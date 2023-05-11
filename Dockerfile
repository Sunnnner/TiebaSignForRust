FROM rust:alpine3.17 as builder

RUN sed -i s#https://dl-cdn.alpinelinux.org#http://mirrors.aliyun.com#g /etc/apk/repositories; \
    apk update ; \
    apk add --no-cache openssl-dev musl-dev bind-tools pkgconfig


WORKDIR /usr/src/myapp
ADD . .

RUN cargo build --release
# RUN strip /usr/src/myapp/target/release/myapp

FROM alpine:3.14.3
COPY --from=builder /usr/src/myapp/target/release/myapp /myapp

RUN echo "0 1 * * * /myapp" >> /var/spool/cron/crontabs/root
RUN crond

CMD ["crond", "-f"]
