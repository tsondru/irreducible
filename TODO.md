# TODO — Open Work on irreducible

> **Note.** Phases 1 + 2 of the original Fong-Spivak integration plan (catgraph v0.10.1 era) are complete and have been removed from this file. Re-exports live at `src/lib.rs:145-147` / `src/functor/fong_spivak.rs`; cospan-chain Frobenius verification lives at `verify_cospan_chain_frobenius` with coverage in `tests/fong_spivak.rs` and `examples/fong_spivak.rs`. The crate is now on catgraph v0.12.0 (Corel co-release), catgraph-surreal v0.10.1, and is at version 0.6.1.
>
> What follows is the still-open architectural work plus a new opportunity opened up by catgraph v0.12.0.

## Phase 3: CospanAlgebra for Z' — lax monoidal coherence

Replace the ad-hoc tensor checks in `verify_symmetric_monoidal_functor` (`src/functor/monoidal.rs:177`) with a `CospanAlgebra`-based verification. The goal is to leverage F&S's `map_cospan` + `lax_monoidal` machinery rather than re-deriving the coherence laws inline.

- [ ] Evaluate whether `IrreducibilityFunctor` can implement or delegate to `CospanAlgebra` for lax monoidal coherence (sketch the shape of the impl: what is the carrier set, what is `map_cospan` doing on cobordism intervals?).
- [ ] If viable: refactor `verify_symmetric_monoidal_functor` to use `CospanAlgebra::map_cospan` + `lax_monoidal` instead of ad-hoc tensor checks. Keep the public `MonoidalFunctorResult` shape stable.
- [ ] Add integration tests comparing the current monoidal verification results with the `CospanAlgebra`-based results — they must agree on every test fixture currently in `monoidal.rs:286+`.
- [ ] If it turns out *not* viable (e.g., the cobordism category isn't naturally a cospan-algebra target), record the negative result in `.claude/docs/decisions/` and close out this phase rather than carrying it indefinitely.

## Phase 4: HypergraphFunctor for Z': T → B — Frobenius preservation

Stronger than the current monoidal check: if both T (computation category) and B (cobordism category) are `HypergraphCategory` instances, then Z' should preserve the Frobenius generators (Eq. 12 in F&S 2019 — `Z'(μ_X) = μ_{Z'(X)}`, etc.).

- [ ] Evaluate whether T (multiway / single-track computation) and B (interval cobordism) can both be structured as `HypergraphCategory` instances. T is the harder direction — multiway evolution is naturally a cospan but the Frobenius structure isn't obvious.
- [ ] If viable: implement `HypergraphFunctor` for `IrreducibilityFunctor`. Add Frobenius-preservation checks alongside the monoidal-functor verification.
- [ ] Use `CospanToFrobeniusFunctor` to decompose cobordism cospans into Frobenius generators, then verify Z' preserves each generator's image.
- [ ] Integration tests for Frobenius preservation: at least one TM and one CA test where the unit/counit/multiplication/comultiplication on the source side match those on the target side after applying Z'.

## Phase 5: Compact closed structure for the Z' ⊣ Z adjunction

`functor/adjunction.rs` currently verifies the triangle identities directly. F&S's `compact_closed` gives a self-dual cup/cap structure that may simplify the adjunction's witness construction (Prop 3.2: name/unname bijection).

- [ ] Evaluate whether `name`/`unname` from `compact_closed` simplifies the witness functions in `ZPrimeAdjunction::verify_triangle_identities`.
- [ ] If viable: refactor the adjunction to use cup/cap pairing for the unit/counit. The triangle-identity check should reduce to a single `name(unname(_))` round-trip on each side.
- [ ] Verify the existing `AdjunctionVerification` results still hold for every test fixture after the refactor — no regressions in `tests/adjunction_laws.rs`.

## Phase 7: Corel for multiway merge events (catgraph v0.12.0)

New as of catgraph v0.12.0. `Corel<Lambda>` is a partition-of-targets structure (F&S 2018 Ex 6.64) with `equivalence_classes`, `merges`, `refines`, `coarsest_common_refinement`, and a `HypergraphCategory` impl. This maps very naturally onto multiway-evolution merge semantics, where two distinct rewrite paths reach the same hypergraph state and the system needs to track which states are identified.

- [ ] Re-export `Corel<Lambda>` and its helpers from `src/functor/fong_spivak.rs` (or a new `src/functor/corel.rs` if it grows).
- [ ] Evaluate whether `MultiwayCospanGraph::edges` can carry a `Corel` annotation when two cospan endpoints unify (i.e., when the multiway graph has a merge node).
- [ ] If viable: add `MultiwayCospanGraph::merges() -> Corel<u32>` exposing the merge partition over vertex IDs across the whole evolution.
- [ ] Use `Corel::coarsest_common_refinement` to compare merge structure across two evolutions of the same initial graph under different rule orderings — this is a candidate confluence test that's stronger than the current Wilson-loop check.
- [ ] Integration tests in `tests/multiway_evolution.rs` verifying merge-partition consistency for known confluent and non-confluent SRS fragments.

## Phase 8: Housekeeping (carry forward across phases above)

- [ ] After each phase: refresh test counts in `CLAUDE.md` (currently "385 base / 427 with full features" + persist suite).
- [ ] After each phase: update `examples/gorard_demo.rs` and `examples/gorard_demo.md` if a new public API lands that strengthens the 9-part presentation.
- [ ] After each phase: tag a release. Suggested cadence — Phase 3 → 0.7.0 (refactor of monoidal verification, possibly breaking), Phase 4 → 0.8.0 (Frobenius preservation), Phase 5 → 0.8.x (adjunction simplification, internal), Phase 7 → 0.9.0 (Corel public surface).
- [ ] If any phase is closed out as "not viable," record the decision in `.claude/docs/decisions/YYYY-MM-DD-<phase>.md` so future readers don't reopen the same investigation.
