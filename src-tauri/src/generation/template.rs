use std::collections::BTreeMap;

pub(crate) fn substitute_braces(
    template: &str,
    resolved_parameters: &BTreeMap<String, f64>,
) -> String {
    let mut result = template.to_owned();
    for (name, value) in resolved_parameters {
        result = result.replace(&format!("{{{name}}}"), &format_number(*value));
    }
    result
}

pub(crate) fn substitute_identifiers(
    template: &str,
    resolved_parameters: &BTreeMap<String, f64>,
) -> String {
    let mut result = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();
    let mut previous_is_identifier_continue = false;
    while let Some(&character) = chars.peek() {
        if (character.is_ascii_alphabetic() || character == '_') && !previous_is_identifier_continue
        {
            let mut identifier = String::new();
            while let Some(&character) = chars.peek() {
                if character.is_ascii_alphanumeric() || character == '_' {
                    identifier.push(character);
                    chars.next();
                } else {
                    break;
                }
            }
            match resolved_parameters.get(&identifier) {
                Some(value) if value.is_sign_negative() => {
                    result.push('(');
                    result.push_str(&format_number(*value));
                    result.push(')');
                }
                Some(value) => result.push_str(&format_number(*value)),
                None => result.push_str(&identifier),
            }
            previous_is_identifier_continue = true;
        } else {
            result.push(character);
            chars.next();
            previous_is_identifier_continue = character.is_ascii_alphanumeric() || character == '_';
        }
    }
    result
}

fn format_number(value: f64) -> String {
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
        pairs
            .iter()
            .map(|(name, value)| (name.to_string(), *value))
            .collect()
    }

    #[test]
    fn substitute_braces_replaces_every_placeholder() {
        let result = substitute_braces(
            "f(x) = {coeff}x - x^2 over [{a}, {b}]",
            &params(&[("coeff", 4.0), ("a", 0.0), ("b", 3.0)]),
        );
        assert_eq!(result, "f(x) = 4x - x^2 over [0, 3]");
    }

    #[test]
    fn substitute_braces_leaves_text_with_no_matching_placeholder_untouched() {
        let result = substitute_braces("no placeholders here", &params(&[("coeff", 4.0)]));
        assert_eq!(result, "no placeholders here");
    }

    #[test]
    fn substitute_identifiers_replaces_exact_parameter_names_only() {
        let result = substitute_identifiers(
            "2*pi*(coeff*b^3/3 - b^4/4)",
            &params(&[("coeff", 4.0), ("b", 3.0)]),
        );
        assert_eq!(result, "2*pi*(4*3^3/3 - 3^4/4)");
    }

    #[test]
    fn substitute_identifiers_leaves_non_parameter_identifiers_untouched() {
        let result = substitute_identifiers("sin(pi/2)", &params(&[("b", 3.0)]));
        assert_eq!(result, "sin(pi/2)");
    }

    #[test]
    fn substitute_identifiers_does_not_partially_match_a_longer_name() {
        let result = substitute_identifiers("coefficient + coeff", &params(&[("coeff", 2.0)]));
        assert_eq!(result, "coefficient + 2");
    }

    #[test]
    fn substitute_identifiers_groups_negative_parameter_values() {
        let result = substitute_identifiers("coeff^2", &params(&[("coeff", -2.0)]));
        assert_eq!(result, "(-2)^2");
    }

    #[test]
    fn substitute_identifiers_does_not_replace_a_scientific_exponent() {
        let result = substitute_identifiers("1e3 + e3", &params(&[("e3", 7.0)]));
        assert_eq!(result, "1e3 + 7");
    }

    #[test]
    fn format_number_omits_trailing_zero_for_whole_numbers() {
        assert_eq!(format_number(4.0), "4");
        assert_eq!(format_number(-2.0), "-2");
        assert_eq!(format_number(2.5), "2.5");
    }

    #[test]
    fn format_number_preserves_large_whole_values() {
        assert_eq!(format_number(1e20), "100000000000000000000");
    }
}
