# Changelog

All notable changes to this crate are documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.2] - 2026-04-26

### Changed

- **catgraph deps bumped to `v0.12.0`** (Corel co-release tag). Shared across `catgraph`, `catgraph-applied`, `catgraph-physics` for Cargo source deduplication. Surface API touched by this crate is unchanged; v0.12.0 is additive (introduces `Corel<Lambda>` partition-of-targets and `Cospan::is_jointly_surjective`).
- **catgraph-surreal dep bumped to `v0.10.1`**. Picks up the engine generalization landed in v0.10.0 — every store now types against `Surreal<engine::any::Any>` instead of `Surreal<engine::local::Db>`.
- **`EvolutionPersistence::new` signature changed**: `&Surreal<Db>` → `&Surreal<Any>`. Callers must construct the connection via `surrealdb::engine::any::connect("mem://")` (or `"surrealkv://path"`, `"ws://host"`, `"http://host"`) instead of `Surreal::new::<Mem>(())`. The same code now works against in-memory, on-disk, WebSocket, and HTTP endpoints. Marked breaking because it propagates through the public `persist` feature surface.
- `surrealdb` dep bumped `3.0.4` → `3.0.5` to match catgraph-surreal's pin and dedupe the crate graph. The `kv-mem` feature is no longer enabled directly here — it is provided transitively by `catgraph-surreal/native-embedded` (default-on).

### Fixed

- Resolves the diamond-dep `Cospan<Lambda>` / `Span<Lambda>` type mismatch that surfaced when consumers pulled both this crate (catgraph v0.11.4 transitively via catgraph-surreal v0.10.0) and catgraph v0.12.0 directly. Both deps now share the same catgraph SHA, so Cargo dedupes the source.

## [0.6.1] - 2026-04-25

Internal-docs cleanup release; no code changes.

### Changed

- Moved internal design docs to `.claude/docs/` so the published crate root only carries user-facing material.
- README component index updated with the Petri-net row that landed in v0.5.0.

## [0.6.0] - 2026-04-14

Manifold-curvature + discrete exterior calculus port to `deep_causality_topology`. The Regge-curvature and DEC substrate that previously lived locally now run on the dc_topology simplicial-complex implementation; `manifold-curvature` and `dec` features compose on top of `dc-geometry`.

### Added

- `dc-geometry` feature gating the dc_topology Regge + DEC substrate (`deep_causality_topology` 0.5.1, `deep_causality_tensor` 0.4.2, `deep_causality_sparse` 0.1.7).
- `MultiwayComplex` 2D simplicial complex from a multiway graph for Stokes integration on closed/non-closed 1-forms.
- 14 dc_topology smoke + bridge tests, 14 Regge curvature tests, 14 DEC tests.

### Changed

- `manifold-curvature` and `dec` features now layer on `dc-geometry` instead of carrying their own simplicial-complex code.

## [0.5.0]

Petri-net computation model — fourth machine type alongside TM, CA, SRS/NTM.

### Added

- `PetriNetMachine`, `PetriExecutionHistory`, `PetriBuilder`, `Marking`, `BuilderError`.
- Linear (smallest-index-enabled) firing rule for deterministic traces.
- `run_multiway_reachability` for non-deterministic exploration.
- Cospan bridge: `PetriNetMachine::transition_as_cospan`.
- `examples/petri_reachability.rs` demonstrating linear + multiway + cospan paths.

## [0.4.4]

### Changed

- catgraph deps bumped to `v0.11.0` (slim baseline split) — `catgraph` (core), `catgraph-physics` (multiway, hypergraph, curvature). catgraph-surreal bumped to `v0.9.0` to match.
- `Cargo.lock` untracked; added to `.gitignore`.

## [0.4.3]

Phase 2.5 — coherence + Stokes rewrite.

### Added

- `multiway_coherence` example (non-confluent fragment failing coherence).
- `multiway_stokes` example (closed vs non-closed 1-forms, gated on `dec`).
- Symmetric monoidal coherence formalization-by-construction over multiway graphs.

[Unreleased]: https://github.com/tsondru/irreducible/compare/v0.6.2...HEAD
[0.6.2]: https://github.com/tsondru/irreducible/releases/tag/v0.6.2
[0.6.1]: https://github.com/tsondru/irreducible/releases/tag/v0.6.1
[0.6.0]: https://github.com/tsondru/irreducible/releases/tag/v0.6.0
[0.5.0]: https://github.com/tsondru/irreducible/releases/tag/v0.5.0
[0.4.4]: https://github.com/tsondru/irreducible/releases/tag/v0.4.4
[0.4.3]: https://github.com/tsondru/irreducible/releases/tag/v0.4.3
