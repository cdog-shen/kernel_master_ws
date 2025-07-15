# FROM alpine:3.16.3
FROM alpine:latest

RUN apk add --no-cache \
    build-base \
    mariadb-dev \
    mariadb-static \
    openssl-dev \
    openssl-libs-static \
    zlib-static \
    git \
    curl


RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"
RUN cd /root && rustup target add x86_64-unknown-linux-musl && git clone https://github.com/cdog-shen/kernel_master_ws.git

WORKDIR /root

ENV RUSTFLAGS="-C link-arg=-L/usr/lib \
               -C link-arg=-L/lib \
               -C link-arg=-Wl,--start-group \
               -C link-arg=-lmariadb \
               -C link-arg=-lssl \
               -C link-arg=-lcrypto \
               -C link-arg=-lz \
               -C link-arg=-Wl,--end-group"

ENTRYPOINT ["/bin/sh", "-c", "while true; do sleep 1000; done"]