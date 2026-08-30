use super::error::KnowledgeError;

const PROBLEM: &str = "## Problem";
const SOLUTION: &str = "## Solution";
const HINTS: &str = "## Hints";
const RECOGNIZED_HEADINGS: [&str; 3] = [PROBLEM, SOLUTION, HINTS];
const EXPECTED_ORDER: [&str; 3] = [PROBLEM, SOLUTION, HINTS];

pub(crate) struct ParsedExampleBody {
    pub problem: String,
    pub solution: String,
    pub hints: Vec<String>,
}

pub(crate) fn parse_example_body(
    entity_id: &str,
    body: &str,
) -> Result<ParsedExampleBody, KnowledgeError> {
    let mut sections: Vec<(&str, Vec<&str>)> = Vec::new();
    let mut current: Option<(&str, Vec<&str>)> = None;
    let mut preamble: Vec<&str> = Vec::new();

    for line in body.lines() {
        let trimmed_end = line.trim_end_matches('\r');
        if let Some(heading) = RECOGNIZED_HEADINGS
            .iter()
            .find(|candidate| trimmed_end == **candidate)
        {
            if let Some(finished) = current.take() {
                sections.push(finished);
            }
            current = Some((*heading, Vec::new()));
        } else if trimmed_end.starts_with("## ") {
            return Err(KnowledgeError::UnknownExampleSection {
                entity_id: entity_id.to_owned(),
                heading: trimmed_end.trim().to_owned(),
            });
        } else if let Some((_, content)) = current.as_mut() {
            content.push(line);
        } else {
            preamble.push(line);
        }
    }
    if let Some(finished) = current.take() {
        sections.push(finished);
    }

    if preamble.iter().any(|line| !line.trim().is_empty()) {
        return Err(KnowledgeError::ContentBeforeProblem {
            entity_id: entity_id.to_owned(),
        });
    }

    let mut seen: Vec<&str> = Vec::new();
    for (heading, _) in &sections {
        if seen.contains(heading) {
            return Err(KnowledgeError::DuplicateExampleSection {
                entity_id: entity_id.to_owned(),
                section: section_name(heading),
            });
        }
        seen.push(heading);
    }

    let mut highest_seen = None;
    for (heading, _) in &sections {
        let position = EXPECTED_ORDER
            .iter()
            .position(|candidate| candidate == heading)
            .expect("heading was already validated as recognized");
        if let Some(highest) = highest_seen {
            if position < highest {
                return Err(KnowledgeError::OutOfOrderExampleSection {
                    entity_id: entity_id.to_owned(),
                    section: section_name(heading),
                });
            }
        }
        highest_seen = Some(position);
    }

    let problem = section_text(&sections, PROBLEM, entity_id, "Problem")?;
    let solution = section_text(&sections, SOLUTION, entity_id, "Solution")?;
    let hints = match sections.iter().find(|(heading, _)| *heading == HINTS) {
        None => Vec::new(),
        Some((_, content)) => parse_hints(entity_id, content)?,
    };

    Ok(ParsedExampleBody {
        problem,
        solution,
        hints,
    })
}

fn section_name(heading: &str) -> &'static str {
    match heading {
        PROBLEM => "Problem",
        SOLUTION => "Solution",
        HINTS => "Hints",
        _ => unreachable!("heading was already validated as recognized"),
    }
}

fn section_text(
    sections: &[(&str, Vec<&str>)],
    heading: &str,
    entity_id: &str,
    section: &'static str,
) -> Result<String, KnowledgeError> {
    let text = sections
        .iter()
        .find(|(candidate, _)| *candidate == heading)
        .map(|(_, content)| content.join("\n").trim().to_owned())
        .ok_or_else(|| KnowledgeError::MissingExampleSection {
            entity_id: entity_id.to_owned(),
            section,
        })?;
    if text.is_empty() {
        return Err(KnowledgeError::MissingExampleSection {
            entity_id: entity_id.to_owned(),
            section,
        });
    }
    Ok(text)
}

fn parse_hints(entity_id: &str, lines: &[&str]) -> Result<Vec<String>, KnowledgeError> {
    let mut hints = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        match line.strip_prefix("- ") {
            Some(text) => hints.push(text.trim_end_matches('\r').trim().to_owned()),
            None => {
                return Err(KnowledgeError::InvalidHintLine {
                    entity_id: entity_id.to_owned(),
                    line: (*line).to_owned(),
                })
            }
        }
    }
    if hints.is_empty() {
        return Err(KnowledgeError::EmptyHintsSection {
            entity_id: entity_id.to_owned(),
        });
    }
    Ok(hints)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_BODY: &str = "## Problem\n\nFind the volume.\n\n## Solution\n\nV = 8pi/3.\n\n## Hints\n\n- Identify the radius first.\n- Then the height.\n";

    #[test]
    fn valid_body_parses_all_three_sections() {
        let parsed = parse_example_body("shell.example_basic", VALID_BODY).unwrap();
        assert_eq!(parsed.problem, "Find the volume.");
        assert_eq!(parsed.solution, "V = 8pi/3.");
        assert_eq!(
            parsed.hints,
            vec!["Identify the radius first.", "Then the height."]
        );
    }

    #[test]
    fn hints_is_optional() {
        let body = "## Problem\n\nFind the volume.\n\n## Solution\n\nV = 8pi/3.\n";
        let parsed = parse_example_body("shell.example_basic", body).unwrap();
        assert!(parsed.hints.is_empty());
    }

    #[test]
    fn hint_order_is_preserved() {
        let body =
            "## Problem\n\nP.\n\n## Solution\n\nS.\n\n## Hints\n\n- first\n- second\n- third\n";
        let parsed = parse_example_body("shell.example_basic", body).unwrap();
        assert_eq!(parsed.hints, vec!["first", "second", "third"]);
    }

    #[test]
    fn missing_problem_is_rejected() {
        let body = "## Solution\n\nV = 8pi/3.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::MissingExampleSection {
                section: "Problem",
                ..
            })
        ));
    }

    #[test]
    fn missing_solution_is_rejected() {
        let body = "## Problem\n\nFind the volume.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::MissingExampleSection {
                section: "Solution",
                ..
            })
        ));
    }

    #[test]
    fn empty_problem_is_rejected() {
        let body = "## Problem\n\n## Solution\n\nV = 8pi/3.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::MissingExampleSection {
                section: "Problem",
                ..
            })
        ));
    }

    #[test]
    fn duplicate_heading_is_rejected() {
        let body = "## Problem\n\nP.\n\n## Solution\n\nS1.\n\n## Solution\n\nS2.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::DuplicateExampleSection {
                section: "Solution",
                ..
            })
        ));
    }

    #[test]
    fn out_of_order_heading_is_rejected() {
        let body = "## Solution\n\nS.\n\n## Problem\n\nP.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::OutOfOrderExampleSection {
                section: "Problem",
                ..
            })
        ));
    }

    #[test]
    fn unknown_heading_is_rejected() {
        let body = "## Problem\n\nP.\n\n## Solution\n\nS.\n\n## Notes\n\nExtra.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::UnknownExampleSection { .. })
        ));
    }

    #[test]
    fn content_before_problem_is_rejected() {
        let body = "Stray intro text.\n\n## Problem\n\nP.\n\n## Solution\n\nS.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::ContentBeforeProblem { .. })
        ));
    }

    #[test]
    fn whitespace_before_problem_is_accepted() {
        let body = "\n\n## Problem\n\nP.\n\n## Solution\n\nS.\n";
        assert!(parse_example_body("shell.example_basic", body).is_ok());
    }

    #[test]
    fn non_list_hints_content_is_rejected() {
        let body =
            "## Problem\n\nP.\n\n## Solution\n\nS.\n\n## Hints\n\nJust a paragraph, not a list.\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::InvalidHintLine { .. })
        ));
    }

    #[test]
    fn multiline_hint_continuation_is_rejected() {
        let body = "## Problem\n\nP.\n\n## Solution\n\nS.\n\n## Hints\n\n- first line\n  continuation line\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::InvalidHintLine { .. })
        ));
    }

    #[test]
    fn empty_hints_section_is_rejected() {
        let body = "## Problem\n\nP.\n\n## Solution\n\nS.\n\n## Hints\n\n";
        assert!(matches!(
            parse_example_body("shell.example_basic", body),
            Err(KnowledgeError::EmptyHintsSection { .. })
        ));
    }

    #[test]
    fn opaque_content_below_recognized_sections_is_preserved_verbatim() {
        let body = "## Problem\n\nP with a **bold** word and a\n### sub-heading inside it.\n\n## Solution\n\nS.\n";
        let parsed = parse_example_body("shell.example_basic", body).unwrap();
        assert!(parsed.problem.contains("### sub-heading inside it."));
    }
}
