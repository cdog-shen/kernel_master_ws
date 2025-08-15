#!/bin/sh

DIRS="cloud-api cmdb-backend jc-commander jc-worker watchman-backend file-agent"

for sys in $SYSS; do
    if [ ! -f "$sys" ]; then
        echo "System $sys does not exist."
        echo "PLZ compile all system component."
        echo "THEN COPY this file into work directory."
        echo "THEN ./start.sh"
        exit 1
    fi
done

mkdir -p log

for sys in $SYSS; do
    nohup ./$sys > ./log/"$sys".log 2>&1 &
done

exit 0
