//! Permanent Practice regression corpus (task 067).
//!
//! Pins what `problem.shell_y_poly` actually *produces* for a fixed list of seeds — resolved
//! parameters, prompt text, every hint in order, response type and canonical solution — so a
//! change to generated content can't pass silently just because the new output is still
//! in-bounds and deterministic (which is all the property tests in `mod.rs` can see).
//!
//! The rules (never delete a case; regenerate only deliberately, as its own reviewable diff)
//! live in `corpus/README.md`, next to the committed cases.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::generation::generate_problem_instance;
use crate::knowledge::{load_knowledge_package, ProblemFamily, ResolvedSolution, ResponseType};

pub(crate) const SHELL_Y_POLY_FAMILY_ID: &str = "problem.shell_y_poly";

/// Every seed with a committed case. **Append only — never remove an entry**, even if the
/// case it pins later looks redundant; see `corpus/README.md`.
///
/// - `0, 1, 3, 4, 6, 7, 10, 11, 12, 15, 17, 20, 22, 26, 27, 37, 39, 46, 49, 100` — the lowest
///   seed reaching each of the family's 20 reachable `(coeff, b)` pairs, so a regression
///   confined to one corner of the parameter space still trips a case.
/// - `42` — the seed the command-layer and registry tests already use.
/// - `u64::MAX` — the far edge of the seed space through the deterministic RNG.
pub(crate) const SHELL_Y_POLY_SEEDS: &[u64] = &[
    0,
    1,
    3,
    4,
    6,
    7,
    10,
    11,
    12,
    15,
    17,
    20,
    22,
    26,
    27,
    37,
    39,
    42,
    46,
    49,
    100,
    u64::MAX,
];

const REGENERATE_ENV: &str = "AXIOM_REGENERATE_CORPUS";

/// One committed case: the full generated instance for one seed, plus the family's response
/// type (which lives on the family, not the instance, but is part of what a learner is served).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CorpusCase {
    pub family_id: String,
    pub seed: u64,
    pub response_type: ResponseType,
    pub resolved_parameters: BTreeMap<String, f64>,
    pub prompt: String,
    pub hints: Vec<String>,
    pub canonical_solution: ResolvedSolution,
}

impl CorpusCase {
    /// The canonical solution as the string a learner would submit for a
    /// `symbolic-expression` family.
    pub(crate) fn canonical_expression(&self) -> &str {
        match &self.canonical_solution {
            ResolvedSolution::Symbolic(expression) => expression,
            ResolvedSolution::Numeric(_) => panic!(
                "{} seed {}: expected a symbolic canonical solution",
                self.family_id, self.seed
            ),
        }
    }
}

pub(crate) fn bundled_shell_y_poly_family() -> ProblemFamily {
    let package_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../knowledge-package");
    load_knowledge_package(&package_root)
        .expect("the bundled knowledge-package must load")
        .problem_families
        .into_iter()
        .find(|family| family.id.as_str() == SHELL_Y_POLY_FAMILY_ID)
        .expect("bundled knowledge-package must contain problem.shell_y_poly")
}

fn shell_y_poly_corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/generation/tests/corpus/shell_y_poly")
}

fn case_path(seed: u64) -> PathBuf {
    shell_y_poly_corpus_dir().join(format!("seed-{seed}.json"))
}

fn generate_case(family: &ProblemFamily, seed: u64) -> CorpusCase {
    let instance = generate_problem_instance(family, seed)
        .unwrap_or_else(|error| panic!("seed {seed} failed to generate: {error}"));
    CorpusCase {
        family_id: instance.family_id.as_str().to_owned(),
        seed: instance.seed,
        response_type: family.response_type,
        resolved_parameters: instance.resolved_parameters,
        prompt: instance.prompt,
        hints: instance.hints,
        canonical_solution: instance.canonical_solution,
    }
}

fn load_case(seed: u64) -> CorpusCase {
    let path = case_path(seed);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "corpus case {} is missing ({error}). Committed cases are never deleted -- \
             see src-tauri/src/generation/tests/corpus/README.md",
            path.display()
        )
    });
    let case: CorpusCase = serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("{} is not a valid corpus case: {error}", path.display()));
    assert_eq!(case.seed, seed, "{} records the wrong seed", path.display());
    assert_eq!(
        case.family_id,
        SHELL_Y_POLY_FAMILY_ID,
        "{} records the wrong family",
        path.display()
    );
    case
}

/// Every committed `problem.shell_y_poly` case, in `SHELL_Y_POLY_SEEDS` order.
pub(crate) fn load_shell_y_poly_cases() -> Vec<CorpusCase> {
    SHELL_Y_POLY_SEEDS
        .iter()
        .map(|&seed| load_case(seed))
        .collect()
}

/// Names every field that differs between the committed case and today's output, so a
/// failure says *what* about the problem changed rather than just that something did.
fn drifted_fields(committed: &CorpusCase, current: &CorpusCase) -> Vec<String> {
    let mut drift = Vec::new();
    let mut compare = |field: &str, expected: String, actual: String| {
        if expected != actual {
            drift.push(format!(
                "  {field}:\n    committed: {expected}\n    generated: {actual}"
            ));
        }
    };

    compare("seed", committed.seed.to_string(), current.seed.to_string());
    compare(
        "family_id",
        committed.family_id.clone(),
        current.family_id.clone(),
    );
    compare(
        "response_type",
        format!("{:?}", committed.response_type),
        format!("{:?}", current.response_type),
    );
    compare(
        "resolved_parameters",
        format!("{:?}", committed.resolved_parameters),
        format!("{:?}", current.resolved_parameters),
    );
    compare(
        "prompt",
        format!("{:?}", committed.prompt),
        format!("{:?}", current.prompt),
    );
    compare(
        "hints.len()",
        committed.hints.len().to_string(),
        current.hints.len().to_string(),
    );
    for (index, (expected, actual)) in committed.hints.iter().zip(&current.hints).enumerate() {
        compare(
            &format!("hints[{index}]"),
            format!("{expected:?}"),
            format!("{actual:?}"),
        );
    }
    compare(
        "canonical_solution",
        format!("{:?}", committed.canonical_solution),
        format!("{:?}", current.canonical_solution),
    );
    drift
}

#[test]
fn corpus_reports_returned_seed_drift() {
    let committed = load_case(42);
    let mut current = committed.clone();
    current.seed = 43;

    assert_eq!(
        drifted_fields(&committed, &current),
        vec!["  seed:\n    committed: 42\n    generated: 43"]
    );
}

#[test]
fn every_committed_shell_y_poly_case_regenerates_identically() {
    let family = bundled_shell_y_poly_family();
    let mut failures = Vec::new();

    for committed in load_shell_y_poly_cases() {
        let current = generate_case(&family, committed.seed);
        let drift = drifted_fields(&committed, &current);
        if !drift.is_empty() {
            failures.push(format!("seed {}:\n{}", committed.seed, drift.join("\n")));
        }
    }

    assert!(
        failures.is_empty(),
        "problem.shell_y_poly output drifted from the committed regression corpus.\n\
         If this change is deliberate, regenerate the corpus as its own reviewable commit -- \
         see src-tauri/src/generation/tests/corpus/README.md.\n\n{}",
        failures.join("\n\n")
    );
}

/// The seed list and the files on disk must agree exactly: a file without a seed means an
/// entry was dropped from the list, a seed without a file means a case was deleted.
#[test]
fn shell_y_poly_corpus_files_match_the_seed_list() {
    let on_disk: BTreeSet<String> = std::fs::read_dir(shell_y_poly_corpus_dir())
        .expect("corpus directory must exist")
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    let expected: BTreeSet<String> = SHELL_Y_POLY_SEEDS
        .iter()
        .map(|seed| format!("seed-{seed}.json"))
        .collect();

    assert_eq!(
        SHELL_Y_POLY_SEEDS.len(),
        expected.len(),
        "SHELL_Y_POLY_SEEDS contains a duplicate seed"
    );
    assert_eq!(
        on_disk, expected,
        "corpus files and SHELL_Y_POLY_SEEDS disagree. Cases are never deleted -- see \
         src-tauri/src/generation/tests/corpus/README.md"
    );
}

#[test]
fn shell_y_poly_corpus_covers_every_reachable_parameter_pair() {
    let covered: BTreeSet<(i64, i64)> = load_shell_y_poly_cases()
        .iter()
        .map(|case| {
            (
                case.resolved_parameters["coeff"] as i64,
                case.resolved_parameters["b"] as i64,
            )
        })
        .collect();
    let reachable: BTreeSet<(i64, i64)> = (2..=6)
        .flat_map(|coeff| (1..=coeff).map(move |b| (coeff, b)))
        .collect();

    assert_eq!(
        covered, reachable,
        "the corpus must pin at least one case for every reachable (coeff, b) pair"
    );
}

/// The single, documented way to (re)write the corpus after a *deliberate* content change.
/// Ignored, and additionally refuses to run without an explicit environment variable, so it
/// can't overwrite the corpus as a side effect of `cargo test -- --ignored`.
///
/// ```text
/// AXIOM_REGENERATE_CORPUS=1 cargo test regenerate_shell_y_poly_corpus -- --ignored
/// ```
#[test]
#[ignore = "rewrites committed corpus files; see corpus/README.md"]
fn regenerate_shell_y_poly_corpus() {
    assert_eq!(
        std::env::var(REGENERATE_ENV).as_deref(),
        Ok("1"),
        "refusing to rewrite the regression corpus without {REGENERATE_ENV}=1"
    );

    let family = bundled_shell_y_poly_family();
    std::fs::create_dir_all(shell_y_poly_corpus_dir()).unwrap();
    for &seed in SHELL_Y_POLY_SEEDS {
        let case = generate_case(&family, seed);
        let mut json = serde_json::to_string_pretty(&case).unwrap();
        json.push('\n');
        std::fs::write(case_path(seed), json).unwrap();
    }
}
