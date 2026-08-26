#!/bin/sh

DIRS="cloud-api cmdb-backend file-agent jc-commander jc-worker share-lib watchman-backend yell"

for dir in $DIRS; do
    if [ ! -d "$dir" ]; then
        echo "Directory $dir does not exist."
        echo "Usage ./build/clean_all_ws.sh"
        exit 1
    fi
done

cargo clean

exit 0
