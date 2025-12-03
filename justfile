export DATE    := `date '+%F'`
export COMMIT  := `GIT_CONFIG_GLOBAL=/dev/null git rev-parse --short=8 HEAD`
export VERSION := "latest"

# List build commands and descriptions
help:
    @just --list --unsorted --list-heading $'🌈 Available commands\n' --list-prefix ' · '
    @echo
    @echo '🚀 Release builds'
    @echo ' · make VERSION=v1.0 build-release build-docker-release'
    @echo
    @echo '💡 Related commands'
    @echo ' · rustup update          - Update Rust toolchain to latest version'
    @echo ' · cargo update           - Update dependencies to latest version'

# Cleanup intermediary files
clean:
    rm -rf target upstate-*.zip

# Run debug version
run +args='':
    cargo run -- {{args}}

# Build local binary
build:
    cargo build --release

# Build multi-architecture binaries
build-release:
    #!/usr/bin/env bash
    set -euxo pipefail
    build-cross() {
        local TARGET=$1 IDENT=$2 DESC=$3
        printf "\n### Cross-compiling %s (%s)\n" "${IDENT}" "${DESC}"
        cross build --release --target "${TARGET}"
        cp target/"${TARGET}"/release/upstate bin/upstate
        zip -rq9 "upstate-${IDENT}.zip" README.md LICENSE install.sh bin/ etc/ man/ --exclude bin/upstate.sh
        rm bin/upstate
    }
    build-cross arm-unknown-linux-gnueabihf    linux-armv6-gnu   "Raspberry Pi 0/1"
    build-cross armv7-unknown-linux-gnueabihf  linux-armv7-gnu   "Raspberry Pi 2/3"
    build-cross aarch64-unknown-linux-gnu      linux-arm64-gnu   "ARMv8/GNU libc"
    build-cross aarch64-unknown-linux-musl     linux-arm64-musl  "ARMv8/musl"
    build-cross x86_64-unknown-linux-gnu       linux-amd64-gnu   "x86-64/GNU libc"
    build-cross x86_64-unknown-linux-musl      linux-amd64-musl  "x86-64/musl"

# Build local Docker image
build-docker:
    docker build . \
        --build-arg DATE=$DATE \
        --build-arg COMMIT=$COMMIT \
        --build-arg VERSION=$VERSION

# Build multi-architecture Docker images
build-docker-release:
    docker buildx build . \
        --build-arg DATE=$DATE \
        --build-arg COMMIT=$COMMIT \
        --build-arg VERSION=$VERSION \
        --platform linux/amd64,linux/arm64 \
        --tag ghcr.io/baraverkstad/upstate:$VERSION \
        --push

# Run code style checks
test:
    cargo clippy
    cargo fmt --check

# Run code style checks + automatic fixes
test-fix:
    cargo clippy --fix --allow-dirty --allow-staged
    cargo fmt
