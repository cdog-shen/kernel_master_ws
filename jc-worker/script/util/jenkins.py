import os
import json
import time
from typing import Dict, Any, Optional
import requests
from requests.auth import HTTPBasicAuth


class JenkinsClient:
    def __init__(
        self,
        base_url: str,
        username: Optional[str] = None,
        password: Optional[str] = None,
        api_token: Optional[str] = None,
        ssl_verify: bool = True,
        timeout: int = 10,
    ):
        """
        base_url: http(s)://jenkins.example.com[:port]
        auth method:
            - username + api_token  (recommended)
            - username + password   (triggers Basic Auth)
        """
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self.verify = ssl_verify

        if api_token:
            self.auth = HTTPBasicAuth(username or "", api_token)
        elif username and password:
            self.auth = HTTPBasicAuth(username, password)
        else:
            raise ValueError("Need (username + api_token) or (username + password)")

        self._session = requests.Session()
        self._session.auth = self.auth
        self._session.verify = ssl_verify

    # utils
    def _request(
        self,
        method: str,
        endpoint: str,
        *,
        params: Optional[Dict[str, Any]] = None,
        data: Any = None,
        headers: Optional[Dict[str, str]] = None,
    ) -> requests.Response:
        url = f"{self.base_url}{endpoint}"
        resp = self._session.request(
            method,
            url,
            params=params,
            data=data,
            headers=headers,
            timeout=self.timeout,
        )
        resp.raise_for_status()
        return resp

    def _flatten_folder(self, folder: str | list[str]) -> str:
        path = ""
        if folder and len(folder) > 0:

            if isinstance(folder, str):
                folder = folder.split("/")

            if isinstance(folder, list):
                for f in folder:
                    path = path + f"/job/{f}"

        return path

    def last_build_number(
        self,
        name: str,
        folder: Optional[str | list[str]] = None,
    ) -> int:
        base = self._flatten_folder(folder) if folder else ""
        path = f"{base}/job/{name}/lastBuild/buildNumber"

        return int(self._request("GET", path).text.strip())

    def get_input_id(
        self, name: str, number: int, folder: Optional[str] = None
    ) -> Optional[str]:
        base = self._flatten_folder(folder) if folder else ""
        data = self._request(
            "GET", f"{base}/job/{name}/{number}/api/json?tree=actions[*]"
        ).json()
        for act in data.get("actions", []):
            if (
                act.get("_class")
                == "org.jenkinsci.plugins.workflow.support.steps.input.InputAction"
            ):
                inputs = act.get("inputs", [])
                if inputs:
                    return inputs[0].get("id")
        return None

    # common
    def info(self) -> Dict[str, Any]:
        """sys info"""
        return self._request("GET", "/api/json").json()

    def job_exists(self, name: str) -> bool:
        try:
            self._request("GET", f"/job/{name}/api/json")
            return True
        except requests.HTTPError as e:
            if e.response.status_code == 404:
                return False
            raise

    def delete_job(self, name: str, folder: Optional[str] = None):
        base = self._flatten_folder(folder) if folder else ""
        path = f"{base}/job/{name}/doDelete"

        self._request("POST", path)

    def build_job(
        self,
        name: str,
        params: Optional[Dict[str, str]] = None,
        folder: Optional[str | list[str]] = None,
    ) -> int:
        """
        build a pipeline, return queue id
        """
        base = self._flatten_folder(folder) if folder else ""
        path = f"{base}/job/{name}/build"

        if params:
            path += "WithParameters"
            self._request("POST", path, data=params)
        else:
            self._request("POST", path)

        return self.last_build_number(name, folder) + 1

    def build_console(
        self, name: str, number: int, folder: Optional[str] = None
    ) -> str:
        base = self._flatten_folder(folder) if folder else ""
        path = f"{base}/job/{name}/{number}/consoleText"

        return self._request("GET", path).text

    def build_result(self, name: str, number: int, folder: Optional[str] = None) -> str:
        base = self._flatten_folder(folder) if folder else ""
        path = f"{base}/job/{name}/{number}/api/json"

        return self._request("GET", path).json()["result"]

    def proceed_or_abort(
        self,
        name: str,
        number: int,
        folder: Optional[str] = None,
        proceed: bool = True,
        input_id: str = "Proceed or Abort",
    ) -> str:
        """
        对指定 Pipeline 构建的 input 步骤做 continue/abort。
        proceed=True  -> 点「Proceed」
        proceed=False -> 点「Abort」
        input_id 可在 job 的 config.xml 里自定义，默认 'Proceed or Abort'
        """
        base = self._flatten_folder(folder) if folder else ""
        url = f"{base}/job/{name}/{number}/input/{input_id}/submit"

        if proceed:
            data = {"proceed": "Proceed", "json": '{"proceed":"","abort":""}'}
        else:
            data = {"abort": "Abort", "json": '{"proceed":"","abort":""}'}

        headers = {
            "Content-Type": "application/x-www-form-urlencoded",
            # "Referer": f"{self.base_url}{base}/job/{name}/{number}/input/",
        }

        resp = self._request("POST", url, data=data, headers=headers)
        return resp.text


# def safe_restart(self):
#     self._request("POST", "/safeRestart")
