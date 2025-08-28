# running check logic
# 1. 确认参数数量

import json


class run:

    @staticmethod
    def pre_running_check(params_dict: str | dict, params_needed: list[str]):
        if type(params_dict) is str:
            params_dict = json.loads(params_dict)
        elif not (type(params_dict) is dict):
            raise TypeError(f"Param: {params_dict} is not JSON or JSON string")

        for key in params_needed:
            if (
                bool(params_dict.get(key, None)) == False
                and params_dict.get(key, None) != 0
            ):
                raise ValueError(f"Error: Need parameter ({key})")

    def __init__(self, params_dict: str | dict):
        self.params = json.loads(params_dict)

    def dispatch(self, fn_handle):
        return fn_handle(self)
