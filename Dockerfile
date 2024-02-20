FROM rust:1-alpine-3.19 AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && cargo build --release && rm -f src/main.rs

COPY src ./src
RUN CARGO_BUILD_INCREMENTAL=true cargo build --release

FROM alpine:3.19 AS runtime
COPY --from=builder /app/target/release/ecci /usr/local/bin/ecci

FROM runtime AS action
COPY entrypoint.sh /entrypoint.sh

ENTRYPOINT [ "/entrypoint.sh" ]
