FROM rust:alpine

LABEL maintainer="shencdog@gmail.com"
RUN apk add --no-cache \
    build-base \
    zlib-static \
    openssl-dev openssl-libs-static \
    mariadb-dev mariadb-static \
    git \
    curl

# RUN sh -c 'rustup target add x86_64-unknown-linux-musl && mkdir -p /root/build'

RUN git clone https://github.com/cdog-shen/kernel_master_ws.git /root/kernel_master_ws

WORKDIR /root/kernel_master_ws

ENV RUSTFLAGS="-C link-arg=-L/usr/lib \
               -C link-arg=-L/lib \
               -C link-arg=-Wl,--start-group \
               -C link-arg=-lmariadb \
               -C link-arg=-lssl \
               -C link-arg=-lcrypto \
               -C link-arg=-lz \
               -C link-arg=-Wl,--end-group"

ENTRYPOINT ["/bin/sh", "-c", "while true; do sleep 1000; done"]
