import smtplib
from email.mime.multipart import MIMEMultipart


def gen_mail_client(host, port, account_addr, access_codes) -> smtplib.SMTP_SSL | None:

    client = smtplib.SMTP_SSL(host, port)
    try:
        client.login(account_addr, access_codes)
    except Exception as e:
        raise Exception("Failed to login : {}".format(e))

    return client


def send_mail(
    client: smtplib.SMTP_SSL, receivers: list[str], msg: MIMEMultipart
) -> dict:

    try:
        client.sendmail(
            client.user,
            receivers,
            msg.as_string(),
        )
    except Exception as e:
        raise Exception("Failed to send mail : {}".format(e))

    return {"code": 200, "status": "success"}
