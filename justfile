export DATE    := `date '+%F'`
export COMMIT  := `GIT_CONFIG_GLOBAL=/dev/null git rev-parse --short=8 HEAD`
export VERSION := "latest"
export DOCKER  := `command -v container || command -v docker || echo 'docker'`

# List build commands and descriptions
help:
    @just --list --unsorted --list-heading $'🌈 Available commands\n' --list-prefix ' · '
    @echo
    @echo '🚀 Release builds'
    @echo ' · just VERSION=v1.0 build-cross'
    @echo
    @echo '💡 Related commands'
    @echo ' · cargo update -v        - Update dependencies to latest version'
    @echo ' · rustup update --no-self-update'
    @echo '                          - Update Rust toolchain to latest version'

# Cleanup intermediary files
clean:
    rm -rf target upstate-*.zip

# Run debug version
run:
    cargo run

# Build local binary
build:
    cargo build --release

# Build multi-architecture binaries
build-cross:
    #!/usr/bin/env bash
    set -euo pipefail
    "$DOCKER" build --pull -f cross-build/Dockerfile \
        --build-arg DATE="$DATE" \
        --build-arg COMMIT="$COMMIT" \
        --build-arg VERSION="$VERSION" \
        -t upstate-cross-build .
    "$DOCKER" run --rm -v "$PWD":/workspace upstate-cross-build
    pkgbuild() {
        local TARGET=$1 IDENT=$2 DESC=$3
        printf "\n### Packaging %s (%s)\n" "$IDENT" "$DESC"
        cp "target/${TARGET}/release/upstate" bin/upstate
        zip -rq9 "upstate-${IDENT}.zip" README.md LICENSE install.sh bin/ etc/ man/ --exclude bin/upstate.sh
        rm bin/upstate
    }
    pkgbuild arm-unknown-linux-gnueabihf   linux-armv6-gnu  "Raspberry Pi 0/1"
    pkgbuild armv7-unknown-linux-gnueabihf linux-armv7-gnu  "Raspberry Pi 2/3"
    pkgbuild aarch64-unknown-linux-gnu     linux-arm64-gnu  "ARMv8/GNU libc"
    pkgbuild aarch64-unknown-linux-musl    linux-arm64-musl "ARMv8/musl"
    pkgbuild x86_64-unknown-linux-gnu      linux-amd64-gnu  "x86-64/GNU libc"
    pkgbuild x86_64-unknown-linux-musl     linux-amd64-musl "x86-64/musl"

# Build Docker image
build-docker:
    "$DOCKER" build --pull . \
        --build-arg DATE="$DATE" \
        --build-arg COMMIT="$COMMIT" \
        --build-arg VERSION="$VERSION"

# Build multi-architecture Docker images
build-docker-release:
    docker buildx build --pull . \
        --build-arg DATE="$DATE" \
        --build-arg COMMIT="$COMMIT" \
        --build-arg VERSION="$VERSION" \
        --platform linux/amd64,linux/arm64 \
        --tag ghcr.io/baraverkstad/upstate:$VERSION \
        --push

# Run code test suite
test:
    cargo clippy --all-targets
    cargo fmt --check
    cargo test

test-fix:
    cargo clippy --fix --allow-dirty --allow-staged
    cargo fmt

# Check for outdated dependencies and toolchain
outdated:
    @echo --== rust toolchain ==--
    @echo "current: $(rustc --version)"
    @echo "latest:  $(curl -sf https://raw.githubusercontent.com/rust-lang/rust/master/RELEASES.md | head -1)"
    @echo
    @echo --== rust dependencies ==--
    cargo outdated
