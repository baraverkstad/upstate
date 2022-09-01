DATE    := $(shell date '+%F')
VERSION := latest

test:
	shellcheck -o all -e SC2249,SC2310,SC2311,SC2312 $(shell find . -name *.sh)

docker:
	docker buildx build . \
		-t baraverkstad/upstate:$(VERSION) \
		--platform linux/amd64,linux/arm64 \
		--push
