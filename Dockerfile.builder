FROM ubuntu:latest

LABEL maintainer="shencdog@gmail.com"

RUN apt update && apt install gcc g++ pkg-config libmysqlclient-dev git curl -y

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# RUN sh -c 'rustup target add x86_64-unknown-linux-musl && mkdir -p /root/build'

RUN cd && git clone https://github.com/cdog-shen/kernel_master_ws.git

WORKDIR /root/kernel_master_ws

ENV PATH="/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

# ENV RUSTFLAGS="-C link-arg=-L/usr/lib \
#                -C link-arg=-L/lib \
#                -C link-arg=-Wl,--start-group \
#                -C link-arg=-lmariadb \
#                -C link-arg=-lssl \
#                -C link-arg=-lcrypto \
#                -C link-arg=-lz \
#                -C link-arg=-Wl,--end-group"
