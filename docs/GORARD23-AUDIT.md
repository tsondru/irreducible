# Gorard 2023 Coverage Audit (irreducible v0.6.3)

> **Paper:** Jonathan Gorard, *A Functorial Perspective on (Multi)computational Irreducibility* ([arXiv:2301.04690v1](https://arxiv.org/abs/2301.04690), 13 Oct 2022, dated Jan 2023).
> **Library:** `irreducible` v0.6.5+ (audit doc landed in v0.6.4 at SHA `6b3d656`; tracking maintained as the crate evolves) — catgraph workspace tag `v0.2.0` ([sustia-llc/catgraph](https://github.com/sustia-llc/catgraph) reboot lineage; the audit's item mapping predates the reboot and is unaffected — the consumer surface carried over intact).
> **Method:** read all 60 pages of the paper end-to-end (intro + §2 + §3 + §4 + §5 + references); cross-walked every numbered equation, definition, theorem, figure caption, and named concept against the irreducible source tree (`src/**`, `tests/**`, `examples/**`). Coverage attribution honours the v0.6.3 shim: implementations that physically live in `catgraph_physics::{interval,temporal_cospan_chain,trace}` are attributed to the irreducible surface (consumers see them as `irreducible::*`).
> **Update cadence:** maintained alongside the crate version. Add a row whenever a new paper item is implemented; flip status (e.g. ⏭️ → ✅) when an action item closes.
>
> **Status legend:**
> - ✅ DONE — implemented and tested against the paper's stated property.
> - ⚠️ PARTIAL — implementation exists but does not fully exhibit the paper's structure (or a corner case is mis-scoped).
> - ⏭️ DEFERRED — planned for a later release.
> - ➖ N/A — discussion / motivational / explicitly stated as future work in the paper itself.
> - 🔗 IN-CATGRAPH — implementation lives in catgraph / catgraph-physics / catgraph-applied; consumed via re-export.
>
> **Companion audits (catgraph workspace):** [`FS19-AUDIT.md`](https://github.com/sustia-llc/catgraph/blob/main/catgraph/docs/FS19-AUDIT.md) (catgraph), [`FS18-AUDIT.md`](https://github.com/sustia-llc/catgraph/blob/main/catgraph-applied/docs/FS18-AUDIT.md) (catgraph-applied), [`BV25-AUDIT.md`](https://github.com/sustia-llc/catgraph/blob/main/catgraph-magnitude/docs/BV25-AUDIT.md) (catgraph-magnitude).

---

## 1. Summary

| Section | DONE | PARTIAL | DEFERRED | N/A | IN-CATGRAPH | Total |
|---|---|---|---|---|---|---|
| §1 Introduction (motivational) | 0 | 0 | 0 | 4 | 0 | 4 |
| §2 Singleway irreducibility (Def 1, Eqs 1–32) | 7 | 2 | 0 | 1 | 1 | 11 |
| §2 Cobordism cat \(\langle \mathcal{B}, \partial, i\rangle\) (Eqs 15, 16, 22–32) | 1 | 2 | 1 | 0 | 0 | 4 |
| §3 Multicomputational irreducibility (SMC, Eqs 33–67) | 6 | 2 | 0 | 0 | 0 | 8 |
| §3 Hypergraph rewriting / DPO (Eqs 70–110) | 5 | 1 | 0 | 1 | 1 | 8 |
| §4 Adjunction with categorical QM / FQFT (Eqs 111–148) | 1 | 1 | 4 | 1 | 0 | 7 |
| §5 Concluding remarks / future directions | 0 | 0 | 5 | 4 | 0 | 9 |
| **TOTAL** | **20** | **8** | **10** | **11** | **2** | **51** |

**Headline numbers (as of irreducible v0.6.3):**
- Of 51 audited items, **11 are N/A** (motivational text or explicitly future-work in the paper itself), **2 IN-CATGRAPH**, leaving **38 implementable items** of which **20 are DONE, 8 PARTIAL, 10 DEFERRED**.
- Of implementable items: **53% DONE / 21% PARTIAL / 26% DEFERRED**.
- The single largest gap is §4 (adjunction with categorical QM / FQFT): one direction of the adjunction (Z': T → B) is implemented end-to-end, but the right adjoint Z: B → T (interval-to-vector-space functor, the propagator side) is essentially absent. Triangle identities exist but verify a one-sided self-roundtrip, not a genuine 𝒯 ⇄ 𝓥ect adjunction.
- The second largest gap is §5 future-work directions (causal 2-cells, glocal multiway, dagger / compact closed deformation), which the paper labels "future investigation" but for which the audit nevertheless tracks present coverage.

---

## 2. Coverage by paper section

### §1 Introduction — motivational

| Item | Status | Location | Notes |
|---|---|---|---|
| Wolfram's NKS computational-irreducibility framing | ➖ | (narrative) | README §"Key Concepts" + the [arXiv:2301.04690](https://arxiv.org/abs/2301.04690) cite. No implementation expected. |
| Recursion-theoretic context (universality, undecidability) | ➖ | (narrative) | No implementation expected. |
| "Irreducibility *is* functoriality" thesis | ➖ → ✅ via §2 | functor/mod.rs | Treated as N/A here; the operationalisation is the §2 row "Z' verifying functoriality". |
| References [1]–[9] (Wolfram, Gorard 2018 Slowdown Theorem) | ➖ | (paper-side) | Slowdown theorem appears in commentary; no quantitative slowdown bound implemented (action item M-1). |

### §2 Singleway irreducibility as functoriality

The paper constructs category 𝒯 (objects = TM configurations, morphisms = transitions) freely from the partial transition function δ: (Q\F)×Γ ⇸ Q×Γ×{L, R, id} (Eqs 1, 8). It then defines functor Z': 𝒯 → ℬ where ob(ℬ) = ℕ and hom(ℬ) = {[n,m] ∩ ℕ | n ≤ m}, with composition `[n,m] ∘ [m,k] = [n,k]` (Eq 9, Eqs 11–13). Computational irreducibility ⇔ Z' is functorial.

| Item | Paper ref | Status | Location | Notes |
|---|---|---|---|---|
| Def 1: reducibility ⇔ ∃ T* with m < n | §2 Def 1 | ⚠️ | functor/mod.rs:104 (`is_sequence_irreducible`), turing.rs:445 (`ExecutionHistory::is_irreducible`) | Partial: the impl operationalises *contiguity-of-intervals* and *cycle-detection-via-fingerprint-repeats*. Cycle detection witnesses an explicit shortcut (T* = T with the cycle elided). It does **not** implement the more general "∃ T* with m < n" — i.e. a search over alternative TMs computing the same f. The paper treats Def 1 as the formal anchor; operational verifiers approximate it via shortcut-search. Document the gap explicitly, or rename the API to `is_no_repeat_shortcut` to avoid overclaim. (Action item I-1.) |
| 1-tape TM as 7-tuple `T = ⟨Q, Γ, b, Σ, q₀, δ, F⟩` | §2 Eq (above 1) | ✅ | machines/turing.rs:28 (`TuringMachine`) | Full data: states, initial_state, accept_states, reject_states, blank, transitions. |
| Partial transition `δ : (Q\F)×Γ ⇸ Q×Γ×{L, R, id}` (id = "no shift") | §2 Eq 1, Eq 8 | ⚠️ | machines/transition.rs:15 (`Direction::{Left,Right,Stay}`) | Naming gap: paper writes the third value as `{L, R, id}` to emphasise that `id` is the categorical *identity-on-tape* (no head shift, supplying the identity morphism in 𝒯). The implementation calls this `Direction::Stay`, breaking the paper-correspondence. Display also emits `S` instead of `id`. Action item M-2: rename or document. The functional behaviour is correct (delta = 0). |
| Object set `ob(𝒯) = Γ^ℵ₀ × Q × ℕ` (tape × state × head) | §2 (after Eq 1) | ✅ | machines/configuration.rs (`Configuration { tape, state, head }`) | Tape is `Vec<Symbol>` with implicit blank padding (countable Γ representation); head is `isize`; state is `u32`. |
| Free category from quiver: composition operator ∘ over (f, X, Y) triples (Eqs 2, 3) | §2 Eqs 2–3 | ✅ | machines/transition.rs (`Transition { from_config, to_config, step }`) | Each `Transition` is exactly the (f, X, Y) triple. Composition is implicit in `ExecutionHistory.transitions: Vec<Transition>`. |
| Associativity of ∘ (Eq 4) | §2 Eq 4 | ➖ | (inherited from Vec append) | Vec composition is associative; not separately tested. |
| Identity axiom: `id_X : X → X` and unit laws (Eqs 5–7) | §2 Eqs 5–7 | ⚠️ | (no identity transition explicit) | The category-theoretic `id_X` corresponds to `Direction::Stay` on a fixed point. There is no test asserting the unit laws `f ∘ id_X = f = id_Y ∘ f` directly on `Transition`. Action item M-3: add unit-law test. |
| Reflexive transitive closure intuition: edge-tagging with step-counts (Fig 2, 3) | §2 Fig 2/3 | ✅ | turing.rs (`ExecutionHistory.transitions`) + functor/mod.rs:83 (`map_morphism`) | Each transition carries its step number; the functor maps step-counts to interval cardinalities. |
| Functor Z' : 𝒯 → ℬ on objects (Eq 10) | §2 Eq 10 | ✅ | functor/mod.rs:75 (`map_step`), computation_state.rs:74 (`to_interval`) | `Z'(X) ∈ ob(ℬ)` realised as `step: usize`. |
| Functor Z' on morphisms (Eq 11): `Z'(f) = [Z'(X), Z'(Y)] ∩ ℕ` | §2 Eq 11 | ✅ | functor/mod.rs:75–84 (`map_step`, `map_morphism`), transition.rs:79 (`to_interval`) | `Transition::to_interval` returns `[step, step+1]`. |
| Composition preservation (Eq 12): `Z'(g ∘ f) = Z'(g) ∪ Z'(f) = [Z'(X), Z'(Z)] ∩ ℕ` | §2 Eq 12 | ✅ | functor/mod.rs:92 (`verify_functoriality`), tests/functoriality.rs | This *is* the irreducibility predicate. Both per-step (`is_composable_with`) and chain-level (`is_sequence_irreducible`) verifiers exist. |
| Identity preservation (Eq 13): `Z'(id_X) = [Z'(X), Z'(X)] ∩ ℕ = {Z'(X)}` | §2 Eq 13 | ⚠️ | functor/mod.rs:74–77 + computation_state.rs:74 | `map_step(s)` returns `[s, s+1]` (1-step interval), and `to_interval` on a state with `complexity = 0` returns `[s, s+1]` via `.max(1)`. The paper Eq 13 says the identity at X maps to the singleton `{Z'(X)} = [s, s] ∩ ℕ` (cardinality 1, zero steps). The implementation conflates "identity" and "elementary step". Either: (a) add a singleton-`identity(s)` constructor for `DiscreteInterval` distinct from the elementary-step interval (`catgraph_physics::interval::DiscreteInterval::singleton`/`identity` already exists at line 67/76 of catgraph-physics — wire it through here); or (b) document explicitly that irreducible's `map_step` is `Z'(elementary-transition)`, not `Z'(id_X)`. Action item I-2. |
| Eq 14 commutative diagram for triangle composition `f, g, g∘f` | §2 Eq 14 | ✅ | tests/functoriality.rs (`test_three_step_chain` and similar) | Three-state composition diagrams are the standard test fixture. |

### §2 (continued) Cobordism category ⟨ℬ, ⊕, ∂, i⟩

The paper extends ℬ from discrete intervals to a full cobordism category (Eqs 15–32): real-valued time `ob(ℬ) = ℝ`, continuous intervals `[x, y]`, finite coproducts ⊕, an additive endofunctor ∂ with ∂² = ∅, a natural transformation i: ∂ ⇒ Id, and a cobordism equivalence relation `M₁ ∼ M₂` (Eq 30).

| Item | Paper ref | Status | Location | Notes |
|---|---|---|---|---|
| Continuous-time variant `ob(ℬ) = ℝ` (Eq 15) | §2 Eq 15 | ⏭️ | (none) | Defer to v0.7.0+. The discrete (ℕ-valued) variant is the operational one for this crate; continuous `ℝ`-valued intervals are listed in the paper as making "the underlying topological intuition manifest" but with no algorithmic claim attached. |
| Cobordism category triple ⟨ℬ, ∂, i⟩ (Eqs 22–25): initial object ∅, additive endofunctor ∂, natural transformation i: ∂ ⇒ Id_ℬ with ∂² = ∅ | §2 Eqs 16, 22–25 | ⚠️ | (partial) | `ParallelIntervals` plays the role of finite-coproduct ⊕ (Eq 21). However, the **boundary endofunctor ∂** is not implemented. There is no `boundary` operation on `DiscreteInterval` returning the pair `{n, m}` (the 0-dimensional manifold given by an interval's endpoints), nor any verifier for `∂(∂([n,m])) = ∅` (Eq 23, ∂² = 0). This is a non-trivial gap because the §4 adjunction with FQFT is conditioned on ℬ being a *cobordism* category, not just a discrete-interval poset. Action item I-3 (deferred to v0.7.0+ alongside §4 expansion). |
| Cobordism equivalence `M₁ ∼ M₂ ⇔ ∃V₁,V₂: M₁ ⊕ ∂V₁ ≅ M₂ ⊕ ∂V₂` (Eqs 30–32) | §2 Eqs 30–32 | ⏭️ | (none) | Requires ∂ first; deferred. |
| Initial object ∅, coproduct injection morphisms i₁, i₂ universality (Eqs 17–21) | §2 Eqs 17–21 | ⚠️ | bifunctor.rs:39 (`TensorProduct::unit` for `ParallelIntervals`) | Unit element + tensor-as-coproduct exists but universality (the unique morphism `M₁ ⊕ M₂ → M*`) is not verified. Action item M-4. |

### §3 Multicomputational irreducibility as monoidal functoriality

The paper promotes 𝒯 and ℬ to symmetric monoidal categories ⟨𝒯, ⊗, I = HALT⟩ and ⟨ℬ, ⊕, ∅⟩ (Eqs 35–47, 48–55). Multicomputational irreducibility ⇔ Z' is a *symmetric monoidal* functor (Eqs 58–67), i.e. preserves tensor product up to coherence isomorphism (μ, ε).

| Item | Paper ref | Status | Location | Notes |
|---|---|---|---|---|
| Multiway evolution graph as quiver freely generating 𝒯 (Fig 6, 7) | §3 (after Eq 35) | ✅ 🔗 | machines/multiway/mod.rs (re-exports `MultiwayEvolutionGraph`) | Lives in catgraph-physics; consumed by `irreducible::machines::multiway::*`. |
| Branchial graphs as foliation Σ_t = level sets of universal time function `t : V → ℤ` (Eqs 33–34) | §3 Eqs 33–34, Fig 8/9 | ✅ 🔗 | machines/multiway/mod.rs (re-exports `extract_branchial_foliation`) | `BranchialGraph` carries the spatial-tensor structure at each step. |
| Tensor product bifunctor `⊗ : 𝒯 × 𝒯 → 𝒯` (Eq 35) | §3 Eq 35 | ✅ | bifunctor.rs:27 (`TensorProduct` trait) | Implemented for `ParallelIntervals`; codomain side. Domain side (`MultiwayEvolutionGraph::tensor_product`) is not directly exposed but is implicit in branchial foliation. |
| Product category 𝒯 × 𝒯 (Eqs 36–39) | §3 Eqs 36–39 | ➖ | (Rust tuple) | Not a separate type; implicit. |
| Associator α (Eqs 40–43) — pentagon coherence | §3 Eqs 40–43 | ✅ | multiway_coherence.rs:106 (`verify_associator`), bifunctor.rs:172 (`verify_associativity`) | Two flavours: `bifunctor::verify_associativity` is *strict* (branch list equality) on `ParallelIntervals`; `multiway_coherence::verify_associator` is *non-strict* (confluence diamond) on multiway graphs. Both pass on confluent fixtures. |
| Left/right unitor λ, ρ (Eqs 44–47) | §3 Eqs 44–47 | ✅ | multiway_coherence.rs:`verify_unitor` (re-exported via `verify_all_coherence`), bifunctor.rs:188 (`verify_unit_laws`) | Same pattern: strict + non-strict checks. |
| Symmetry/braiding σ (Eqs 48–55) — hexagon coherence + involutivity (Eq 55) | §3 Eqs 48–55 | ✅ | multiway_coherence.rs:`verify_braiding`, bifunctor.rs:201 (`verify_symmetry`) | Hexagon coherence (Eq 50) is checked indirectly via confluence-diamond commutativity, not by an explicit hexagon verifier. Action item M-5: add an explicit hexagon test on a fixture multiway graph. |
| Braided-but-not-symmetric variant (Eqs 56–57) | §3 Eqs 56–57 | ➖ | (paper notes the case is unnecessary for symmetric MCs) | Out of scope for v0.6.x. |
| Symmetric monoidal functor Z' : ⟨𝒯, ⊗, HALT⟩ → ⟨ℬ, ⊕, ∅⟩ (Eq 58) | §3 Eq 58 | ⚠️ | functor/monoidal.rs:177 (`verify_symmetric_monoidal_functor`) | Implementation verifies tensor preservation step-by-step on the foliation (`compute_actual_parallel`, line 242). However: (a) the **HALT unit object** (paper: I = HALT, Eq 58) is not represented anywhere — there is no dedicated `MultiwayEvolutionGraph` HALT node, no `Z'(I) → ε` coherence map (Eq 59 ε : ∅ → Z'(I)). The paper is explicit that HALT plays the role of the unit; the implementation defaults to the empty `ParallelIntervals`. Action item I-4. (b) The coherence-map equality `Z'(α^𝒯) = α^ℬ ∘ μ ∘ (μ ⊕ id)` (Eq 60–61, the lax monoidal coherence diagram) is not separately verified beyond strict component equality. |
| Coherence maps ε : ∅ → Z'(I) and μ : Z'(X) ⊕ Z'(Y) → Z'(X ⊗ Y) (Eq 59) | §3 Eq 59 | ⚠️ | (implicit; flat tensor only) | Treated as identities (strict monoidal). The paper-spec for **lax** monoidal allows non-isomorphism μ; **strong** requires isomorphism; **strict** identity. The implementation is *strict* in the monoidal category sense, which is a stricter framing than the paper's lax setup. This is fine for `Direction::Stay`-flat domains but does not exhibit the lax structure. Action item I-5. |
| Lax / strong / strict monoidal functor distinction (paper line 27, after Eq 65) | §3 (text) | ⚠️ | (no naming distinction in code) | The implementation is silently strict; calling code refers to "monoidal functor" without specifying flavour. Document or expose. |
| Symmetry coherence Eq 66–67: `μ_{Y,X} ∘ σ^ℬ_{Z'(X),Z'(Y)} = Z'(σ^𝒯_{X,Y}) ∘ μ_{X,Y}` | §3 Eqs 66–67 | ⚠️ | multiway_coherence.rs `verify_braiding` | Verified at the multiway-graph level (parallel-events commute) but not at the functor-coherence level (Eq 66). The two are equivalent under strict monoidality; under lax, they differ. Action item M-6. |
| Multicomputational irreducibility ⇔ additivity under tensor product (Eqs 67–69, lines 14–16) | §3 (text after Eq 67) | ⚠️ | functor/monoidal.rs:225 (`compute_actual_parallel`) | The paper is emphatic: multicomputational reducibility = *subadditivity of complexity under ⊗*; multicomputationally irreducible ⇔ `f⊗g requires ≥ n+m steps`. The implementation checks structural equivalence of `ParallelIntervals` shape but does **not** explicitly compute "minimum steps" for tensor composition — instead it equates "tensor preservation" with "every active branch advances by one step at every foliation slice", which is a sufficient-but-not-necessary condition. **Crucially:** `Complexity::parallel(self, other) = max(self, other)` in `complexity.rs:85` directly contradicts the paper's *additive* multicomputational complexity model (Eqs 67–69 set parallel composition complexity = sum of individual complexities). This is the most material paper-fidelity gap in §3. Action item I-6 (potentially blocking depending on consumer expectations). |
| State-evolution function vs state-equivalence function orthogonality (paper line 28, around Eq 70) | §3 (text) | ⚠️ | machines/multiway/string_rewrite.rs (SRS), machines/multiway/ntm.rs (NTM) | The paper emphasises that for NTMs the state-equivalence function is *trivial* (tape+state+head equality); for hypergraph rewriting it is *non-trivial* (hypergraph isomorphism). Both surfaces exist in irreducible (NTM via `tests/computation_types.rs`, hypergraph via `machines/hypergraph/`); however no analytic surface (e.g. `state_equivalence_complexity` per machine type) tracks this orthogonality explicitly. Action item M-7. |

### §3 Hypergraph rewriting / DPO

| Item | Paper ref | Status | Location | Notes |
|---|---|---|---|---|
| Directed hypergraph `H = ⟨V, E⟩`, `E ⊆ 𝒫(V)\{∅}` (Eq 70) | §3 Eq 70 | ✅ 🔗 | machines/hypergraph/mod.rs (re-exports `Hypergraph`, `Hyperedge` from catgraph-physics) | |
| Hypergraph rewriting rule as span `L ← K → R` of monomorphisms (Eq 71) | §3 Eq 71 | ✅ 🔗 | machines/hypergraph/mod.rs (`RewriteRule`, `RewriteSpan`) + catgraph-physics rewrite_span.rs:46 (`RewriteRule::to_span()`) | The categorical `Span<u32>` representation is one method-call away from any `RewriteRule`. |
| Mono left-cancellative property (Eqs 72–74) | §3 Eqs 72–74 | ⚠️ | (catgraph-physics `Span` does not separately enforce monomorphism) | The construction is type-named `Span` but the underlying maps `l: K → L`, `r: K → R` are not proven injective on insertion. Wolfram-style rewrite rules happen to be injective in practice but no run-time check. Action item M-8. |
| Double-pushout (DPO) construction (Eqs 75–81) | §3 Eqs 75–81 | ✅ | catgraph-physics::hypergraph::evolution.rs (HypergraphEvolution::run) | |
| Adhesive category requirements (van-Kampen square, Eqs 82–108) | §3 Eqs 82–108 | ⏭️ | (none) | The paper acknowledges that the hypergraph category with subhypergraph inclusions is *not* strictly adhesive (full subcategory of an adhesive ambient via "selective adhesivity"). v0.6.3 does not verify any of the van-Kampen / pullback / pushout-along-mono conditions. Listed in §5 as future work; deferred. |
| Concurrency theorem composition `p₁ *_E p₂` (Eq 109) — gives ∘ in 𝒯 | §3 Eq 109 | ⚠️ | machines/hypergraph/catgraph_bridge.rs (`MultiwayCospan::compose`) | Compositional structure is implicit in the multiway-evolution chain; no first-class `concurrent_composition` method on `RewriteRule`. Action item M-9 (deferred — cleaner once the §3 Eq 110 parallel-production surface lands). |
| Parallelism theorem composition `p₁ + p₂` (Eq 110) — gives ⊗ in 𝒯 | §3 Eq 110 | ⚠️ | (implicit via foliation) | Same gap as Eq 109. |
| Causal invariance / Wilson-loop holonomy | §5 (Fig 17, future-work) but partly anchored in §3 via DPO | ✅ | machines/hypergraph/mod.rs (`CausalInvarianceResult`, `WilsonLoop`) | irreducible exposes catgraph-physics's `CausalInvarianceResult` + `WilsonLoop` + `plaquette_action`. This is *forward* from where the paper has it; the paper treats causal invariance as an aspirational §5 future direction (causal 2-cells in a weak 2-category). |

### §4 Adjunction with categorical QM / FQFT

The paper's §4 is the headline-conceptual section: it argues that Z' : 𝒯 → ℬ is *left adjoint* to a propagator functor Z : 𝓑ord^Riem → 𝓥ect, so that "(multi)computational irreducibility is dual to locality of time evolution" (Eqs 122–125). It also discusses dagger / compact / dagger-compact structures (Eqs 129–148).

| Item | Paper ref | Status | Location | Notes |
|---|---|---|---|---|
| Adjunction Z' ⊣ Z (Eqs 118–124) | §4 Eqs 118–124 | ⚠️ | adjunction.rs:34 (`ZPrimeOps`), functor/adjunction.rs:34 (`ZPrimeAdjunction`) | The implementation defines Z' : `ComputationState → DiscreteInterval` and Z : `DiscreteInterval → ComputationState`, then verifies "triangle identities" via `verify_triangle_1` / `verify_triangle_2` (functor/adjunction.rs:55–69). However: the paper's Z is not `DiscreteInterval → ComputationState`; it is `Z : 𝓑ord^Riem → 𝓥ect` — the *propagator* functor sending a manifold to its space-of-states, sending a cobordism to its Hilbert-space evolution operator. Calling `Z(interval)` "the computation state at that interval" is a categorical pun, not the paper's adjunction. The triangle identities therefore reduce to a tautological `to_interval ∘ z ∘ to_interval = to_interval` roundtrip on `(step, complexity)` pairs (verified in `tests/adjunction_laws.rs`). **This is the most architectural drift from the paper in v0.6.x.** Action item A-1 (architectural; surface in v0.7.0 plan). |
| Triangle identity 1: ε_{Z'(c)} ∘ Z'(η_c) = id_{Z'(c)} (Eq 122–123) | §4 Eqs 122–123 | ⚠️ | functor/adjunction.rs:55–61 | Tautological under the current Z definition. |
| Triangle identity 2: Z(ε_i) ∘ η_{Z(i)} = id_{Z(i)} (Eq 124) | §4 Eq 124 | ⚠️ | functor/adjunction.rs:63–69 | Same. |
| Quantum-mechanical time-evolution functor `Z : 𝓑ord_1^Riem → 𝓥ect` (Eqs 111–117) | §4 Eqs 111–117 | ⏭️ | (none) | Atiyah-Segal sewing laws (Eq 113) absent. No `Vect` or `Hilbert` types exist in irreducible; the paper takes them from categorical-quantum-mechanics literature (Abramsky-Coecke). Lattice-gauge infrastructure (Wilson-loop / Polyakov-loop / Bessel-ratio, e.g. `deep_causality_topology`'s lattice module) may seed a v0.7.0+ Z stub. Deferred. |
| Atiyah-Segal sewing laws Z(M ∪ M') = Z(M') ∘ Z(M) (Eq 113) | §4 Eq 113 | ⏭️ | (none) | Same as above. |
| Higher-dim adjunction `Z' : ⟨𝒯, ⊗, I⟩ → ⟨ℬ, ⊕, ∅⟩` ⊣ `Z : 𝓑ord_d^Riem → 𝓥ect` (Eq 125) | §4 Eq 125 | ⏭️ | (none) | Deferred. |
| Dagger structure † (Eqs 129–137) — involutive contravariant endofunctor | §4 Eqs 129–137 | ⏭️ | (none) | Paper notes (Eq 137) that any Wolfram-style hypergraph rewrite rule has a canonical † given by `(L ← K → R) ↦ (R ← K → L)`; this could be implemented today on `RewriteRule` (~10 lines). Deferred to v0.7.0+. |
| Compact closed structure (Eqs 138–144) — duals X*, η, ε, yanking | §4 Eqs 138–144 | ⏭️ | (none) | The cup/cap/name/unname infrastructure does exist in catgraph (`functor/fong_spivak.rs:28`, re-exporting `catgraph::compact_closed::*`) but it is anchored to F&S "Seven Sketches" §3.1 — a cospan-category compactness, not Vect-style FdVect compactness. The paper-fidelity claim (compact-closed Vect cf. Eq 138) is not exercised. ⚠️ partial-credit alternative: count this as ✅ if the F&S compact-closed surface is treated as a *witness* of the paper's compact-closed claim. Action item A-2 (architectural — clarify framing). |
| Dagger-compact category (Eqs 143–148) — interplay σ ∘ ε† = η | §4 Eqs 143–148 | ⏭️ | (none) | Deferred. |
| Hypergraph dual `H* = ⟨V* = E, E* = …⟩` (Eqs 147–148) | §4 Eqs 147–148 | ⏭️ | (none) | Could be added today via vertex-edge swap on `Hypergraph`; deferred. |
| Computational-complexity meaning of `f†` ("irreducibility of reversal", paper line 28 of §4) | §4 (text + §5) | ⏭️ | (none) | Speculative direction; deferred. |

### §5 Concluding remarks / future directions

The paper's §5 lists several speculative extensions. We track them here with status because future irreducible plans may want to consume them.

| Item | Paper ref | Status | Location | Notes |
|---|---|---|---|---|
| Multicomputational complexity classes survey | §5 (lines 22–33) | ➖ | — | Open research direction. |
| Causal invariance ⇒ weak 2-category, 2-cells = causal edges (Fig 17, §5 lines 34–60) | §5 Fig 17 | ⏭️ | (none) | The paper-figure is reproducible-in-principle from `MultiwayEvolutionGraph` (catgraph-physics) but no `causal_2_cells` API exists. Listed as deferred. |
| Causal categories à la Coecke-Lal (§5 paragraph after Fig 17) | §5 (text) | ⏭️ | (none) | Tied to the previous row. |
| "Causal irreducibility": Z' distorting causal 2-cell composition | §5 (text) | ⏭️ | (none) | Paper future-work; no implementation. |
| Reversibility / cryptographic-irreducibility duality (one-way functions) | §5 (text) | ⏭️ | (none) | Paper future-work. |
| Glocal multiway systems (token-shattering) | §5 (text) + Figs 18, 19 | ⏭️ | (none) | Listed as future work. The catgraph-physics base lacks a `GlocalMultiwayGraph` type. |
| Spatial vs branchial tensor products (rig categories) | §5 (text) | ⏭️ | (none) | Paper acknowledges compatibility conditions are unknown; explicit `rig` framing would consume `catgraph-applied::Rig` if attempted. |
| Computational vs multicomputational entropy disambiguation (§5, lines 58–80) | §5 (text) | ➖ | — | Motivational. |
| Connection to Arsiwalla / Shulman cohesive HoTT (§5 line 82) | §5 (text) | ➖ | — | Motivational. |

---

## 3. Coverage by definition / theorem / proposition / example

The paper does not number theorems or propositions; the only formally numbered objects are equations and **Definition 1**. So this table tracks named *concepts* introduced in section text and named *figures*, with their concrete acceptance witnesses where the audit treats the item as DONE.

| Paper item | Status | Location | Concrete witness / acceptance |
|---|---|---|---|
| Definition 1 (reducibility ⇔ ∃T* with m<n) | ⚠️ | turing.rs:445 | Approximation only (no-shortcut-by-cycle). See §2 row above. |
| Figure 1 (rule 2506 graphical) | ➖ | — | Not implemented as a fixture. The paper's *2-state, 2-color rule 2506* could be reproduced as a `TuringMachine` constant alongside `busy_beaver_2_2` (turing.rs:194); doing so would let `tests/functoriality.rs` test the exact paper figure data. **Action item M-10**. |
| Figure 2 (rule 2506 evolution graph from `{0,1,0,0}`, 4 steps) | ➖ | — | Same as above — depends on M-10 fixture. |
| Figure 3 (edge-tagging with step-counts) | ✅ | turing.rs `to_intervals` | The morphism-tagging structure is implemented; not validated against rule-2506 specifically. |
| Figure 4 (vertex/edge-tagged free category, Z' with intermediate step lists) | ✅ | turing.rs + functor/mod.rs:106 (`is_sequence_irreducible`) | |
| Figure 5/6/7 (rules 2506+3506 as parallel composition; multiway graph for 3 steps) | ➖ | — | Same as Figs 1–2 — would benefit from a paper-fixture rule fixture. |
| Figure 8/9 (default foliation Σ_t and branchial graphs) | ✅ 🔗 | machines/multiway/mod.rs (re-exports `extract_branchial_foliation` + `BranchialGraph`) | Foliation tested in `tests/multiway_evolution.rs` and `tests/multiway_coherence.rs`. |
| Figure 10 (NTM evolution graph with step counts; subadditive ∘ and ⊗) | ⚠️ | tests/multiway_coherence.rs | Subadditivity check missing. See action item I-6. |
| Figure 11 (vertex/edge-tagged version of Fig 10) | ⚠️ | (same) | Same. |
| Figure 12 (set substitution rule `{{x,y},{x,z}} → {{x,z},{x,w},{y,w},{z,w}}`) | ➖ | — | Could be a `RewriteRule::set_substitution_paper_fig12` constant. **Action item M-11**. |
| Figures 13–16 (multiway hypergraph evolution, 3 steps, with cospan analysis) | ⚠️ | machines/hypergraph/catgraph_bridge.rs | Full multiway-cospan plumbing exists; not validated against the paper's specific 3-step double-self-loop fixture. Depends on M-11. |
| Figure 17 (multiway evolution causal graph, gray + orange edges) | ⏭️ | (none) | Paper future-work (§5). Deferred. |
| Figures 18–19 (glocal multiway evolution causal graph, glocal branchial graph) | ⏭️ | (none) | Paper future-work (§5). Deferred. |

---

## 4. Acceptance gates

Quantitative checks the v0.6.3 surface should pass against paper claims. Modeled on the BV25-AUDIT three-residual pattern.

| # | Gate | Status today | Test reference | Paper anchor |
|---|---|---|---|---|
| 1 | Busy Beaver BB(2,2) halts at exactly 6 steps with four 1s on tape | ✅ | turing.rs:656 (`test_busy_beaver_2_2`) | §2 (text — example); known result. |
| 2 | BB(2,2) is irreducible (no shortcuts; complexity ratio = 1.0) | ✅ | turing.rs:674 (`test_busy_beaver_irreducible`) | §2 Eq 12 functoriality predicate. |
| 3 | Cycling TM on blank tape is reducible (shortcut found) | ✅ | turing.rs:709 (`test_cycling_tm`) | §2 Def 1. |
| 4 | Adjunction triangle identities hold on a sequence of `ComputationState`s | ⚠️ | tests/adjunction_laws.rs (11 tests) | §4 Eqs 122–124 — but the implementation triangulates the *self-roundtrip* of `to_interval`, not the paper's 𝒯 ⇄ 𝓥ect adjunction. Passes by tautology. **Surface in v0.7.0 plan.** |
| 5 | `verify_associativity / verify_unit_laws / verify_symmetry` on `ParallelIntervals` | ✅ | tests/bifunctor_laws.rs | §3 Eqs 40–55. Strict variant; passes by Vec-equality. |
| 6 | Non-strict associator/braiding/unitor coherence on confluent multiway graph | ✅ | tests/multiway_coherence.rs (10 tests) | §3 Eqs 40–55 lifted to multiway substrate. |
| 7 | Non-confluent fragment fails coherence | ✅ | examples/multiway_coherence.rs | §3 (negative falsifiability of monoidal hypothesis). |
| 8 | Frobenius decomposition preserves composition on cospan chain (irreducible's three-perspective agreement) | ✅ | tests/fong_spivak.rs | F&S §2-3 (consumed via catgraph). |
| 9 | NTM with subadditive parallel composition (paper-Fig-10 fixture) detects multicomputational *reducibility* | ❌ | (no test) | §3 Eqs 67–69. Missing because `Complexity::parallel = max` discards the subadditivity signal. **Blocking for "this crate verifies multicomputational irreducibility per the paper".** [Action item I-6](#7-action-items) — scheduled for **v0.7.0** (path (a): patch `Complexity::parallel` to `+`, single paper-faithful model, breaking; ratified 2026-05-05). |
| 10 | Hypergraph DPO + Wilson-loop holonomy detects causal invariance | ✅ 🔗 | tests/hypergraph_rewriting.rs (21 tests) | §3 (DPO) + §5 future-work-via-anticipation. |
| 11 | DEC `d² = 0` on confluence-diamond 2-complex (feature `dec`) | ✅ | tests/multiway_stokes.rs | §5 Stokes-perspective interpretation; goes beyond paper-explicit content. |

---

## 5. Out of scope (v0.6.x)

Items intentionally not implemented in v0.6.x with rationale:

- **Continuous-time ℝ-valued cobordism category (Eq 15).** Discrete ℕ-valued category is the operational substrate; continuous variant is "topological intuition" per the paper itself with no algorithmic claim attached. Re-evaluate if a downstream consumer (catgraph-coalition? catgraph-coalition-dl?) needs Riemannian-flavoured intervals.
- **Z : 𝓑ord^Riem → 𝓥ect propagator functor** (Eqs 111–128, §4). Categorical-quantum-mechanics infrastructure (FdVect, Hilbert spaces, Schrödinger evolution) is far outside the crate's stated scope. The "adjunction-with-FQFT" framing of §4 is the paper's interpretive contribution, not an operational claim.
- **Adhesive-category van-Kampen verification (Eqs 82–108).** Out of scope; defer to a hypothetical adhesive-category crate.
- **Glocal multiway / dagger / compact-closed deformations (§5).** Paper-explicit future-work; out of scope for v0.6.x.

---

## 6. Deferred to future versions

| Target | Item | Notes |
|---|---|---|
| **v0.7.0** | Drop the three v0.6.3 deprecation shims (`interval`, `temporal_cospan_chain`, `trace`) per CHANGELOG promise | Not a paper-fidelity item; tracked in CHANGELOG. |
| **v0.7.0** | Action item I-2: separate `DiscreteInterval::identity(s)` (singleton, Z'(id_X) per Eq 13) from `map_step(s)` (1-step interval) | Paper Eq 13. Wire through `catgraph_physics::interval::DiscreteInterval::singleton`. |
| **v0.7.0** | Action item M-2: rename `Direction::Stay → Direction::Id` (or `Forward`) and emit `id` (or `F`) in `Display` to match paper Eqs 1, 8 | Naming gap. |
| **v0.7.0** | Action item M-3: add explicit unit-law test on `Transition` per Eqs 5–7 | One test. |
| **v0.7.0** | Action item M-10/M-11: add paper-fixture rules (rule 2506, rule 3506, set-substitution-Fig-12) as `TuringMachine` / `RewriteRule` constants | Lets the audit's Figures-as-tests gates flip ✅. |
| **v0.7.0** | Action item I-1: rename `is_irreducible` → `is_no_repeat_shortcut` OR document the gap from Def 1 | Naming / scope clarity. |
| **v0.7.0** | Action item I-6: replace `Complexity::parallel = max` with paper-faithful subadditivity check (paper Eqs 67–69) | Largest paper-fidelity gap in §3; potentially blocking if a downstream consumer claims "this crate verifies multicomputational irreducibility per Gorard". |
| **v0.8.0+** | Action item A-1: surface the architectural drift in `ZPrimeAdjunction` triangle identities. Either rename the trait or add a `caveat:` doc explaining the implementation is a one-sided self-roundtrip, not the paper's 𝒯 ⇄ 𝓥ect adjunction | Architectural. |
| **v0.8.0+** | Action item A-2: clarify framing of `cup`/`cap`/`name`/`unname` re-exports — F&S compact-closed cospans vs paper's FdVect compact closure (Eqs 138–144) | Architectural. |
| **v0.8.0+** | Action item I-3: implement boundary endofunctor ∂ + ∂² = ∅ verifier on `DiscreteInterval` per Eqs 22–23 | Required substrate for any honest §4 adjunction work. |
| **v0.8.0+** | Action item I-4: explicit HALT unit object + ε coherence map (paper Eq 58, 59) | §3 paper-fidelity. |
| **v0.8.0+** | Hypergraph dual `H*` per Eqs 147–148 (vertex-edge swap) | One-line addition; deferred for sequencing. |
| **v0.8.0+** | Action item M-4: universality check for coproduct injection morphisms (Eqs 17–21) | One verifier. |
| **v0.8.0+** | Action item M-5: explicit hexagon coherence test for braiding (Eq 50) on a fixture multiway graph | One test. |
| **v0.8.0+** | Action item M-6: functor-coherence variant of Eq 66–67 (vs current multiway-graph variant) | One test. |
| **v0.8.0+** | Action item M-7: per-machine `state_equivalence_complexity` annotation surfacing the paper's evolution-vs-equivalence orthogonality | API addition. |
| **v0.8.0+** | Action item M-8: monomorphism check on `RewriteSpan::l, r` (Eqs 72–74) | One verifier. |
| **v0.8.0+** | Action item M-9: first-class `concurrent_composition` and `parallel_composition` on `RewriteRule` (Eqs 109–110) | API + tests. |

---

## 7. Action items (summary)

Promoted from §2–§5 above. Triage classes:

| # | Tag | Severity | Description | Cycle |
|---|---|---|---|---|
| I-1 | important | rename / scope | `is_irreducible` API approximates Def 1 only via cycle detection — overclaim risk | v0.7.0 patch |
| I-2 | important | paper-fidelity | `Z'(id_X)` should be singleton `[s,s]` (Eq 13), not `[s,s+1]` | v0.7.0 |
| I-3 | important | substrate | Boundary endofunctor ∂ on `DiscreteInterval` + ∂²=∅ verifier (Eqs 22–25) | v0.8.0+ |
| I-4 | important | paper-fidelity | HALT unit object + ε coherence map (Eq 58, 59) | v0.8.0+ |
| I-5 | important | naming | Lax / strong / strict monoidal functor flavour not surfaced in API | v0.7.0 doc |
| I-6 | **blocking-for-claim** | paper-fidelity | `Complexity::parallel = max` contradicts paper additivity (Eqs 67–69) | v0.7.0 |
| A-1 | architectural | adjunction | Triangle identities verify a self-roundtrip, not paper's 𝒯 ⇄ 𝓥ect | v0.7.0 doc; fix v0.8.0+ |
| A-2 | architectural | naming | Compact-closed cup/cap re-exports anchored at F&S, not paper §4 Eqs 138–144 | v0.7.0 doc |
| M-1 through M-11 | minor | various | See "Deferred to future versions" | v0.7.0 / v0.8.0+ |

---

*Maintained alongside `irreducible/CHANGELOG.md`. Any new release that adds a paper-anchored item should update both files.*
