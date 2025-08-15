#!/bin/sh

DIRS="cloud-api cmdb-backend jc-commander jc-worker watchman-backend file-agent"

mkdir -p ./target/log

for dir in $DIRS; do
    if [ ! -d "$dir" ]; then
        echo "Directory $dir does not exist."
        echo "Usage ./build/start_all_by_docker_compose.sh"
        exit 1
    fi
done

cp -f ./cloud-api/cloud_api.template.toml ./target/cloud_api.toml
cp -f ./cmdb-backend/cmdb.template.toml ./target/cmdb.toml
cp -f ./jc-commander/job_center_commander.template.toml ./target/job_center_commander.toml
cp -f ./jc-worker/job_center_worker.template.toml ./target/job_center_worker.toml
cp -f ./watchman-backend/watchman.template.toml ./target/watchman.toml

if [ $(type docker-compose) == "" ]; then
    echo "docker-compose NOT in ur PATH."
    echo "PLZ install docker-compose first."
    exit 1
fi

docker-compose --profile run up -d
