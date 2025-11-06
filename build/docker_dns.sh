#!/bin/bash
# 根据当前运行的容器，生成 /etc/hosts 可解析的条目

HOSTS_FILE="/etc/hosts"
DOCKER_NETWORK="kernel_master_ws_default"
MARKER_BEGIN="# >>> Docker containers >>>"
MARKER_END="# <<< Docker containers <<<"

# 生成最新段落
gen_entries() {
    echo "$MARKER_BEGIN"
    docker ps -q \
    | xargs -I{} docker inspect {} \
    | jq -r "
        select(.[0].NetworkSettings.Networks[\"$DOCKER_NETWORK\"].IPAddress != null) |
        \"\(.[0].NetworkSettings.Networks[\"$DOCKER_NETWORK\"].IPAddress) \(.[0].Config.Hostname) \(.[0].Name[1:])\""
    echo "$MARKER_END"
}

# 替换 hosts 中旧段落
update_hosts() {
    # 删除旧标记段落
    sed -i "/^$MARKER_BEGIN$/,/^$MARKER_END$/d" "$HOSTS_FILE"
    # 追加新段落
    gen_entries >> "$HOSTS_FILE"
}

# 首次执行
update_hosts
