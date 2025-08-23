FROM rust:1-alpine3.22 AS builder

WORKDIR /usr/src/app

RUN apk add --no-cache build-base openssl-dev openssl-libs-static

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM alpine:3.22 AS final

COPY --from=builder /usr/src/app/target/release/bililive_bark /bililive_bark

ENTRYPOINT ["/bililive_bark"]
