DATE    := $(shell date '+%F')
VERSION := latest

all:
	@echo '🌈 Makefile commands'
	@grep -E -A 1 '^#' Makefile | awk 'BEGIN { RS = "--\n"; FS = "\n" }; { sub("#+ +", "", $$1); sub(":.*", "", $$2); printf " · make %-18s- %s\n", $$2, $$1}'
	@echo
	@echo '🚀 Release builds'
	@echo ' · make VERSION=v1.0 docker'

# Run code style checks
test:
	shellcheck -o all -e SC2249,SC2310,SC2311,SC2312 $(shell find . -name *.sh)

# Build and publish Docker image
docker:
	docker buildx build . \
		-t ghcr.io/baraverkstad/upstate:$(VERSION) \
		--platform linux/amd64,linux/arm/v6,linux/arm64 \
		--push
