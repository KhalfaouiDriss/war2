# 01: Project Foundation

**What to build:** Rust workspace with cargo configuration, basic directory structure, and CI pipeline that builds, tests, and lints.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Cargo workspace with `sim`, `gateway`, `shard`, `state-service`, `client` crates
- [ ] `rust-toolchain.toml` pinning stable version
- [ ] GitHub Actions CI: `cargo build --release`, `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`
- [ ] `.gitignore` for Rust, Node, WASM artifacts
- [ ] `Makefile` with `build`, `test`, `lint`, `clean` targets
- [ ] README with project overview and build instructions