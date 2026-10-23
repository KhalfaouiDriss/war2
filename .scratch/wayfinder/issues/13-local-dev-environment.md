# Ticket: Local Dev Environment (Docker Compose + Makefile) — RESOLVED

## Decision

**Services** (5 containers):
| Service | Image | Ports | Purpose |
|---------|-------|-------|---------|
| `gateway` | war2-gateway:dev | 8080:8080 | TLS termination, auth, WS proxy |
| `shard-1` | war2-shard:dev | 9001:9001 | Simulation shard 1 |
| `shard-2` | war2-shard:dev | 9002:9002 | Simulation shard 2 |
| `redis` | redis:7-alpine | 6379:6379 | Streams, cache, sessions |
| `postgres` | postgres:15-alpine | 5432:5432 | Persistent storage |
| `client-dev` | war2-client:dev | 3000:3000 | Vite dev server + WASM |

**Network**: `war2-net` (bridge), all services resolve by name

**Volumes**:
- `postgres-data` → `/var/lib/postgresql/data`
- `redis-data` → `/data`
- `./sim:/app/sim` (bind mount for cargo watch)
- `./client:/app/client` (bind mount for Vite HMR)

---

## Docker Compose (`docker-compose.yml`)

```yaml
version: '3.8'

services:
  gateway:
    build:
      context: .
      dockerfile: docker/gateway.Dockerfile
      target: dev
    ports: ["8080:8080"]
    environment:
      - RUST_LOG=debug
      - JWT_SECRET=${JWT_SECRET}
      - REDIS_URL=redis://redis:6379
      - POSTGRES_URL=postgres://war2:war2@postgres:5432/war2
      - SHARD_ADDRESSES=shard-1:9001,shard-2:9002
    depends_on: [redis, postgres, shard-1, shard-2]
    volumes:
      - ./gateway:/app/gateway
    command: cargo watch -x "run --bin gateway"

  shard-1:
    build:
      context: .
      dockerfile: docker/shard.Dockerfile
      target: dev
    ports: ["9001:9001"]
    environment:
      - RUST_LOG=debug
      - SHARD_ID=shard-1
      - REDIS_URL=redis://redis:6379
      - POSTGRES_URL=postgres://war2:war2@postgres:5432/war2
      - REGION_MIN_X=0
      - REGION_MAX_X=25000
      - REGION_MIN_Y=0
      - REGION_MAX_Y=50000
    depends_on: [redis, postgres]
    volumes:
      - ./sim:/app/sim
    command: cargo watch -x "run --bin shard"

  shard-2:
    build:
      context: .
      dockerfile: docker/shard.Dockerfile
      target: dev
    ports: ["9002:9002"]
    environment:
      - RUST_LOG=debug
      - SHARD_ID=shard-2
      - REDIS_URL=redis://redis:6379
      - POSTGRES_URL=postgres://war2:war2@postgres:5432/war2
      - REGION_MIN_X=25000
      - REGION_MAX_X=50000
      - REGION_MIN_Y=0
      - REGION_MAX_Y=50000
    depends_on: [redis, postgres]
    volumes:
      - ./sim:/app/sim
    command: cargo watch -x "run --bin shard"

  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]
    volumes: [redis-data:/data]
    command: redis-server --appendonly yes --maxmemory 512mb --maxmemory-policy allkeys-lru

  postgres:
    image: postgres:15-alpine
    ports: ["5432:5432"]
    environment:
      - POSTGRES_DB=war2
      - POSTGRES_USER=war2
      - POSTGRES_PASSWORD=war2
    volumes:
      - postgres-data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d  # auto-run on init

  client-dev:
    build:
      context: .
      dockerfile: docker/client.Dockerfile
      target: dev
    ports: ["3000:3000"]
    environment:
      - VITE_GATEWAY_URL=ws://localhost:8080
    volumes:
      - ./client:/app/client
      - /app/client/node_modules  # anonymous volume for node_modules
    command: npm run dev

volumes:
  postgres-data:
  redis-data:

networks:
  default:
    name: war2-net
```

---

## Dockerfiles (Multi-stage)

### `docker/gateway.Dockerfile`
```dockerfile
FROM rust:1.75 AS builder
WORKDIR /app
COPY gateway/ .
RUN cargo build --release --bin gateway

FROM rust:1.75-slim AS dev
WORKDIR /app
RUN cargo install cargo-watch
COPY --from=builder /app/target/release/gateway /usr/local/bin/gateway
COPY gateway/ .
CMD ["cargo", "watch", "-x", "run --bin gateway"]
```

### `docker/shard.Dockerfile`
```dockerfile
FROM rust:1.75 AS builder
WORKDIR /app
COPY sim/ .
RUN cargo build --release --bin shard

FROM rust:1.75-slim AS dev
WORKDIR /app
RUN cargo install cargo-watch
COPY --from=builder /app/target/release/shard /usr/local/bin/shard
COPY sim/ .
CMD ["cargo", "watch", "-x", "run --bin shard"]
```

### `docker/client.Dockerfile`
```dockerfile
FROM node:20-alpine AS dev
WORKDIR /app
COPY client/package*.json ./
RUN npm ci
COPY client/ .
EXPOSE 3000
CMD ["npm", "run", "dev"]
```

---

## Makefile

```makefile
.PHONY: up down logs build test clean wasm lint

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
	docker compose exec gateway cargo test
	docker compose exec shard-1 cargo test
	docker compose exec shard-2 cargo test
	cd client && npm test

# Lint
lint:
	docker compose exec gateway cargo clippy -- -D warnings
	docker compose exec shard-1 cargo clippy -- -D warnings
	cd client && npm run lint

# Clean everything
clean:
	docker compose down -v
	rm -rf client/pkg sim/target

# Database migrations
migrate:
	docker compose exec postgres psql -U war2 -d war2 -f /docker-entrypoint-initdb.d/001_init.sql

# Seed test data
seed:
	docker compose exec gateway cargo run --bin seed

# Full rebuild
rebuild: clean up
```

---

## Hot Reload

| Component | Mechanism |
|-----------|-----------|
| **Gateway/Shard (Rust)** | `cargo watch` in container → rebuilds on `.rs` change, restarts binary |
| **Client (TypeScript)** | Vite HMR → instant module replacement in browser |
| **WASM** | `wasm-pack build` via `make wasm` → Vite picks up new `.wasm` + `.js` |
| **Database** | Migrations auto-run on `postgres` container init (idempotent) |

**Workflow**:
```bash
make up          # starts everything
# edit sim/src/lib.rs → cargo watch rebuilds → shard restarts
# edit client/src/App.tsx → Vite HMR updates browser
# make wasm        # rebuild WASM after sim changes
```

---

## Environment Variables (`.env`)

```bash
JWT_SECRET=your-256-bit-secret-here
RUST_LOG=debug
POSTGRES_PASSWORD=war2
```

---

## Verification

```bash
# Check all services healthy
make up && sleep 10 && curl -f http://localhost:8080/health && curl -f http://localhost:3000

# Run integration tests
make test

# View logs
make logs
```

---

## Blocking → Unblocked

- ✅ Completes all 13 tickets
- ✅ Map fully specified → ready for `/to-spec`