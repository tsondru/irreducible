//! Phase 0 smoke tests for the `dc-geometry` feature.
//!
//! These tests exercise the `deep_causality_topology` substrate end-to-end:
//! a hand-built triangle (3 vertices / 3 edges / 1 face) and the Phase 0
//! edge-only bridge from `BranchialGraph` to `SimplicialComplex`.
//!
//! See `.claude/plans/2026-04-14-dc-topology-substrate.md` for the phased plan.

#![cfg(feature = "dc-geometry")]

use catgraph_physics::multiway::BranchialGraph;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    Manifold, ManifoldTopology, Simplex, SimplicialComplex, SimplicialComplexBuilder,
};
use irreducible::StringRewriteSystem;
use irreducible::geometry::dc_bridge::branchial_to_simplicial;
use irreducible::geometry::hodge::unit_hodge_operators;
use irreducible::geometry::regge_metric::flat_regge_geometry;

/// End-to-end check on a flat equilateral triangle:
/// - Euler characteristic of a closed triangle (with its face) is 1.
/// - All deficit angles at interior bones are numerically zero.
/// - `exterior_derivative(0)` produces a tensor whose shape matches the 1-skeleton.
#[test]
fn flat_triangle_has_euler_char_one_and_zero_curvature() {
    // Build a 2D triangle: adding the 2-simplex {0,1,2} auto-adds its 3 edges
    // and 3 vertices via SimplicialComplexBuilder's closure property.
    let mut builder = SimplicialComplexBuilder::new(2);
    builder
        .add_simplex(Simplex::new(vec![0, 1, 2]))
        .expect("adding triangle should succeed");
    let base: SimplicialComplex<f64> = builder.build().expect("complex build");

    // deep_causality_topology 0.6 validates the Hodge ⋆ surface eagerly at
    // `Manifold::with_metric`; a builder-built complex carries no coordinates,
    // so pre-supply unit Hodge operators (this smoke test asserts topology and
    // Regge curvature only — no assertion depends on ⋆ values).
    let complex = SimplicialComplex::new(
        base.skeletons().clone(),
        base.boundary_operators().clone(),
        base.coboundary_operators().clone(),
        unit_hodge_operators(&base),
    );

    // Flat metric: 3 edges of length 1.
    let metric = flat_regge_geometry(3).expect("flat metric should construct");

    // Manifold data: one scalar per simplex (3 vertices + 3 edges + 1 triangle = 7).
    // The data tensor is required by Manifold::with_metric; contents are not
    // exercised by this smoke test.
    let data = CausalTensor::new(vec![0.0_f64; 7], vec![7]).expect("data tensor");
    let manifold = Manifold::with_metric(complex.clone(), data, Some(metric.clone()), 0)
        .expect("manifold construction on flat triangle");

    // Topology sanity: closed triangle has chi = 1 (V - E + F = 3 - 3 + 1).
    assert_eq!(manifold.euler_characteristic(), 1);

    // Curvature sanity: every deficit angle must be ~0 on a flat (equilateral,
    // unit-length) triangle. Interior bones sum to 2pi; boundary bones are
    // zeroed by the dc_topology implementation.
    let ricci = metric
        .calculate_ricci_curvature(&complex)
        .expect("ricci on flat triangle");
    for v in ricci.as_slice() {
        assert!(
            v.abs() < 1e-10,
            "flat triangle should have zero deficit everywhere, got {v}"
        );
    }

    // DEC smoke: exterior_derivative(0) maps a 0-form (one coeff per vertex)
    // to a 1-form (one coeff per edge). We don't assert on numeric values —
    // Phase 2 will cover DEC semantics in depth.
    let d0 = manifold.exterior_derivative(0);
    assert_eq!(
        d0.len(),
        3,
        "d of a 0-form should produce one coefficient per 1-simplex"
    );
}

/// The Phase 0 bridge (edge-only) produces a complex with the expected number
/// of 0- and 1-simplices from a tiny multiway evolution. No 2-simplices are
/// asserted — they're deferred to Phase 2.
#[test]
fn branchial_bridge_produces_valid_complex() {
    // Minimal multiway evolution: a single rule with one rewrite, run a few
    // steps so a BranchialGraph at step > 0 has nodes to project.
    let srs = StringRewriteSystem::new(vec![("A", "AA")]);
    let evolution = srs.run_multiway("A", 3, 16);

    // Walk steps until we find one with >= 2 nodes (so the bridge sees more
    // than a singleton). For "A" -> "AA" this is typically step 1 or 2.
    let branchial = (0..=3)
        .map(|step| BranchialGraph::from_evolution_at_step(&evolution, step))
        .find(|bg| !bg.nodes.is_empty())
        .expect("evolution should have at least one non-empty step");

    let complex =
        branchial_to_simplicial(&branchial).expect("Phase 0 bridge should succeed on SRS");

    // 0-skeleton vertex count matches the branchial node count.
    let vertex_count = complex.skeletons()[0].simplices().len();
    assert_eq!(vertex_count, branchial.nodes.len());

    // 1-skeleton edge count is at most the branchial edge count (self-loops
    // are filtered; BranchialGraph does not generally emit them but the
    // bridge guards against it defensively).
    let edge_count = complex.skeletons()[1].simplices().len();
    assert!(
        edge_count <= branchial.edges.len(),
        "bridge produced more 1-simplices ({edge_count}) than branchial edges ({})",
        branchial.edges.len()
    );

    // Phase 0 invariant: no 2-simplices yet.
    assert_eq!(
        complex.skeletons()[2].simplices().len(),
        0,
        "Phase 0 bridge must not emit 2-simplices"
    );
}
