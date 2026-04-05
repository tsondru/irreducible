//! Demonstrates the manifold curvature pipeline:
//! `StringRewriteSystem` -> multiway evolution -> branchial foliation -> MDS embedding -> Riemannian curvature
//!
//! Run with: `cargo run --example manifold_curvature --features manifold-curvature`

#[cfg(not(feature = "manifold-curvature"))]
fn main() {
    eprintln!("This example requires the `manifold-curvature` feature:");
    eprintln!("  cargo run --example manifold_curvature --features manifold-curvature");
}

#[cfg(feature = "manifold-curvature")]
fn main() {
    manifold_demo::run();
}

#[cfg(feature = "manifold-curvature")]
mod manifold_demo {
    use irreducible::machines::multiway::{
        ManifoldCurvature, ShortestPathMDS, StringRewriteSystem,
    };
    use irreducible::extract_branchial_foliation;

    pub fn run() {
        println!("=== Manifold Curvature Pipeline ===\n");

        // A branching SRS with sustained growth:
        // "AB" -> "BA" swaps, "A" -> "AA" doubles, giving multiple match positions
        let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
        let evolution = srs.run_multiway("AB", 4, 200);

        let stats = evolution.statistics();
        println!(
            "SRS evolution: {} nodes, {} forks, max depth {}",
            stats.total_nodes, stats.fork_count, stats.max_depth
        );

        let foliation = extract_branchial_foliation(&evolution);
        println!("Branchial foliation: {} time steps\n", foliation.len());

        let embedding = ShortestPathMDS::<3>;

        for branchial in &foliation {
            if branchial.node_count() <= 1 {
                println!(
                    "Step {}: {} node(s) — skipping (need >1 for curvature)",
                    branchial.step,
                    branchial.node_count()
                );
                continue;
            }

            let curvature = ManifoldCurvature::from_branchial(branchial, &embedding);
            println!("{curvature}");
            println!();
        }
    }
}
