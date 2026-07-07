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
cargo run --example gorard_demo                              # 9-part paper walkthrough
```

Rust 2024 edition, MSRV 1.90.

## Dependencies

- `catgraph` / `catgraph-applied` / `catgraph-physics` — git tag on the
  private `sustia-llc/catgraph` workspace (SSH). **ONE tag string shared by
  all three** (dual-SHA prevention; see the Cargo.toml comments).
- `deep_causality_{topology,tensor,sparse}` — currently git-rev-pinned
  pre-release (one shared rev string); swap to crates.io pins once released.
- CI authenticates the private catgraph dep with a read-only deploy key +
  `webfactory/ssh-agent`; `Cargo.lock` is committed and CI builds `--locked`.

## Feature flags

| Feature | Gates |
|---------|-------|
| *(none)* | Core library — purely computational, no I/O, no async |
| `dc-geometry` | deep_causality_topology Regge + DEC substrate |
| `manifold-curvature` | Regge deficit-angle curvature on branchial complexes |
| `dec` | Discrete exterior calculus on multiway complexes |
| `lapack` | LAPACK eigendecomposition (needs `libopenblas-dev`; not in CI) |

`persist` (SurrealDB evolution-trace storage) was **removed** pending the
catgraph-surreal reboot — pre-reboot catgraph-surreal pins the retired
catgraph lineage (type-identity mismatch). Restore path: issue #15.

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

Work is tracked as GitHub issues; TODO.md is retired.
