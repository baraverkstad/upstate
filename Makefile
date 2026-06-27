export DATE    := $(shell date '+%F')
export COMMIT  := $(shell GIT_CONFIG_GLOBAL=/dev/null git rev-parse --short=8 HEAD)
export VERSION := latest
export DOCKER := $(shell command -v container || command -v docker || echo 'docker')

define PKGBUILD
	@printf "\n### Packaging %s (%s)\n" "$(strip $2)" "$(strip $3)"
	cp target/$(strip $1)/release/upstate bin/upstate
	zip -rq9 upstate-$(strip $2).zip README.md LICENSE install.sh bin/ etc/ man/ --exclude bin/upstate.sh
	rm bin/upstate
endef

all:
	@echo '🌈 Makefile commands'
	@grep -E -A 1 '^#' Makefile | awk 'BEGIN { RS = "--\n"; FS = "\n" }; { sub("#+ +", "", $$1); sub(":.*", "", $$2); printf " · make %-18s- %s\n", $$2, $$1}'
	@echo
	@echo '🚀 Release builds'
	@echo ' · make VERSION=v1.0 build-cross'
	@echo
	@echo '💡 Related commands'
	@echo ' · rustup update          - Update Rust toolchain to latest version'
	@echo ' · cargo update -v        - Update dependencies to latest version'

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
	$(DOCKER) build --pull -f cross-build/Dockerfile \
		--build-arg DATE=$(DATE) \
		--build-arg COMMIT=$(COMMIT) \
		--build-arg VERSION=$(VERSION) \
		-t upstate-cross-build .
	$(DOCKER) run --rm -v $(PWD):/workspace upstate-cross-build
	$(call PKGBUILD, arm-unknown-linux-gnueabihf,   linux-armv6-gnu,  Raspberry Pi 0/1)
	$(call PKGBUILD, armv7-unknown-linux-gnueabihf, linux-armv7-gnu,  Raspberry Pi 2/3)
	$(call PKGBUILD, aarch64-unknown-linux-gnu,     linux-arm64-gnu,  ARMv8/GNU libc)
	$(call PKGBUILD, aarch64-unknown-linux-musl,    linux-arm64-musl, ARMv8/musl)
	$(call PKGBUILD, x86_64-unknown-linux-gnu,      linux-amd64-gnu,  x86-64/GNU libc)
	$(call PKGBUILD, x86_64-unknown-linux-musl,     linux-amd64-musl, x86-64/musl)

# Build Docker image
build-docker:
	$(DOCKER) build --pull . \
		--build-arg DATE=$(DATE) \
		--build-arg COMMIT=$(COMMIT) \
		--build-arg VERSION=$(VERSION)

build-docker-release:
	docker buildx build --pull . \
		--build-arg DATE=$(DATE) \
		--build-arg COMMIT=$(COMMIT) \
		--build-arg VERSION=$(VERSION) \
		--platform linux/amd64,linux/arm64 \
		--tag ghcr.io/baraverkstad/upstate:$(VERSION) \
		--push

# Run code style checks
test:
	cargo clippy
	cargo fmt --check

test-fix:
	cargo clippy --fix --allow-dirty --allow-staged
	cargo fmt

# Check for outdated dependencies and toolchain
outdated:
	@echo --== rust toolchain ==--
	@echo "current: $$(rustc --version)"
	@echo "latest:  $$(curl -sf https://raw.githubusercontent.com/rust-lang/rust/master/RELEASES.md | head -1)"
	@echo
	@echo --== rust dependencies ==--
	cargo outdated
