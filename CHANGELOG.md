# Changelog

All notable changes to this crate are documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.4] - 2026-05-05

Three-reviewer post-shipping patch on v0.6.3 (per workspace CLAUDE.md
release rule 7 — `superpowers:code-reviewer` + `rust-v2:rust-dev-v2` +
deep paper-fidelity audit against Gorard 2023, modeled on yesterday's
catgraph-magnitude v0.2.1 deep-pass pattern). Audit doc landed at
[`docs/GORARD23-AUDIT.md`](docs/GORARD23-AUDIT.md) (51 audited items —
38 implementable: 20 DONE, 8 PARTIAL, 10 DEFERRED).

This patch covers four mechanical, non-API-breaking findings. The
larger paper-fidelity findings (P-I6 `Complexity::parallel = max` →
additive, P-A1 `ZPrimeAdjunction` self-roundtrip drift, P-M2
`Direction::Stay` → `Direction::Id` rename, plus seven smaller
fidelity items) are folded into the v0.7.0 design phase and tracked
in the audit doc.

### Changed

- **`CLAUDE.md` "Trace Analysis" + "IrreducibilityTrace Trait"
  sections updated** to reflect the v0.6.3 sub-trait + blanket-impl
  shape. The pre-v0.6.3 standalone-trait body remained in the doc
  and would have led future agentic work or new contributors to
  write `impl IrreducibilityTrace for T { fn step_count(...) ... }`
  blocks that fail to compile (the trait now has no body to
  implement — the blanket already satisfies it). Updated the
  symbol table at line 161 + the directory tree at line 36 +
  added an explicit "Do NOT add new methods to `IrreducibilityTrace`"
  note. Reviewer C-I1.
- **`src/trace.rs:21-30` rustdoc clarified**: the deprecation warning
  on `IrreducibilityTrace` fires at the **bound site** (`fn f<T:
  IrreducibilityTrace>(...)`), not at method-call sites — method
  calls on `T: IrreducibilityTrace` resolve through the supertrait
  to `StepTrace::method` and emit no warning. One deprecation hit
  per bound is the intended trade-off of the sub-trait pattern over
  duplicating method signatures. The pre-patch wording "fires the
  `#[deprecated]` warning at consumer call sites" was inaccurate.
  Reviewer R-I1.
- **`src/trace.rs:30` blanket impl narrowed** from `impl<T: StepTrace
  + ?Sized> IrreducibilityTrace for T {}` to `impl<T: StepTrace>
  IrreducibilityTrace for T {}`. Upstream `StepTrace` is implicitly
  `Sized`-bound, so the `?Sized` widening was harmless (the bound
  was unsatisfiable for `!Sized` `T`) but asymmetric vs. upstream
  and slightly misleading at the API surface. Matching narrowing
  applied to `tests/shim_aliases.rs` test bounds. Reviewer R-I2.
- **`tests/shim_aliases.rs:11-15` doc comment clarified**: replaced
  the inaccurate "and vice versa via the blanket" with an explicit
  bidirectional account (the blanket gives `StepTrace ⇒
  IrreducibilityTrace`; the supertrait bound gives the reverse).
  Reviewer M-1 from code-review.
- **CHANGELOG v0.6.3 test-count notation corrected**: the line
  "200 lib + 12 integration (incl. 1 new alias-round-trip) +
  1 shim_aliases + 9 doctests" double-counted `shim_aliases.rs` and
  understated the integration-test file count. Replaced with the
  accurate "200 lib + 15 integration test files" wording. Reviewer
  C-I2.

### Added

- **`docs/GORARD23-AUDIT.md`** — paper-coverage audit against
  Gorard 2023 (arXiv:2301.04690v1). 248 lines. Mirrors the
  catgraph-magnitude `BV25-AUDIT.md` structure: front matter +
  status legend + summary table + per-section coverage + per-
  definition/theorem/example coverage + acceptance gates +
  out-of-scope + deferred + action items. Tracks 51 audited
  items: 20 DONE, 8 PARTIAL, 10 DEFERRED, 11 N/A, 2 IN-CATGRAPH.

### Deferred (to v0.7.0 design phase or v0.8.0+)

The deep paper audit surfaced two architectural drifts and one
sequence of smaller fidelity items. All are tracked in
`docs/GORARD23-AUDIT.md`:

- **P-I6 (Blocking, paper)** — `Complexity::parallel(self, other)`
  in `src/complexity.rs:85` returns `max(self, other)` (wall-clock
  time), but Gorard §3 Eqs 67-69 specify the **additive
  multicomputational** model: `f⊗g` requires *at least* `n + m`
  steps. **Decision (2026-05-05): patch to `+` in v0.7.0**, mirroring
  the precedent already in `catgraph_physics::ParallelIntervals`
  which exposes both `total_complexity` (sum, paper-faithful) and
  `max_complexity` (wall-clock) with explicit rustdoc framing.
  irreducible's `Complexity::parallel` will gain the dual-method
  surface in v0.7.0.
- **P-A1 (Architectural, paper)** — `ZPrimeAdjunction` triangle
  identities verify a self-roundtrip (`to_interval ∘ z ∘ to_interval
  = to_interval`), not the paper's `𝒯 ⇄ 𝓥ect` adjunction. The 11
  passing tests in `tests/adjunction_laws.rs` exercise Vec-equality,
  not adjointness. The paper's right adjoint Z is `𝓑ord^Riem → 𝓥ect`
  (Atiyah-Segal propagator); irreducible's Z is `DiscreteInterval →
  ComputationState` — a categorical pun. **Decision (2026-05-05):
  build a real Z stub on rebozo's `causality:topology-lattice-gauge`
  substrate in v0.8.0+**, aligning with paper §4. v0.7.0 doc-warns
  the current `ZPrimeAdjunction` to flag the framing gap.
- **P-M2 (paper)** — `Direction::Stay` should be `Direction::Id`
  (paper Eqs 1, 8 explicitly write the third δ value as `id` to
  emphasise the categorical identity-on-tape morphism). Display
  emits `S` instead of `id`. v0.7.0 (breaking enum-variant rename).
- **P-I1, P-I2, P-I5, P-M3 to P-M11** — smaller fidelity items
  (`is_irreducible` paper-Def-1 approximation; `Z'(id_X)` singleton;
  lax/strong/strict monoidal flavour; transition unit-laws; coproduct
  universality; hexagon coherence; monomorphism check on
  `RewriteSpan::l, r`; concurrent rewrite composition; rule 2506 +
  rule 3506 + Fig 12 set-substitution paper-figure fixtures). All
  v0.7.0 plan annotations.
- **P-I3, P-I4** — cobordism endofunctor `∂` (Eqs 22-25, `∂² = ∅`)
  + HALT unit object (paper §3 Eq 58: `I = HALT`) + ε coherence
  (Eq 59). Substrate-level work; v0.8.0+.

## [0.6.3] - 2026-05-05

H.1 cross-repo follow-up close-out: workspace umbrella pin bump from
catgraph `v0.12.0` to `v0.13.0` (SHA `4f8bda8`) + re-export shim
conversion of `interval`, `temporal_cospan_chain`, `trace` into thin
`pub use catgraph_physics::*;` shims, ending the cross-repo drift
between irreducible and catgraph-physics.

Per the `catgraph-physics v0.3.0` CHANGELOG cross-repo follow-up
commitment (lines 47-52). Tracked in catgraph workspace's
`.claude/plans/2026-04-28-pre-phase-6b-hardening.md` H.1.

### Changed

- **Workspace umbrella pin bumped to `v0.13.0`.** The three catgraph
  git deps (`catgraph` / `catgraph-applied` / `catgraph-physics`)
  repinned from tag `v0.12.0` to `v0.13.0`. v0.13.0 is the additive
  umbrella co-release adding `catgraph-dl v0.2.0`; the three crates
  this crate depends on are unchanged at the source level
  (catgraph v0.12.2, catgraph-applied v0.5.4, catgraph-physics v0.3.0).
- **`catgraph-surreal` pin bumped from `v0.10.1` to `v0.11.1`.**
  v0.11.1 was cut alongside this release as the +1 H.4-pattern
  crate-tag — it is the same code as v0.11.0 plus a single-line
  catgraph-deps tag-string bump from `v0.12.0` to `v0.13.0` so all
  four pins in this crate's resolved graph collapse to one catgraph
  source identity. Without that bump, the v0.13.0 umbrella + a
  `v0.10.1`-or-v0.11.0 catgraph-surreal in the same graph produce
  two cargo source identities for catgraph and the canonical H.4
  boundary-type mismatch on `Cospan<u32>` / `Span<u32>`.
- **`src/interval.rs` reduced to a re-export shim** (387 LOC →
  8 LOC) pointing at `catgraph_physics::interval::{DiscreteInterval,
  ParallelIntervals}`. No type renames; consumer paths
  `irreducible::interval::*` and `irreducible::DiscreteInterval` /
  `irreducible::ParallelIntervals` are unchanged.
- **`src/temporal_cospan_chain.rs` reduced to a re-export shim**
  (342 LOC → 23 LOC) pointing at
  `catgraph_physics::temporal_cospan_chain::{ConservationResult,
  TemporalComplex, TemporalComplexError}`.
- **`src/trace.rs` reduced to a re-export shim** (320 LOC → 30 LOC)
  pointing at `catgraph_physics::trace::{analyze_trace, detect_repeats,
  is_irreducible, RepeatDetection, StepTrace, TraceAnalysis}`.
- **Internal call sites migrated** to the new names (`StepTrace`,
  `TemporalComplexError`) in `src/machines/{turing,
  cellular_automaton,petri/*,trace,mod}.rs` and `src/functor/
  stokes_integration.rs`. Paths still go through the local shim
  modules (`crate::trace::StepTrace`); the deprecated aliases serve
  external consumers only.

### Deprecated

- `irreducible::temporal_cospan_chain::StokesError` — renamed to
  `TemporalComplexError` in `catgraph-physics` v0.3.0. The old name
  is preserved as a `#[deprecated]` type alias for the v0.6.x cycle
  and will be removed in v0.7.0. (Implemented as `pub type
  StokesError = TemporalComplexError;` rather than `pub use ... as
  StokesError;` because `#[deprecated]` on a `pub use` re-export
  does not propagate the warning to consumer call sites in current
  rustc — the type-alias form does.)
- `irreducible::trace::IrreducibilityTrace` — renamed to `StepTrace`
  in `catgraph-physics` v0.3.0. The old name is preserved as a
  `#[deprecated]` sub-trait with a blanket impl over `StepTrace`,
  so any `StepTrace` implementor automatically satisfies
  `IrreducibilityTrace` (and bound positions
  `fn f<T: IrreducibilityTrace>` still compile, but emit the
  deprecation warning).

### Test count

- Lib-internal tests dropped from 233 to 200 (the 33 unit tests
  inside the three shimmed modules now live in catgraph-physics
  v0.3.0 alongside the implementation).
- Integration tests (`tests/interval_laws.rs`, `tests/
  temporal_cospan_chain.rs`) **stay** — they are now smoke tests
  for the shim contract.
- `tests/temporal_cospan_chain.rs` migrated to use the new
  `TemporalComplexError` name in most assertions; one
  `#[allow(deprecated)]` test (`deprecated_stokes_error_alias_round_trips`)
  exercises the alias to confirm the round-trip.
- New `tests/shim_aliases.rs` (1 test) confirms the
  `IrreducibilityTrace` sub-trait + blanket-impl is accepted in
  bound position over a real `StepTrace` implementor
  (`TuringMachine::ExecutionHistory`).
- Net: **200 lib + 15 integration test files (incl. the new
  `shim_aliases.rs` file + 1 new `deprecated_stokes_error_alias_round_trips`
  test inside `temporal_cospan_chain.rs`) + 9 doctests = green, no
  deprecation warnings in the irreducible source.** Lib-internal
  count drops from 233 → 200 because the 33 unit tests inside the
  shimmed modules now live in catgraph-physics v0.3.0 alongside the
  implementation.

### Cross-repo follow-up (DELIVERED + queued)

- DELIVERED: `catgraph-physics v0.3.0` CHANGELOG lines 47-52
  promised this re-export shim conversion at irreducible v0.6.3.
- DELIVERED: `catgraph-surreal v0.11.1` (single-line tag bump,
  same SHA as v0.11.0 plus one Cargo.toml line) — needed to clear
  the dual-SHA boundary-type mismatch.
- QUEUED for `irreducible v0.7.0`: drop the three shim files
  entirely + remove the two deprecated aliases (`StokesError`,
  `IrreducibilityTrace`) + delete `tests/shim_aliases.rs`. Per
  `catgraph-physics v0.3.0` CHANGELOG: "irreducible v0.7.0 (later):
  drop the deprecated local modules entirely."

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

[Unreleased]: https://github.com/tsondru/irreducible/compare/v0.6.4...HEAD
[0.6.4]: https://github.com/tsondru/irreducible/releases/tag/v0.6.4
[0.6.3]: https://github.com/tsondru/irreducible/releases/tag/v0.6.3
[0.6.2]: https://github.com/tsondru/irreducible/releases/tag/v0.6.2
[0.6.1]: https://github.com/tsondru/irreducible/releases/tag/v0.6.1
[0.6.0]: https://github.com/tsondru/irreducible/releases/tag/v0.6.0
[0.5.0]: https://github.com/tsondru/irreducible/releases/tag/v0.5.0
[0.4.4]: https://github.com/tsondru/irreducible/releases/tag/v0.4.4
[0.4.3]: https://github.com/tsondru/irreducible/releases/tag/v0.4.3
