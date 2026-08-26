#!/bin/sh

DIRS="cloud-api cmdb-backend file-agent jc-commander jc-worker share-lib watchman-backend yell"
BINS="cloud-api cmdb-backend file-agent jc-commander jc-worker watchman-backend yell"

for dir in $DIRS; do
    if [ ! -d "$dir" ]; then
        echo "Directory $dir does not exist."
        echo "Usage ./build/build_all_ws.sh"
        exit 1
    fi
done

cargo build --release --workspace

mkdir -p ./dist

for bin in $BINS; do
    cp -f target/release/$bin ./dist/
done

exit 0
