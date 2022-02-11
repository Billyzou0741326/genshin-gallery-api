FROM rust:1.58-alpine3.14 AS builder
WORKDIR /usr/src/app
COPY . .
RUN apk add --no-cache musl-dev && cargo install --path .

FROM alpine:3.14
RUN apk add --no-cache musl-dev
COPY --from=builder /usr/local/cargo/bin/genshin-gallery-api /usr/local/bin/
ENTRYPOINT ["genshin-gallery-api"]
