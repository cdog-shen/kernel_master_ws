import os
import json
import requests

WATCHMAN_HOST = os.getenv("WATCHMAN_HOST", "10.16.18.128")
WATCHMAN_PORT = os.getenv("WATCHMAN_PORT", 8000)
URL = f"http://{WATCHMAN_HOST}:{WATCHMAN_PORT}/api/subsystem_call/cloud_api"


def call(api_name: str, cloud_user: str, region: str, params: dict = {}) -> dict:
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

    response = requests.request("POST", URL, json=payload, headers=headers)
    return response.json()
