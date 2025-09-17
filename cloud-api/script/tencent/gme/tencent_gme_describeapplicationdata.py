# 查询APP ID用量
# 参数列表
# 1. ak: SecretID
# 2. sk: SecretKEY
# 3. region: 地区
# 4. params: 请求参数
# 5. endpoint: 接口地址 (暂时固定, 不影响返回结果)
# param示例:
# {
#   "BizId": 1400801101,
#   "StartDate": "2025-09-16",
#   "EndDate": "2025-09-16"
# }

import json
import sys
import os
from package_import import ApiClient_GME

FILE_NAME = os.path.basename(__file__)


try:
    if len(sys.argv) != 5:
        print(f"Usage: python {FILE_NAME} <AK> <SK> <region> <params>")
        sys.exit(1)

    AK = sys.argv[1]
    SK = sys.argv[2]
    region = sys.argv[3]
    params = json.loads(sys.argv[4])
    endpoint = "gme.tencentcloudapi.com"

    client = ApiClient_GME(AK, SK, region, endpoint)

    req = client.ModelsHandler().DescribeApplicationDataRequest()
    req.from_json_string(jsonStr=json.dumps(params))

    resp = client.ClientHandler().DescribeApplicationData(req)

    print(resp.to_json_string(), end="")


except Exception as err:
    import traceback

    print(traceback.format_exc())
