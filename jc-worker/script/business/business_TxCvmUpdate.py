import sys
import json

from pathlib import Path
from typing import Tuple


ROOT_DIR = Path(__file__).parent.parent
sys.path.append(str(ROOT_DIR))

from util.data_struct import *
from util.auth import *
from util.runner import run
from util import cloud_api
from util import cmdb

# file meta
SCRIPT_NAME = Path(__file__).stem
PARAMS = sys.argv[1]
DEEDED_PARAMS = []


# main function
def main(runner: run) -> Tuple[int, str, dict | list[dict]]:
    # get cloud data from Cloud-API subsystem
    cloud_data = cloud_api.call(
        "tencent_cvm_describeinstances",
        "Unknow-api-haoxin",
        "ap-shanghai",
    )

    if cloud_data.get("code", 500) != 200:
        return (
            500,
            "cloud_api",
            {
                "api": "tencent_lighthouse_describeinstances",
                "info": cloud_data.get("data"),
            },
        )

    cloud_data: dict = cloud_data.get("data", {}).get("data", {})

    # compare and update
    for instance in cloud_data.get("InstanceSet", []):
        current_data = cmdb.call(
            "get", "cloudserver", {"instance_name": instance.get("InstanceName")}
        )
        if current_data.get("code", 500) != 200:
            return (
                500,
                "CMDB",
                {
                    "api": "table_get",
                    "info": cloud_data.get("data"),
                },
            )
        elif len(current_data.get("data", {}).get("data", [])) == 0:
            new_resp = cmdb.call(
                "new",
                "cloudserver",
                {
                    "provider": "Tencent",
                    "zone": instance.get("Placement", {}).get("Zone"),
                    "instance_id": instance.get("InstanceId"),
                    "instance_name": instance.get("InstanceName"),
                    "plantform": (
                        "windows"
                        if "windows" in instance.get("OsName", "").lower()
                        else "Unix"
                    ),
                    "status": instance.get("InstanceState"),
                    "tag": "[]",
                    "private_ip": instance.get("PrivateIpAddresses", [])[0],
                    "full_info": "{}",
                    "attach_info": "{}",
                },
            )
            if new_resp.get("code", 500) != 200:
                return (
                    500,
                    "CMDB",
                    {
                        "api": "table_new",
                        "info": new_resp.get("data"),
                    },
                )
        else:
            update_resp = cmdb.call(
                "update",
                "light_ecs",
                {
                    "id": current_data.get("data").get("data")[0].get("id"),
                    "provider": "Tencent",
                    "zone": instance.get("Placement", {}).get("Zone"),
                    "instance_id": instance.get("InstanceId"),
                    "instance_name": instance.get("InstanceName"),
                    "plantform": (
                        "windows"
                        if "windows" in instance.get("OsName", "").lower()
                        else "Unix"
                    ),
                    "status": instance.get("InstanceState"),
                    "tag": current_data.get("data").get("data")[0].get("tag"),
                    "private_ip": instance.get("PrivateIpAddresses", [])[0],
                    "full_info": current_data.get("data")
                    .get("data")[0]
                    .get("full_info"),
                    "attach_info": current_data.get("data")
                    .get("data")[0]
                    .get("attach_info"),
                },
            )
            if update_resp.get("code", 500) != 200:
                return (
                    500,
                    "CMDB",
                    {
                        "api": "table_update",
                        "info": update_resp.get("data"),
                    },
                )

    return (200, runner.params.get("test_name"), {"info": "ok"})


if __name__ == "__main__":
    try:
        JWT_check()
        run.pre_running_check(params_dict=PARAMS, params_needed=DEEDED_PARAMS)
        runner = run(params_dict=PARAMS)

        (code, msg, data) = runner.dispatch(fn_handle=main)

        if code != 200:
            print(gen_error_msg(SCRIPT_NAME, str(data), code), end="")
        else:
            print(gen_ok_msg(msg, data), end="")

    except Exception as error:
        import traceback

        print(gen_trace_msg(SCRIPT_NAME, traceback.format_exc()), end="")
