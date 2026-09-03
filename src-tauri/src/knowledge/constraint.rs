use std::collections::BTreeMap;
use std::iter::Peekable;
use std::str::Chars;

use serde::{Deserialize, Serialize};

use super::error::KnowledgeError;

const MAX_NESTING_DEPTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArithOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompareOp {
    Eq,
    Ne,
    Ge,
    Le,
    Gt,
    Lt,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Term {
    Param(String),
    Literal(f64),
    BinaryOp {
        op: ArithOp,
        left: Box<Term>,
        right: Box<Term>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstraintExpr {
    Comparison {
        left: Term,
        op: CompareOp,
        right: Term,
    },
    All(Vec<ConstraintExpr>),
}

impl Term {
    pub(crate) fn evaluate(&self, params: &BTreeMap<String, f64>) -> f64 {
        match self {
            Self::Param(name) => params[name],
            Self::Literal(value) => *value,
            Self::BinaryOp { op, left, right } => {
                let left = left.evaluate(params);
                let right = right.evaluate(params);
                match op {
                    ArithOp::Add => left + right,
                    ArithOp::Sub => left - right,
                    ArithOp::Mul => left * right,
                    ArithOp::Div => left / right,
                }
            }
        }
    }
}

impl ConstraintExpr {
    pub(crate) fn holds(&self, params: &BTreeMap<String, f64>) -> bool {
        match self {
            Self::Comparison { left, op, right } => {
                let left = left.evaluate(params);
                let right = right.evaluate(params);
                match op {
                    CompareOp::Eq => left == right,
                    CompareOp::Ne => left != right,
                    CompareOp::Ge => left >= right,
                    CompareOp::Le => left <= right,
                    CompareOp::Gt => left > right,
                    CompareOp::Lt => left < right,
                }
            }
            Self::All(parts) => parts.iter().all(|part| part.holds(params)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    Eq,
    Ne,
    Ge,
    Le,
    Gt,
    Lt,
    LParen,
    RParen,
    And,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut chars: Peekable<Chars> = input.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c.is_ascii_digit() || c == '.' {
            let mut number = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() || c == '.' {
                    number.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token::Number(
                number
                    .parse()
                    .map_err(|_| format!("invalid number: {number}"))?,
            ));
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let mut identifier = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    identifier.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(if identifier == "and" {
                Token::And
            } else {
                Token::Ident(identifier)
            });
            continue;
        }
        chars.next();
        tokens.push(match c {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '=' if chars.next_if_eq(&'=').is_some() => Token::Eq,
            '!' if chars.next_if_eq(&'=').is_some() => Token::Ne,
            '>' if chars.next_if_eq(&'=').is_some() => Token::Ge,
            '<' if chars.next_if_eq(&'=').is_some() => Token::Le,
            '>' => Token::Gt,
            '<' => Token::Lt,
            '=' => return Err("expected '==' but found a single '='".to_owned()),
            '!' => return Err("expected '!=' but found a bare '!'".to_owned()),
            other => return Err(format!("unexpected character: {other}")),
        });
    }
    Ok(tokens)
}

struct ExprParser {
    tokens: Vec<Token>,
    pos: usize,
}

struct ParsedTerm {
    term: Term,
    depth: usize,
}

impl ParsedTerm {
    fn leaf(term: Term) -> Self {
        Self { term, depth: 0 }
    }

    fn binary(op: ArithOp, left: Self, right: Self) -> Result<Self, String> {
        let depth = 1 + left.depth.max(right.depth);
        if depth > MAX_NESTING_DEPTH {
            return Err(format!(
                "constraint tree exceeds maximum depth {MAX_NESTING_DEPTH}"
            ));
        }
        Ok(Self {
            term: Term::BinaryOp {
                op,
                left: Box::new(left.term),
                right: Box::new(right.term),
            },
            depth,
        })
    }
}

impl ExprParser {
    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        token
    }

    fn parse_conjunction(&mut self) -> Result<ConstraintExpr, String> {
        let mut comparisons = vec![self.parse_comparison()?];
        while self.tokens.get(self.pos) == Some(&Token::And) {
            self.pos += 1;
            comparisons.push(self.parse_comparison()?);
        }
        if comparisons.len() == 1 {
            Ok(comparisons.remove(0))
        } else {
            Ok(ConstraintExpr::All(comparisons))
        }
    }

    fn parse_comparison(&mut self) -> Result<ConstraintExpr, String> {
        let left = self.parse_term(0)?.term;
        let op = match self.advance() {
            Some(Token::Eq) => CompareOp::Eq,
            Some(Token::Ne) => CompareOp::Ne,
            Some(Token::Ge) => CompareOp::Ge,
            Some(Token::Le) => CompareOp::Le,
            Some(Token::Gt) => CompareOp::Gt,
            Some(Token::Lt) => CompareOp::Lt,
            other => return Err(format!("expected a comparison operator, found {other:?}")),
        };
        let right = self.parse_term(0)?.term;
        Ok(ConstraintExpr::Comparison { left, op, right })
    }

    fn parse_term(&mut self, depth: usize) -> Result<ParsedTerm, String> {
        let mut left = self.parse_factor(depth)?;
        loop {
            let op = match self.tokens.get(self.pos) {
                Some(Token::Plus) => ArithOp::Add,
                Some(Token::Minus) => ArithOp::Sub,
                _ => break,
            };
            self.pos += 1;
            left = ParsedTerm::binary(op, left, self.parse_factor(depth)?)?;
        }
        Ok(left)
    }

    fn parse_factor(&mut self, depth: usize) -> Result<ParsedTerm, String> {
        let mut left = self.parse_atom(depth)?;
        loop {
            let op = match self.tokens.get(self.pos) {
                Some(Token::Star) => ArithOp::Mul,
                Some(Token::Slash) => ArithOp::Div,
                _ => break,
            };
            self.pos += 1;
            left = ParsedTerm::binary(op, left, self.parse_atom(depth)?)?;
        }
        Ok(left)
    }

    fn parse_atom(&mut self, depth: usize) -> Result<ParsedTerm, String> {
        if depth > MAX_NESTING_DEPTH {
            return Err(format!(
                "constraint nesting exceeds maximum depth {MAX_NESTING_DEPTH}"
            ));
        }
        match self.advance() {
            Some(Token::Number(value)) => Ok(ParsedTerm::leaf(Term::Literal(value))),
            Some(Token::Ident(name)) => Ok(ParsedTerm::leaf(Term::Param(name))),
            Some(Token::Minus) => {
                let atom = self.parse_atom(depth + 1)?;
                if let Term::Literal(value) = atom.term {
                    Ok(ParsedTerm::leaf(Term::Literal(-value)))
                } else {
                    ParsedTerm::binary(ArithOp::Sub, ParsedTerm::leaf(Term::Literal(0.0)), atom)
                }
            }
            Some(Token::LParen) => {
                let term = self.parse_term(depth + 1)?;
                match self.advance() {
                    Some(Token::RParen) => Ok(term),
                    other => Err(format!("expected ')', found {other:?}")),
                }
            }
            other => Err(format!(
                "expected a number, identifier, '-', or '(', found {other:?}"
            )),
        }
    }
}

pub(crate) fn is_parameter_name(name: &str) -> bool {
    matches!(tokenize(name).as_deref(), Ok([Token::Ident(parsed)]) if parsed == name)
}

pub(crate) fn parse_constraint(
    entity_id: &str,
    input: &str,
) -> Result<ConstraintExpr, KnowledgeError> {
    let result = (|| {
        let mut parser = ExprParser {
            tokens: tokenize(input)?,
            pos: 0,
        };
        let expression = parser.parse_conjunction()?;
        if parser.pos != parser.tokens.len() {
            return Err(format!("unexpected trailing input in constraint: {input}"));
        }
        Ok(expression)
    })();
    result.map_err(|message| KnowledgeError::ConstraintParseError {
        entity_id: entity_id.to_owned(),
        constraint: input.to_owned(),
        message,
    })
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
    fn evaluates_literals_and_parameters() {
        let values = params(&[("b", 3.0)]);
        assert_eq!(Term::Literal(2.0).evaluate(&values), 2.0);
        assert_eq!(Term::Param("b".to_owned()).evaluate(&values), 3.0);
    }

    #[test]
    fn evaluates_every_arithmetic_operator() {
        let values = params(&[]);
        fn binary(op: ArithOp, left: f64, right: f64) -> Term {
            Term::BinaryOp {
                op,
                left: Box::new(Term::Literal(left)),
                right: Box::new(Term::Literal(right)),
            }
        }
        assert_eq!(binary(ArithOp::Add, 2.0, 3.0).evaluate(&values), 5.0);
        assert_eq!(binary(ArithOp::Sub, 5.0, 3.0).evaluate(&values), 2.0);
        assert_eq!(binary(ArithOp::Mul, 4.0, 3.0).evaluate(&values), 12.0);
        assert_eq!(binary(ArithOp::Div, 9.0, 3.0).evaluate(&values), 3.0);
    }

    #[test]
    fn holds_evaluates_every_comparison_operator() {
        let values = params(&[]);
        fn comparison(op: CompareOp, left: f64, right: f64) -> ConstraintExpr {
            ConstraintExpr::Comparison {
                left: Term::Literal(left),
                op,
                right: Term::Literal(right),
            }
        }
        assert!(comparison(CompareOp::Eq, 1.0, 1.0).holds(&values));
        assert!(comparison(CompareOp::Ne, 1.0, 2.0).holds(&values));
        assert!(comparison(CompareOp::Ge, 2.0, 2.0).holds(&values));
        assert!(comparison(CompareOp::Le, 2.0, 2.0).holds(&values));
        assert!(comparison(CompareOp::Gt, 3.0, 2.0).holds(&values));
        assert!(comparison(CompareOp::Lt, 2.0, 3.0).holds(&values));
        assert!(!comparison(CompareOp::Gt, 2.0, 3.0).holds(&values));
    }

    #[test]
    fn holds_requires_every_conjunct_to_hold() {
        let values = params(&[("b", 4.0)]);
        let expression = parse_constraint("family", "b >= 1 and b <= 10").unwrap();
        assert!(expression.holds(&values));

        let expression = parse_constraint("family", "b >= 1 and b <= 3").unwrap();
        assert!(!expression.holds(&values));
    }

    #[test]
    fn parses_conjunctions_and_arithmetic_with_precedence() {
        let expression = parse_constraint("family", "a == 2 * (3 + 1) and b <= a").unwrap();
        assert!(matches!(expression, ConstraintExpr::All(parts) if parts.len() == 2));
    }

    #[test]
    fn rejects_trailing_garbage_with_context() {
        assert!(
            matches!(parse_constraint("problem.family", "a < b c"), Err(KnowledgeError::ConstraintParseError { entity_id, .. }) if entity_id == "problem.family")
        );
    }

    #[test]
    fn limits_parenthesis_and_unary_minus_nesting() {
        for prefix in ["(", "-", "-("] {
            let suffix = if prefix.contains('(') { ")" } else { "" };
            let limit = MAX_NESTING_DEPTH / prefix.len();
            let input = format!("b >= {}5{}", prefix.repeat(limit), suffix.repeat(limit));
            assert!(parse_constraint("problem.family", &input).is_ok());
            for depth in [limit + 1, 10_000] {
                let input = format!("b >= {}5{}", prefix.repeat(depth), suffix.repeat(depth));
                assert!(matches!(
                    parse_constraint("problem.family", &input),
                    Err(KnowledgeError::ConstraintParseError { entity_id, constraint, message })
                        if entity_id == "problem.family"
                            && constraint == input
                            && message.contains("maximum depth")
                ));
            }
        }
    }

    #[test]
    fn parses_unary_minus_with_atom_precedence() {
        fn binary(op: ArithOp, left: Term, right: Term) -> Term {
            Term::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        let a = Term::Param("a".to_owned());
        let negative_a = binary(ArithOp::Sub, Term::Literal(0.0), a.clone());
        for (input, right) in [
            ("-5", Term::Literal(-5.0)),
            ("--5", Term::Literal(5.0)),
            ("-a", negative_a.clone()),
            (
                "-a * 2",
                binary(ArithOp::Mul, negative_a, Term::Literal(2.0)),
            ),
            (
                "a * -2",
                binary(ArithOp::Mul, a.clone(), Term::Literal(-2.0)),
            ),
            (
                "a - -2",
                binary(ArithOp::Sub, a.clone(), Term::Literal(-2.0)),
            ),
            (
                "-(a + 2)",
                binary(
                    ArithOp::Sub,
                    Term::Literal(0.0),
                    binary(ArithOp::Add, a, Term::Literal(2.0)),
                ),
            ),
        ] {
            assert_eq!(
                parse_constraint("problem.family", &format!("b >= {input}")).unwrap(),
                ConstraintExpr::Comparison {
                    left: Term::Param("b".to_owned()),
                    op: CompareOp::Ge,
                    right,
                },
                "incorrect unary-minus parse for {input}"
            );
        }
        for input in ["b >= -", "b >= -()", "b >= -*5"] {
            assert!(matches!(
                parse_constraint("problem.family", input),
                Err(KnowledgeError::ConstraintParseError { .. })
            ));
        }
    }

    #[test]
    fn bounds_actual_tree_depth_across_operators_and_grouping() {
        for operator in ["+", "-", "*", "/"] {
            let at_limit = format!("{}1", format!("1{operator}").repeat(MAX_NESTING_DEPTH));
            assert!(parse_constraint("family", &format!("b >= {at_limit}")).is_ok());
            for too_deep in [
                format!("{at_limit}{operator}1"),
                format!("({at_limit}) + 1"),
                format!("1 + ({at_limit})"),
                format!("-({at_limit})"),
            ] {
                assert!(
                    matches!(parse_constraint("family", &format!("b >= {too_deep}")),
                    Err(KnowledgeError::ConstraintParseError { message, .. }) if message.contains("tree exceeds maximum depth")),
                    "accepted {too_deep}"
                );
            }
        }
    }
}
