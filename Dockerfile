FROM rust:latest

RUN apt update -y && \
    apt upgrade -y && \
    apt install -y g++-mingw-w64-x86-64

RUN rustup target add x86_64-pc-windows-gnu && \
    rustup target add x86_64-unknown-linux-gnu
