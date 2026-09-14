# irreducible

Computational irreducibility as functoriality in Rust — Jonathan Gorard,
*A Functorial Perspective on (Multi)computational Irreducibility*
(arXiv:2301.04690). A computation is irreducible iff the functor
Z′: T → B (computations → cobordism intervals) preserves composition.

Example consumer of the [catgraph](https://github.com/sustia-llc/catgraph)
core + physics layer. Machine zoo: Turing machines, elementary CA, string
rewriting, nondeterministic TMs, hypergraph DPO rewriting, Petri nets.

## Build & test

```sh
cargo build --workspace
cargo test  --workspace                                      # every change: green before merge
cargo test  --workspace --features manifold-curvature,dec    # feature leg (CI-gated)
cargo clippy --workspace --all-targets -- -D warnings        # the CI gate
cargo clippy --workspace --all-targets -- -W clippy::pedantic  # advisory local pass
cargo fmt   --all --check
cargo run --example gorard_demo                              # 12-part paper walkthrough
```

Rust 2024 edition, MSRV 1.98 (measured cross-feature maximum; the default
feature set builds on 1.90, `persist` 1.94, `dc-geometry` 1.98).

## Dependencies

- `catgraph` / `catgraph-applied` / `catgraph-physics` — git tag on the
  public `sustia-llc/catgraph` workspace (HTTPS). **ONE tag string shared by
  all three** (dual-SHA prevention; see the Cargo.toml comments).
- `catgraph-surreal` — git tag on the public `sustia-llc/catgraph-surreal`
  repo (HTTPS), optional behind `persist`. It resolves catgraph from the
  same git URL, so its catgraph tag and the three above move together.
- `deep_causality_{topology,tensor,linear}` — crates.io pins
  (`0.10` / `0.5` / `0.1`), optional behind `dc-geometry`.
- All git deps fetch over anonymous HTTPS; `Cargo.lock` is committed and CI
  builds `--locked`.

## Feature flags

| Feature | Gates |
|---------|-------|
| *(none)* | Core library — purely computational, no I/O, no async |
| `dc-geometry` | deep_causality_topology Regge + DEC substrate |
| `manifold-curvature` | Regge deficit-angle curvature on branchial complexes |
| `dec` | Discrete exterior calculus on multiway complexes |
| `lapack` | LAPACK eigendecomposition (needs `libopenblas-dev`; not in CI) |
| `persist` | SurrealDB cospan-chain storage; selects no engine |
| `persist-mem` | `persist` on the in-memory engine (endpoint `memory`) |
| `persist-rocksdb` | `persist` on the RocksDB engine (endpoint `rocksdb://path`) |

## Rules

1. **The paper is the spec.** Gorard 2023 anchors the API; the mapping lives
   in `docs/GORARD23-AUDIT.md`.
2. **Every change is green** — `cargo test` + clippy `-D warnings` + fmt
   before merge; work lands via branch + PR on CI.
3. **Never a path dep in committed state.** The commented `[patch]` block is
   for local lock-step only; all deps sharing a git URL move on one
   tag/rev string.
4. **No `.unwrap()` in production code** — `?` → thiserror, or
   `.expect("invariant: WHY")`; `.unwrap()` only in `#[cfg(test)]` and doc
   examples.
5. **Upstream gaps are upstream issues.** If a catgraph surface is missing or
   wrong, file it on catgraph — never work around it downstream.

Work is tracked as GitHub issues.
