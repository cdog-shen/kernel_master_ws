#!/bin/sh

DIRS="cloud-api cmdb-backend jc-commander jc-worker watchman-backend file-agent share-lib"

for dir in $DIRS; do
    if [ ! -d "$dir" ]; then
        echo "Directory $dir does not exist."
        echo "Usage ./build/clean_all_ws.sh"
        exit 1
    fi
done

for dir in $DIRS; do
    cd $dir && cargo clean
    cd -
done

exit 0
