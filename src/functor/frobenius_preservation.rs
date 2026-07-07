//! Frobenius preservation for Z': T → B (F&S §2.3 Eq. 12; issue #12).
//!
//! ## Evaluation outcome (acceptance item 1)
//!
//! Both categories are structured as hypergraph categories **by encoding
//! into the free one** (Thm 3.14): T-morphisms are the multiway step
//! cospans of [`multiway_step_cospans`](super::interval_algebra::multiway_step_cospans)
//! and B-morphisms are cobordism cospans — both `Cospan<u32>`, which
//! carries the `HypergraphCategory` impl. Under this encoding the Frobenius
//! generators of T **are the multiway event types**:
//!
//! | generator | multiway event |
//! |---|---|
//! | μ (multiplication) | merge (two branches reach one state) |
//! | δ (comultiplication) | fork (one state rewrites two ways) |
//! | ε (counit) | branch death (no continuation) |
//! | η (unit) | branch creation (the root; never mid-evolution) |
//!
//! Z' itself acts on these shapes as the **identity hypergraph functor**
//! (label map `id`, morphism map `clone`): Gorard's Z' forgets the
//! computational content of states but preserves the entire boundary /
//! event structure — the interval content lives in the cospan-algebra
//! action ([`IntervalCospanAlgebra`](super::interval_algebra::IntervalCospanAlgebra)),
//! not in the cospan shape. The trait laws (Eq. 12, functoriality,
//! monoidality) are not enforced by the compiler; [`verify_frobenius_preservation`]
//! checks them where they are decidable:
//!
//! 1. **Eq. 12 on generators**, twice: at the cospan level
//!    (`map_mor(gen_T) ≟ gen_B`, via `structurally_equal`) and through
//!    [`CospanToFrobeniusFunctor`] (Prop 3.8: the decomposition of a
//!    generator cospan must be that generator, via `FrobeniusMorphism`
//!    equality).
//! 2. **Spider factorization per event**: every apex component (l parents,
//!    r children) must equal the pushout composite of its Frobenius
//!    generator recipe — (l−1) multiplications, then either a counit
//!    (r = 0, branch death) or (r−1) comultiplications. This is the
//!    falsifiable Prop 3.8 content at cospan level.
//! 3. **Boundary preservation through decomposition**: `cospan_to_frobenius`
//!    of each step cospan reproduces the cospan's boundaries (the decidable
//!    slice — the free hypergraph category has no diagram normal form
//!    upstream, so full string-diagram equality is out of reach; same
//!    limitation recorded on issues #13/#11).
//!
//! Whole-step reassembly (⊗ of component spiders ≟ step cospan) is *not*
//! checked: branchial node order may interleave components, so reassembly
//! requires braiding permutations — deferred until a consumer needs it.

use std::hash::Hash;

use catgraph::category::{Composable, ComposableMutating, HasIdentity};
use catgraph::cospan::Cospan;
use catgraph::errors::CatgraphError;
use catgraph::frobenius::FrobeniusMorphism;
use catgraph::hypergraph_category::HypergraphCategory;
use catgraph::hypergraph_functor::{CospanToFrobeniusFunctor, HypergraphFunctor};
use catgraph::monoidal::Monoidal;
use catgraph_physics::multiway::MultiwayEvolutionGraph;

use super::IrreducibilityFunctor;
use super::interval_algebra::multiway_step_cospans;

/// Z' as a hypergraph functor on the free hypergraph category (Thm 3.14):
/// identity on labels, identity on cospan shapes. See the
/// [module docs](self) for why this is the honest encoding of Gorard's Z'
/// at the shape level.
impl HypergraphFunctor<u32, u32, Cospan<u32>, Cospan<u32>> for IrreducibilityFunctor {
    fn map_ob(&self, x: u32) -> u32 {
        x
    }

    fn map_mor(&self, f: &Cospan<u32>) -> Result<Cospan<u32>, CatgraphError> {
        Ok(f.clone())
    }
}

/// Result of Frobenius-preservation verification for Z' (Eq. 12).
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct FrobeniusPreservationResult {
    /// Eq. 12 for η at the cospan level and through the Prop 3.8 functor.
    pub unit_preserved: bool,
    /// Eq. 12 for ε.
    pub counit_preserved: bool,
    /// Eq. 12 for μ.
    pub multiplication_preserved: bool,
    /// Eq. 12 for δ.
    pub comultiplication_preserved: bool,
    /// `cospan_to_frobenius` of every step cospan reproduces its boundaries.
    pub decomposition_boundaries_preserved: bool,
    /// Per-step spider-factorization checks.
    pub per_step: Vec<StepFrobeniusCheck>,
}

impl FrobeniusPreservationResult {
    /// True when every generator equation, boundary check, and per-step
    /// factorization holds.
    #[must_use]
    pub fn all_hold(&self) -> bool {
        self.unit_preserved
            && self.counit_preserved
            && self.multiplication_preserved
            && self.comultiplication_preserved
            && self.decomposition_boundaries_preserved
            && self.per_step.iter().all(|s| s.factorizations_hold)
    }
}

/// Spider-factorization check for one evolution step.
#[derive(Clone, Debug)]
pub struct StepFrobeniusCheck {
    /// The time step.
    pub step: usize,
    /// Number of apex components (events) checked at this step.
    pub components_checked: usize,
    /// Whether every event factored through its Frobenius generator recipe.
    pub factorizations_hold: bool,
}

/// The single label of the single-typed step-cospan encoding.
const Z: u32 = 0;

/// Build the Frobenius generator recipe for an (l, r) spider in `Cospan`:
/// (l−1) multiplications folding l wires to one, then a counit if `r == 0`
/// or (r−1) comultiplications expanding to r wires.
///
/// Requires `l >= 1` (multiway events always have at least one parent).
fn spider_recipe(l: usize, r: usize) -> Result<Cospan<u32>, CatgraphError> {
    let mut composite: Cospan<u32> = Cospan::identity(&vec![Z; l]);

    // Fold: w wires -> w-1 wires via mu ⊗ id^(w-2).
    let mut w = l;
    while w > 1 {
        let mut layer = Cospan::multiplication(Z);
        layer.monoidal(Cospan::identity(&vec![Z; w - 2]));
        composite = composite.compose(&layer)?;
        w -= 1;
    }

    if r == 0 {
        // Branch death: cap the surviving wire.
        composite = composite.compose(&Cospan::counit(Z))?;
    } else {
        // Expand: w wires -> w+1 wires via delta ⊗ id^(w-1).
        while w < r {
            let mut layer = Cospan::comultiplication(Z);
            layer.monoidal(Cospan::identity(&vec![Z; w - 1]));
            composite = composite.compose(&layer)?;
            w += 1;
        }
    }

    Ok(composite)
}

/// The (l, r) spider as a single-apex cospan — the shape every multiway
/// event takes inside a step cospan.
fn spider_cospan(l: usize, r: usize) -> Cospan<u32> {
    Cospan::new(vec![0; l], vec![0; r], vec![Z])
}

/// Eq. 12 for one generator: the functor's image at the cospan level must
/// be the target generator, and the Prop 3.8 decomposition of the generator
/// cospan must be the generator `FrobeniusMorphism`.
fn generator_preserved(
    functor: &IrreducibilityFunctor,
    decomposer: &CospanToFrobeniusFunctor<()>,
    source: &Cospan<u32>,
    target_frobenius: &FrobeniusMorphism<u32, ()>,
) -> Result<bool, CatgraphError> {
    let mapped: Cospan<u32> = functor.map_mor(source)?;
    let cospan_level = mapped.structurally_equal(source);

    let decomposed: FrobeniusMorphism<u32, ()> = decomposer.map_mor(&mapped)?;
    let frobenius_level = decomposed == *target_frobenius;

    Ok(cospan_level && frobenius_level)
}

/// Verify that Z' preserves Frobenius structure on a multiway evolution
/// (F&S §2.3 Eq. 12 + Prop 3.8). See the [module docs](self) for exactly
/// what is decidable and checked.
///
/// # Errors
///
/// Propagates [`CatgraphError`] from cospan composition or decomposition;
/// also errors if a step cospan contains an event with no parent branch
/// (a spontaneous creation mid-evolution, which multiway BFS never
/// produces).
pub fn verify_frobenius_preservation<S: Clone + Hash, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
) -> Result<FrobeniusPreservationResult, CatgraphError> {
    let functor = IrreducibilityFunctor::new();
    let decomposer = CospanToFrobeniusFunctor::<()>::new();

    // Eq. 12 on the four generators.
    let unit_preserved = generator_preserved(
        &functor,
        &decomposer,
        &Cospan::unit(Z),
        &FrobeniusMorphism::unit(Z),
    )?;
    let counit_preserved = generator_preserved(
        &functor,
        &decomposer,
        &Cospan::counit(Z),
        &FrobeniusMorphism::counit(Z),
    )?;
    let multiplication_preserved = generator_preserved(
        &functor,
        &decomposer,
        &Cospan::multiplication(Z),
        &FrobeniusMorphism::multiplication(Z),
    )?;
    let comultiplication_preserved = generator_preserved(
        &functor,
        &decomposer,
        &Cospan::comultiplication(Z),
        &FrobeniusMorphism::comultiplication(Z),
    )?;

    // Per-step: boundary preservation through decomposition + spider
    // factorization per apex component.
    let chain = multiway_step_cospans(graph);
    let mut decomposition_boundaries_preserved = true;
    let mut per_step = Vec::with_capacity(chain.len());

    for (step, cospan) in chain.iter().enumerate() {
        let image: Cospan<u32> = functor.map_mor(cospan)?;

        let decomposed: FrobeniusMorphism<u32, ()> = decomposer.map_mor(&image)?;
        let expected_domain = vec![Z; image.left_to_middle().len()];
        let expected_codomain = vec![Z; image.right_to_middle().len()];
        if decomposed.domain() != expected_domain || decomposed.codomain() != expected_codomain {
            decomposition_boundaries_preserved = false;
        }

        // Census per apex vertex: (l parents, r children).
        let apex_count = image.middle().len();
        let mut lefts = vec![0usize; apex_count];
        let mut rights = vec![0usize; apex_count];
        for &a in image.left_to_middle() {
            lefts[a] += 1;
        }
        for &a in image.right_to_middle() {
            rights[a] += 1;
        }

        let mut factorizations_hold = true;
        let mut components_checked = 0;
        for (l, r) in lefts.iter().copied().zip(rights.iter().copied()) {
            if l == 0 && r == 0 {
                continue; // unreachable: apex vertices come from boundary nodes
            }
            if l == 0 {
                return Err(CatgraphError::Composition {
                    message: format!(
                        "step {step}: event with {r} children but no parent branch; \
                         spontaneous creation is not a multiway event"
                    ),
                });
            }
            components_checked += 1;
            let recipe = spider_recipe(l, r)?;
            if !recipe.structurally_equal(&spider_cospan(l, r)) {
                factorizations_hold = false;
            }
        }

        per_step.push(StepFrobeniusCheck {
            step,
            components_checked,
            factorizations_hold,
        });
    }

    Ok(FrobeniusPreservationResult {
        unit_preserved,
        counit_preserved,
        multiplication_preserved,
        comultiplication_preserved,
        decomposition_boundaries_preserved,
        per_step,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::multiway::StringRewriteSystem;

    #[test]
    fn generator_recipes_reproduce_spiders() {
        // Sequential event: identity wire.
        assert!(
            spider_recipe(1, 1)
                .unwrap()
                .structurally_equal(&spider_cospan(1, 1))
        );
        // Fork: delta.
        assert!(
            spider_recipe(1, 2)
                .unwrap()
                .structurally_equal(&spider_cospan(1, 2))
        );
        // Merge: mu.
        assert!(
            spider_recipe(2, 1)
                .unwrap()
                .structurally_equal(&spider_cospan(2, 1))
        );
        // Branch death: epsilon.
        assert!(
            spider_recipe(1, 0)
                .unwrap()
                .structurally_equal(&spider_cospan(1, 0))
        );
        // Wide events: 3-way fork, 3-way merge, merge-then-fork.
        assert!(
            spider_recipe(1, 3)
                .unwrap()
                .structurally_equal(&spider_cospan(1, 3))
        );
        assert!(
            spider_recipe(3, 1)
                .unwrap()
                .structurally_equal(&spider_cospan(3, 1))
        );
        assert!(
            spider_recipe(2, 2)
                .unwrap()
                .structurally_equal(&spider_cospan(2, 2))
        );
        // Dying merge: two parents, no children.
        assert!(
            spider_recipe(2, 0)
                .unwrap()
                .structurally_equal(&spider_cospan(2, 0))
        );
    }

    #[test]
    fn frobenius_preservation_on_forking_srs() {
        let srs = StringRewriteSystem::new(vec![("A", "AB"), ("A", "BA")]);
        let evolution = srs.run_multiway("A", 3, 32);
        let result = verify_frobenius_preservation(&evolution).expect("verification runs");
        assert!(result.unit_preserved);
        assert!(result.counit_preserved);
        assert!(result.multiplication_preserved);
        assert!(result.comultiplication_preserved);
        assert!(result.all_hold(), "{result:?}");
        // The fork at step 0 must have been checked as a real event.
        assert!(result.per_step[0].components_checked >= 1);
    }

    #[test]
    fn frobenius_preservation_on_merging_srs() {
        // Cyclic swap: branches re-converge on shared states (merges).
        let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("BA", "AB")]);
        let evolution = srs.run_multiway("AB", 4, 32);
        let result = verify_frobenius_preservation(&evolution).expect("verification runs");
        assert!(result.all_hold(), "{result:?}");
    }
}
