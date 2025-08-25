from typing import Tuple
import requests


CARD_COMPONENTS = {
    "source": lambda icon_url, title, color=0: {
        "icon_url": icon_url,
        "desc": title,
        "desc_color": color,
    },
    "main_title": lambda title, desc: {
        "title": title,
        "desc": desc,
    },
    "card_image": lambda image_url, aspect_ratio=2.25: {
        "url": image_url,
        "aspect_ratio": aspect_ratio,
    },
    "jump_item": lambda url, title, _type=1: {
        "type": _type,
        "url": url,
        "title": title,
    },
}


# main function
def send_to_group(webhook_url: str, card_type: str, content: str | dict) -> dict:

    headers = {"Content-Type": "application/json"}
    if card_type == "text":
        payload = {
            "msgtype": "text",
            "text": {"content": (content if content else "💡 测试 \n\t--Job center")},
        }
    elif card_type == "markdown":
        payload = {
            "msgtype": "markdown_v2",
            "markdown_v2": {
                "content": (content if content else "# 测试Markdown\n\n- 啥也没有")
            },
        }
    elif card_type == "card" and isinstance(content, dict):
        payload = {
            "msgtype": "template_card",
            "template_card": {
                "card_type": "text_notice",
                "source": CARD_COMPONENTS["source"](
                    icon_url=content.get(
                        "icon_url",
                        "https://wework.qpic.cn/wwpic/252813_jOfDHtcISzuodLa_1629280209/0",
                    ),
                    title=content.get("source_title", "测试卡片"),
                ),
                "main_title": CARD_COMPONENTS["main_title"](
                    title=content.get("main_title", "测试卡片"),
                    desc=content.get("main_desc", ""),
                ),
                "jump_list": [
                    CARD_COMPONENTS["jump_item"](
                        url=item.get("url", ""),
                        title=item.get("title", ""),
                    )
                    for item in content.get("jump_list", [])
                ],
                "card_action": {
                    "type": 1,
                    "url": content.get("card_url", ""),
                },
            },
        }
    else:
        return {"code": 500, "msg": "Invalid message type", "data": {}}

    info = requests.post(
        webhook_url,
        json=payload,
        headers=headers,
        timeout=10,
    ).json()

    return {"code": 200, "msg": "Send Message Success", "data": info}
