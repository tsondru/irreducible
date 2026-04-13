//! Integration tests for the temporal cospan chain bridge.

use catgraph::category::Composable;
use irreducible::temporal_cospan_chain::{ConservationResult, StokesError, TemporalComplex};
use irreducible::{DiscreteInterval, ElementaryCA, TuringMachine};

#[test]
fn build_temporal_complex_from_tm_execution() {
    let bb = TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);
    let intervals = history.to_intervals();
    let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    assert_eq!(complex.num_time_steps(), 7);
    assert_eq!(complex.num_intervals(), 6);
    assert_eq!(complex.time_points(), &[0, 1, 2, 3, 4, 5, 6]);
}

#[test]
fn build_temporal_complex_from_ca_execution() {
    let ca = ElementaryCA::rule_30(11);
    let initial = ca.single_cell_initial();
    let history = ca.run(initial, 10);
    let intervals = history.to_intervals();
    let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    assert_eq!(complex.num_time_steps(), 11);
    assert_eq!(complex.num_intervals(), 10);
}

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
    assert!(result.is_contiguous);
    assert!(result.is_monotonic);
    assert!(result.is_well_formed());
    assert!((result.total_complexity - 3.0).abs() < 1e-10);
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
}

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
    assert_eq!(cospans[0].middle(), &[0u32, 2]);
    assert_eq!(cospans[1].middle(), &[2u32, 5]);
    assert_eq!(cospans[2].middle(), &[5u32, 7]);
}

#[test]
fn cospan_chain_is_composable() {
    let intervals = vec![
        DiscreteInterval::new(0, 1),
        DiscreteInterval::new(1, 2),
        DiscreteInterval::new(2, 3),
    ];
    let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    let cospans = complex.to_cospan_chain();
    for i in 0..cospans.len() - 1 {
        assert!(cospans[i].composable(&cospans[i + 1]).is_ok());
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
    assert_eq!(composite.domain(), vec![0u32]);
    assert_eq!(composite.codomain(), vec![10u32]);
}

#[test]
fn single_interval_complex() {
    let intervals = vec![DiscreteInterval::new(4, 9)];
    let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    assert_eq!(complex.num_time_steps(), 2);
    assert_eq!(complex.num_intervals(), 1);
    let result = complex.verify_conservation();
    assert!(result.is_conserved);
    assert!((result.total_complexity - 5.0).abs() < 1e-10);
}

#[test]
fn empty_intervals_error() {
    let result = TemporalComplex::from_intervals(&[]);
    assert!(matches!(result, Err(StokesError::EmptyIntervals)));
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
    assert!((result.average_complexity() - 8.0 / 3.0).abs() < 1e-10);
}

#[test]
fn empty_conservation_result_methods() {
    let result = ConservationResult {
        is_conserved: true,
        is_contiguous: true,
        is_monotonic: true,
        total_complexity: 0.0,
        num_intervals: 0,
        num_time_steps: 0,
    };
    assert!((result.average_complexity()).abs() < 1e-10);
    assert!(result.is_well_formed());
}
