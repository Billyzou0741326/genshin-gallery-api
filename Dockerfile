FROM museaqours/genshin-gallery-api-buildbase:0.1.0 AS builder
WORKDIR /usr/src/app
COPY . .
RUN CARGO_TARGET_DIR=target cargo install --path . \
    && strip /usr/local/cargo/bin/genshin-gallery-api

FROM alpine:3.14
RUN apk add --no-cache musl-dev
COPY --from=builder /usr/local/cargo/bin/genshin-gallery-api /usr/local/bin/
ENTRYPOINT ["genshin-gallery-api"]
