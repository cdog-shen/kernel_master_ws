#!/usr/bin/env groovy

/**
 * Yell 通知服务 — Jenkins Pipeline 调用封装
 *
 * 使用方法（Jenkins Pipeline）:
 *
 *   1. 直接发送通知:
 *      yell.sendNotification(
 *          yellUrl: 'http://yell-host:9005',
 *          token: 'uuid 550e8400-e29b-41d4-a716-446655440000',
 *          title: '系统告警',
 *          body: 'CPU 使用率超过 90%',
 *          recipients: [
 *              [channelType: 'bark', instance: 'bark-main', recipient: 'DEVICE_KEY'],
 *              [channelType: 'smtp', instance: 'smtp-main', recipient: 'ops@example.com']
 *          ]
 *      )
 *
 *   2. 通过 alias 发送（无需指定渠道细节）:
 *      yell.sendNotification(
 *          yellUrl: 'http://yell-host:9005',
 *          token: 'uuid 550e8400-e29b-41d4-a716-446655440000',
 *          title: '系统告警',
 *          body: 'CPU 使用率超过 90%',
 *          recipients: 'ops-team'
 *      )
 *
 *   3. 使用模板发送:
 *      yell.sendWithTemplate(
 *          yellUrl: 'http://yell-host:9005',
 *          token: 'uuid 550e8400-e29b-41d4-a716-446655440000',
 *          templateName: 'bark_urgent_alert',
 *          variables: [level: '严重', service: 'CMDB', message: '连接超时', group: 'cmdb-alerts', url: 'https://monitor.example.com/alert/1'],
 *          recipients: [
 *              [channelType: 'bark', instance: 'bark-main', recipient: 'DEVICE_KEY']
 *          ]
 *      )
 *
 *   4. 使用 Gotify 模板发送:
 *      yell.sendWithTemplate(
 *          yellUrl: 'http://yell-host:9005',
 *          token: 'uuid 550e8400-e29b-41d4-a716-446655440000',
 *          templateName: 'gotify_urgent',
 *          variables: [level: '严重', service: 'CMDB', message: '连接超时', url: 'https://monitor.example.com/alert/1'],
 *          recipients: [
 *              [channelType: 'gotify', instance: 'gotify-main', recipient: 'APP_TOKEN']
 *          ]
 *      )
 *
 *   5. 使用 Teams 模板发送:
 *      yell.sendWithTemplate(
 *          yellUrl: 'http://yell-host:9005',
 *          token: 'uuid 550e8400-e29b-41d4-a716-446655440000',
 *          templateName: 'teams_urgent',
 *          variables: [level: '严重', service: 'CMDB', message: '连接超时', url: 'https://monitor.example.com/alert/1'],
 *          recipients: [
 *              [channelType: 'teams', instance: 'teams-main', recipient: '']
 *          ]
 *      )
 *
 *   6. 使用通用 Webhook 发送:
 *      yell.sendWithTemplate(
 *          yellUrl: 'http://yell-host:9005',
 *          token: 'uuid 550e8400-e29b-41d4-a716-446655440000',
 *          templateName: 'webhook_json',
 *          variables: [title: '告警', body: 'CPU过高', level: 'critical'],
 *          recipients: [
 *              [channelType: 'webhook', instance: 'webhook-main', recipient: '']
 *          ]
 *      )
 *
 *   7. 全局配置（通过 environment 块设置环境变量）:
 *      environment {
 *          YELL_URL = 'http://yell-host:9005'
 *          YELL_TOKEN = 'uuid xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx'
 *      }
 *      yell.sendNotification(title: '...', body: '...', recipients: [...])
 */

class YellClient implements Serializable {
    def steps
    String yellUrl
    String token

    YellClient(steps, String yellUrl = null, String token = null) {
        this.steps = steps
        this.yellUrl = yellUrl ?: steps.env.YELL_URL ?: 'http://localhost:9005'
        this.token = token ?: steps.env.YELL_TOKEN ?: ''
    }

    /**
     * 直接发送通知
     *
     * @param args 参数 Map
     *   - title:      消息标题
     *   - body:       消息正文（必填）
     *   - format:     格式 text/html/markdown（默认 text）
     *   - priority:   优先级 low/normal/high/urgent（默认 normal）
     *   - tags:       标签列表（可选）
     *   - url:        关联链接（可选）
     *   - mentions:   提醒列表（可选）
     *   - recipients: 接收者列表（必填）
     *       每个元素: [channelType: '...', instance: '...', recipient: '...']
     *   - yellUrl:    覆盖全局 URL（可选）
     *   - token:      覆盖全局 Token（可选）
     */
    def sendNotification(Map args) {
        def url = args.yellUrl ?: this.yellUrl
        def authToken = args.token ?: this.token
        def body = args.body ?: args.message ?: error('body/message is required')

        def payload = [
            title    : args.title ?: '',
            body     : body,
            format   : args.format ?: 'text',
            priority : args.priority ?: 'normal',
            recipients: parseRecipients(args.recipients)
        ]

        if (args.tags) payload.tags = args.tags
        if (args.url) payload.url = args.url
        if (args.mentions) payload.mentions = args.mentions

        return doPost("${url}/notify/send", authToken, payload)
    }

    /**
     * 使用模板发送通知
     *
     * @param args 参数 Map
     *   - templateName: 模板名称（必填）
     *   - variables:    模板变量 Map（可选）
     *   - recipients:   接收者列表（必填）
     *   - yellUrl:      覆盖全局 URL（可选）
     *   - token:        覆盖全局 Token（可选）
     */
    def sendWithTemplate(Map args) {
        def url = args.yellUrl ?: this.yellUrl
        def authToken = args.token ?: this.token
        def templateName = args.templateName ?: args.template ?: error('templateName is required')

        def payload = [
            template_name: templateName,
            variables    : args.variables ?: [:],
            recipients   : parseRecipients(args.recipients)
        ]

        return doPost("${url}/notify/template", authToken, payload)
    }

    /**
     * 解析 recipients 参数，支持多种写法
     */
    private List parseRecipients(def recipients) {
        if (!recipients) error('recipients is required')

        def result = []
        recipients.each { r ->
            def item = [:]
            item.channel_type = r.channelType ?: r.channel_type ?: error("Missing channelType in recipient: ${r}")
            item.instance = r.instance ?: error("Missing instance in recipient: ${r}")
            item.recipient = r.recipient ?: error("Missing recipient in recipient: ${r}")
            result.add(item)
        }
        return result
    }

    /**
     * 发送 HTTP POST 请求
     *
     * 优先使用 httpRequest 插件，回退到 sh curl
     */
    private def doPost(String url, String token, Map payload) {
        def json = groovy.json.JsonOutput.toJson(payload)

        // 优先尝试 httpRequest 插件（Jenkins HTTP Request Plugin）
        if (steps.metaClass.respondsTo(steps, 'httpRequest', Map)) {
            def response = steps.httpRequest(
                url: url,
                httpMode: 'POST',
                contentType: 'APPLICATION_JSON',
                requestBody: json,
                customHeaders: [[name: 'Authorization', value: token]],
                validResponseCodes: '200:299',
                consoleLogResponseBody: false
            )
            steps.echo "Yell response: ${response.content}"
            return groovy.json.JsonSlurperClassic.newInstance().parseText(response.content)
        }

        // 回退: 使用 curl 命令
        def escapedJson = json.replace("'", "'\\''")
        def curlCmd = "curl -s -w '\\n%{http_code}' -X POST '${url}' -H 'Authorization: ${token}' -H 'Content-Type: application/json' -d '${escapedJson}'"

        def output = steps.sh(script: curlCmd, returnStdout: true).trim()

        def lines = output.split('\n')
        def httpCode = lines[-1].trim()
        def body = lines[0..-2].join('\n')

        if (httpCode != '200') {
            error("Yell API error: HTTP ${httpCode}, body: ${body}")
        }

        steps.echo "Yell response: ${body}"
        return groovy.json.JsonSlurperClassic.newInstance().parseText(body)
    }

    private def error(String msg) {
        steps.error("[Yell] ${msg}")
    }
}

// ============================================================
// Jenkins Shared Library / Pipeline 顶层入口
// ============================================================
// 在 Pipeline 中通过 load 或 shared library 引用后，可直接调用:
//   yell.sendNotification(...)
//   yell.sendWithTemplate(...)
//
// 默认配置通过环境变量设置:
//   environment {
//       YELL_URL = 'http://yell-host:9005'
//       YELL_TOKEN = 'uuid xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx'
//   }

/**
 * 直接发送通知
 */
def sendNotification(Map args) {
    def client = new YellClient(this, args.yellUrl, args.token)
    return client.sendNotification(args)
}

/**
 * 使用模板发送通知
 */
def sendWithTemplate(Map args) {
    def client = new YellClient(this, args.yellUrl, args.token)
    return client.sendWithTemplate(args)
}
