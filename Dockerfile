FROM alpine:3.23 AS build
ARG DATE
ARG COMMIT
ARG VERSION
RUN apk --no-cache upgrade && \
    apk --no-cache add cargo && \
    mkdir /build
ADD Cargo.toml Cargo.lock /build/
ADD src /build/src/
ENV DATE=${DATE} COMMIT=${COMMIT} VERSION=${VERSION}
RUN cd /build && \
    cargo build --release

FROM alpine:3.23
RUN apk --no-cache upgrade && \
    apk --no-cache add libgcc
COPY --from=build /build/target/release/upstate /usr/local/bin/upstate
CMD ["/usr/local/bin/upstate"]
