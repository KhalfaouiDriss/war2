.PHONY: up down logs build test lint clean wasm migrate seed rebuild

# Start all services
up:
	docker compose up -d --build

# Start with logs
up-logs:
	docker compose up --build

# Stop all services
down:
	docker compose down

# View logs
logs:
	docker compose logs -f

# Build WASM for client
wasm:
	cd sim && wasm-pack build --target web --out-dir ../client/pkg

# Run tests
test:
	cargo test --workspace

# Run tests with output
test-verbose:
	cargo test --workspace -- --nocapture

# Lint
lint:
	cargo clippy --workspace -- -D warnings
	cargo fmt --check

# Clean everything
clean:
	docker compose down -v
	rm -rf client/pkg sim/target gateway/target shard/target state-service/target

# Database migrations
migrate:
	docker compose exec postgres psql -U war2 -d war2 -f /docker-entrypoint-initdb.d/001_init.sql

# Seed test data
seed:
	docker compose exec gateway cargo run --bin seed

# Full rebuild
rebuild: clean up

# Build all
build:
	cargo build --workspace --release

# Check formatting
fmt:
	cargo fmt --all

# Run clippy
clippy:
	cargo clippy --workspace -- -D warnings

# Generate docs
doc:
	cargo doc --workspace --no-deps