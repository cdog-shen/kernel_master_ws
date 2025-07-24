import os
import time
import requests
from enum import Enum

WATCHMAN_HOST = os.getenv("WATCHMAN_HOST", "127.0.0.1")
WATCHMAN_PORT = os.getenv("WATCHMAN_PORT", 8000)
URL = f"http://{WATCHMAN_HOST}:{WATCHMAN_PORT}/api/subsystem_call/cmdb"


class Operation(str, Enum):
    query = "query"
    new = "new"
    update = "update"
    delete = "delete"


def call(operation: Operation, table: str, param: dict = {}):
    JWT = os.getenv("WATCHMAN_JWT")

    payload = {
        "target": "table",
        "operation": operation,
        "data": {"table": table, operation: param},
    }
    headers = {"Authorization": JWT, "content-type": "application/json"}

    time.sleep(0.1)  # Avoid rate limit

    response = requests.request("POST", URL, json=payload, headers=headers)
    return response.json()
