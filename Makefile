export DATE    := $(shell date '+%F')
export COMMIT  := $(shell git rev-parse --short=8 HEAD)
export VERSION := latest

define CROSSBUILD
	@printf "\n### Cross-compiling %s (%s)\n" "$(strip $2)" "$(strip $3)"
	cross build --release --target $(strip $1)
	cp target/$(strip $1)/release/upstate bin/upstate
	zip -rq9 upstate-$(strip $2).zip README.md LICENSE install.sh bin/ etc/ man/ --exclude bin/upstate.sh
	rm bin/upstate
endef

all:
	@echo '🌈 Makefile commands'
	@grep -E -A 1 '^#' Makefile | awk 'BEGIN { RS = "--\n"; FS = "\n" }; { sub("#+ +", "", $$1); sub(":.*", "", $$2); printf " · make %-18s- %s\n", $$2, $$1}'
	@echo
	@echo '🚀 Release builds'
	@echo ' · make VERSION=v1.0 build-release'
	@echo
	@echo '💡 Related commands'
	@echo ' · cargo fmt              - Format all Rust source code'
	@echo ' · cargo update           - Update dependencies to latest version'

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
	$(call CROSSBUILD, arm-unknown-linux-gnueabihf,   linux-armv6-gnu,  Raspberry Pi 0/1)
	$(call CROSSBUILD, armv7-unknown-linux-gnueabihf, linux-armv7-gnu,  Raspberry Pi 2/3/4)
	$(call CROSSBUILD, aarch64-unknown-linux-gnu,     linux-arm64-gnu,  ARMv8/GNU libc)
	$(call CROSSBUILD, aarch64-unknown-linux-musl,    linux-arm64-musl, ARMv8/musl)
	$(call CROSSBUILD, x86_64-unknown-linux-gnu,      linux-amd64-gnu,  x86-64/GNU libc)
	$(call CROSSBUILD, x86_64-unknown-linux-musl,     linux-amd64-musl, x86-64/musl)

# Build Docker image
build-docker:
	docker build . \
		--build-arg DATE=$(DATE) \
		--build-arg COMMIT=$(COMMIT) \
		--build-arg VERSION=$(VERSION)

build-docker-release:
	docker buildx build
		--build-arg DATE=$(DATE) \
		--build-arg COMMIT=$(COMMIT) \
		--build-arg VERSION=$(VERSION) \
		--platform linux/amd64,linux/arm64 \
		--tag ghcr.io/baraverkstad/upstate:$(VERSION) \
		--push

# Run code style checks
test:
	shellcheck -o all -e SC2249,SC2310,SC2311,SC2312 $(shell find . -name '*.sh')
