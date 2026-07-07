# Changelog

All notable changes to this crate are documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Reboot alignment: irreducible becomes the example consumer of the
rebooted catgraph's core + physics layer.

### Changed

- **catgraph dependencies repointed** from the retired
  `tsondru/catgraph` lineage (`v0.13.0`) to the rebooted
  `sustia-llc/catgraph` (`v0.2.0`, SSH; single tag string shared by
  `catgraph` / `catgraph-applied` / `catgraph-physics` for dual-SHA
  prevention). No API changes were required in library code — the
  reboot's consumer surface is drop-in for this crate.
- **deep_causality git-pinned pre-release**: `deep_causality_topology`
  0.6.1 / `deep_causality_tensor` 0.4.4 / `deep_causality_sparse`
  0.2.0 via a single shared git rev (versions not yet on crates.io;
  swap back to crates.io pins once released). Adapted to the 0.6
  `Manifold` API: type parameters are now `Manifold<K: ChainComplex, F>`,
  Hodge ⋆ availability is validated eagerly at `with_metric`, and the
  differential operators (`codifferential`, `laplacian`) source
  Hodge ⋆ from the Regge metric (a metric-less `Manifold` panics).
  Test-only fixes in `tests/dc_topology_smoke.rs` and
  `tests/multiway_stokes.rs`.
- **`persist` feature quarantined**: `catgraph-surreal` still pins the
  pre-reboot catgraph lineage; mixing it with reboot catgraph is a
  type-identity mismatch. The dep stays declared (now SSH) but the
  feature must not be enabled in CI or tests until catgraph-surreal
  is realigned.
- **CI introduced** (`.github/workflows/ci.yml`): fmt, clippy
  `-D warnings`, tests on default features plus a
  `manifold-curvature,dec` feature leg; private git-dep auth via a
  read-only deploy key + `webfactory/ssh-agent`.
- **`Cargo.lock` is now committed** (removed from `.gitignore`): CI
  builds `--locked`, which avoids fetching the quarantined private
  `catgraph-surreal` manifest during lock generation.

## [0.6.5] - 2026-05-05

Top-up post-shipping reviewer pass on v0.6.3 + v0.6.4. Workspace
CLAUDE.md release rule 7 specifies the canonical triumvirate
`superpowers:code-reviewer` + `rust-v2:rust-dev-v2` +
`rust-v2:rust-practical`; the prior pass had two silent substitutions
(`feature-dev:code-reviewer` instead of `superpowers:code-reviewer` —
not the same agent — and `general-purpose` deep-paper audit
substituted for `rust-v2:rust-practical` instead of being added
alongside it). The user re-dispatched the missing two reviewers; this
v0.6.5 patch carries their findings. Workspace CLAUDE.md release
rule 7 was simultaneously tightened to lock the canonical triumvirate
and forbid silent substitution.

Four mechanical findings, all doc / Cargo.toml-comment level. No
code or behavioral change.

### Changed

- **`CLAUDE.md` `[workspace.dependencies]` block updated** from the
  pre-v0.6.3 pin strings (`v0.12.0` for the catgraph triple,
  `v0.10.1` for catgraph-surreal) to the live pins (`v0.13.0` /
  `v0.11.1`). The previous pre-v0.6.3 strings would have led
  agentic work or new contributors writing follow-up cross-repo
  deps to use the wrong pin. Also updated the "Fong-Spivak
  Categorical Infrastructure" section header + the integration-status
  paragraph (line 214) + the deferred-work table (line 494) to
  reference the umbrella convention rather than a stale
  point-release tag. Reviewers N-I1 + I-2.
- **`Cargo.toml` commented `[patch.*]` block completed**. The block
  listed `catgraph` and `catgraph-physics` as the path-patch targets
  but **omitted `catgraph-applied`** even though it is a
  non-optional `[dependencies]` entry pulling from the same git URL.
  Anyone uncommenting the block for cross-repo dev would
  accidentally split the catgraph source identity (path-patched
  `catgraph` + git-tagged `catgraph-applied` from the same URL =
  H.4 dual-SHA). Added the missing line + an explicit
  `IMPORTANT (H.4 dual-SHA prevention)` comment block citing the
  workspace H.4 precedent. Reviewer rust-v2:rust-practical I-1.
- **`docs/GORARD23-AUDIT.md` header version updated** from
  `v0.6.3 / e099cb9` to a version-neutral framing covering the doc's
  ongoing maintenance. The audit doc was *committed* at v0.6.4 SHA
  `6b3d656` but the header still claimed v0.6.3, creating a
  release-marker drift. Reviewer N-I2.
- **`docs/GORARD23-AUDIT.md` Acceptance Gate 9 (NTM
  subadditive-parallel-composition test) cross-references action
  item I-6** with the v0.7.0 ratified-decision link. Previously
  Gate 9 said "Blocking" with no scheduled-fix pointer. Reviewer N-M1.
- **`src/lib.rs:12` rustdoc** clarified that `Complexity` +
  `ComputationState` remain local types, while `DiscreteInterval`,
  `ParallelIntervals`, `TemporalComplex`, and `StepTrace` are
  re-exported from `catgraph_physics` via the v0.6.3 shim modules
  (which v0.7.0 will drop). Previously the line said all three were
  "re-exported from catgraph", which was true pre-shim but is now
  misleading. Reviewer N-M2.

### Architectural follow-up (v0.7.0 plan annotation)

- **A-1 (rust-v2:rust-practical)**: v0.7.0's release procedure
  should add an explicit pre-tagging gate "verify catgraph-surreal's
  umbrella tag matches the irreducible catgraph pin" — when v0.7.0
  drops the shim files, if catgraph-surreal has not yet bumped to
  the umbrella tag of that release, the H.4 dual-SHA pattern can
  recur. Tracked in the v0.7.0 design phase.

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
  steps. **Decision (2026-05-05): patch to `+` in v0.7.0** —
  paper-faithful **single model**, breaking change for any consumer
  relying on the wall-clock semantics. `Complexity` is a single
  scalar representing the abstract complexity of `f⊗g`; the
  paper-faithful answer is the sum. (Note: `catgraph_physics::
  ParallelIntervals` exposes both `total_complexity` (sum) and
  `max_complexity` (max) as separate methods because it models
  *multi-branch* parallel computations where wall-clock is a
  meaningfully separate quantity. The dual surface is right for
  `ParallelIntervals` but not for `Complexity`.)
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
commitment.

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

- Moved internal design docs out of the published crate root so it only carries user-facing material.
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

[Unreleased]: https://github.com/tsondru/irreducible/compare/v0.6.5...HEAD
[0.6.5]: https://github.com/tsondru/irreducible/releases/tag/v0.6.5
[0.6.4]: https://github.com/tsondru/irreducible/releases/tag/v0.6.4
[0.6.3]: https://github.com/tsondru/irreducible/releases/tag/v0.6.3
[0.6.2]: https://github.com/tsondru/irreducible/releases/tag/v0.6.2
[0.6.1]: https://github.com/tsondru/irreducible/releases/tag/v0.6.1
[0.6.0]: https://github.com/tsondru/irreducible/releases/tag/v0.6.0
[0.5.0]: https://github.com/tsondru/irreducible/releases/tag/v0.5.0
[0.4.4]: https://github.com/tsondru/irreducible/releases/tag/v0.4.4
[0.4.3]: https://github.com/tsondru/irreducible/releases/tag/v0.4.3
