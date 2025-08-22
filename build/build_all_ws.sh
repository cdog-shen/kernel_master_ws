#!/bin/sh

DIRS="cloud-api cmdb-backend jc-commander jc-worker share-lib watchman-backend file-agent share-lib"

for dir in $DIRS; do
    if [ ! -d "$dir" ]; then
        echo "Directory $dir does not exist."
        echo "Usage ./build/build_all_ws.sh"
        exit 1
    fi
done

mkdir -p ./target

for dir in $DIRS; do
    cd $dir && cargo build --release
    cp -f target/release/$dir ../target/
    cd -
done

exit 0
