# Contributing to Grafito

Thanks for considering a contribution! Grafito is 100% original Rust under
GPL-3.0-or-later.

## Getting started

```bash
git clone https://github.com/Diez111/Grafito.git
cd Grafito
cargo run -p grafito-app --release
```

Requires Rust ≥ 1.92 and a C compiler (for the wgpu stack). The default
build needs no C math libraries; the opt-in `cas-nativo` feature needs
GMP/MPFR/MPC + FLINT.

## The rules

1. **Run the gates before opening a PR:**

   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --locked -- -D warnings
   cargo test --workspace --locked        # OOMs on 27 GB machines: prefer per-crate runs
   cargo test -p grafito-core --locked -j1   # per-crate with -j1 is the standard loop
   ```

2. **No `unwrap()`/`expect()` in production code** — the workspace denies
   them; tests may use them.
3. **Determinism is sacred**: domain maps are `BTreeMap`, saves are
   byte-identical, exercise generators are seeded (`same seed ⇒ same
   exercise`). Never introduce wall-clock or unseeded randomness into the
   brain.
4. **Every feature ships with tests**, seeded/deterministic where
   applicable; stale tests and dead code get updated or deleted, not left
   behind.
5. **New user-facing strings ship in all 6 locales** (ES/EN/PT/IT/FR/DE) in
   the same pass, with the `MSG_COUNT`/per-catalog count pins updated
   together.
6. **New animation templates** must join all three registries
   (`CANONICAL_TEMPLATES` ↔ `NATIVE_TEMPLATES` ↔ `PLANTILLAS_COMBO`) plus
   the agent mirror, and the golden hashes re-pinned — the sync test will
   tell you.
7. **Layer rules**: brain crates (L0–L2) never depend on egui/wgpu; only
   `grafito-app` knows everything. See
   [docs/architecture.md](docs/architecture.md).
8. **Credits are part of the gate**: if you port an idea or pattern from
   another project, add it to the acknowledgments table in
   [README.es.md](README.es.md) with its verified license.

## Conventional Commits

Use `feat:`, `fix:`, `refactor:`, `test:`, `docs:` prefixes.

## Reporting bugs

Open a GitHub issue with the reproduction steps (a `.grafito` document or
command sequence is ideal). Security vulnerabilities: follow
[SECURITY.md](.github/SECURITY.md) — do NOT open public issues for them.
