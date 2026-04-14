//! Hodge star operator constructors for 2D simplicial complexes with unit edge
//! lengths.
//!
//! `deep_causality_topology::SimplicialComplexBuilder::build` produces a
//! complex with empty `hodge_star_operators`. The DEC pipeline
//! (`codifferential`, `laplacian`) requires those operators to be populated,
//! so this module supplies them for the flat / unit-edge-length case.
//!
//! # Unit equilateral geometry
//!
//! We assume all 1-simplices have length `1` and all 2-simplices are
//! equilateral triangles of area `√3 / 4`. For this geometry:
//!
//! - **`⋆₀`**: diagonal `V × V`. Each vertex `v` has dual volume
//!   `(√3 / 12) × (number of incident 2-simplices)`. That is `1/3 · area`
//!   contributed by each incident triangle.
//! - **`⋆₁`**: diagonal `E × E`. Each edge `e` has dual-length-to-primal-length
//!   ratio `|*e| / |e|`. For equilateral unit triangles, the circumcenter is
//!   at distance `√3 / 6` from each edge midpoint. So a **boundary** edge
//!   (one incident triangle) has `|*e| = √3 / 6`; an **interior** edge (two
//!   incident triangles) has `|*e| = √3 / 3`. With `|e| = 1`, the diagonal
//!   entries are those values verbatim.
//! - **`⋆₂`**: diagonal `F × F`. Each triangle has dual 0-volume
//!   `1 / area = 4 / √3`.
//!
//! # Output type
//!
//! All three operators are returned as `deep_causality_sparse::CsrMatrix<f64>`
//! — the same type `SimplicialComplex::new` accepts in its
//! `hodge_star_operators` slot.

use deep_causality_sparse::CsrMatrix;
use deep_causality_topology::SimplicialComplex;

/// Area of a unit-edge equilateral triangle: `√3 / 4`.
const AREA_UNIT_EQUILATERAL: f64 = 0.433_012_701_892_219_3;

/// One-third of the area: contribution per vertex per incident triangle.
const AREA_THIRD: f64 = AREA_UNIT_EQUILATERAL / 3.0;

/// Circumcenter-to-edge-midpoint distance for a unit equilateral triangle:
/// `√3 / 6`.
const INRADIUS: f64 = 0.288_675_134_594_812_9;

/// Dual length for a **boundary** edge (one incident triangle): `√3 / 6`.
const DUAL_LEN_BOUNDARY: f64 = INRADIUS;

/// Dual length for an **interior** edge (two incident triangles): `√3 / 3`.
const DUAL_LEN_INTERIOR: f64 = 2.0 * INRADIUS;

/// Build the `⋆₀` (0-form → 2-form dual) Hodge star operator on a 2D complex
/// with unit-edge equilateral triangles.
///
/// The operator is diagonal `V × V` where `V` is the number of 0-simplices.
/// Each vertex entry equals `(√3 / 12) × (number of 2-simplices containing that
/// vertex)`. Isolated vertices (with no incident triangles) receive the value
/// `0.0`; we do not emit a zero-valued triplet for them — `⋆₀` is truly empty
/// on the corresponding row.
///
/// # Panics
///
/// Does not panic in practice: `from_triplets` only fails on out-of-bounds
/// indices, and every emitted triplet has `i < v_count`.
#[must_use]
pub fn unit_hodge_0(complex: &SimplicialComplex<f64>) -> CsrMatrix<f64> {
    let skeletons = complex.skeletons();
    let v_count = skeletons
        .first()
        .map_or(0, |skel| skel.simplices().len());

    let mut per_vertex_weight = vec![0.0_f64; v_count];
    if let Some(tri_skel) = skeletons.get(2) {
        for tri in tri_skel.simplices() {
            for &v in tri.vertices() {
                if v < v_count {
                    per_vertex_weight[v] += AREA_THIRD;
                }
            }
        }
    }

    let triplets: Vec<(usize, usize, f64)> = per_vertex_weight
        .iter()
        .enumerate()
        .filter(|(_, w)| w.abs() > 0.0)
        .map(|(i, w)| (i, i, *w))
        .collect();

    CsrMatrix::from_triplets(v_count, v_count, &triplets)
        .expect("diagonal triplets always have valid (row, col) within (V, V)")
}

/// Build the `⋆₁` (1-form → 1-form dual) Hodge star operator.
///
/// Diagonal `E × E` matrix. Each edge's entry is `√3 / 6` if it borders
/// exactly one 2-simplex (boundary edge), `√3 / 3` if it borders two
/// (interior edge), and `0` otherwise (edge with no incident triangle —
/// typical on a purely 1-dimensional stratum, or for diagnostic edges that
/// failed closure).
///
/// # Panics
///
/// Does not panic in practice: `from_triplets` only fails on out-of-bounds
/// indices, and every emitted triplet has `i < e_count`.
#[must_use]
pub fn unit_hodge_1(complex: &SimplicialComplex<f64>) -> CsrMatrix<f64> {
    let skeletons = complex.skeletons();
    let e_count = skeletons
        .get(1)
        .map_or(0, |skel| skel.simplices().len());

    let mut incident_tri_count = vec![0_usize; e_count];
    if let (Some(edge_skel), Some(tri_skel)) = (skeletons.get(1), skeletons.get(2)) {
        for tri in tri_skel.simplices() {
            // Equivalent to enumerating the 3 edges of the triangle.
            let v = tri.vertices();
            for &(i, j) in &[(0_usize, 1_usize), (0, 2), (1, 2)] {
                let edge = deep_causality_topology::Simplex::new(vec![v[i], v[j]]);
                if let Some(e_idx) = edge_skel.get_index(&edge) {
                    incident_tri_count[e_idx] += 1;
                }
            }
        }
    }

    let triplets: Vec<(usize, usize, f64)> = incident_tri_count
        .iter()
        .enumerate()
        .filter_map(|(i, &n)| match n {
            1 => Some((i, i, DUAL_LEN_BOUNDARY)),
            2 => Some((i, i, DUAL_LEN_INTERIOR)),
            _ => None,
        })
        .collect();

    CsrMatrix::from_triplets(e_count, e_count, &triplets)
        .expect("diagonal triplets always have valid (row, col) within (E, E)")
}

/// Build the `⋆₂` (2-form → 0-form dual) Hodge star operator.
///
/// Diagonal `F × F` matrix with every entry equal to `1 / area = 4 / √3`.
///
/// # Panics
///
/// Does not panic in practice: `from_triplets` only fails on out-of-bounds
/// indices, and every emitted triplet has `i < f_count`.
#[must_use]
pub fn unit_hodge_2(complex: &SimplicialComplex<f64>) -> CsrMatrix<f64> {
    let skeletons = complex.skeletons();
    let f_count = skeletons
        .get(2)
        .map_or(0, |skel| skel.simplices().len());

    let inv_area = 1.0 / AREA_UNIT_EQUILATERAL;
    let triplets: Vec<(usize, usize, f64)> = (0..f_count).map(|i| (i, i, inv_area)).collect();

    CsrMatrix::from_triplets(f_count, f_count, &triplets)
        .expect("diagonal triplets always have valid (row, col) within (F, F)")
}

/// Build the full `[⋆₀, ⋆₁, ⋆₂]` Hodge star stack for a 2D simplicial complex
/// with unit-edge equilateral geometry.
///
/// This is the exact type `SimplicialComplex::new(..., hodge_star_operators)`
/// expects. Rebuild the complex with these operators to enable
/// `Manifold::codifferential`, `Manifold::hodge_star`, and
/// `Manifold::laplacian`.
#[must_use]
pub fn unit_hodge_operators(complex: &SimplicialComplex<f64>) -> Vec<CsrMatrix<f64>> {
    vec![
        unit_hodge_0(complex),
        unit_hodge_1(complex),
        unit_hodge_2(complex),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use deep_causality_topology::{Simplex, SimplicialComplexBuilder};

    fn single_triangle() -> SimplicialComplex<f64> {
        let mut b = SimplicialComplexBuilder::new(2);
        b.add_simplex(Simplex::new(vec![0, 1, 2]))
            .expect("single triangle fits the builder");
        b.build::<f64>().expect("single-triangle build")
    }

    fn two_triangles_sharing_edge() -> SimplicialComplex<f64> {
        // Triangles {0,1,2} and {1,2,3} share edge {1,2}.
        // Boundary edges (1 incident tri): {0,1}, {0,2}, {1,3}, {2,3}  → 4 boundary edges.
        // Interior edge (2 incident tris): {1,2}                         → 1 interior edge.
        let mut b = SimplicialComplexBuilder::new(2);
        b.add_simplex(Simplex::new(vec![0, 1, 2])).unwrap();
        b.add_simplex(Simplex::new(vec![1, 2, 3])).unwrap();
        b.build::<f64>().expect("two-triangle build")
    }

    fn csr_diag(mat: &CsrMatrix<f64>, i: usize) -> f64 {
        let start = mat.row_indices()[i];
        let end = mat.row_indices()[i + 1];
        for k in start..end {
            if mat.col_indices()[k] == i {
                return mat.values()[k];
            }
        }
        0.0
    }

    #[test]
    fn unit_hodge_0_on_single_triangle_is_identity_scaled() {
        // Each of the 3 vertices is incident to exactly 1 triangle.
        // Expected diagonal: area / 3 = √3 / 12 ≈ 0.144_337_567.
        let complex = single_triangle();
        let star0 = unit_hodge_0(&complex);

        assert_eq!(star0.shape(), (3, 3));
        for v in 0..3 {
            assert!(
                (csr_diag(&star0, v) - AREA_THIRD).abs() < 1e-12,
                "vertex {v} star0 expected {AREA_THIRD}, got {}",
                csr_diag(&star0, v)
            );
        }
    }

    #[test]
    fn unit_hodge_2_is_4_over_sqrt3() {
        let complex = single_triangle();
        let star2 = unit_hodge_2(&complex);

        assert_eq!(star2.shape(), (1, 1));
        let expected = 1.0 / AREA_UNIT_EQUILATERAL;
        assert!(
            (csr_diag(&star2, 0) - expected).abs() < 1e-12,
            "star2 on single tri expected {expected}, got {}",
            csr_diag(&star2, 0)
        );
    }

    #[test]
    fn unit_hodge_1_boundary_vs_interior() {
        let complex = two_triangles_sharing_edge();
        let star1 = unit_hodge_1(&complex);

        let edge_skel = &complex.skeletons()[1];
        assert_eq!(edge_skel.simplices().len(), 5, "4 boundary + 1 interior");
        assert_eq!(star1.shape(), (5, 5));

        // Locate the interior edge {1,2}.
        let interior_idx = edge_skel
            .get_index(&Simplex::new(vec![1, 2]))
            .expect("edge {1,2} must exist");

        // Boundary edges (the other 4).
        let interior_val = csr_diag(&star1, interior_idx);
        assert!(
            (interior_val - DUAL_LEN_INTERIOR).abs() < 1e-12,
            "interior edge star1 expected {DUAL_LEN_INTERIOR}, got {interior_val}"
        );

        for e_idx in 0..5 {
            if e_idx == interior_idx {
                continue;
            }
            let v = csr_diag(&star1, e_idx);
            assert!(
                (v - DUAL_LEN_BOUNDARY).abs() < 1e-12,
                "boundary edge {e_idx} star1 expected {DUAL_LEN_BOUNDARY}, got {v}"
            );
        }
    }

    #[test]
    fn unit_hodge_operators_returns_stack_of_three() {
        let complex = single_triangle();
        let ops = unit_hodge_operators(&complex);
        assert_eq!(ops.len(), 3, "⋆₀, ⋆₁, ⋆₂");
        assert_eq!(ops[0].shape(), (3, 3));
        assert_eq!(ops[1].shape(), (3, 3));
        assert_eq!(ops[2].shape(), (1, 1));
    }
}
