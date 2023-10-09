FROM alpine:3.18 as build
ARG DATE
ARG COMMIT
ARG VERSION
RUN apk add --no-cache \
        cargo && \
    mkdir /build
ADD Cargo.toml Cargo.lock /build/
ADD src /build/src/
ENV DATE=${DATE} COMMIT=${COMMIT} VERSION=${VERSION}
RUN cd /build && \
    cargo build --release

FROM alpine:3.18
RUN apk add --no-cache \
        libgcc
COPY --from=build /build/target/release/upstate /usr/local/bin/upstate
ADD etc/upstate.conf /usr/local/etc/
ADD man/man1 /usr/local/man/
# VOLUME /etc/upstate.conf
# VOLUME /var/run
CMD ["/usr/local/bin/upstate"]
