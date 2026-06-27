FROM rust:latest

ARG DATE
ARG COMMIT
ARG VERSION

RUN apt-get install -y \
        gcc-arm-linux-gnueabihf \
        gcc-aarch64-linux-gnu \
        gcc-x86-64-linux-gnu \
        musl-tools \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add \
        arm-unknown-linux-gnueabihf \
        armv7-unknown-linux-gnueabihf \
        aarch64-unknown-linux-gnu \
        aarch64-unknown-linux-musl \
        x86_64-unknown-linux-gnu \
        x86_64-unknown-linux-musl

COPY cross-build/config.toml /usr/local/cargo/config.toml

WORKDIR /workspace

ENV DATE=${DATE} COMMIT=${COMMIT} VERSION=${VERSION}

CMD cargo build --release --target arm-unknown-linux-gnueabihf && \
    cargo build --release --target armv7-unknown-linux-gnueabihf && \
    cargo build --release --target aarch64-unknown-linux-gnu && \
    cargo build --release --target aarch64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-gnu && \
    cargo build --release --target x86_64-unknown-linux-musl
