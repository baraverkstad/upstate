export DATE    := $(shell date '+%F')
export COMMIT  := $(shell git rev-parse --short=8 HEAD)
export VERSION := latest

all:
	@echo '🌈 Makefile commands'
	@grep -E -A 1 '^#' Makefile | awk 'BEGIN { RS = "--\n"; FS = "\n" }; { sub("#+ +", "", $$1); sub(":.*", "", $$2); printf " · make %-18s- %s\n", $$2, $$1}'
	@echo
	@echo '🚀 Release builds'
	@echo ' · make VERSION=v1.0 build build-docker package'
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

# Build release targets
build:
	cargo build --release
	@echo "Building for arm-unknown-linux-gnueabihf (Raspberry Pi 0/1)"
	cross build --release --target arm-unknown-linux-gnueabihf
	@echo "Building for armv7-unknown-linux-gnueabihf (Raspberry Pi 2/3/4)"
	cross build --release --target armv7-unknown-linux-gnueabihf
	@echo "Building for aarch64-unknown-linux-gnu (ARMv8, GNU libc)"
	cross build --release --target aarch64-unknown-linux-gnu
	@echo "Building for aarch64-unknown-linux-musl (ARMv8, musl)"
	cross build --release --target aarch64-unknown-linux-musl
	@echo "Building for x86_64-unknown-linux-gnu (Intel x86-64, GNU libc)"
	cross build --release --target x86_64-unknown-linux-gnu
	@echo "Building for x86_64-unknown-linux-musl (Intel x86-64, musl)"
	cross build --release --target x86_64-unknown-linux-musl

# Build and publish Docker image
build-docker:
	docker buildx build . \
		--build-arg DATE=$(DATE) \
		--build-arg COMMIT=$(COMMIT) \
		--build-arg VERSION=$(VERSION) \
		-t ghcr.io/baraverkstad/upstate:$(VERSION) \
		--platform linux/amd64,linux/arm64 \
		--push

# Run code style checks
test:
	shellcheck -o all -e SC2249,SC2310,SC2311,SC2312 $(shell find . -name '*.sh')

# Package binaries for release
package: upstate-linux-armv6-gnu.zip upstate-linux-armv7-gnu.zip \
		upstate-linux-arm64-gnu.zip upstate-linux-arm64-musl.zip \
		upstate-linux-amd64-gnu.zip upstate-linux-amd64-musl.zip

upstate-linux-armv6-gnu.zip: tmp-arm-unknown-linux-gnueabihf.zip
	@mv $< $@

upstate-linux-armv7-gnu.zip: tmp-armv7-unknown-linux-gnueabihf.zip
	@mv $< $@

upstate-linux-arm64-gnu.zip: tmp-aarch64-unknown-linux-gnu.zip
	@mv $< $@

upstate-linux-arm64-musl.zip: tmp-aarch64-unknown-linux-musl.zip
	@mv $< $@

upstate-linux-amd64-gnu.zip: tmp-x86_64-unknown-linux-gnu.zip
	@mv $< $@

upstate-linux-amd64-musl.zip: tmp-x86_64-unknown-linux-musl.zip
	@mv $< $@

tmp-%.zip: target/$*/release/upstate
	ln -rs target/$*/release/upstate bin/upstate
	zip -rq9 $@ README.md LICENSE install.sh bin/ etc/ man/ -x bin/upstate.sh
	rm bin/upstate
