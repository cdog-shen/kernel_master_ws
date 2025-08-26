# -*- coding=utf-8
# 查询所有桶
# 参数列表
# 1. ak: SecretID
# 2. sk: SecretKEY
# 3. region: 地区
# 4. params: 请求参数
# 5. endpoint: 接口地址 (暂时固定, 不影响返回结果)
# param示例:
# {
# }

import json
import sys
import os
from package_import import ApiClient_COS

FILE_NAME = os.path.basename(__file__)


try:
    if len(sys.argv) != 5:
        print(f"Usage: python {FILE_NAME} <AK> <SK> <region> <params>")
        sys.exit(1)

    AK = sys.argv[1]
    SK = sys.argv[2]
    region = sys.argv[3]
    params = json.loads(sys.argv[4])
    endpoint = "cls.tencentcloudapi.com"

    client = ApiClient_COS(AK, SK, region, endpoint)

    resp = client.ClientHandler().list_objects(
        Bucket=params.get("bucket", None), Prefix=params.get("prefix", None)
    )

    print(json.dumps(resp), end="")


except Exception as err:
    import traceback

    print(traceback.format_exc())
