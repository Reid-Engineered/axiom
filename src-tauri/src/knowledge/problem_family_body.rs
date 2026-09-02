use super::error::KnowledgeError;

const PROMPT: &str = "## Prompt";
const SOLUTION: &str = "## Solution";
const HINTS: &str = "## Hints";
const RECOGNIZED_HEADINGS: [&str; 3] = [PROMPT, SOLUTION, HINTS];
const EXPECTED_ORDER: [&str; 3] = [PROMPT, SOLUTION, HINTS];

pub(crate) struct ParsedProblemFamilyBody {
    pub prompt: String,
    pub solution_structure: String,
    pub hint_texts: Vec<String>,
}

pub(crate) fn parse_problem_family_body(
    entity_id: &str,
    body: &str,
) -> Result<ParsedProblemFamilyBody, KnowledgeError> {
    let mut sections: Vec<(&str, Vec<&str>)> = Vec::new();
    let mut current: Option<(&str, Vec<&str>)> = None;
    let mut preamble = Vec::new();
    for line in body.lines() {
        let trimmed_end = line.trim_end_matches('\r');
        if let Some(heading) = RECOGNIZED_HEADINGS.iter().find(|h| trimmed_end == **h) {
            if let Some(finished) = current.take() {
                sections.push(finished);
            }
            current = Some((*heading, Vec::new()));
        } else if trimmed_end.starts_with("## ") {
            return Err(KnowledgeError::UnknownProblemFamilySection {
                entity_id: entity_id.to_owned(),
                heading: trimmed_end.trim().to_owned(),
            });
        } else if let Some((_, content)) = current.as_mut() {
            content.push(line);
        } else {
            preamble.push(line);
        }
    }
    if let Some(finished) = current {
        sections.push(finished);
    }
    if preamble.iter().any(|line| !line.trim().is_empty()) {
        return Err(KnowledgeError::ContentBeforePrompt {
            entity_id: entity_id.to_owned(),
        });
    }

    let mut seen = Vec::new();
    for (heading, _) in &sections {
        if seen.contains(heading) {
            return Err(KnowledgeError::DuplicateProblemFamilySection {
                entity_id: entity_id.to_owned(),
                section: section_name(heading),
            });
        }
        seen.push(*heading);
    }
    let mut highest_seen = None;
    for (heading, _) in &sections {
        let position = EXPECTED_ORDER
            .iter()
            .position(|h| h == heading)
            .expect("recognized heading");
        if highest_seen.is_some_and(|highest| position < highest) {
            return Err(KnowledgeError::OutOfOrderProblemFamilySection {
                entity_id: entity_id.to_owned(),
                section: section_name(heading),
            });
        }
        highest_seen = Some(position);
    }

    let prompt = section_text(&sections, PROMPT, entity_id, "Prompt")?;
    let solution_structure = section_text(&sections, SOLUTION, entity_id, "Solution")?;
    let hint_texts = sections
        .iter()
        .find(|(heading, _)| *heading == HINTS)
        .map_or_else(
            || Ok(Vec::new()),
            |(_, lines)| parse_hint_lines(entity_id, lines),
        )?;
    Ok(ParsedProblemFamilyBody {
        prompt,
        solution_structure,
        hint_texts,
    })
}

fn section_name(heading: &str) -> &'static str {
    match heading {
        PROMPT => "Prompt",
        SOLUTION => "Solution",
        HINTS => "Hints",
        _ => unreachable!(),
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
        .map(|(_, lines)| lines.join("\n").trim().to_owned())
        .ok_or_else(|| KnowledgeError::MissingProblemFamilySection {
            entity_id: entity_id.to_owned(),
            section,
        })?;
    if text.is_empty() {
        return Err(KnowledgeError::MissingProblemFamilySection {
            entity_id: entity_id.to_owned(),
            section,
        });
    }
    Ok(text)
}

fn parse_hint_lines(entity_id: &str, lines: &[&str]) -> Result<Vec<String>, KnowledgeError> {
    lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.strip_prefix("- ")
                .map(|text| text.trim_end_matches('\r').trim().to_owned())
                .ok_or_else(|| KnowledgeError::InvalidProblemFamilyHintLine {
                    entity_id: entity_id.to_owned(),
                    line: (*line).to_owned(),
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_body_and_preserves_hint_order() {
        let body = "## Prompt\n\nP.\n\n## Solution\n\nS.\n\n## Hints\n\n- first\n- second\n";
        let parsed = parse_problem_family_body("problem.a", body).unwrap();
        assert_eq!(parsed.hint_texts, vec!["first", "second"]);
    }

    #[test]
    fn rejects_duplicate_and_out_of_order_sections() {
        let duplicate = "## Prompt\nP\n## Solution\nS\n## Solution\nS2";
        assert!(matches!(
            parse_problem_family_body("problem.a", duplicate),
            Err(KnowledgeError::DuplicateProblemFamilySection { .. })
        ));
        let out_of_order = "## Solution\nS\n## Prompt\nP";
        assert!(matches!(
            parse_problem_family_body("problem.a", out_of_order),
            Err(KnowledgeError::OutOfOrderProblemFamilySection { .. })
        ));
    }
}
