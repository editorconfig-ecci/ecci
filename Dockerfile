FROM rust:1-alpine AS builder

WORKDIR /app
RUN apk add --no-cache editorconfig-dev=0.12.11-r0

# Fetching is keyed only by the manifests and lockfile. Keep it before the
# source copy so a source-only change can reuse this layer from BuildKit's cache.
COPY Cargo.toml Cargo.lock ./
COPY crates/ecci/Cargo.toml crates/ecci/Cargo.toml
COPY crates/ecci-checker/Cargo.toml crates/ecci-checker/Cargo.toml
COPY crates/ecci-editorconfig/Cargo.toml crates/ecci-editorconfig/Cargo.toml
COPY crates/ecci-report/Cargo.toml crates/ecci-report/Cargo.toml
# Cargo validates every workspace member before fetching. These placeholder
# targets make the manifest-only workspace valid without copying real sources.
RUN mkdir -p crates/ecci/src crates/ecci-checker/src crates/ecci-editorconfig/src crates/ecci-report/src \
    && touch crates/ecci/src/main.rs crates/ecci-checker/src/lib.rs crates/ecci-editorconfig/src/lib.rs crates/ecci-report/src/lib.rs
RUN cargo fetch --locked

COPY crates ./crates
RUN RUSTFLAGS="-C target-feature=-crt-static -L native=/usr/lib" cargo build --locked --release --package ecci

FROM alpine:3.19 AS runtime
RUN apk add --no-cache libeditorconfig=0.12.6-r2 libgcc=13.2.1_git20231014-r0
COPY --from=builder /app/target/release/ecci /usr/local/bin/ecci

FROM runtime AS action
COPY entrypoint.sh /entrypoint.sh

ENTRYPOINT [ "/entrypoint.sh" ]
