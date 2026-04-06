#![cfg(feature = "manifold-curvature")]
//! Integration tests for the manifold curvature pipeline.
//!
//! Tests the full pipeline: `StringRewriteSystem` -> multiway evolution ->
//! branchial foliation -> MDS embedding -> Riemannian curvature, plus
//! `DiscreteCurvature` trait conformance and edge cases.

use irreducible::machines::multiway::{ManifoldCurvature, ShortestPathMDS, StringRewriteSystem};
use irreducible::{extract_branchial_foliation, DiscreteCurvature};

// ---------------------------------------------------------------------------
// Full pipeline: SRS -> multiway -> branchial -> manifold curvature
// ---------------------------------------------------------------------------

#[test]
fn srs_multiway_to_manifold_curvature_pipeline() {
    let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
    let evolution = srs.run_multiway("AB", 4, 200);

    let foliation = extract_branchial_foliation(&evolution);
    assert!(!foliation.is_empty(), "foliation must have at least one step");

    let embedding = ShortestPathMDS::<3>;

    for branchial in &foliation {
        if branchial.node_count() <= 1 {
            continue;
        }

        let curvature = ManifoldCurvature::from_branchial(branchial, &embedding);

        // dimension() returns the number of branchial graph nodes
        assert_eq!(curvature.dimension(), branchial.node_count());

        // step() matches the branchial time step
        assert_eq!(curvature.step(), branchial.step);

        // Scalar curvature must be finite (no NaN / Inf)
        assert!(
            curvature.scalar_curvature().is_finite(),
            "scalar curvature must be finite at step {}",
            branchial.step,
        );

        // ShortestPathMDS embeds into flat Euclidean space (identity metric),
        // so all curvatures are zero and is_flat() must hold.
        assert!(
            curvature.is_flat(),
            "Euclidean MDS embedding should be flat at step {}",
            branchial.step,
        );
    }
}

// ---------------------------------------------------------------------------
// DiscreteCurvature trait method conformance
// ---------------------------------------------------------------------------

#[test]
fn manifold_curvature_trait_methods() {
    let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
    let evolution = srs.run_multiway("AB", 4, 200);

    let foliation = extract_branchial_foliation(&evolution);

    // Find the first branchial with at least 2 nodes
    let branchial = foliation
        .iter()
        .find(|b| b.node_count() > 1)
        .expect("should have at least one branchial with >1 node");

    let curvature = ManifoldCurvature::from_branchial(branchial, &ShortestPathMDS::<3>);

    // Ricci curvature at each vertex must be finite
    for v in 0..curvature.dimension() {
        assert!(
            curvature.ricci_curvature(v).is_finite(),
            "ricci_curvature({v}) must be finite",
        );
    }

    // Sectional curvature between vertex pairs must be finite
    if curvature.dimension() >= 2 {
        assert!(
            curvature.sectional_curvature(0, 1).is_finite(),
            "sectional_curvature(0, 1) must be finite",
        );
    }

    // Irreducibility indicator is non-negative
    assert!(
        curvature.irreducibility_indicator() >= 0.0,
        "irreducibility_indicator must be >= 0.0",
    );
}

// ---------------------------------------------------------------------------
// Edge cases: empty and single-node branchial graphs
// ---------------------------------------------------------------------------

#[test]
fn manifold_curvature_empty_and_single_node() {
    use irreducible::machines::multiway::BranchialGraph;

    // --- Empty branchial graph ---
    let empty = BranchialGraph {
        step: 0,
        nodes: Vec::new(),
        edges: Vec::new(),
    };
    let empty_curvature = ManifoldCurvature::from_branchial(&empty, &ShortestPathMDS::<3>);

    assert_eq!(empty_curvature.dimension(), 0);
    assert!(empty_curvature.is_flat());
    assert!(empty_curvature.scalar_curvature().is_finite());
    assert!((empty_curvature.irreducibility_indicator() - 0.0).abs() < f64::EPSILON);

    // --- Single-node branchial graph ---
    use irreducible::machines::multiway::{BranchId, MultiwayNodeId};

    let single_node_id = MultiwayNodeId::new(BranchId(0), 0);
    let single = BranchialGraph {
        step: 1,
        nodes: vec![single_node_id],
        edges: Vec::new(),
    };
    let single_curvature = ManifoldCurvature::from_branchial(&single, &ShortestPathMDS::<3>);

    assert_eq!(single_curvature.dimension(), 1);
    assert!(single_curvature.is_flat());
    assert!(single_curvature.scalar_curvature().is_finite());
    assert_eq!(single_curvature.step(), 1);
}
