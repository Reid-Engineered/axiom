use std::path::Path;

use super::error::KnowledgeError;

const DELIMITER: &str = "+++";

pub(crate) fn split_frontmatter(
    path: &Path,
    raw: &str,
) -> Result<(String, String), KnowledgeError> {
    if raw.starts_with('\u{FEFF}') {
        return Err(KnowledgeError::Bom {
            path: path.to_owned(),
        });
    }

    let normalized = raw.replace("\r\n", "\n");
    let mut lines = normalized.split_terminator('\n');

    match lines.next() {
        Some(DELIMITER) => {}
        _ => {
            return Err(KnowledgeError::MissingFrontmatterDelimiter {
                path: path.to_owned(),
            })
        }
    }

    let mut toml_lines: Vec<&str> = Vec::new();
    let mut closed = false;
    let mut remaining: Vec<&str> = Vec::new();
    for line in lines.by_ref() {
        if line == DELIMITER {
            closed = true;
            break;
        }
        toml_lines.push(line);
    }
    if !closed {
        return Err(KnowledgeError::UnterminatedFrontmatter {
            path: path.to_owned(),
        });
    }
    remaining.extend(lines);

    let mut body_start = 0;
    while body_start < remaining.len() && remaining[body_start].trim().is_empty() {
        body_start += 1;
    }

    let toml_text = toml_lines.join("\n") + "\n";
    let body_text = if body_start >= remaining.len() {
        String::new()
    } else {
        remaining[body_start..].join("\n") + "\n"
    };

    Ok((toml_text, body_text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn p() -> &'static Path {
        Path::new("concepts/shell.method_vertical_axis.md")
    }

    #[test]
    fn splits_toml_and_body() {
        let raw = "+++\nid = \"shell.a\"\n+++\n\nBody text.\n";
        let (toml, body) = split_frontmatter(p(), raw).unwrap();
        assert_eq!(toml, "id = \"shell.a\"\n");
        assert_eq!(body, "Body text.\n");
    }

    #[test]
    fn trims_leading_blank_lines_from_body_but_preserves_internal_blank_lines() {
        let raw = "+++\nid = \"shell.a\"\n+++\n\n\nFirst paragraph.\n\nSecond paragraph.\n";
        let (_, body) = split_frontmatter(p(), raw).unwrap();
        assert_eq!(body, "First paragraph.\n\nSecond paragraph.\n");
    }

    #[test]
    fn accepts_crlf_line_endings() {
        let raw = "+++\r\nid = \"shell.a\"\r\n+++\r\n\r\nBody.\r\n";
        let (toml, body) = split_frontmatter(p(), raw).unwrap();
        assert_eq!(toml, "id = \"shell.a\"\n");
        assert_eq!(body, "Body.\n");
    }

    #[test]
    fn rejects_bom() {
        let raw = "\u{FEFF}+++\nid = \"shell.a\"\n+++\n\nBody.\n";
        assert!(matches!(
            split_frontmatter(p(), raw),
            Err(KnowledgeError::Bom { .. })
        ));
    }

    #[test]
    fn rejects_missing_opening_delimiter() {
        let raw = "id = \"shell.a\"\n+++\n\nBody.\n";
        assert!(matches!(
            split_frontmatter(p(), raw),
            Err(KnowledgeError::MissingFrontmatterDelimiter { .. })
        ));
    }

    #[test]
    fn rejects_leading_blank_line_before_opening_delimiter() {
        let raw = "\n+++\nid = \"shell.a\"\n+++\n\nBody.\n";
        assert!(matches!(
            split_frontmatter(p(), raw),
            Err(KnowledgeError::MissingFrontmatterDelimiter { .. })
        ));
    }

    #[test]
    fn rejects_unterminated_frontmatter() {
        let raw = "+++\nid = \"shell.a\"\n";
        assert!(matches!(
            split_frontmatter(p(), raw),
            Err(KnowledgeError::UnterminatedFrontmatter { .. })
        ));
    }
}
