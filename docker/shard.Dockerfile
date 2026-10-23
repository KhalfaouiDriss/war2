FROM rust:1.75 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY shard/Cargo.toml shard/Cargo.toml
COPY sim/Cargo.toml sim/Cargo.toml
RUN mkdir -p shard/src sim/src && \
    echo "fn main() {}" > shard/src/main.rs && \
    echo "pub fn dummy() {}" > sim/src/lib.rs && \
    cargo build --release --bin shard 2>/dev/null || true

COPY shard/src shard/src
COPY sim/src sim/src
RUN cargo build --release --bin shard

FROM rust:1.75-slim AS dev
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-watch
COPY --from=builder /app/target/release/shard /usr/local/bin/shard
COPY shard/ shard/
COPY sim/ sim/
CMD ["cargo", "watch", "-x", "run --bin shard"]