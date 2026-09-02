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
        let left = self.parse_term(0)?;
        let op = match self.advance() {
            Some(Token::Eq) => CompareOp::Eq,
            Some(Token::Ne) => CompareOp::Ne,
            Some(Token::Ge) => CompareOp::Ge,
            Some(Token::Le) => CompareOp::Le,
            Some(Token::Gt) => CompareOp::Gt,
            Some(Token::Lt) => CompareOp::Lt,
            other => return Err(format!("expected a comparison operator, found {other:?}")),
        };
        let right = self.parse_term(0)?;
        Ok(ConstraintExpr::Comparison { left, op, right })
    }

    fn parse_term(&mut self, depth: usize) -> Result<Term, String> {
        let mut left = self.parse_factor(depth)?;
        loop {
            let op = match self.tokens.get(self.pos) {
                Some(Token::Plus) => ArithOp::Add,
                Some(Token::Minus) => ArithOp::Sub,
                _ => break,
            };
            self.pos += 1;
            left = Term::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(self.parse_factor(depth)?),
            };
        }
        Ok(left)
    }

    fn parse_factor(&mut self, depth: usize) -> Result<Term, String> {
        let mut left = self.parse_atom(depth)?;
        loop {
            let op = match self.tokens.get(self.pos) {
                Some(Token::Star) => ArithOp::Mul,
                Some(Token::Slash) => ArithOp::Div,
                _ => break,
            };
            self.pos += 1;
            left = Term::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(self.parse_atom(depth)?),
            };
        }
        Ok(left)
    }

    fn parse_atom(&mut self, depth: usize) -> Result<Term, String> {
        if depth > MAX_NESTING_DEPTH {
            return Err(format!(
                "constraint nesting exceeds maximum depth {MAX_NESTING_DEPTH}"
            ));
        }
        match self.advance() {
            Some(Token::Number(value)) => Ok(Term::Literal(value)),
            Some(Token::Ident(name)) => Ok(Term::Param(name)),
            Some(Token::Minus) => match self.parse_atom(depth + 1)? {
                Term::Literal(value) => Ok(Term::Literal(-value)),
                term => Ok(Term::BinaryOp {
                    op: ArithOp::Sub,
                    left: Box::new(Term::Literal(0.0)),
                    right: Box::new(term),
                }),
            },
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
}
