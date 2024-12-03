FROM rust:1.83.0-alpine3.20 AS builder

RUN apk add musl-dev \
    && rustup override set nightly

WORKDIR /app

COPY Cargo.toml .
COPY Cargo.lock .
COPY src ./src
COPY migrations ./migrations
COPY .env .
COPY assets ./assets

RUN cargo build --release

FROM alpine:3.20.3 AS base

WORKDIR /app

COPY --from=builder /app/target/release/mvn-rs ./maven

ENTRYPOINT ["/app/maven"]
