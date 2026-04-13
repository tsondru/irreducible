# Phase 2.5 Design Decisions

**Status: DONE** -- implemented in irreducible v0.4.3 (2026-04-13).

Captured 2026-04-13 before implementation. Upstream spec:
`catgraph/.claude/refactor/phase-2.5-coherence-stokes-rewrite.md`

---

## Q1: "Multiway = non-strict SMC" — formalization by construction

**Decision:** Do not block on finding a published reference that proves multiway
systems form a non-strict symmetric monoidal category. Instead, treat the Phase 2.5
implementation as a formalization-by-construction.

**Rationale:**

catgraph-physics v0.1.0 ships the data structures that encode the SMC interpretation:

- `ConfluenceDiamond` — the 2-simplex witnessing that two paths reconverge
- `parallel_independent_events(node_id)` — enumerates tensor-product pairs at a fork
- `events_commute(a, b)` — causal commutativity predicate (common descendant exists)

The intuition is well-established in the Wolfram Physics literature: parallel
independent events are "tensor," sequential dependent events are "composition,"
and causal invariance is the statement that the associator / braiding natural
transformations exist (confluence witnesses). Gorard (arXiv:2301.04690) frames
multicomputational irreducibility as functoriality of Z': T → B, which presupposes
monoidal structure on T.

What's missing is a single published proof that "multiway evolution graphs with
confluence form a non-strict SMC" in exactly the sense needed. Rather than waiting
for one, we formalize the claim operationally:

1. `verify_associator` checks that `(e₁⊗e₂)⊗e₃` and `e₁⊗(e₂⊗e₃)` are confluent
   in the multiway graph (witness path exists). Returns `Err(NonConfluent)` if not.
2. `verify_braiding` checks commutativity of `e₁⊗e₂` and `e₂⊗e₁`.
3. `verify_unitor` checks identity-event composition.
4. Proptests construct non-confluent fragments and assert the checks **fail**.

If the checks pass on well-formed graphs and fail on malformed ones, the
implementation *is* the formalization. Docs will note:

> The category-theoretic claim is verified computationally, not proved
> symbolically. Candidate references: Gorard arXiv:2301.04690 (functorial
> irreducibility), Wolfram Physics Project (causal invariance as commutativity),
> Baez & Dolan (opetopic approach to higher categories).

---

## Q2: No `IrreducibilityTrace` impl for multiway systems

**Decision:** Do not add a multiway implementation of `IrreducibilityTrace` in
Phase 2.5. The trait stays scoped to deterministic sequential machines (TM, CA).

**Rationale:**

`IrreducibilityTrace` assumes linear sequential execution:

```rust
pub trait IrreducibilityTrace {
    fn state_fingerprints(&self) -> Vec<u64>;   // ordered sequence
    fn to_intervals(&self) -> Vec<DiscreteInterval>;  // composable chain
    fn step_count(&self) -> usize;              // single count
    fn halted(&self) -> bool;                   // single terminus
}
```

Multiway systems are DAGs with branching and merging. Any projection onto this
trait destroys tensor structure — the exact structure Phase 2.5 coherence
verification needs to inspect.

The existing approach is correct and sufficient:

- `branch_intervals()` extracts per-branch interval sequences from a multiway graph
- `verify_multiway_functoriality()` analyzes the branch collection as a whole
- Phase 2.5 coherence checks operate on the full `MultiwayEvolutionGraph`, not on
  individual branches

If per-branch trace analysis later proves useful, a thin `MultiwayBranchTrace`
wrapper (one root-to-leaf path in the DAG) could implement the trait. That's a
separate, smaller scope item — not a Phase 2.5 concern.
