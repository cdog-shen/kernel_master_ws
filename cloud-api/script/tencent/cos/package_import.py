# -*- coding: utf-8 -*-
import sys
import json
import types
import logging
from qcloud_cos import CosConfig
from qcloud_cos import CosS3Client

# 正常情况日志级别使用INFO，需要定位时可以修改为DEBUG，此时SDK会打印和服务端的通信信息
logging.basicConfig(level=logging.INFO, stream=sys.stdout)


class ApiClient_COS:
    def __init__(
        self,
        AK,
        SK,
        region="ap-shanghai",
        endpoint="cvm.tencentcloudapi.com",
    ):
        try:
            config = CosConfig(Region=region, SecretId=AK, SecretKey=SK, Token=None)
            self.client = CosS3Client(config)

        except Exception as err:
            print(err)
            return err

    def ClientHandler(self):
        return self.client

    # def ModelsHandler(self):
    #     return models
