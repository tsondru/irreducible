//! Construction helpers for [`ReggeGeometry<f64>`].
//!
//! # Phase 0 scope
//!
//! Only a **flat baseline** is provided: all edge lengths are `1.0`. This
//! matches the current observable behavior of [`crate::machines::multiway::ManifoldCurvature`]
//! (identity-metric `ShortestPathMDS` embedding produces uniformly-flat local
//! geometry).
//!
//! Non-flat edge-length assignments (spherical / hyperbolic Regge metrics that
//! induce *systematically* nonzero curvature from the branchial structure
//! alone) become the new Phase 2 of Deferred Work #9 in `CLAUDE.md`. They are
//! out of scope for the Phase 0 `dc_topology` port and will ship in a later
//! plan.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ReggeGeometry, TopologyError};

/// Build a flat [`ReggeGeometry<f64>`] on `edge_count` edges, all of length
/// `1.0`.
///
/// # Errors
///
/// Propagates [`TopologyError`] if [`CausalTensor::new`] rejects the
/// `(data, shape)` pair. Under the invariant that `data.len() == edge_count`,
/// this only fails if the underlying tensor crate changes its validation
/// contract.
#[must_use = "the returned Result carries the constructed metric or a topology error"]
pub fn flat_regge_geometry(edge_count: usize) -> Result<ReggeGeometry<f64>, TopologyError> {
    let edge_lengths = CausalTensor::new(vec![1.0_f64; edge_count], vec![edge_count])
        .map_err(TopologyError::from)?;
    Ok(ReggeGeometry::new(edge_lengths))
}
