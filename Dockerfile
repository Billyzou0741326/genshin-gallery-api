FROM rust:1.58-alpine3.14 AS dev-env
WORKDIR /usr/src/app
RUN cargo init --bin .
COPY Cargo.* ./
RUN touch src/lib.rs \
    && apk add --no-cache musl-dev binutils \
    && cargo build

FROM dev-env AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path . \
    && strip /usr/local/cargo/bin/genshin-gallery-api

FROM alpine:3.14
RUN apk add --no-cache musl-dev
COPY --from=builder /usr/local/cargo/bin/genshin-gallery-api /usr/local/bin/
ENTRYPOINT ["genshin-gallery-api"]
