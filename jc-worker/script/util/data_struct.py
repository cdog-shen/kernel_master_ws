import json


def gen_error_msg(
    SCRIPT_NAME: str,
    error_info: str,
    code: int = 500,
):
    return json.dumps(
        {
            "code": code,
            "msg": f"Error on running {SCRIPT_NAME}",
            "data": {"info": f"{error_info}"},
        }
    )


def gen_trace_msg(
    SCRIPT_NAME: str,
    trace_info: str,
    code: int = 500,
):
    return json.dumps(
        {
            "code": code,
            "msg": f"Error on running {SCRIPT_NAME}",
            "data": {"traceback": f"{trace_info}"},
        }
    )


def gen_ok_msg(msg: str, data: dict | list[dict]):
    return json.dumps(
        {
            "code": 200,
            "msg": f"{msg}",
            "data": data,
        }
    )
