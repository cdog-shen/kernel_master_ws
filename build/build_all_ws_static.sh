#!/bin/sh

DIRS="cloud-api cmdb-backend jc-commander jc-worker watchman-backend file-agent"

for dir in $DIRS; do
    if [ ! -d "$dir" ]; then
        echo "Directory $dir does not exist."
        echo "Usage ./build/build_all_ws_static.sh"
        exit 1
    fi
done

mkdir -p ./target

for dir in $DIRS; do
    cd $dir && cargo build --release --target=x86_64-unknown-linux-musl
    cp target/x86_64-unknown-linux-musl/release/$dir ../target/
    cd -
done

exit 0
