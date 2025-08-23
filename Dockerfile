FROM rust:1-alpine3.22 AS builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM scratch AS final

COPY --from=builder /usr/src/app/target/release/bililive_bark /bililive_bark

ENTRYPOINT ["/bililive_bark"]
