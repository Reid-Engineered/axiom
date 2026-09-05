use std::collections::BTreeSet;
use std::path::Path;

use crate::generation::generate_problem_instance;
use crate::knowledge::{load_knowledge_package, ResolvedSolution};

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

fn bundled_shell_y_poly_family() -> crate::knowledge::ProblemFamily {
    let package_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../knowledge-package");
    let package = load_knowledge_package(&package_root)
        .expect("the bundled knowledge-package must load -- a broken bundle is a build problem");
    package
        .problem_families
        .into_iter()
        .find(|family| family.id.as_str() == "problem.shell_y_poly")
        .expect("bundled knowledge-package must contain problem.shell_y_poly")
}

/// Same seed, same instance -- the property the whole attempt store depends on, since an
/// attempt persists only its seed and is replayed through the generator on every load.
#[test]
fn bundled_shell_y_poly_is_deterministic_for_every_seed() {
    let family = bundled_shell_y_poly_family();

    for seed in 0..2_000u64 {
        let first = generate_problem_instance(&family, seed)
            .unwrap_or_else(|error| panic!("seed {seed} failed to generate: {error}"));
        let second = generate_problem_instance(&family, seed).unwrap();
        assert_eq!(
            first, second,
            "seed {seed} did not generate deterministically"
        );
    }
}

/// Every instance the bundled family can produce must be a well-formed, geometrically real
/// problem: parameters inside their declared bounds, a non-negative shell height across the
/// whole interval, no leftover templating, and a canonical solution that both agrees with
/// the hand-derived closed form `2*pi*(c*b^3/3 - b^4/4)` and is a positive finite volume.
///
/// The family's reachable parameter space is exactly 20 `(coeff, b)` pairs
/// (`coeff` in 2..=6, `b` in 1..=coeff), and the final assertion pins that all 20 were
/// actually reached -- so this is exhaustive over the family's content, not a spot check.
#[test]
fn bundled_shell_y_poly_instances_are_valid_across_ten_thousand_seeds() {
    let family = bundled_shell_y_poly_family();
    let math = mathcore::MathCore::new();
    let mut seen_pairs = BTreeSet::new();

    for seed in 0..10_000u64 {
        let instance = generate_problem_instance(&family, seed)
            .unwrap_or_else(|error| panic!("seed {seed} failed to generate: {error}"));

        let coeff = instance.resolved_parameters["coeff"];
        let a = instance.resolved_parameters["a"];
        let b = instance.resolved_parameters["b"];

        assert!(
            (2.0..=6.0).contains(&coeff) && coeff.fract() == 0.0,
            "seed {seed}: coeff={coeff} outside the declared integer range 2..=6"
        );
        assert_eq!(
            a, 0.0,
            "seed {seed}: a={a}, but the shell radius r(x) = x needs a = 0"
        );
        assert!(
            b >= 1.0 && b <= coeff && b.fract() == 0.0,
            "seed {seed}: b={b} outside the declared integer range 1..=coeff ({coeff})"
        );

        // h(x) = x(coeff - x) >= 0 on [0, b] is what makes "bounded above by f, below by
        // the x-axis" true; b <= coeff is the bound that guarantees it.
        const SAMPLE_POINTS: u32 = 50;
        for index in 0..=SAMPLE_POINTS {
            let x = b * (f64::from(index) / f64::from(SAMPLE_POINTS));
            let height = coeff * x - x * x;
            assert!(
                height >= 0.0,
                "seed {seed}: h({x}) = {height} < 0 for coeff={coeff}, b={b}"
            );
        }

        assert_eq!(
            instance.hints.len(),
            4,
            "seed {seed}: expected 4 hint levels"
        );
        for name in family.parameters.keys() {
            let placeholder = format!("{{{name}}}");
            assert!(
                !instance.prompt.contains(&placeholder),
                "seed {seed}: prompt still contains {placeholder}"
            );
            for hint in &instance.hints {
                assert!(
                    !hint.contains(&placeholder),
                    "seed {seed}: hint still contains {placeholder}"
                );
            }
        }

        let ResolvedSolution::Symbolic(expression) = &instance.canonical_solution else {
            panic!("seed {seed}: problem.shell_y_poly is a symbolic-expression family");
        };
        let value = math
            .calculate(expression)
            .unwrap_or_else(|error| panic!("seed {seed}: {expression:?} did not parse: {error}"));
        let expected = 2.0 * std::f64::consts::PI * (coeff * b.powi(3) / 3.0 - b.powi(4) / 4.0);
        assert!(
            (value - expected).abs() <= 1e-9 * expected.abs(),
            "seed {seed}: {expression:?} evaluated to {value}, hand-derived closed form gives {expected}"
        );
        assert!(
            value.is_finite() && value > 0.0,
            "seed {seed}: volume {value} is not a positive finite number"
        );

        seen_pairs.insert((coeff as i64, b as i64));
    }

    let reachable_pairs: BTreeSet<(i64, i64)> = (2..=6)
        .flat_map(|coeff| (1..=coeff).map(move |b| (coeff, b)))
        .collect();
    assert_eq!(
        seen_pairs, reachable_pairs,
        "10,000 seeds did not cover every reachable (coeff, b) pair"
    );
}
