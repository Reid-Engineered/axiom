use std::path::Path;

use super::error::KnowledgeError;
use super::example_body::parse_example_body;
use super::frontmatter::split_frontmatter;
use super::identifier::{ConceptId, ObjectiveId};
use super::provenance::convert_provenance_refs;
use super::raw::RawExampleFrontmatter;
use super::types::Example;

pub(crate) fn parse_example_file(path: &Path, raw: &str) -> Result<Example, KnowledgeError> {
    let (toml_text, body) = split_frontmatter(path, raw)?;
    let raw_frontmatter: RawExampleFrontmatter =
        toml::from_str(&toml_text).map_err(|source| KnowledgeError::TomlSyntax {
            path: path.to_owned(),
            source,
        })?;

    let id = super::identifier::ExampleId::new(raw_frontmatter.id)?;
    let concept_id = ConceptId::new(raw_frontmatter.concept_id)?;
    let objective_ids = raw_frontmatter
        .objective_ids
        .into_iter()
        .map(ObjectiveId::new)
        .collect::<Result<Vec<_>, _>>()?;

    let parsed_body = parse_example_body(id.as_str(), &body)?;
    let provenance_refs = convert_provenance_refs(id.as_str(), raw_frontmatter.provenance_refs)?;

    Ok(Example {
        id,
        concept_id,
        objective_ids,
        problem: parsed_body.problem,
        solution: parsed_body.solution,
        hints: parsed_body.hints,
        provenance_refs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const VALID_EXAMPLE: &str = r#"+++
id = "shell.example_y_poly"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height"]

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Example 2.13"
+++

## Problem

Find the volume of the solid formed by revolving the region bounded above by
`f(x) = 4x - x^2` and below by the x-axis over `[0, 3]` around the *y*-axis.

## Solution

V = 63*pi/2.

## Hints

- Identify the shell radius and height as functions of `x` before integrating.
"#;

    #[test]
    fn valid_example_parses() {
        let example =
            parse_example_file(Path::new("examples/shell.example_y_poly.md"), VALID_EXAMPLE)
                .unwrap();
        assert_eq!(example.id.as_str(), "shell.example_y_poly");
        assert_eq!(example.concept_id.as_str(), "shell.method_vertical_axis");
        assert_eq!(example.objective_ids.len(), 1);
        assert!(example.problem.starts_with("Find the volume"));
        assert_eq!(example.solution, "V = 63*pi/2.");
        assert_eq!(example.hints.len(), 1);
        assert_eq!(example.provenance_refs.len(), 1);
    }

    #[test]
    fn objective_ids_may_be_empty() {
        let raw = VALID_EXAMPLE.replace(r#"objective_ids = ["shell.setup_radius_height"]"#, "");
        let example =
            parse_example_file(Path::new("examples/shell.example_y_poly.md"), &raw).unwrap();
        assert!(example.objective_ids.is_empty());
    }

    #[test]
    fn missing_problem_section_is_rejected() {
        let raw = VALID_EXAMPLE.replace(
            "## Problem\n\nFind the volume of the solid formed by revolving the region bounded above by\n`f(x) = 4x - x^2` and below by the x-axis over `[0, 3]` around the *y*-axis.\n\n",
            "",
        );
        assert!(matches!(
            parse_example_file(Path::new("examples/shell.example_y_poly.md"), &raw),
            Err(KnowledgeError::MissingExampleSection {
                section: "Problem",
                ..
            })
        ));
    }
}
