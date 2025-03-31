# test
import sys
import json

from pathlib import Path
from typing import Tuple

ROOT_DIR = Path(__file__).parent.parent
sys.path.append(str(ROOT_DIR))

from util.data_struct import *
from util.auth import *
from util.runner import run

# file meta
SCRIPT_NAME = Path(__file__).stem
PARAMS = sys.argv[1]
DEEDED_PARAMS = ["test_name"]


# main function
def main(runner: run) -> Tuple[int, str, dict | list[dict]]:
    return (200, runner.params.get("test_name"), {"info": "ok"})


if __name__ == "__main__":
    try:
        JWT_check()
        run.pre_running_check(params_dict=PARAMS, params_needed=DEEDED_PARAMS)
        runner = run(params_dict=PARAMS)

        (code, msg, data) = runner.dispatch(fn_handle=main)

        if code != 200:
            print(gen_error_msg(SCRIPT_NAME, data, code))
        else:
            print(gen_ok_msg(msg, data))

    except Exception as error:
        import traceback

        print(gen_trace_msg(SCRIPT_NAME, traceback.format_exc()))
