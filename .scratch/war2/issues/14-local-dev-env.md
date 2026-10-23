# 14: Local Dev Environment

**What to build:** 6 services (gateway, 2 shards, redis, postgres, client-dev), docker-compose + Makefile, cargo watch + Vite HMR, SQL migrations on startup, 2 shards for partition testing.

**Blocked by:** 01-project-foundation

**Status:** ready-for-agent

- [ ] `docker-compose.yml`: gateway (8080), shard-1 (9001), shard-2 (9002), redis (6379), postgres (5432), client-dev (3000)
- [ ] Multi-stage Dockerfiles: builder (release) → dev (cargo watch / npm dev)
- [ ] Bind mounts: `./sim:/app/sim`, `./gateway:/app/gateway`, `./client:/app/client`
- [ ] `Makefile`: `up`, `down`, `logs`, `wasm`, `test`, `lint`, `clean`, `migrate`, `seed`, `rebuild`
- [ ] Postgres auto-migrations: `./migrations/*.sql` mounted to `/docker-entrypoint-initdb.d`
- [ ] Redis: appendonly, maxmemory 512mb, LRU eviction
- [ ] Environment: `.env` with `JWT_SECRET`, `POSTGRES_PASSWORD`
- [ ] Verification: `make up && sleep 10 && curl -f localhost:8080/health && curl -f localhost:3000`