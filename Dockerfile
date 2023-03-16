FROM alpine:3.17
RUN apk add --no-cache \
        bash \
        ncurses \
        procps
ADD bin/upstate.sh /usr/local/bin/upstate
ADD etc/upstate.conf /usr/local/etc/
ADD man/man1 /usr/local/man/
# VOLUME /etc/upstate.conf
# VOLUME /var/run
CMD ["/usr/local/bin/upstate"]
