# TODO — catgraph v0.10.1 Fong-Spivak Integration

## Phase 1: Re-export new types (small, immediate)

- [ ] Re-export `HypergraphCategory` trait from `catgraph::hypergraph_category` through `src/functor/` or `src/lib.rs`
- [ ] Re-export `HypergraphFunctor`, `RelabelingFunctor`, `CospanToFrobeniusFunctor` from `catgraph::hypergraph_functor`
- [ ] Re-export `CospanAlgebra`, `PartitionAlgebra`, `NameAlgebra` from `catgraph::cospan_algebra`
- [ ] Re-export `cup`, `cap`, `cup_single`, `cap_single`, `name`, `unname` from `catgraph::compact_closed`
- [ ] Add integration tests verifying the re-exports are usable from irreducible's public API
- [ ] Run `cargo test --workspace` and `cargo clippy -- -W clippy::pedantic`

## Phase 2: HypergraphCategory in cospan verification (medium)

- [ ] Verify Stokes cospan chains satisfy `HypergraphCategory` Frobenius axioms — add test in `tests/stokes_integration.rs`
- [ ] Verify hypergraph evolution cospan chains satisfy `HypergraphCategory` — add test in `tests/catgraph_bridge.rs`
- [ ] Use `HypergraphCategory::cup`/`cap` in `StokesIrreducibility::to_cospan_chain()` if it strengthens the verification
- [ ] Add example in `examples/` demonstrating Frobenius structure on cobordism cospans

## Phase 3: CospanAlgebra for the functor Z' (larger, architectural)

- [ ] Evaluate whether `IrreducibilityFunctor` can implement or delegate to `CospanAlgebra` for lax monoidal coherence
- [ ] If viable: refactor `verify_symmetric_monoidal_functor()` to use `CospanAlgebra::map_cospan` + `lax_monoidal` instead of ad-hoc tensor checks
- [ ] Add integration tests comparing current monoidal verification results with `CospanAlgebra`-based results (must agree)

## Phase 4: HypergraphFunctor for Z': T → B (larger, architectural)

- [ ] Evaluate whether both T (computation category) and B (cobordism category) can be structured as `HypergraphCategory` instances
- [ ] If viable: implement `HypergraphFunctor` for Z', gaining Frobenius-preserving verification (Eq. 12) — stronger than current monoidal check
- [ ] Use `CospanToFrobeniusFunctor` to decompose cobordism cospans into Frobenius generators for verification
- [ ] Add integration tests for Frobenius preservation: `Z'(μ_X) = μ_{Z'(X)}` etc.

## Phase 5: Compact closed structure for adjunction (medium)

- [ ] Evaluate whether `name`/`unname` bijection from `compact_closed` simplifies `ZPrimeAdjunction` in `functor/adjunction.rs`
- [ ] If viable: refactor Z' ⊣ Z adjunction to use compact closed cup/cap pairing
- [ ] Verify triangle identities still hold after refactor

## Phase 6: Documentation and examples

- [ ] Update `examples/gorard_demo.rs` to showcase Fong-Spivak integration
- [ ] Update `examples/gorard_demo.md` with Fong-Spivak context
- [ ] Update CLAUDE.md test counts after all phases
- [ ] Bump version to 0.4.0
