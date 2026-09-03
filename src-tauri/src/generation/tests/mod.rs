use std::path::Path;

use crate::generation::generate_problem_instance;
use crate::knowledge::load_knowledge_package;

fn shell_y_poly_family() -> crate::knowledge::ProblemFamily {
    let fixture_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/knowledge/tests/fixtures/canonical");
    let package = load_knowledge_package(&fixture_root).unwrap();
    package
        .problem_families
        .into_iter()
        .find(|family| family.id.as_str() == "problem.shell_y_poly")
        .expect("fixture must contain problem.shell_y_poly")
}

/// The declared `b <= coeff` bound must keep `coeff*x - x^2` non-negative throughout the
/// generated interval, preserving the problem family's geometric premise.
#[test]
fn shell_y_poly_height_stays_non_negative_across_ten_thousand_seeds() {
    let family = shell_y_poly_family();

    for seed in 0..10_000u64 {
        let instance = generate_problem_instance(&family, seed)
            .unwrap_or_else(|error| panic!("seed {seed} failed to generate: {error}"));

        let coeff = instance.resolved_parameters["coeff"];
        let b = instance.resolved_parameters["b"];

        const SAMPLE_POINTS: u32 = 50;
        for index in 0..=SAMPLE_POINTS {
            let x = b * (index as f64 / SAMPLE_POINTS as f64);
            let height = coeff * x - x * x;
            assert!(
                height >= 0.0,
                "seed {seed}: h({x}) = {height} < 0 for coeff={coeff}, b={b}"
            );
        }
    }
}
