FROM rust:1.65.0-alpine3.16 as builder

RUN sed -i s#https://dl-cdn.alpinelinux.org#http://mirrors.aliyun.com#g /etc/apk/repositories; \
    apk update ; \
    apk add --no-cache openssl-dev musl-dev bind-tools pkgconfig

ADD . /src
WORKDIR /src

RUN cargo build --release && strip target/release/tiebaSign

# RUN strip /usr/src/myapp/target/release/myapp

FROM alpine:3.14.3

RUN apk add --no-cache libc6-compat

COPY --from=builder /src/target/release/tiebaSign /tiebaSign

RUN echo "0 1 * * * /tiebaSign" >> /var/spool/cron/crontabs/root
RUN crond

CMD ["crond", "-f"]
