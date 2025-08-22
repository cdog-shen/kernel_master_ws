import requests
import os

WATCHMAN_HOST = os.getenv("WATCHMAN_HOST", "127.0.0.1")
WATCHMAN_PORT = os.getenv("WATCHMAN_PORT", 8000)
CHECK_URL = f"http://{WATCHMAN_HOST}:{WATCHMAN_PORT}/km/watchman/api/auth/me/2"
LOGIN_URL = f"http://{WATCHMAN_HOST}:{WATCHMAN_PORT}/km/watchman/api/auth/login"
OP_USERNAME = os.getenv("OP_USERNAME", "script_caller")
OP_PASSWD = os.getenv("OP_PASSWD", "script_caller")
JWT = os.getenv("WATCHMAN_JWT")


def login():
    global JWT
    count = 0
    while True:
        payload = {"username": f"{OP_USERNAME}", "passwd": f"{OP_PASSWD}"}
        headers = {"content-type": "application/json"}

        response = requests.request("POST", LOGIN_URL, json=payload, headers=headers)
        resp_json = response.json()

        if resp_json.get("code", 500) != 200:
            count += 1
            if count > 3:
                raise Exception(f"login error {resp_json}")
            else:
                continue

        elif resp_json.get("code", 500) == 200:
            jwt_str = f"{resp_json.get('data').get('token_type')} {resp_json.get('data').get('token')}"
            os.environ["WATCHMAN_JWT"] = jwt_str
            JWT = jwt_str
            break


def JWT_check():
    global JWT
    count = 0

    while True:
        if JWT is None:
            login()

        headers = {"Authorization": f"{JWT}"}

        response = requests.request("GET", CHECK_URL, headers=headers)
        resp_status = response.status_code

        if resp_status != 200:
            count += 1
            if count > 3:
                raise Exception(f"Check error")
            else:
                JWT = None
                os.environ["WATCHMAN_JWT"] = ""
                continue

        elif resp_status == 200:
            break
