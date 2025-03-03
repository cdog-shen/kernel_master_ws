# 获取实例列表
# 参数列表:
# ak: SecretID
# sk: SecretKEY
# region: 地区

import json
import types
from package_import import ApiClient_Lighthouse, TencentCloudSDKException
import sys


try:
    if len(sys.argv) != 5:
        print("Usage: python tencent_lighthouse_describeinstances.py <AK> <SK> <region> <params>")
        sys.exit(1)

    AK = sys.argv[1]
    SK = sys.argv[2]
    region = sys.argv[3]
    params = json.loads(sys.argv[4])
    endpoint = "lighthouse.tencentcloudapi.com"

    client = ApiClient_Lighthouse(AK, SK, endpoint, region)

    req = client.ModelsHandler().DescribeInstancesRequest()
    req.from_json_string(jsonStr=json.dumps(params))

    resp = client.ClientHandler().DescribeInstances(req)

    print(resp.to_json_string())


except TencentCloudSDKException as err:
    print(err)
