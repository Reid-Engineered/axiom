use std::iter::Peekable;
use std::str::Chars;

use serde::{Deserialize, Serialize};

use super::error::KnowledgeError;

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
        let left = self.parse_term()?;
        let op = match self.advance() {
            Some(Token::Eq) => CompareOp::Eq,
            Some(Token::Ne) => CompareOp::Ne,
            Some(Token::Ge) => CompareOp::Ge,
            Some(Token::Le) => CompareOp::Le,
            Some(Token::Gt) => CompareOp::Gt,
            Some(Token::Lt) => CompareOp::Lt,
            other => return Err(format!("expected a comparison operator, found {other:?}")),
        };
        let right = self.parse_term()?;
        Ok(ConstraintExpr::Comparison { left, op, right })
    }

    fn parse_term(&mut self) -> Result<Term, String> {
        let mut left = self.parse_factor()?;
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
                right: Box::new(self.parse_factor()?),
            };
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Term, String> {
        let mut left = self.parse_atom()?;
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
                right: Box::new(self.parse_atom()?),
            };
        }
        Ok(left)
    }

    fn parse_atom(&mut self) -> Result<Term, String> {
        match self.advance() {
            Some(Token::Number(value)) => Ok(Term::Literal(value)),
            Some(Token::Ident(name)) => Ok(Term::Param(name)),
            Some(Token::LParen) => {
                let term = self.parse_term()?;
                match self.advance() {
                    Some(Token::RParen) => Ok(term),
                    other => Err(format!("expected ')', found {other:?}")),
                }
            }
            other => Err(format!(
                "expected a number, identifier, or '(', found {other:?}"
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
}
