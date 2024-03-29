FROM vadage/rust-base:debian-10-slim

COPY Cargo.toml /app
COPY Cargo.lock /app
COPY src /app/src
COPY migrations /app/migrations
COPY .env /app
COPY assets /app/assets

RUN cargo build --release
