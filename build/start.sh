#!/bin/sh

DIRS="cloud-api cmdb-backend file-agent jc-commander jc-worker watchman-backend yell"

for sys in $DIRS; do
    if [ ! -f "$sys" ]; then
        echo "System $sys does not exist."
        echo "PLZ compile all system component."
        echo "THEN COPY this file into work directory."
        echo "THEN ./start.sh"
        exit 1
    fi
done

mkdir -p log

for sys in $DIRS; do
    nohup ./$sys > ./log/"$sys".log 2>&1 &
done

exit 0
