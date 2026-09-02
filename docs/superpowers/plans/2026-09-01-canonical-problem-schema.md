# Canonical Problem Schema Implementation Plan

> **For agentic workers:** this plan is handed to Codex to implement directly against `.ai/tasks/`, `AGENTS.md`, and `CLAUDE.md`'s normal workflow — not executed via `superpowers:subagent-driven-development` or `superpowers:executing-plans`. Steps use checkbox (`- [ ]`) syntax for tracking regardless of executor.

**Goal:** Add `ProblemFamily` (authored, generative-problem-template Knowledge Package
content) and `ProblemInstance` (runtime generation-output shape) to `src-tauri/src/knowledge/`,
following every convention that module already established for `Concept`/`Objective`/`Example`.

**Architecture:** Every new file mirrors an existing sibling exactly — `problem_family.rs`
mirrors `objective.rs`/`example.rs`'s `parse_x_file()` shape, `problem_family_body.rs`
mirrors `example_body.rs`'s heading-parser shape, new `raw.rs`/`types.rs`/`error.rs`
additions extend the existing enums/structs rather than introducing parallel machinery. The
one genuinely new piece is `constraint.rs`: a small hand-written recursive-descent parser
(no new Cargo dependency) turning authored strings like `"b <= coeff"` into a real,
evaluable expression tree.

**Tech Stack:** Rust, `serde`/`toml` (already a dependency), no new crates.

**Spec:** `docs/superpowers/specs/2026-09-01-canonical-problem-schema-design.md`

## Global Constraints

- New code lives in `src-tauri/src/knowledge/` only — no Tauri command, no frontend
  change, no wiring outside this module (confirmed: nothing in `src-tauri/src/commands/`
  references `KnowledgePackage` yet).
- Every new file follows the exact conventions already established:
  `#[serde(deny_unknown_fields)]` on every `Raw*` struct, `#[serde(default)]` on every
  optional frontmatter field, `pub(crate)` on internal parse/validate functions,
  `KnowledgeError` as the one error type (new variants added to the existing flat enum in
  `error.rs`, with `Display` arms), inline `#[cfg(test)] mod tests` in the file the code
  under test lives in.
- `ProblemFamily` files use the `+++`-delimited TOML frontmatter + Markdown body format —
  identical grammar to every other entity (`src-tauri/src/knowledge/frontmatter.rs`'s
  `split_frontmatter`, reused unchanged).
- `ProblemFamily` loads in the *same* `discover_entities`/`validate_references` pass as
  `Concept`/`Objective`/`Example` — not a separate loader, sharing the in-memory
  concept/objective maps already built during that pass.
- Domain-validity constraints (`"for all x in [a,b]"`-style claims) are explicitly **out of
  scope** for this schema — only parameter-arithmetic constraints get parsed/evaluated here.

---

### Task 1: `ProblemFamilyId` and core value types

**Files:**
- Modify: `src-tauri/src/knowledge/identifier.rs`
- Modify: `src-tauri/src/knowledge/types.rs`
- Modify: `src-tauri/src/knowledge/mod.rs`

**Interfaces:**
- Produces: `ProblemFamilyId` (via the existing `knowledge_id!` macro), `ParameterType`,
  `Bound`, `ParameterSpec`, `ResponseType`, `CanonicalSolution`, `Hint`,
  `ProblemFamilyStatus`, `ProblemFamily` — every later task in this plan consumes these
  exact names and fields.

- [ ] **Step 1: Add `ProblemFamilyId`**

In `src-tauri/src/knowledge/identifier.rs`, find this line:

```rust
knowledge_id!(SourceId);
```

Add immediately after it:

```rust
knowledge_id!(ProblemFamilyId);
```

- [ ] **Step 2: Write the failing test for `ProblemFamily` types**

In `src-tauri/src/knowledge/types.rs`, add to the existing `#[cfg(test)] mod tests` block
(inside the `mod tests { ... }` that already exists at the bottom of the file, alongside
the existing `use` statements and `example_round_trips_through_json` test):

```rust
    #[test]
    fn problem_family_round_trips_through_json() {
        use crate::knowledge::identifier::ProblemFamilyId;

        let family = ProblemFamily {
            id: ProblemFamilyId::new("problem.shell_y_poly").unwrap(),
            concept_id: ConceptId::new("shell.method_vertical_axis").unwrap(),
            objective_ids: vec![ObjectiveId::new("shell.setup_radius_height_y_axis").unwrap()],
            difficulty: DifficultyRange { min: 1, max: 2 },
            generator: GeneratorRef {
                id: "gen-shell-y-poly".to_owned(),
                version: 1,
            },
            parameters: std::collections::BTreeMap::from([(
                "coeff".to_owned(),
                ParameterSpec {
                    kind: ParameterType::Integer,
                    value: None,
                    min: Some(Bound::Literal(2.0)),
                    max: Some(Bound::Literal(6.0)),
                    description: Some("Linear coefficient".to_owned()),
                },
            )]),
            constraints: vec![],
            prompt: "Define R...".to_owned(),
            solution_structure: "V = ...".to_owned(),
            response_type: ResponseType::SymbolicExpression,
            canonical_solution: CanonicalSolution::Symbolic {
                expression: "2*pi*(coeff*b^3/3 - b^4/4)".to_owned(),
            },
            hints: vec![Hint {
                level: 1,
                text: "Identify the shell radius and height.".to_owned(),
            }],
            provenance_refs: vec![ProvenanceRef {
                source_id: SourceId::new("src.openstax_calc2").unwrap(),
                locator: None,
                kind: ProvenanceKind::Direct,
            }],
            status: ProblemFamilyStatus::Verified,
        };

        let json = serde_json::to_string(&family).unwrap();
        let round_tripped: ProblemFamily = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, family);
    }
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test --locked problem_family_round_trips_through_json`
Expected: FAIL — `ProblemFamily` (and the other new types) not found in this scope.

- [ ] **Step 4: Write the types**

In `src-tauri/src/knowledge/types.rs`, add after the existing `Example` struct definition
(and its derive line matches the file's existing pattern —
`#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]` — copy it exactly):

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProblemFamily {
    pub id: ProblemFamilyId,
    pub concept_id: ConceptId,
    pub objective_ids: Vec<ObjectiveId>,
    pub difficulty: DifficultyRange,
    pub generator: GeneratorRef,
    pub parameters: std::collections::BTreeMap<String, ParameterSpec>,
    pub constraints: Vec<crate::knowledge::constraint::ConstraintExpr>,
    pub prompt: String,
    pub solution_structure: String,
    pub response_type: ResponseType,
    pub canonical_solution: CanonicalSolution,
    pub hints: Vec<Hint>,
    pub provenance_refs: Vec<ProvenanceRef>,
    pub status: ProblemFamilyStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DifficultyRange {
    pub min: u8,
    pub max: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneratorRef {
    pub id: String,
    pub version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    Integer,
    Float,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Bound {
    Literal(f64),
    Reference {
        parameter: String,
        #[serde(default)]
        offset: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterSpec {
    #[serde(rename = "type")]
    pub kind: ParameterType,
    #[serde(default)]
    pub value: Option<Bound>,
    #[serde(default)]
    pub min: Option<Bound>,
    #[serde(default)]
    pub max: Option<Bound>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResponseType {
    SymbolicExpression,
    Numeric,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CanonicalSolution {
    Symbolic { expression: String },
    Numeric { value: f64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hint {
    pub level: u32,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProblemFamilyStatus {
    Verified,
    NeedsReview,
}
```

Note: `CanonicalSolution` uses `#[serde(tag = "kind", ...)]` for the *domain type*'s JSON
round-trip (this is what `serde_json` needs to disambiguate the variant) — this is separate
from the raw TOML frontmatter shape, which Task 3 handles differently (the TOML frontmatter
doesn't have a `kind` tag; which variant to build is inferred from `response_type` instead).

Add `use super::identifier::ProblemFamilyId;` to the existing `use` line at the top of
`types.rs` that already imports `ConceptId, ExampleId, KnowledgePackageId, ObjectiveId,
SourceId` — add `ProblemFamilyId` to that same import list.

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo test --locked problem_family_round_trips_through_json`
Expected: still FAILS at this point — `crate::knowledge::constraint::ConstraintExpr` doesn't
exist yet (Task 2 creates it). This is expected; proceed to Task 2 before re-running.

- [ ] **Step 6: Add a stub `constraint` module so Task 1 compiles standalone**

In `src-tauri/src/knowledge/mod.rs`, find:

```rust
mod concept;
```

Add immediately before it:

```rust
mod constraint;
```

Create `src-tauri/src/knowledge/constraint.rs` with just enough to compile (Task 2 replaces
this with the real parser):

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstraintExpr {
    Placeholder,
}
```

- [ ] **Step 7: Run the test to verify it passes**

Run: `cargo test --locked problem_family_round_trips_through_json`
Expected: PASS.

- [ ] **Step 8: Update `mod.rs` exports**

In `src-tauri/src/knowledge/mod.rs`, find:

```rust
pub use identifier::{ConceptId, ExampleId, KnowledgePackageId, ObjectiveId, SourceId};
```

Replace with:

```rust
pub use identifier::{
    ConceptId, ExampleId, KnowledgePackageId, ObjectiveId, ProblemFamilyId, SourceId,
};
```

Find:

```rust
pub use types::{
    Concept, Example, KnowledgePackage, Objective, ProvenanceKind, ProvenanceRef, Source,
    SourceLocator,
};
```

Replace with:

```rust
pub use types::{
    Bound, CanonicalSolution, Concept, DifficultyRange, Example, GeneratorRef, Hint,
    KnowledgePackage, Objective, ParameterSpec, ParameterType, ProblemFamily,
    ProblemFamilyStatus, ProvenanceKind, ProvenanceRef, ResponseType, Source, SourceLocator,
};
```

- [ ] **Step 9: Run the full workspace test suite and lints**

Run: `cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`
Expected: all pass. (`cargo fmt` may reformat the code you just wrote — if so, run
`cargo fmt --all` once, then re-run the check.)

- [ ] **Step 10: Commit**

```bash
git add src-tauri/src/knowledge/identifier.rs src-tauri/src/knowledge/types.rs src-tauri/src/knowledge/mod.rs src-tauri/src/knowledge/constraint.rs
git commit -m "feat(knowledge): add ProblemFamily core types and ProblemFamilyId"
```

---

### Task 2: Constraint expression parser

**Files:**
- Modify: `src-tauri/src/knowledge/constraint.rs` (replaces Task 1's stub)
- Modify: `src-tauri/src/knowledge/error.rs`

**Interfaces:**
- Consumes: `KnowledgeError` (from Task 1's context, already exists).
- Produces: `ConstraintExpr`, `Term`, `ArithOp`, `CompareOp` (real types, replacing the
  stub), `pub(crate) fn parse_constraint(entity_id: &str, input: &str) ->
  Result<ConstraintExpr, KnowledgeError>` — Task 4 calls this directly.

- [ ] **Step 1: Add the error variant**

In `src-tauri/src/knowledge/error.rs`, find the `KnowledgeError` enum's last variant
(`ReverseDuplicateRelated { first, second }`) and add immediately after it, before the
enum's closing `}`:

```rust
    ConstraintParseError {
        entity_id: String,
        constraint: String,
        message: String,
    },
```

In the `Display` impl's `match self { ... }`, find the last arm
(`Self::ReverseDuplicateRelated { first, second } => write!(...)`) and add immediately
after its closing `}`, before the match's closing `}`:

```rust
            Self::ConstraintParseError {
                entity_id,
                constraint,
                message,
            } => write!(
                f,
                "{entity_id}: constraint \"{constraint}\" failed to parse: {message}"
            ),
```

- [ ] **Step 2: Write the failing tests**

Replace the entire contents of `src-tauri/src/knowledge/constraint.rs` with:

```rust
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_simple_comparison() {
        let expr = parse_constraint("family", "b <= coeff").unwrap();
        assert_eq!(
            expr,
            ConstraintExpr::Comparison {
                left: Term::Param("b".to_owned()),
                op: CompareOp::Le,
                right: Term::Param("coeff".to_owned()),
            }
        );
    }

    #[test]
    fn parses_a_conjunction() {
        let expr = parse_constraint("family", "coeff >= 2 and coeff <= 6").unwrap();
        assert_eq!(
            expr,
            ConstraintExpr::All(vec![
                ConstraintExpr::Comparison {
                    left: Term::Param("coeff".to_owned()),
                    op: CompareOp::Ge,
                    right: Term::Literal(2.0),
                },
                ConstraintExpr::Comparison {
                    left: Term::Param("coeff".to_owned()),
                    op: CompareOp::Le,
                    right: Term::Literal(6.0),
                },
            ])
        );
    }

    #[test]
    fn parses_arithmetic_terms() {
        let expr = parse_constraint("family", "a + b < 10").unwrap();
        assert_eq!(
            expr,
            ConstraintExpr::Comparison {
                left: Term::BinaryOp {
                    op: ArithOp::Add,
                    left: Box::new(Term::Param("a".to_owned())),
                    right: Box::new(Term::Param("b".to_owned())),
                },
                op: CompareOp::Lt,
                right: Term::Literal(10.0),
            }
        );
    }

    #[test]
    fn respects_operator_precedence() {
        // "a == 2 * 3 + 1" must parse as a == ((2*3)+1), not a == (2*(3+1))
        let expr = parse_constraint("family", "a == 2 * 3 + 1").unwrap();
        assert_eq!(
            expr,
            ConstraintExpr::Comparison {
                left: Term::Param("a".to_owned()),
                op: CompareOp::Eq,
                right: Term::BinaryOp {
                    op: ArithOp::Add,
                    left: Box::new(Term::BinaryOp {
                        op: ArithOp::Mul,
                        left: Box::new(Term::Literal(2.0)),
                        right: Box::new(Term::Literal(3.0)),
                    }),
                    right: Box::new(Term::Literal(1.0)),
                },
            }
        );
    }

    #[test]
    fn parenthesized_terms_override_precedence() {
        let expr = parse_constraint("family", "a == 2 * (3 + 1)").unwrap();
        assert_eq!(
            expr,
            ConstraintExpr::Comparison {
                left: Term::Param("a".to_owned()),
                op: CompareOp::Eq,
                right: Term::BinaryOp {
                    op: ArithOp::Mul,
                    left: Box::new(Term::Literal(2.0)),
                    right: Box::new(Term::BinaryOp {
                        op: ArithOp::Add,
                        left: Box::new(Term::Literal(3.0)),
                        right: Box::new(Term::Literal(1.0)),
                    }),
                },
            }
        );
    }

    #[test]
    fn syntax_error_is_reported_with_context() {
        assert!(matches!(
            parse_constraint("problem.family_x", "b <= "),
            Err(KnowledgeError::ConstraintParseError { entity_id, constraint, .. })
                if entity_id == "problem.family_x" && constraint == "b <= "
        ));
    }

    #[test]
    fn trailing_garbage_is_rejected() {
        assert!(matches!(
            parse_constraint("family", "a < b c"),
            Err(KnowledgeError::ConstraintParseError { .. })
        ));
    }

    #[test]
    fn all_six_comparison_operators_parse() {
        for (text, expected) in [
            ("a == b", CompareOp::Eq),
            ("a != b", CompareOp::Ne),
            ("a >= b", CompareOp::Ge),
            ("a <= b", CompareOp::Le),
            ("a > b", CompareOp::Gt),
            ("a < b", CompareOp::Lt),
        ] {
            let expr = parse_constraint("family", text).unwrap();
            assert_eq!(
                expr,
                ConstraintExpr::Comparison {
                    left: Term::Param("a".to_owned()),
                    op: expected,
                    right: Term::Param("b".to_owned()),
                }
            );
        }
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test --locked --lib knowledge::constraint`
Expected: FAIL — `parse_constraint` not found.

- [ ] **Step 4: Implement the tokenizer and recursive-descent parser**

Add the following to `src-tauri/src/knowledge/constraint.rs`, *above* the `#[cfg(test)]`
line (so it's part of the real module, not the test module):

```rust
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
            let mut num = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() || c == '.' {
                    num.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            let value = num
                .parse::<f64>()
                .map_err(|_| format!("invalid number: {num}"))?;
            tokens.push(Token::Number(value));
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let mut ident = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    ident.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if ident == "and" {
                tokens.push(Token::And);
            } else {
                tokens.push(Token::Ident(ident));
            }
            continue;
        }
        match c {
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Eq);
                } else {
                    return Err("expected '==' but found a single '='".to_owned());
                }
            }
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Ne);
                } else {
                    return Err("expected '!=' but found a bare '!'".to_owned());
                }
            }
            '>' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Ge);
                } else {
                    tokens.push(Token::Gt);
                }
            }
            '<' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Le);
                } else {
                    tokens.push(Token::Lt);
                }
            }
            other => return Err(format!("unexpected character: {other}")),
        }
    }

    Ok(tokens)
}

struct ExprParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl ExprParser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        token
    }

    fn parse_conjunction(&mut self) -> Result<ConstraintExpr, String> {
        let mut comparisons = vec![self.parse_comparison()?];
        while self.peek() == Some(&Token::And) {
            self.advance();
            comparisons.push(self.parse_comparison()?);
        }
        if comparisons.len() == 1 {
            Ok(comparisons.into_iter().next().expect("just pushed one"))
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
            let op = match self.peek() {
                Some(Token::Plus) => ArithOp::Add,
                Some(Token::Minus) => ArithOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_factor()?;
            left = Term::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Term, String> {
        let mut left = self.parse_atom()?;
        loop {
            let op = match self.peek() {
                Some(Token::Star) => ArithOp::Mul,
                Some(Token::Slash) => ArithOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_atom()?;
            left = Term::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_atom(&mut self) -> Result<Term, String> {
        match self.advance() {
            Some(Token::Number(value)) => Ok(Term::Literal(value)),
            Some(Token::Ident(name)) => Ok(Term::Param(name)),
            Some(Token::LParen) => {
                let inner = self.parse_term()?;
                match self.advance() {
                    Some(Token::RParen) => Ok(inner),
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
        let tokens = tokenize(input)?;
        let mut parser = ExprParser::new(tokens);
        let expr = parser.parse_conjunction()?;
        if parser.pos != parser.tokens.len() {
            return Err(format!("unexpected trailing input in constraint: {input}"));
        }
        Ok(expr)
    })();

    result.map_err(|message| KnowledgeError::ConstraintParseError {
        entity_id: entity_id.to_owned(),
        constraint: input.to_owned(),
        message,
    })
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test --locked --lib knowledge::constraint`
Expected: PASS, 8 tests.

- [ ] **Step 6: Run the full suite and lints**

Run: `cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`
Expected: all pass.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/knowledge/constraint.rs src-tauri/src/knowledge/error.rs
git commit -m "feat(knowledge): add constraint expression parser"
```

---

### Task 3: Raw frontmatter and body parser

**Files:**
- Modify: `src-tauri/src/knowledge/raw.rs`
- Create: `src-tauri/src/knowledge/problem_family_body.rs`
- Modify: `src-tauri/src/knowledge/mod.rs`
- Modify: `src-tauri/src/knowledge/error.rs`

**Interfaces:**
- Consumes: `KnowledgeError` (Task 1/2).
- Produces: `RawProblemFamilyFrontmatter`, `RawParameterSpec`, `RawBound`, `RawHint` (raw.rs
  additions); `pub(crate) fn parse_problem_family_body(entity_id: &str, body: &str) ->
  Result<ParsedProblemFamilyBody, KnowledgeError>` where `ParsedProblemFamilyBody { prompt:
  String, solution_structure: String, hint_texts: Vec<String> }` — Task 4 calls both
  directly.

- [ ] **Step 1: Add the new error variants**

In `src-tauri/src/knowledge/error.rs`, add these variants to the `KnowledgeError` enum
(after `ConstraintParseError` from Task 2):

```rust
    MissingProblemFamilySection {
        entity_id: String,
        section: &'static str,
    },
    DuplicateProblemFamilySection {
        entity_id: String,
        section: &'static str,
    },
    OutOfOrderProblemFamilySection {
        entity_id: String,
        section: &'static str,
    },
    UnknownProblemFamilySection {
        entity_id: String,
        heading: String,
    },
    ContentBeforePrompt {
        entity_id: String,
    },
    InvalidProblemFamilyHintLine {
        entity_id: String,
        line: String,
    },
    ProblemFamilyHintCountMismatch {
        entity_id: String,
        frontmatter_count: usize,
        body_count: usize,
    },
```

Add matching `Display` arms (after `ConstraintParseError`'s arm):

```rust
            Self::MissingProblemFamilySection { entity_id, section } => write!(f, "problem family {entity_id} is missing required section ## {section}"),
            Self::DuplicateProblemFamilySection { entity_id, section } => write!(f, "problem family {entity_id} declares ## {section} more than once"),
            Self::OutOfOrderProblemFamilySection { entity_id, section } => write!(f, "problem family {entity_id}: ## {section} appears out of the required Prompt/Solution/Hints order"),
            Self::UnknownProblemFamilySection { entity_id, heading } => write!(f, "problem family {entity_id} contains unrecognized heading: {heading}"),
            Self::ContentBeforePrompt { entity_id } => write!(f, "problem family {entity_id} has non-whitespace content before ## Prompt"),
            Self::InvalidProblemFamilyHintLine { entity_id, line } => write!(f, "problem family {entity_id}: invalid line under ## Hints (expected \"- <hint>\"): {line}"),
            Self::ProblemFamilyHintCountMismatch { entity_id, frontmatter_count, body_count } => write!(f, "problem family {entity_id} declares {frontmatter_count} hint(s) in frontmatter but ## Hints has {body_count} line(s)"),
```

- [ ] **Step 2: Add raw frontmatter structs**

In `src-tauri/src/knowledge/raw.rs`, add after the existing `RawExampleFrontmatter` struct:

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawProblemFamilyFrontmatter {
    pub id: String,
    pub concept_id: String,
    #[serde(default)]
    pub objective_ids: Vec<String>,
    pub difficulty: RawDifficultyRange,
    pub generator: RawGeneratorRef,
    #[serde(default)]
    pub parameters: std::collections::BTreeMap<String, RawParameterSpec>,
    #[serde(default)]
    pub constraints: Vec<String>,
    pub response_type: String,
    pub canonical_solution: RawCanonicalSolution,
    #[serde(default)]
    pub hints: Vec<RawHint>,
    #[serde(default)]
    pub provenance_refs: Vec<RawProvenanceRef>,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawDifficultyRange {
    pub min: u8,
    pub max: u8,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawGeneratorRef {
    pub id: String,
    pub version: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum RawBound {
    Literal(f64),
    Reference {
        parameter: String,
        #[serde(default)]
        offset: f64,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawParameterSpec {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub value: Option<RawBound>,
    #[serde(default)]
    pub min: Option<RawBound>,
    #[serde(default)]
    pub max: Option<RawBound>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawCanonicalSolution {
    #[serde(default)]
    pub expression: Option<String>,
    #[serde(default)]
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawHint {
    pub level: u32,
}
```

Note: `RawCanonicalSolution` deliberately has both fields optional at the raw-parse layer —
which one is *required* depends on `response_type`, so that check happens in Task 4's
structural validation, not here (this file's job is only "does this TOML shape parse,"
matching how `RawParameterSpec.kind` stays a raw `String` here and gets validated into
`ParameterType` later too).

- [ ] **Step 3: Write the failing tests for the body parser**

Create `src-tauri/src/knowledge/problem_family_body.rs`:

```rust
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
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_BODY: &str = "## Prompt\n\nFind the volume.\n\n## Solution\n\nV = 8pi/3.\n\n## Hints\n\n- Identify the radius first.\n- Then the height.\n";

    #[test]
    fn valid_body_parses_all_three_sections() {
        let parsed = parse_problem_family_body("problem.shell_basic", VALID_BODY).unwrap();
        assert_eq!(parsed.prompt, "Find the volume.");
        assert_eq!(parsed.solution_structure, "V = 8pi/3.");
        assert_eq!(
            parsed.hint_texts,
            vec!["Identify the radius first.", "Then the height."]
        );
    }

    #[test]
    fn hints_is_optional() {
        let body = "## Prompt\n\nFind the volume.\n\n## Solution\n\nV = 8pi/3.\n";
        let parsed = parse_problem_family_body("problem.shell_basic", body).unwrap();
        assert!(parsed.hint_texts.is_empty());
    }

    #[test]
    fn hint_order_is_preserved() {
        let body =
            "## Prompt\n\nP.\n\n## Solution\n\nS.\n\n## Hints\n\n- first\n- second\n- third\n";
        let parsed = parse_problem_family_body("problem.shell_basic", body).unwrap();
        assert_eq!(parsed.hint_texts, vec!["first", "second", "third"]);
    }

    #[test]
    fn missing_prompt_is_rejected() {
        let body = "## Solution\n\nV = 8pi/3.\n";
        assert!(matches!(
            parse_problem_family_body("problem.shell_basic", body),
            Err(KnowledgeError::MissingProblemFamilySection {
                section: "Prompt",
                ..
            })
        ));
    }

    #[test]
    fn missing_solution_is_rejected() {
        let body = "## Prompt\n\nFind the volume.\n";
        assert!(matches!(
            parse_problem_family_body("problem.shell_basic", body),
            Err(KnowledgeError::MissingProblemFamilySection {
                section: "Solution",
                ..
            })
        ));
    }

    #[test]
    fn duplicate_heading_is_rejected() {
        let body = "## Prompt\n\nP.\n\n## Solution\n\nS1.\n\n## Solution\n\nS2.\n";
        assert!(matches!(
            parse_problem_family_body("problem.shell_basic", body),
            Err(KnowledgeError::DuplicateProblemFamilySection {
                section: "Solution",
                ..
            })
        ));
    }

    #[test]
    fn out_of_order_heading_is_rejected() {
        let body = "## Solution\n\nS.\n\n## Prompt\n\nP.\n";
        assert!(matches!(
            parse_problem_family_body("problem.shell_basic", body),
            Err(KnowledgeError::OutOfOrderProblemFamilySection {
                section: "Prompt",
                ..
            })
        ));
    }

    #[test]
    fn unknown_heading_is_rejected() {
        let body = "## Prompt\n\nP.\n\n## Solution\n\nS.\n\n## Notes\n\nExtra.\n";
        assert!(matches!(
            parse_problem_family_body("problem.shell_basic", body),
            Err(KnowledgeError::UnknownProblemFamilySection { .. })
        ));
    }

    #[test]
    fn content_before_prompt_is_rejected() {
        let body = "Stray intro text.\n\n## Prompt\n\nP.\n\n## Solution\n\nS.\n";
        assert!(matches!(
            parse_problem_family_body("problem.shell_basic", body),
            Err(KnowledgeError::ContentBeforePrompt { .. })
        ));
    }

    #[test]
    fn non_list_hints_content_is_rejected() {
        let body =
            "## Prompt\n\nP.\n\n## Solution\n\nS.\n\n## Hints\n\nJust a paragraph, not a list.\n";
        assert!(matches!(
            parse_problem_family_body("problem.shell_basic", body),
            Err(KnowledgeError::InvalidProblemFamilyHintLine { .. })
        ));
    }
}
```

- [ ] **Step 4: Run the tests to verify they fail**

Run: `cargo test --locked --lib knowledge::problem_family_body`
Expected: FAIL — `unimplemented!()` panics on the first test.

- [ ] **Step 5: Implement the body parser**

Replace the `unimplemented!()` function body in `src-tauri/src/knowledge/problem_family_body.rs`
with (this is a direct structural port of `example_body.rs`'s `parse_example_body`, renamed
for this entity's headings and error variants):

```rust
pub(crate) fn parse_problem_family_body(
    entity_id: &str,
    body: &str,
) -> Result<ParsedProblemFamilyBody, KnowledgeError> {
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
    if let Some(finished) = current.take() {
        sections.push(finished);
    }

    if preamble.iter().any(|line| !line.trim().is_empty()) {
        return Err(KnowledgeError::ContentBeforePrompt {
            entity_id: entity_id.to_owned(),
        });
    }

    let mut seen: Vec<&str> = Vec::new();
    for (heading, _) in &sections {
        if seen.contains(heading) {
            return Err(KnowledgeError::DuplicateProblemFamilySection {
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
                return Err(KnowledgeError::OutOfOrderProblemFamilySection {
                    entity_id: entity_id.to_owned(),
                    section: section_name(heading),
                });
            }
        }
        highest_seen = Some(position);
    }

    let prompt = section_text(&sections, PROMPT, entity_id, "Prompt")?;
    let solution_structure = section_text(&sections, SOLUTION, entity_id, "Solution")?;
    let hint_texts = match sections.iter().find(|(heading, _)| *heading == HINTS) {
        None => Vec::new(),
        Some((_, content)) => parse_hint_lines(entity_id, content)?,
    };

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
    let mut hints = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        match line.strip_prefix("- ") {
            Some(text) => hints.push(text.trim_end_matches('\r').trim().to_owned()),
            None => {
                return Err(KnowledgeError::InvalidProblemFamilyHintLine {
                    entity_id: entity_id.to_owned(),
                    line: (*line).to_owned(),
                })
            }
        }
    }
    Ok(hints)
}
```

Note: unlike `example_body.rs`'s `parse_hints`, this omits the `EmptyHintsSection` rejection
— an empty `## Hints` section (zero bullet lines) is allowed to produce `Vec::new()`, since
Task 4's frontmatter/body hint-count cross-check (comparing this against the frontmatter's
`hints` array length) is what actually catches a real mismatch; a `## Hints` heading present
with zero lines and zero frontmatter entries is a legitimate (if slightly odd) way to author
"no hints," not an error on its own.

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test --locked --lib knowledge::problem_family_body`
Expected: PASS, 9 tests.

- [ ] **Step 7: Register the new module**

In `src-tauri/src/knowledge/mod.rs`, find:

```rust
mod package;
```

Add immediately after it:

```rust
mod problem_family_body;
```

- [ ] **Step 8: Run the full suite and lints**

Run: `cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`
Expected: all pass.

- [ ] **Step 9: Commit**

```bash
git add src-tauri/src/knowledge/raw.rs src-tauri/src/knowledge/problem_family_body.rs src-tauri/src/knowledge/mod.rs src-tauri/src/knowledge/error.rs
git commit -m "feat(knowledge): add ProblemFamily raw frontmatter and body parser"
```

---

### Task 4: `parse_problem_family_file` and per-family structural validation

**Files:**
- Create: `src-tauri/src/knowledge/problem_family.rs`
- Modify: `src-tauri/src/knowledge/mod.rs`
- Modify: `src-tauri/src/knowledge/error.rs`

**Interfaces:**
- Consumes: everything from Tasks 1-3 (`ProblemFamily` and its field types, `parse_constraint`,
  `RawProblemFamilyFrontmatter` and its nested raw types, `parse_problem_family_body`,
  `split_frontmatter`, `convert_provenance_refs`).
- Produces: `pub(crate) fn parse_problem_family_file(path: &Path, raw: &str) ->
  Result<ProblemFamily, KnowledgeError>` — Task 5's `discover.rs` wiring calls this exactly
  the way `parse_objective_file`/`parse_example_file` are already called.

- [ ] **Step 1: Add the remaining error variants**

In `src-tauri/src/knowledge/error.rs`, add (after Task 3's variants):

```rust
    UnknownParameterType {
        entity_id: String,
        value: String,
    },
    DanglingParameterReference {
        entity_id: String,
        parameter: String,
        target: String,
    },
    ParameterReferenceCycle {
        entity_id: String,
        cycle: Vec<String>,
    },
    ParameterValueAndBoundsConflict {
        entity_id: String,
        parameter: String,
    },
    ConstraintUnknownParameter {
        entity_id: String,
        parameter: String,
    },
    UnknownResponseType {
        entity_id: String,
        value: String,
    },
    ResponseTypeSolutionMismatch {
        entity_id: String,
        response_type: &'static str,
    },
    InvalidDifficultyRange {
        entity_id: String,
        min: u8,
        max: u8,
    },
    DuplicateHintLevel {
        entity_id: String,
        level: u32,
    },
    UnknownProblemFamilyStatus {
        entity_id: String,
        value: String,
    },
```

Add matching `Display` arms:

```rust
            Self::UnknownParameterType { entity_id, value } => write!(f, "{entity_id} declares unknown parameter type: {value}"),
            Self::DanglingParameterReference { entity_id, parameter, target } => write!(f, "{entity_id}.parameters.{parameter} references undeclared parameter: {target}"),
            Self::ParameterReferenceCycle { entity_id, cycle } => write!(f, "{entity_id} has a parameter-reference cycle: {}", cycle.join(" -> ")),
            Self::ParameterValueAndBoundsConflict { entity_id, parameter } => write!(f, "{entity_id}.parameters.{parameter} declares both a fixed value and min/max bounds"),
            Self::ConstraintUnknownParameter { entity_id, parameter } => write!(f, "{entity_id} has a constraint referencing undeclared parameter: {parameter}"),
            Self::UnknownResponseType { entity_id, value } => write!(f, "{entity_id} declares unknown response_type: {value}"),
            Self::ResponseTypeSolutionMismatch { entity_id, response_type } => write!(f, "{entity_id}'s canonical_solution does not match response_type {response_type}"),
            Self::InvalidDifficultyRange { entity_id, min, max } => write!(f, "{entity_id} has an invalid difficulty range: min {min} > max {max}"),
            Self::DuplicateHintLevel { entity_id, level } => write!(f, "{entity_id} declares hint level {level} more than once"),
            Self::UnknownProblemFamilyStatus { entity_id, value } => write!(f, "{entity_id} declares unknown status: {value}"),
```

- [ ] **Step 2: Write the failing tests**

Create `src-tauri/src/knowledge/problem_family.rs`:

```rust
use std::path::Path;

use super::error::KnowledgeError;

pub(crate) fn parse_problem_family_file(
    path: &Path,
    raw: &str,
) -> Result<super::types::ProblemFamily, KnowledgeError> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_FAMILY: &str = r#"+++
id = "problem.shell_y_poly"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height_y_axis"]
difficulty = { min = 1, max = 2 }
generator = { id = "gen-shell-y-poly", version = 1 }
response_type = "symbolic-expression"
status = "verified"

[parameters.coeff]
type = "integer"
min = 2
max = 6

[parameters.b]
type = "integer"
min = 1
max = { parameter = "coeff" }

[canonical_solution]
expression = "2*pi*(coeff*b^3/3 - b^4/4)"

[[hints]]
level = 1

[[provenance_refs]]
source_id = "src.openstax_calc2"
kind = "direct"
+++

## Prompt

Define R and find the volume.

## Solution

V = 2*pi*(...)

## Hints

- Identify the shell radius and height.
"#;

    #[test]
    fn valid_family_parses() {
        let family = parse_problem_family_file(
            Path::new("problems/problem.shell_y_poly.md"),
            VALID_FAMILY,
        )
        .unwrap();
        assert_eq!(family.id.as_str(), "problem.shell_y_poly");
        assert_eq!(family.concept_id.as_str(), "shell.method_vertical_axis");
        assert_eq!(family.parameters.len(), 2);
        assert_eq!(family.hints.len(), 1);
        assert_eq!(family.hints[0].text, "Identify the shell radius and height.");
    }

    #[test]
    fn dangling_parameter_reference_is_rejected() {
        let raw = VALID_FAMILY.replace(
            r#"max = { parameter = "coeff" }"#,
            r#"max = { parameter = "nonexistent" }"#,
        );
        assert!(matches!(
            parse_problem_family_file(Path::new("problems/x.md"), &raw),
            Err(KnowledgeError::DanglingParameterReference { .. })
        ));
    }

    #[test]
    fn parameter_reference_cycle_is_rejected() {
        let raw = VALID_FAMILY.replace(
            "[parameters.coeff]\ntype = \"integer\"\nmin = 2\nmax = 6",
            "[parameters.coeff]\ntype = \"integer\"\nmin = 2\nmax = { parameter = \"b\" }",
        );
        assert!(matches!(
            parse_problem_family_file(Path::new("problems/x.md"), &raw),
            Err(KnowledgeError::ParameterReferenceCycle { .. })
        ));
    }

    #[test]
    fn constraint_referencing_unknown_parameter_is_rejected() {
        let raw = VALID_FAMILY.replace(
            "response_type = \"symbolic-expression\"",
            "response_type = \"symbolic-expression\"\nconstraints = [\"z <= coeff\"]",
        );
        assert!(matches!(
            parse_problem_family_file(Path::new("problems/x.md"), &raw),
            Err(KnowledgeError::ConstraintUnknownParameter { .. })
        ));
    }

    #[test]
    fn response_type_solution_mismatch_is_rejected() {
        let raw = VALID_FAMILY.replace(
            "[canonical_solution]\nexpression = \"2*pi*(coeff*b^3/3 - b^4/4)\"",
            "[canonical_solution]\nvalue = 1.0",
        );
        assert!(matches!(
            parse_problem_family_file(Path::new("problems/x.md"), &raw),
            Err(KnowledgeError::ResponseTypeSolutionMismatch { .. })
        ));
    }

    #[test]
    fn invalid_difficulty_range_is_rejected() {
        let raw = VALID_FAMILY.replace("difficulty = { min = 1, max = 2 }", "difficulty = { min = 3, max = 1 }");
        assert!(matches!(
            parse_problem_family_file(Path::new("problems/x.md"), &raw),
            Err(KnowledgeError::InvalidDifficultyRange { .. })
        ));
    }

    #[test]
    fn hint_count_mismatch_is_rejected() {
        let raw = VALID_FAMILY.replace("[[hints]]\nlevel = 1", "[[hints]]\nlevel = 1\n\n[[hints]]\nlevel = 2");
        assert!(matches!(
            parse_problem_family_file(Path::new("problems/x.md"), &raw),
            Err(KnowledgeError::ProblemFamilyHintCountMismatch { .. })
        ));
    }

    #[test]
    fn duplicate_hint_level_is_rejected() {
        let raw = VALID_FAMILY
            .replace("[[hints]]\nlevel = 1", "[[hints]]\nlevel = 1\n\n[[hints]]\nlevel = 1")
            .replace(
                "## Hints\n\n- Identify the shell radius and height.\n",
                "## Hints\n\n- Identify the shell radius and height.\n- Second hint.\n",
            );
        assert!(matches!(
            parse_problem_family_file(Path::new("problems/x.md"), &raw),
            Err(KnowledgeError::DuplicateHintLevel { .. })
        ));
    }

    #[test]
    fn value_and_bounds_conflict_is_rejected() {
        let raw = VALID_FAMILY.replace(
            "[parameters.coeff]\ntype = \"integer\"\nmin = 2\nmax = 6",
            "[parameters.coeff]\ntype = \"integer\"\nvalue = 3\nmin = 2\nmax = 6",
        );
        assert!(matches!(
            parse_problem_family_file(Path::new("problems/x.md"), &raw),
            Err(KnowledgeError::ParameterValueAndBoundsConflict { .. })
        ));
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test --locked --lib knowledge::problem_family`
Expected: FAIL — `unimplemented!()` panics.

- [ ] **Step 4: Implement `parse_problem_family_file`**

Replace the `unimplemented!()` function in `src-tauri/src/knowledge/problem_family.rs` with
the full implementation. Replace the file's top `use` block and function with:

```rust
use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::constraint::{parse_constraint, ConstraintExpr, Term};
use super::error::KnowledgeError;
use super::frontmatter::split_frontmatter;
use super::identifier::{ConceptId, ObjectiveId, ProblemFamilyId};
use super::problem_family_body::parse_problem_family_body;
use super::provenance::convert_provenance_refs;
use super::raw::{RawBound, RawCanonicalSolution, RawParameterSpec, RawProblemFamilyFrontmatter};
use super::types::{
    Bound, CanonicalSolution, DifficultyRange, GeneratorRef, Hint, ParameterSpec, ParameterType,
    ProblemFamily, ProblemFamilyStatus, ResponseType,
};

pub(crate) fn parse_problem_family_file(
    path: &Path,
    raw: &str,
) -> Result<ProblemFamily, KnowledgeError> {
    let (toml_text, body) = split_frontmatter(path, raw)?;
    let fm: RawProblemFamilyFrontmatter =
        toml::from_str(&toml_text).map_err(|source| KnowledgeError::TomlSyntax {
            path: path.to_owned(),
            source,
        })?;

    let id = ProblemFamilyId::new(fm.id)?;
    let entity_id = id.as_str().to_owned();
    let concept_id = ConceptId::new(fm.concept_id)?;
    let objective_ids: Vec<ObjectiveId> = fm
        .objective_ids
        .into_iter()
        .map(ObjectiveId::new)
        .collect::<Result<_, _>>()?;

    if fm.difficulty.min > fm.difficulty.max {
        return Err(KnowledgeError::InvalidDifficultyRange {
            entity_id: entity_id.clone(),
            min: fm.difficulty.min,
            max: fm.difficulty.max,
        });
    }
    let difficulty = DifficultyRange {
        min: fm.difficulty.min,
        max: fm.difficulty.max,
    };
    crate::modules::identifier::validate_identifier(&fm.generator.id).map_err(|_| {
        KnowledgeError::InvalidIdentifier {
            value: fm.generator.id.clone(),
        }
    })?;
    let generator = GeneratorRef {
        id: fm.generator.id,
        version: fm.generator.version,
    };

    let parameters = convert_parameters(&entity_id, fm.parameters)?;
    validate_parameter_references(&entity_id, &parameters)?;

    let constraints: Vec<ConstraintExpr> = fm
        .constraints
        .iter()
        .map(|text| parse_constraint(&entity_id, text))
        .collect::<Result<_, _>>()?;
    for constraint in &constraints {
        validate_constraint_parameters(&entity_id, constraint, &parameters)?;
    }

    let response_type = match fm.response_type.as_str() {
        "symbolic-expression" => ResponseType::SymbolicExpression,
        "numeric" => ResponseType::Numeric,
        other => {
            return Err(KnowledgeError::UnknownResponseType {
                entity_id: entity_id.clone(),
                value: other.to_owned(),
            })
        }
    };
    let canonical_solution = convert_canonical_solution(&entity_id, response_type, fm.canonical_solution)?;

    let status = match fm.status.as_str() {
        "verified" => ProblemFamilyStatus::Verified,
        "needs-review" => ProblemFamilyStatus::NeedsReview,
        other => {
            return Err(KnowledgeError::UnknownProblemFamilyStatus {
                entity_id: entity_id.clone(),
                value: other.to_owned(),
            })
        }
    };

    let provenance_refs = convert_provenance_refs(&entity_id, fm.provenance_refs)?;

    let parsed_body = parse_problem_family_body(&entity_id, &body)?;

    if fm.hints.len() != parsed_body.hint_texts.len() {
        return Err(KnowledgeError::ProblemFamilyHintCountMismatch {
            entity_id: entity_id.clone(),
            frontmatter_count: fm.hints.len(),
            body_count: parsed_body.hint_texts.len(),
        });
    }
    let mut seen_levels = HashSet::new();
    let mut hints = Vec::with_capacity(fm.hints.len());
    for (raw_hint, text) in fm.hints.into_iter().zip(parsed_body.hint_texts.into_iter()) {
        if !seen_levels.insert(raw_hint.level) {
            return Err(KnowledgeError::DuplicateHintLevel {
                entity_id: entity_id.clone(),
                level: raw_hint.level,
            });
        }
        hints.push(Hint {
            level: raw_hint.level,
            text,
        });
    }

    Ok(ProblemFamily {
        id,
        concept_id,
        objective_ids,
        difficulty,
        generator,
        parameters,
        constraints,
        prompt: parsed_body.prompt,
        solution_structure: parsed_body.solution_structure,
        response_type,
        canonical_solution,
        hints,
        provenance_refs,
        status,
    })
}

fn convert_parameters(
    entity_id: &str,
    raw: std::collections::BTreeMap<String, RawParameterSpec>,
) -> Result<std::collections::BTreeMap<String, ParameterSpec>, KnowledgeError> {
    let mut result = std::collections::BTreeMap::new();
    for (name, raw_spec) in raw {
        let kind = match raw_spec.kind.as_str() {
            "integer" => ParameterType::Integer,
            "float" => ParameterType::Float,
            other => {
                return Err(KnowledgeError::UnknownParameterType {
                    entity_id: entity_id.to_owned(),
                    value: other.to_owned(),
                })
            }
        };
        let value = raw_spec.value.map(convert_bound);
        let min = raw_spec.min.map(convert_bound);
        let max = raw_spec.max.map(convert_bound);
        if value.is_some() && (min.is_some() || max.is_some()) {
            return Err(KnowledgeError::ParameterValueAndBoundsConflict {
                entity_id: entity_id.to_owned(),
                parameter: name,
            });
        }
        result.insert(
            name,
            ParameterSpec {
                kind,
                value,
                min,
                max,
                description: raw_spec.description,
            },
        );
    }
    Ok(result)
}

fn convert_bound(raw: RawBound) -> Bound {
    match raw {
        RawBound::Literal(value) => Bound::Literal(value),
        RawBound::Reference { parameter, offset } => Bound::Reference { parameter, offset },
    }
}

fn validate_parameter_references(
    entity_id: &str,
    parameters: &std::collections::BTreeMap<String, ParameterSpec>,
) -> Result<(), KnowledgeError> {
    for (name, spec) in parameters {
        for bound in [&spec.value, &spec.min, &spec.max].into_iter().flatten() {
            if let Bound::Reference { parameter, .. } = bound {
                if !parameters.contains_key(parameter) {
                    return Err(KnowledgeError::DanglingParameterReference {
                        entity_id: entity_id.to_owned(),
                        parameter: name.clone(),
                        target: parameter.clone(),
                    });
                }
            }
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Gray,
        Black,
    }

    fn visit(
        name: &str,
        parameters: &std::collections::BTreeMap<String, ParameterSpec>,
        colors: &mut HashMap<String, Color>,
        path: &mut Vec<String>,
        entity_id: &str,
    ) -> Result<(), KnowledgeError> {
        match colors.get(name) {
            Some(Color::Black) => return Ok(()),
            Some(Color::Gray) => {
                let start = path.iter().position(|n| n == name).unwrap_or(0);
                let mut cycle: Vec<String> = path[start..].to_vec();
                cycle.push(name.to_owned());
                return Err(KnowledgeError::ParameterReferenceCycle {
                    entity_id: entity_id.to_owned(),
                    cycle,
                });
            }
            _ => {}
        }
        colors.insert(name.to_owned(), Color::Gray);
        path.push(name.to_owned());
        if let Some(spec) = parameters.get(name) {
            for bound in [&spec.value, &spec.min, &spec.max].into_iter().flatten() {
                if let Bound::Reference { parameter, .. } = bound {
                    visit(parameter, parameters, colors, path, entity_id)?;
                }
            }
        }
        path.pop();
        colors.insert(name.to_owned(), Color::Black);
        Ok(())
    }

    let mut colors: HashMap<String, Color> =
        parameters.keys().map(|k| (k.clone(), Color::White)).collect();
    let mut path = Vec::new();
    for name in parameters.keys() {
        if colors.get(name) == Some(&Color::White) {
            visit(name, parameters, &mut colors, &mut path, entity_id)?;
        }
    }
    Ok(())
}

fn validate_constraint_parameters(
    entity_id: &str,
    constraint: &ConstraintExpr,
    parameters: &std::collections::BTreeMap<String, ParameterSpec>,
) -> Result<(), KnowledgeError> {
    fn check_term(
        term: &Term,
        parameters: &std::collections::BTreeMap<String, ParameterSpec>,
        entity_id: &str,
    ) -> Result<(), KnowledgeError> {
        match term {
            Term::Param(name) => {
                if !parameters.contains_key(name) {
                    return Err(KnowledgeError::ConstraintUnknownParameter {
                        entity_id: entity_id.to_owned(),
                        parameter: name.clone(),
                    });
                }
                Ok(())
            }
            Term::Literal(_) => Ok(()),
            Term::BinaryOp { left, right, .. } => {
                check_term(left, parameters, entity_id)?;
                check_term(right, parameters, entity_id)
            }
        }
    }

    match constraint {
        ConstraintExpr::Comparison { left, right, .. } => {
            check_term(left, parameters, entity_id)?;
            check_term(right, parameters, entity_id)
        }
        ConstraintExpr::All(inner) => {
            for expr in inner {
                validate_constraint_parameters(entity_id, expr, parameters)?;
            }
            Ok(())
        }
    }
}

fn convert_canonical_solution(
    entity_id: &str,
    response_type: ResponseType,
    raw: RawCanonicalSolution,
) -> Result<CanonicalSolution, KnowledgeError> {
    match (response_type, raw.expression, raw.value) {
        (ResponseType::SymbolicExpression, Some(expression), None) => {
            Ok(CanonicalSolution::Symbolic { expression })
        }
        (ResponseType::Numeric, None, Some(value)) => Ok(CanonicalSolution::Numeric { value }),
        _ => Err(KnowledgeError::ResponseTypeSolutionMismatch {
            entity_id: entity_id.to_owned(),
            response_type: match response_type {
                ResponseType::SymbolicExpression => "symbolic-expression",
                ResponseType::Numeric => "numeric",
            },
        }),
    }
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test --locked --lib knowledge::problem_family`
Expected: PASS, 9 tests.

- [ ] **Step 6: Register the module**

In `src-tauri/src/knowledge/mod.rs`, find:

```rust
mod objective;
```

Add immediately after it:

```rust
mod problem_family;
```

- [ ] **Step 7: Run the full suite and lints**

Run: `cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`
Expected: all pass.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/knowledge/problem_family.rs src-tauri/src/knowledge/mod.rs src-tauri/src/knowledge/error.rs
git commit -m "feat(knowledge): add parse_problem_family_file with structural validation"
```

---

### Task 5: Wire into package discovery, cross-entity validation, and `KnowledgePackage`

**Files:**
- Modify: `src-tauri/src/knowledge/discover.rs`
- Modify: `src-tauri/src/knowledge/validate.rs`
- Modify: `src-tauri/src/knowledge/types.rs`
- Modify: `src-tauri/src/knowledge/loader.rs`
- Modify: `src-tauri/src/knowledge/mod.rs`
- Modify: `src-tauri/src/knowledge/error.rs`

**Interfaces:**
- Consumes: `parse_problem_family_file` (Task 4), `ProblemFamily` (Task 1).
- Produces: `KnowledgePackage.problem_families: Vec<ProblemFamily>` — populated end to end
  by `load_knowledge_package`, the module's one public entry point.

- [ ] **Step 1: Add the cross-entity error variant**

In `src-tauri/src/knowledge/error.rs`, add:

```rust
    ProblemFamilyCrossConceptObjective {
        problem_family_id: String,
        objective_id: String,
    },
```

And its `Display` arm:

```rust
            Self::ProblemFamilyCrossConceptObjective { problem_family_id, objective_id } => write!(
                f,
                "problem family {problem_family_id} references objective {objective_id} belonging to a different concept"
            ),
```

- [ ] **Step 2: Write the failing test for discovery**

In `src-tauri/src/knowledge/discover.rs`, add to the existing `#[cfg(test)] mod tests`
block:

```rust
    const VALID_PROBLEM_FAMILY: &str = "+++\nid = \"problem.a\"\nconcept_id = \"shell.a\"\ndifficulty = { min = 1, max = 1 }\ngenerator = { id = \"gen-a\", version = 1 }\nresponse_type = \"numeric\"\nstatus = \"verified\"\n\n[canonical_solution]\nvalue = 1.0\n\n[[provenance_refs]]\nsource_id = \"src.x\"\nkind = \"direct\"\n+++\n\n## Prompt\n\nP.\n\n## Solution\n\nS.\n";

    #[test]
    fn discovers_problem_family_files() {
        let root = temp_package_dir("discovers_problem_family_files");
        let problems_dir = root.join("problems");
        fs::create_dir_all(&problems_dir).unwrap();
        fs::write(problems_dir.join("problem.a.md"), VALID_PROBLEM_FAMILY).unwrap();
        let discovered = discover_entities(&root).unwrap();
        assert_eq!(discovered.problem_families.len(), 1);
        assert_eq!(discovered.problem_families[0].id.as_str(), "problem.a");
    }

    #[test]
    fn missing_problems_directory_yields_zero_problem_families() {
        let root = temp_package_dir("missing_problems_directory_yields_zero_problem_families");
        let discovered = discover_entities(&root).unwrap();
        assert!(discovered.problem_families.is_empty());
    }
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test --locked --lib knowledge::discover`
Expected: FAIL — `DiscoveredEntities` has no field `problem_families`.

- [ ] **Step 4: Wire `problem_families` into discovery**

In `src-tauri/src/knowledge/discover.rs`, update the imports at the top:

```rust
use super::concept::parse_concept_file;
use super::error::KnowledgeError;
use super::example::parse_example_file;
use super::objective::parse_objective_file;
use super::problem_family::parse_problem_family_file;
use super::types::{Concept, Example, Objective, ProblemFamily};
```

Update `DiscoveredEntities`:

```rust
pub(crate) struct DiscoveredEntities {
    pub concepts: Vec<Concept>,
    pub objectives: Vec<Objective>,
    pub examples: Vec<Example>,
    pub problem_families: Vec<ProblemFamily>,
}
```

Update `discover_entities`:

```rust
pub(crate) fn discover_entities(root: &Path) -> Result<DiscoveredEntities, KnowledgeError> {
    let concepts = load_entities(root, "concepts", parse_concept_file, |c: &Concept| {
        c.id.as_str().to_owned()
    })?;
    let objectives = load_entities(root, "objectives", parse_objective_file, |o: &Objective| {
        o.id.as_str().to_owned()
    })?;
    let examples = load_entities(root, "examples", parse_example_file, |e: &Example| {
        e.id.as_str().to_owned()
    })?;
    let problem_families = load_entities(
        root,
        "problems",
        parse_problem_family_file,
        |p: &ProblemFamily| p.id.as_str().to_owned(),
    )?;

    Ok(DiscoveredEntities {
        concepts,
        objectives,
        examples,
        problem_families,
    })
}
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo test --locked --lib knowledge::discover`
Expected: PASS.

- [ ] **Step 6: Write the failing test for cross-entity validation**

In `src-tauri/src/knowledge/validate.rs`, add to the existing `#[cfg(test)] mod tests`
block (alongside the existing `example`/`concept`/`objective` test helpers, add a matching
`problem_family` helper):

```rust
    use crate::knowledge::types::{DifficultyRange, GeneratorRef, ProblemFamily, ResponseType, CanonicalSolution, ProblemFamilyStatus};
    use crate::knowledge::identifier::ProblemFamilyId;

    fn problem_family(id: &str, concept_id: &str, objective_ids: Vec<&str>) -> ProblemFamily {
        ProblemFamily {
            id: ProblemFamilyId::new(id).unwrap(),
            concept_id: ConceptId::new(concept_id).unwrap(),
            objective_ids: objective_ids
                .into_iter()
                .map(|o| ObjectiveId::new(o).unwrap())
                .collect(),
            difficulty: DifficultyRange { min: 1, max: 1 },
            generator: GeneratorRef {
                id: "gen".to_owned(),
                version: 1,
            },
            parameters: std::collections::BTreeMap::new(),
            constraints: vec![],
            prompt: "p".to_owned(),
            solution_structure: "s".to_owned(),
            response_type: ResponseType::Numeric,
            canonical_solution: CanonicalSolution::Numeric { value: 1.0 },
            hints: vec![],
            provenance_refs: provenance("src.a"),
            status: ProblemFamilyStatus::Verified,
        }
    }

    #[test]
    fn valid_problem_family_references_resolve() {
        let entities = DiscoveredEntities {
            concepts: vec![concept("shell.a")],
            objectives: vec![objective("shell.obj", "shell.a")],
            examples: vec![],
            problem_families: vec![problem_family("problem.a", "shell.a", vec!["shell.obj"])],
        };
        assert!(validate_references(&entities, &[source("src.a")]).is_ok());
    }

    #[test]
    fn problem_family_unresolved_concept_is_rejected() {
        let entities = DiscoveredEntities {
            concepts: vec![],
            objectives: vec![],
            examples: vec![],
            problem_families: vec![problem_family("problem.a", "shell.missing", vec![])],
        };
        assert!(matches!(
            validate_references(&entities, &[source("src.a")]),
            Err(KnowledgeError::UnresolvedConcept { .. })
        ));
    }

    #[test]
    fn problem_family_cross_concept_objective_is_rejected() {
        let entities = DiscoveredEntities {
            concepts: vec![concept("shell.a"), concept("shell.b")],
            objectives: vec![objective("shell.obj", "shell.b")],
            examples: vec![],
            problem_families: vec![problem_family("problem.a", "shell.a", vec!["shell.obj"])],
        };
        assert!(matches!(
            validate_references(&entities, &[source("src.a")]),
            Err(KnowledgeError::ProblemFamilyCrossConceptObjective { .. })
        ));
    }
```

Also update the existing `valid_references_resolve` and other pre-existing tests in this
file's `#[cfg(test)] mod tests` block: every place they construct `DiscoveredEntities { ...
}` will now fail to compile because the struct has a new required field. Add
`problem_families: vec![],` to every existing `DiscoveredEntities { ... }` literal already
in this file's test module (there are five: `valid_references_resolve`,
`unresolved_objective_concept_id_is_rejected`, `unresolved_example_objective_id_is_rejected`,
`cross_concept_objective_is_rejected`, `unresolved_provenance_source_is_rejected`).

- [ ] **Step 7: Run the test to verify it fails**

Run: `cargo test --locked --lib knowledge::validate`
Expected: FAIL to compile — `validate_references` doesn't check `problem_families` yet, and
(before Step 8) the function signature/logic doesn't reference the new field at all yet, so
the new tests fail their assertions.

- [ ] **Step 8: Extend `validate_references`**

In `src-tauri/src/knowledge/validate.rs`, update the imports:

```rust
use super::identifier::SourceId;
use super::types::{ProvenanceRef, Source};
```

stays the same, but the function body needs a new block. Find the closing of the `example`
loop (the `for example in &entities.examples { ... }` block) and add immediately after it,
still inside `validate_references`, before the `validate_provenance_sources(...)` calls:

```rust
    for problem_family in &entities.problem_families {
        if !concept_ids.contains(&problem_family.concept_id) {
            return Err(KnowledgeError::UnresolvedConcept {
                entity_id: problem_family.id.as_str().to_owned(),
                field: "concept_id",
                target: problem_family.concept_id.as_str().to_owned(),
            });
        }
        for objective_id in &problem_family.objective_ids {
            let Some(objective) = objectives_by_id.get(objective_id) else {
                return Err(KnowledgeError::UnresolvedObjective {
                    entity_id: problem_family.id.as_str().to_owned(),
                    field: "objective_ids",
                    target: objective_id.as_str().to_owned(),
                });
            };
            if objective.concept_id != problem_family.concept_id {
                return Err(KnowledgeError::ProblemFamilyCrossConceptObjective {
                    problem_family_id: problem_family.id.as_str().to_owned(),
                    objective_id: objective_id.as_str().to_owned(),
                });
            }
        }
    }
```

And add the `problem_families` provenance check alongside the existing three
`validate_provenance_sources(...)` calls:

```rust
    validate_provenance_sources(
        entities
            .problem_families
            .iter()
            .map(|p| (p.id.as_str(), &p.provenance_refs)),
        &source_ids,
    )?;
```

- [ ] **Step 9: Run the test to verify it passes**

Run: `cargo test --locked --lib knowledge::validate`
Expected: PASS.

- [ ] **Step 10: Add `problem_families` to `KnowledgePackage` and wire the loader**

In `src-tauri/src/knowledge/types.rs`, update `KnowledgePackage`:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgePackage {
    pub id: KnowledgePackageId,
    pub schema_version: u32,
    pub version: Version,
    pub title: String,
    pub description: String,
    pub concepts: Vec<Concept>,
    pub objectives: Vec<Objective>,
    pub examples: Vec<Example>,
    pub problem_families: Vec<ProblemFamily>,
    pub sources: Vec<Source>,
}
```

In `src-tauri/src/knowledge/loader.rs`, update the `Ok(KnowledgePackage { ... })`
construction at the end of `load_knowledge_package`:

```rust
    Ok(KnowledgePackage {
        id: identity.id,
        schema_version: identity.schema_version,
        version: identity.version,
        title: identity.title,
        description: identity.description,
        concepts: entities.concepts,
        objectives: entities.objectives,
        examples: entities.examples,
        problem_families: entities.problem_families,
        sources,
    })
```

Also update `loader.rs`'s own tests: `write_minimal_valid_package` doesn't need changes
(it's fine for a minimal package to have zero problem families — the `problems/` directory
simply won't exist, matching how `examples/` already behaves when absent), but the assertion
in `minimal_valid_package_loads` can optionally confirm this — add one line:

```rust
        assert!(package.problem_families.is_empty());
```

right after the existing `assert_eq!(package.sources.len(), 1);` line in that test.

- [ ] **Step 11: Update `mod.rs` exports**

In `src-tauri/src/knowledge/mod.rs`, add `problem_family` to the module declarations (find
`mod problem_family;` from Task 4 — already there). Update the type re-export list one more
time to add `ProblemFamily` if not already present (it was added in Task 1's Step 8 — verify
it's still there) and `ConstraintExpr`/`Term`/`ArithOp`/`CompareOp` from `constraint`:

```rust
pub use constraint::{ArithOp, CompareOp, ConstraintExpr, Term};
```

Add this line to `mod.rs` near the other `pub use` lines.

- [ ] **Step 12: Run the full suite and lints**

Run: `cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`
Expected: all pass.

- [ ] **Step 13: Commit**

```bash
git add src-tauri/src/knowledge/discover.rs src-tauri/src/knowledge/validate.rs src-tauri/src/knowledge/types.rs src-tauri/src/knowledge/loader.rs src-tauri/src/knowledge/mod.rs src-tauri/src/knowledge/error.rs
git commit -m "feat(knowledge): wire ProblemFamily into package discovery and cross-entity validation"
```

---

### Task 6: `ProblemInstance` runtime types

**Files:**
- Modify: `src-tauri/src/knowledge/types.rs`
- Modify: `src-tauri/src/knowledge/mod.rs`

**Interfaces:**
- Produces: `ProblemInstance`, `ResolvedSolution` — sub-project 4 (Practice Core Utility)
  will produce values of this shape from `practice.generate`; nothing in this plan
  constructs one.

- [ ] **Step 1: Write the failing test**

Add to `src-tauri/src/knowledge/types.rs`'s `#[cfg(test)] mod tests` block:

```rust
    #[test]
    fn problem_instance_round_trips_through_json() {
        let instance = ProblemInstance {
            family_id: ProblemFamilyId::new("problem.shell_y_poly").unwrap(),
            seed: 42,
            resolved_parameters: std::collections::BTreeMap::from([
                ("coeff".to_owned(), 4.0),
                ("b".to_owned(), 3.0),
            ]),
            prompt: "Define R with coeff=4, b=3...".to_owned(),
            canonical_solution: ResolvedSolution::Symbolic("2*pi*(4*27/3 - 81/4)".to_owned()),
            hints: vec!["Identify the shell radius.".to_owned()],
        };

        let json = serde_json::to_string(&instance).unwrap();
        let round_tripped: ProblemInstance = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, instance);
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --locked problem_instance_round_trips_through_json`
Expected: FAIL — `ProblemInstance` not found.

- [ ] **Step 3: Add the types**

In `src-tauri/src/knowledge/types.rs`, add after `ProblemFamilyStatus`:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProblemInstance {
    pub family_id: ProblemFamilyId,
    pub seed: u64,
    pub resolved_parameters: std::collections::BTreeMap<String, f64>,
    pub prompt: String,
    pub canonical_solution: ResolvedSolution,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ResolvedSolution {
    Symbolic(String),
    Numeric(f64),
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test --locked problem_instance_round_trips_through_json`
Expected: PASS.

- [ ] **Step 5: Update `mod.rs` exports**

In `src-tauri/src/knowledge/mod.rs`, find the `pub use types::{ ... }` block and add
`ProblemInstance, ResolvedSolution` to it, keeping the list alphabetically grouped the way
it already is.

- [ ] **Step 6: Run the full suite and lints**

Run: `cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`
Expected: all pass.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/knowledge/types.rs src-tauri/src/knowledge/mod.rs
git commit -m "feat(knowledge): add ProblemInstance and ResolvedSolution runtime types"
```

---

### Task 7: Canonical fixture and conformance suite

**Files:**
- Create: `src-tauri/src/knowledge/tests/fixtures/canonical/problems/problem.shell_y_poly.md`
- Modify: `src-tauri/src/knowledge/tests/fixtures/canonical/package.toml` (if it needs a
  matching source — check first, see Step 1)
- Create: `src-tauri/src/knowledge/tests/conformance_problem_family.rs` (or the equivalent
  location the existing conformance suite already uses — see Step 1)

**Interfaces:** none — this task only adds fixtures and tests exercising everything Tasks
1-6 already built; it produces nothing later tasks depend on.

- [ ] **Step 1: Inspect the existing canonical fixture and conformance test setup**

Before writing anything, read:
- `src-tauri/src/knowledge/tests/fixtures/canonical/package.toml`
- `src-tauri/src/knowledge/tests/fixtures/canonical/sources.toml`
- `src-tauri/src/knowledge/tests/` (the directory itself — find whatever file currently
  runs `load_knowledge_package` against the canonical fixture directory as an integration
  test, likely named something like `canonical.rs` or similar under
  `src-tauri/src/knowledge/tests/`)

Confirm the canonical fixture's `sources.toml` already declares a source usable for the new
`problems/problem.shell_y_poly.md` fixture's `provenance_refs` (the real `pf-shell-y-poly.json`
prototype cited `src-openstax-calc2-rule2-6` and `src-openstax-calc2-ex2-13` — check whether
an equivalent `src.openstax_calc2`-style id already exists in the canonical fixture's
`sources.toml`; if so, reuse it exactly as named there instead of inventing a new one).

- [ ] **Step 2: Create the canonical `ProblemFamily` fixture**

Create `src-tauri/src/knowledge/tests/fixtures/canonical/problems/problem.shell_y_poly.md`,
migrating the real prototype content recovered from git history (commit
`17d8aa36c7c9be2dd51d697cbfa61dc5a0035b5e^:knowledge-package/problem-families/pf-shell-y-poly.json`)
into the new format:

```markdown
+++
id = "problem.shell_y_poly"
concept_id = "shell.method_vertical_axis"
objective_ids = ["shell.setup_radius_height_y_axis", "shell.compute_volume_y_axis_single_curve"]
difficulty = { min = 1, max = 2 }
generator = { id = "gen-shell-y-poly", version = 1 }
response_type = "symbolic-expression"
status = "verified"

[parameters.coeff]
type = "integer"
min = 2
max = 6
description = "Linear coefficient for quadratic curve f(x) = c*x - x^2"

[parameters.a]
type = "integer"
value = 0
description = "Left boundary of the interval"

[parameters.b]
type = "integer"
min = 1
max = { parameter = "coeff" }
description = "Right boundary; its inclusive maximum is the sampled coeff value"

[canonical_solution]
expression = "2*pi*(coeff*b^3/3 - b^4/4)"

[[hints]]
level = 1

[[hints]]
level = 2

[[hints]]
level = 3

[[hints]]
level = 4

[[provenance_refs]]
source_id = "REPLACE_WITH_WHATEVER_SOURCE_ID_STEP_1_FOUND"
kind = "direct"
[provenance_refs.locator]
section = "2.3"
label = "Rule 2.6"
+++

## Prompt

Define R as the region bounded above by the graph of f(x) = {coeff}x - x^2 and below by the
x-axis over the interval [{a}, {b}]. Find the volume of the solid of revolution formed by
revolving R around the y-axis.

## Solution

V = \int_{a}^{b} 2\pi x f(x) dx = 2\pi \int_{0}^{b} ({coeff}x^2 - x^3) dx
  = 2\pi [{coeff}x^3/3 - x^4/4]_{0}^{b}

## Hints

- Identify the shell radius and shell height as functions of x for rotation around the y-axis.
- For a region bounded by y = f(x) revolved around the y-axis, the shell radius is r(x) = x and the height is h(x) = {coeff}x - x^2.
- Set up the definite integral: V = \int_{0}^{{b}} 2\pi x ({coeff}x - x^2) dx.
- Evaluate the antiderivative 2\pi [{coeff}*x^3/3 - x^4/4] from 0 to {b}.
```

Replace `REPLACE_WITH_WHATEVER_SOURCE_ID_STEP_1_FOUND` with the actual source id Step 1
found in the canonical fixture's `sources.toml` (this is a real value you determine by
reading that file, not a placeholder to leave in the committed file).

- [ ] **Step 3: Write the failing conformance test**

Following whatever pattern Step 1's inspection found for the existing conformance suite
(mirror its exact structure — module path, how it invokes `load_knowledge_package` against
the canonical fixture directory, how it asserts on the result), add a test confirming the
canonical fixture (now including `problems/problem.shell_y_poly.md`) loads successfully end
to end and that the loaded `ProblemFamily` is present:

```rust
#[test]
fn canonical_package_includes_the_migrated_problem_family() {
    let package = load_knowledge_package(canonical_fixture_root()).unwrap();
    assert_eq!(package.problem_families.len(), 1);
    let family = &package.problem_families[0];
    assert_eq!(family.id.as_str(), "problem.shell_y_poly");
    assert_eq!(family.parameters.len(), 3);
    assert_eq!(family.hints.len(), 4);
}
```

(Use whatever the existing conformance test file's actual helper function for locating the
canonical fixture root is called — read the file from Step 1 to get its exact name; do not
invent a new one if one already exists.)

- [ ] **Step 4: Run the test to verify it fails**

Run: `cargo test --locked canonical_package_includes_the_migrated_problem_family`
Expected: FAIL until Step 2's fixture is correctly in place and loads cleanly — if it fails
with a validation error instead of "test not found," fix the fixture file (Step 2) to
resolve it, since the fixture itself is real content this task is responsible for getting
right, not scaffolding to discard.

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo test --locked canonical_package_includes_the_migrated_problem_family`
Expected: PASS.

- [ ] **Step 6: Add conformance cases for each rejection class**

Add table-driven or individually-named tests (matching whichever style the existing
conformance suite already uses) covering, at minimum, one case from each class already unit
-tested in Task 4 but now exercised through the *full* `load_knowledge_package` path (a
broken `problem_families/` file inside an otherwise-valid canonical-shaped package):
dangling parameter reference, parameter reference cycle, constraint referencing an unknown
parameter, response_type/canonical_solution mismatch, invalid difficulty range, hint count
mismatch, duplicate hint level, unresolved `concept_id`, unresolved `objective_ids`, and
cross-concept objective. Each case: copy the canonical fixture directory to a temp dir,
corrupt just the one field under test in `problems/problem.shell_y_poly.md`, assert
`load_knowledge_package` returns the specific expected `KnowledgeError` variant.

- [ ] **Step 7: Run the full suite and lints**

Run: `cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`
Expected: all pass.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/knowledge/tests/
git commit -m "test(knowledge): add canonical ProblemFamily fixture and conformance suite"
```

## Self-Review Notes

**Spec coverage:** §3 `ProblemFamily` fields (Task 1) ✓. Constraint expressions (Task 2) ✓.
§4 `ProblemInstance` (Task 6) ✓. §5 file format — `+++` TOML+MD, `problems/` directory,
long-form content in the body (Tasks 3-4) ✓. §6 validation — all ten rules enumerated in the
spec have a corresponding check: concept_id/objective_ids resolution and same-concept rule
(Task 5), parameter reference resolution + cycle check (Task 4), constraint parse +
parameter check (Tasks 2, 4), response_type/canonical_solution match (Task 4), difficulty
range (Task 4), hint level uniqueness (Task 4), generator.id grammar (Task 1, via the
existing `ProblemFamilyId`-style identifier validation reused for the `id` field itself —
note: `generator.id` itself is a plain `String` in this plan, per the spec's own §6 note
that it's "validated against the same identifier grammar... but not checked against an
actually-registered function" — this plan does NOT separately validate `generator.id`'s
grammar, which is a gap against the spec; see the note below), provenance_refs (reuses
existing rule, Task 5) ✓. §7 testing — canonical fixture + conformance suite (Task 7) ✓.

**Gap found during self-review, fixed inline:** the spec's §6 says `generator.id` "is
validated against the same identifier grammar as every other id in this package" — the
first draft of Task 4 stored `generator.id` as a plain `String` with no grammar check.
Fixed directly in Task 4 Step 4's code above: a
`crate::modules::identifier::validate_identifier(&fm.generator.id)` call (the exact function
`identifier.rs`'s `knowledge_id!` macro already uses for every other id), mapping failure to
the existing `KnowledgeError::InvalidIdentifier` variant, immediately before constructing
`generator`. No new `GeneratorId` newtype needed — `GeneratorRef` isn't referenced elsewhere
the way `ConceptId`/`ObjectiveId` are, so a full newtype would be unnecessary weight for a
single validation call.

**Placeholder scan:** no TBD/TODO in code; the fixture file's
`REPLACE_WITH_WHATEVER_SOURCE_ID_STEP_1_FOUND` is a deliberate, explicitly-flagged
fill-in-a-real-value instruction (Task 7 Step 2 says so directly), not a plan placeholder —
the value it needs depends on reading an existing file first, which the task's own Step 1
requires before Step 2 is written.

**Type/name consistency:** `ProblemFamily`, `ParameterSpec`, `Bound`, `ResponseType`,
`CanonicalSolution`, `Hint`, `ProblemFamilyStatus` (Task 1) match exactly what Tasks 3-6
import and construct. `ConstraintExpr`/`Term`/`ArithOp`/`CompareOp` (Task 2) match what
Task 4 imports (`super::constraint::{parse_constraint, ConstraintExpr, Term}`) and what
Task 5 re-exports. `parse_problem_family_body`/`ParsedProblemFamilyBody` (Task 3) match
Task 4's usage exactly (`prompt`, `solution_structure`, `hint_texts` field names identical
in both). `DiscoveredEntities.problem_families` (Task 5) matches what Task 7's conformance
suite reads via `package.problem_families`.
