# 获取实例列表
# 参数列表:
# ak: SecretID
# sk: SecretKEY
# region: 地区

import json
import sys
import os
from package_import import ApiClient_Lighthouse

FILE_NAME = os.path.basename(__file__)


try:
    if len(sys.argv) != 5:
        print(f"Usage: python {FILE_NAME} <AK> <SK> <region> <params>")
        sys.exit(1)

    AK = sys.argv[1]
    SK = sys.argv[2]
    region = sys.argv[3]
    params = json.loads(sys.argv[4])
    endpoint = "lighthouse.tencentcloudapi.com"

    client = ApiClient_Lighthouse(AK, SK, region, endpoint)

    req = client.ModelsHandler().DescribeInstancesRequest()
    req.from_json_string(jsonStr=json.dumps(params))

    resp = client.ClientHandler().DescribeInstances(req)

    print(resp.to_json_string(), end="")

except Exception as err:
    import traceback

    print(traceback.format_exc())
