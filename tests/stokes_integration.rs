//! Integration tests for Stokes integration and temporal complexes.
//!
//! Tests TemporalComplex construction from TM and CA executions,
//! boundary operator properties, conservation law verification,
//! cospan chain generation, and error cases.
//!
//! NOTE (v0.4.1): exercises deprecated `exterior_derivative` and `is_closed`
//! (see `src/stokes.rs` module docs). Kept green until v0.4.3 Phase 2.5.
#![allow(deprecated)]

use catgraph::category::Composable;
use irreducible::{
    DiscreteInterval, ElementaryCA, StokesError, StokesIrreducibility, TemporalComplex,
    TuringMachine,
};

// ---------------------------------------------------------------------------
// Temporal complex from TM execution
// ---------------------------------------------------------------------------

#[test]
fn build_temporal_complex_from_tm_execution() {
    let bb = TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);

    let intervals = history.to_intervals();
    let complex = TemporalComplex::from_intervals(&intervals).unwrap();

    // 6 steps -> 7 time points, 6 intervals
    assert_eq!(complex.num_time_steps(), 7);
    assert_eq!(complex.num_intervals(), 6);
    assert_eq!(complex.time_points(), &[0, 1, 2, 3, 4, 5, 6]);
}

// ---------------------------------------------------------------------------
// Temporal complex from CA execution
// ---------------------------------------------------------------------------

#[test]
fn build_temporal_complex_from_ca_execution() {
    let ca = ElementaryCA::rule_30(11);
    let initial = ca.single_cell_initial();
    let history = ca.run(initial, 10);

    let intervals = history.to_intervals();
    let complex = TemporalComplex::from_intervals(&intervals).unwrap();

    assert_eq!(complex.num_time_steps(), 11); // 0 through 10
    assert_eq!(complex.num_intervals(), 10);
}

// ---------------------------------------------------------------------------
// Boundary operator properties
// ---------------------------------------------------------------------------

#[test]
fn exterior_derivative_is_zero_for_1d_complex() {
    let intervals = vec![
        DiscreteInterval::new(0, 3),
        DiscreteInterval::new(3, 7),
        DiscreteInterval::new(7, 10),
    ];

    let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    let form = complex.intervals_to_form();
    let d_form = complex.exterior_derivative(&form);

    // In dim-1, dw is vacuously zero (no 2-simplices)
    assert!(d_form.iter().all(|&c| c.abs() < 1e-10));
}

#[test]
fn form_coefficients_match_step_counts() {
    let intervals = vec![
        DiscreteInterval::new(0, 2),
        DiscreteInterval::new(2, 5),
        DiscreteInterval::new(5, 7),
    ];

    let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    let form = complex.intervals_to_form();

    assert_eq!(form, vec![2.0, 3.0, 2.0]);
    assert_eq!(complex.step_counts(), &[2, 3, 2]);
}

// ---------------------------------------------------------------------------
// Conservation law verification
// ---------------------------------------------------------------------------

#[test]
fn conservation_holds_for_contiguous_intervals() {
    let intervals = vec![
        DiscreteInterval::new(0, 1),
        DiscreteInterval::new(1, 2),
        DiscreteInterval::new(2, 3),
    ];

    let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    let result = complex.verify_conservation();

    assert!(result.is_conserved);
    assert!(result.is_closed);
    assert!(result.is_contiguous);
    assert!(result.is_monotonic);
    assert!((result.total_complexity - 3.0).abs() < 1e-10);
    assert!(result.is_well_formed());
}

#[test]
fn stokes_irreducibility_for_tm_execution() {
    let bb = TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);

    let intervals = history.to_intervals();
    let analysis = StokesIrreducibility::analyze(&intervals).unwrap();

    assert!(analysis.is_irreducible());
    assert!((analysis.conservation_ratio() - 1.0).abs() < 1e-10);
    assert!((analysis.integrated_complexity - 6.0).abs() < 1e-10);
}

#[test]
fn stokes_irreducibility_for_ca_execution() {
    let ca = ElementaryCA::rule_30(11);
    let initial = ca.single_cell_initial();
    let history = ca.run(initial, 10);

    let intervals = history.to_intervals();
    let analysis = StokesIrreducibility::analyze(&intervals).unwrap();

    assert!(analysis.is_irreducible());
    assert!((analysis.conservation_ratio() - 1.0).abs() < 1e-10);
}

#[test]
fn average_complexity_computation() {
    let intervals = vec![
        DiscreteInterval::new(0, 2),
        DiscreteInterval::new(2, 6),
        DiscreteInterval::new(6, 8),
    ];

    let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    let result = complex.verify_conservation();

    // Total = 8, intervals = 3, average = 8/3
    assert!((result.average_complexity() - 8.0 / 3.0).abs() < 1e-10);
}

// ---------------------------------------------------------------------------
// Cospan chain from temporal complex
// ---------------------------------------------------------------------------

#[test]
fn cospan_chain_from_temporal_complex() {
    let intervals = vec![
        DiscreteInterval::new(0, 2),
        DiscreteInterval::new(2, 5),
        DiscreteInterval::new(5, 7),
    ];

    let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    let cospans = complex.to_cospan_chain();

    assert_eq!(cospans.len(), 3);

    // Verify labels are actual time point values
    assert_eq!(cospans[0].middle(), &[0u32, 2]);
    assert_eq!(cospans[1].middle(), &[2u32, 5]);
    assert_eq!(cospans[2].middle(), &[5u32, 7]);
}

#[test]
fn cospan_chain_is_composable_via_catgraph() {
    let intervals = vec![
        DiscreteInterval::new(0, 1),
        DiscreteInterval::new(1, 2),
        DiscreteInterval::new(2, 3),
    ];

    let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    let cospans = complex.to_cospan_chain();

    for i in 0..cospans.len() - 1 {
        assert!(
            cospans[i].composable(&cospans[i + 1]).is_ok(),
            "Stokes cospans {} and {} should be composable",
            i,
            i + 1
        );
    }
}

#[test]
fn compose_full_cospan_chain() {
    let intervals = vec![
        DiscreteInterval::new(0, 3),
        DiscreteInterval::new(3, 7),
        DiscreteInterval::new(7, 10),
    ];

    let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    let composite = complex.compose_cospan_chain().unwrap();

    // Domain should be [t_0], codomain should be [t_n]
    assert_eq!(composite.domain(), vec![0u32]);
    assert_eq!(composite.codomain(), vec![10u32]);
}

// ---------------------------------------------------------------------------
// Error cases
// ---------------------------------------------------------------------------

#[test]
fn empty_intervals_error() {
    let result = TemporalComplex::from_intervals(&[]);
    assert!(matches!(result, Err(StokesError::EmptyIntervals)));
}

#[test]
fn stokes_analyze_empty_intervals_error() {
    let result = StokesIrreducibility::analyze(&[]);
    assert!(result.is_err());
}
