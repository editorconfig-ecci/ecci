FROM rust:1-alpine AS builder

WORKDIR /app
RUN apk add --no-cache editorconfig-dev
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN RUSTFLAGS="-C target-feature=-crt-static -L native=/usr/lib" cargo build --locked --release --package ecci

FROM alpine:3.19 AS runtime
RUN apk add --no-cache libeditorconfig libgcc
COPY --from=builder /app/target/release/ecci /usr/local/bin/ecci

FROM runtime AS action
COPY entrypoint.sh /entrypoint.sh

ENTRYPOINT [ "/entrypoint.sh" ]
