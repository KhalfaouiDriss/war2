.PHONY: help up down logs build test lint clean wasm migrate seed rebuild fmt clippy doc

# Default target
help:
	@echo "War2 - Persistent World RTS/MMO"
	@echo ""
	@echo "Available targets:"
	@echo "  make up          - Start all services (detached)"
	@echo "  make up-logs     - Start all services with logs"
	@echo "  make down        - Stop all services"
	@echo "  make logs        - View logs"
	@echo "  make build       - Build all services (release)"
	@echo "  make test        - Run all tests"
	@echo "  make lint        - Run clippy and fmt check"
	@echo "  make wasm        - Build WASM for client"
	@echo "  make migrate     - Run database migrations"
	@echo "  make clean       - Stop services and remove all build artifacts"
	@echo "  make rebuild     - Clean and rebuild everything"
	@echo "  make fmt         - Format code"
	@echo "  make clippy      - Run clippy"
	@echo "  make doc         - Generate documentation"

# Start all services (detached)
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