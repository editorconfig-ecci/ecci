FROM rust:1-alpine AS builder

WORKDIR /app
RUN apk add --no-cache editorconfig-dev=0.12.11-r0
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN RUSTFLAGS="-C target-feature=-crt-static -L native=/usr/lib" cargo build --locked --release --package ecci

FROM alpine:3.19 AS runtime
RUN apk add --no-cache libeditorconfig=0.12.6-r2 libgcc=13.2.1_git20231014-r0
COPY --from=builder /app/target/release/ecci /usr/local/bin/ecci

FROM runtime AS action
COPY entrypoint.sh /entrypoint.sh

ENTRYPOINT [ "/entrypoint.sh" ]
