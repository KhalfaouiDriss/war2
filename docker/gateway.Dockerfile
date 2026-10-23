FROM rust:1.75 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY gateway/Cargo.toml gateway/Cargo.toml
COPY sim/Cargo.toml sim/Cargo.toml
RUN mkdir -p gateway/src sim/src && \
    echo "fn main() {}" > gateway/src/main.rs && \
    echo "pub fn dummy() {}" > sim/src/lib.rs && \
    cargo build --release --bin gateway 2>/dev/null || true

COPY gateway/src gateway/src
COPY sim/src sim/src
RUN cargo build --release --bin gateway

FROM rust:1.75-slim AS dev
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-watch
COPY --from=builder /app/target/release/gateway /usr/local/bin/gateway
COPY gateway/ gateway/
CMD ["cargo", "watch", "-x", "run --bin gateway"]