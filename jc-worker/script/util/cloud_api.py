import os
import json
import time
import requests

WATCHMAN_HOST = os.getenv("WATCHMAN_HOST", "127.0.0.1")
WATCHMAN_PORT = os.getenv("WATCHMAN_PORT", 8000)
WATCHMAN_SSL = os.getenv("WATCHMAN_SSL", "disable").lower() == "enable"
WATCHMAN_VERIFY = os.getenv("WATCHMAN_VERIFY", "disable").lower() == "enable"
URL = f"{'https' if WATCHMAN_SSL else 'http'}://{WATCHMAN_HOST}:{WATCHMAN_PORT}/km/watchman/api/subsystem_call/cloud_api"
LIMIT = 100


def merge_json(json1, json2):
    """
    auto merge JSON OBJ and ARR。

    :param json1: json 1
    :param json2: json 2
    :return: merged json

    merge rule:
    Object & array: merge
    string & number: overwrite
    """
    # data1 = json.loads(json1)
    # data2 = json.loads(json2)

    def merge_dicts(d1, d2):
        for key in d2:
            if key in d1:
                if isinstance(d1[key], dict) and isinstance(d2[key], dict):
                    merge_dicts(d1[key], d2[key])
                elif isinstance(d1[key], list) and isinstance(d2[key], list):
                    d1[key].extend(d2[key])
                else:
                    d1[key] = d2[key]
            else:
                d1[key] = d2[key]
        return d1

    merged_json = merge_dicts(json1, json2)

    # merged_json = json.dumps(merged_data, indent=2)
    return merged_json


def call(
    api_name: str,
    cloud_user: str,
    region: str,
    params: dict = {"Offset": 0, "Limit": LIMIT},
) -> dict:
    JWT = os.getenv("WATCHMAN_JWT")
    payload = {
        "target": "script",
        "operation": "call",
        "data": {
            "api_name": api_name,
            "cloud_user": cloud_user,
            "region": region,
            "params": json.dumps(params),
        },
    }
    headers = {
        "Authorization": JWT,
        "content-type": "application/json",
    }

    time.sleep(0.1)  # Avoid rate limit

    response = requests.request(
        "POST", URL, json=payload, headers=headers, verify=WATCHMAN_VERIFY
    )

    if response.status_code != 200:
        try:
            return response.json()
        except Exception as e:
            raise Exception(f"login error, not json response {response.text}") from e

    offset = int(params.get("Offset", 0))

    try:
        resp_json = response.json()
    except Exception as e:
        raise Exception(f"login error, not json response {response.text}") from e
    
    if resp_json.get("data", {}).get("data", {}).get("TotalCount"):
        total_count = int(
            resp_json.get("data", {}).get("data", {}).get("TotalCount")
        )
    else:
        total_count = 1

    if LIMIT * (offset + 1) < total_count:
        next_resp_json = call(
            api_name=api_name,
            cloud_user=cloud_user,
            region=region,
            params={"Offset": offset + 1, "Limit": LIMIT},
        )
        return merge_json(resp_json, next_resp_json)
    else:
        return resp_json
