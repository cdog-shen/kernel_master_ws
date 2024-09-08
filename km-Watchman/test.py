import rsa


def rsaEncrypt(message):
    """
    RSA加密函数
    传入需要加密的内容,进行RSA加密,并返回密文 & 私钥 & 公钥
    :param message: 需要加密的内容,明文
    :return: 密文 & 私钥 & 公钥
    """
    (key_pub , key_pri)  = rsa.newkeys(1024)
    # print(key_pri)
    # print(key_pub)
    content = message.encode("utf-8")
    crypto = rsa.encrypt(content, key_pub)
    return (crypto, key_pri, key_pub)


def rsaDecrypt(message, key_pri):
    """
    RSA 解密函数,传入密文 & 私钥,得到明文；
    :param message: 密文
    :param key_pri: 私钥
    :return: 明文
    """
    content = rsa.decrypt(message, key_pri)
    return content.decode("utf-8")


# 公钥加密,私钥解密

message = "I Love China. 我爱你中国！"

print("加密前: {} \n".format(message))

crypto, key_pri, key_pub = rsaEncrypt(message)

print("加密后: {} \n-------------------".format(crypto))
print("秘钥为: {} \n-------------------".format(key_pri))
print("公钥为: {} \n-------------------".format(key_pub))

content = rsaDecrypt(crypto, key_pri)
print("明文为: {} \n-------------------".format(content))
