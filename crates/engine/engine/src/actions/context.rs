//! Key contexts and keymap binding predicates.
//!
//! Each focusable element contributes a [`KeyContext`] to the
//! [`ContextStack`], ordered from the shallowest (outermost) to the deepest
//! (focused) element. Keymap sections carry an optional [`Predicate`] parsed
//! from strings like `"Editor && mode == insert"`.
//!
//! Grammar (precedence: `()` > `>` > `!` > `&&` > `||`):
//!
//! ```text
//! expr    := or
//! or      := and ('||' and)*
//! and     := unary ('&&' unary)*
//! unary   := '!' unary | primary
//! primary := atom ('>' atom)*
//! atom    := '(' expr ')' | ident ('==' ident)?
//! ```
//!
//! A bare `ident` matches a context whose name is `ident` or that carries an
//! attribute/flag with that key. `a == b` matches a context attribute.
//! `A > B` requires `A` to match an *ancestor* (not necessarily immediate)
//! of the context matched by `B`.

use std::fmt;

// ---------------------------------------------------------------------------
// KeyContext & ContextStack
// ---------------------------------------------------------------------------

/// The key context contributed by one element in the focus path.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyContext {
    pub name: String,
    /// Key/value attributes. Flags are attributes with an empty value; a
    /// bare identifier predicate matches the attribute key regardless of
    /// its value.
    pub attrs: Vec<(String, String)>,
}

impl KeyContext {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            attrs: Vec::new(),
        }
    }

    /// Builder: adds a key/value attribute (`mode == insert`).
    pub fn with_attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.push((key.into(), value.into()));
        self
    }

    /// Builder: adds a flag attribute, matched by a bare identifier.
    pub fn with_flag(mut self, name: impl Into<String>) -> Self {
        self.attrs.push((name.into(), String::new()));
        self
    }

    /// Value of attribute `key`, if present.
    pub fn attr(&self, key: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// `true` when `ident` is this context's name or one of its attribute
    /// keys (flag-style match).
    pub fn matches_ident(&self, ident: &str) -> bool {
        self.name == ident || self.attrs.iter().any(|(k, _)| k == ident)
    }
}

/// Focus path of key contexts, from the shallowest to the deepest element.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContextStack {
    contexts: Vec<KeyContext>,
}

impl ContextStack {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a context as the new deepest entry.
    pub fn push(&mut self, context: KeyContext) {
        self.contexts.push(context);
    }

    /// Pops the deepest context.
    pub fn pop(&mut self) -> Option<KeyContext> {
        self.contexts.pop()
    }

    pub fn contexts(&self) -> &[KeyContext] {
        &self.contexts
    }

    pub fn len(&self) -> usize {
        self.contexts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.contexts.is_empty()
    }
}

impl From<Vec<KeyContext>> for ContextStack {
    fn from(contexts: Vec<KeyContext>) -> Self {
        Self { contexts }
    }
}

// ---------------------------------------------------------------------------
// Predicate
// ---------------------------------------------------------------------------

/// Parsed keymap context predicate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Predicate {
    /// Bare identifier: matches context name or attribute key.
    Identifier(String),
    /// `key == value` attribute test.
    Equals(String, String),
    Not(Box<Predicate>),
    And(Box<Predicate>, Box<Predicate>),
    Or(Box<Predicate>, Box<Predicate>),
    /// `A > B`: `A` must match an ancestor of the context matched by `B`.
    Child(Box<Predicate>, Box<Predicate>),
}

/// Error produced when a predicate string cannot be parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PredicateParseError {
    message: String,
}

impl PredicateParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PredicateParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid context predicate: {}", self.message)
    }
}

impl std::error::Error for PredicateParseError {}

impl Predicate {
    /// Parses a predicate string such as `"Editor && mode == insert"`.
    pub fn parse(input: &str) -> Result<Self, PredicateParseError> {
        let tokens = tokenize(input)?;
        let mut parser = Parser {
            tokens: &tokens,
            index: 0,
        };
        let expr = parser.parse_or()?;
        if let Some((token, pos)) = parser.peek() {
            return Err(PredicateParseError::new(format!(
                "unexpected `{token}` at position {pos} after complete expression in `{input}`"
            )));
        }
        Ok(expr)
    }

    /// Evaluates this predicate anchored at the deepest context of `contexts`
    /// (`Child` ancestors may match any shallower prefix).
    pub fn eval(&self, contexts: &[KeyContext]) -> bool {
        let Some(deepest) = contexts.last() else {
            return false;
        };
        match self {
            Self::Identifier(ident) => deepest.matches_ident(ident),
            Self::Equals(key, value) => deepest.attr(key) == Some(value.as_str()),
            Self::Not(inner) => !inner.eval(contexts),
            Self::And(a, b) => a.eval(contexts) && b.eval(contexts),
            Self::Or(a, b) => a.eval(contexts) || b.eval(contexts),
            Self::Child(ancestor, descendant) => {
                descendant.eval(contexts)
                    && (0..contexts.len() - 1)
                        .rev()
                        .any(|i| ancestor.eval(&contexts[..=i]))
            }
        }
    }

    /// Index of the deepest context in `stack` at which this predicate
    /// matches, or `None`. Deeper matches give bindings higher precedence.
    pub fn match_depth(&self, stack: &ContextStack) -> Option<usize> {
        (0..stack.len())
            .rev()
            .find(|&i| self.eval(&stack.contexts()[..=i]))
    }

    /// `true` when some context in the stack satisfies this predicate.
    pub fn matches(&self, stack: &ContextStack) -> bool {
        self.match_depth(stack).is_some()
    }
}

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Ident(String),
    And,
    Or,
    Not,
    Equals,
    Child,
    LParen,
    RParen,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Ident(name) => write!(f, "{name}"),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::Not => write!(f, "!"),
            Token::Equals => write!(f, "=="),
            Token::Child => write!(f, ">"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
        }
    }
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '.'
}

fn tokenize(input: &str) -> Result<Vec<(Token, usize)>, PredicateParseError> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some(&(pos, c)) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            '(' => {
                chars.next();
                tokens.push((Token::LParen, pos));
            }
            ')' => {
                chars.next();
                tokens.push((Token::RParen, pos));
            }
            '!' => {
                chars.next();
                tokens.push((Token::Not, pos));
            }
            '>' => {
                chars.next();
                tokens.push((Token::Child, pos));
            }
            '&' | '|' | '=' => {
                chars.next();
                if chars.peek().map(|&(_, next)| next) == Some(c) {
                    chars.next();
                    let token = match c {
                        '&' => Token::And,
                        '|' => Token::Or,
                        _ => Token::Equals,
                    };
                    tokens.push((token, pos));
                } else {
                    return Err(PredicateParseError::new(format!(
                        "expected `{c}{c}` at position {pos} in `{input}` (single `{c}` is not \
                         an operator)"
                    )));
                }
            }
            c if is_ident_char(c) => {
                let mut ident = String::new();
                while let Some(&(_, c)) = chars.peek() {
                    if is_ident_char(c) {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push((Token::Ident(ident), pos));
            }
            other => {
                return Err(PredicateParseError::new(format!(
                    "unexpected character `{other}` at position {pos} in `{input}`"
                )));
            }
        }
    }

    Ok(tokens)
}

// ---------------------------------------------------------------------------
// Parser (recursive descent)
// ---------------------------------------------------------------------------

struct Parser<'a> {
    tokens: &'a [(Token, usize)],
    index: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&(Token, usize)> {
        self.tokens.get(self.index)
    }

    fn advance(&mut self) -> Option<&(Token, usize)> {
        let token = self.tokens.get(self.index);
        if token.is_some() {
            self.index += 1;
        }
        token
    }

    fn eat(&mut self, expected: &Token) -> bool {
        if self.peek().map(|(t, _)| t) == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn parse_or(&mut self) -> Result<Predicate, PredicateParseError> {
        let mut left = self.parse_and()?;
        while self.eat(&Token::Or) {
            let right = self.parse_and()?;
            left = Predicate::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Predicate, PredicateParseError> {
        let mut left = self.parse_unary()?;
        while self.eat(&Token::And) {
            let right = self.parse_unary()?;
            left = Predicate::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Predicate, PredicateParseError> {
        if self.eat(&Token::Not) {
            let inner = self.parse_unary()?;
            Ok(Predicate::Not(Box::new(inner)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Predicate, PredicateParseError> {
        let mut left = self.parse_atom()?;
        while self.eat(&Token::Child) {
            let right = self.parse_atom()?;
            left = Predicate::Child(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_atom(&mut self) -> Result<Predicate, PredicateParseError> {
        match self.advance() {
            Some((Token::LParen, pos)) => {
                let pos = *pos;
                let expr = self.parse_or()?;
                if !self.eat(&Token::RParen) {
                    return Err(PredicateParseError::new(format!(
                        "unclosed `(` opened at position {pos}"
                    )));
                }
                Ok(expr)
            }
            Some((Token::Ident(name), _)) => {
                let name = name.clone();
                if self.eat(&Token::Equals) {
                    match self.advance() {
                        Some((Token::Ident(value), _)) => {
                            Ok(Predicate::Equals(name, value.clone()))
                        }
                        Some((token, pos)) => Err(PredicateParseError::new(format!(
                            "expected value after `==`, found `{token}` at position {pos}"
                        ))),
                        None => Err(PredicateParseError::new(
                            "expected value after `==`, found end of input",
                        )),
                    }
                } else {
                    Ok(Predicate::Identifier(name))
                }
            }
            Some((token, pos)) => Err(PredicateParseError::new(format!(
                "expected identifier or `(`, found `{token}` at position {pos}"
            ))),
            None => Err(PredicateParseError::new(
                "expected identifier or `(`, found end of input",
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn ident(name: &str) -> Predicate {
        Predicate::Identifier(name.to_string())
    }

    fn stack(names: &[&str]) -> ContextStack {
        names
            .iter()
            .map(|n| KeyContext::new(*n))
            .collect::<Vec<_>>()
            .into()
    }

    // -- Parsing ------------------------------------------------------------

    #[test]
    fn parses_identifier() {
        assert_eq!(Predicate::parse("Editor").unwrap(), ident("Editor"));
    }

    #[test]
    fn parses_equals() {
        assert_eq!(
            Predicate::parse("mode == insert").unwrap(),
            Predicate::Equals("mode".into(), "insert".into())
        );
        // Whitespace around `==` is optional.
        assert_eq!(
            Predicate::parse("mode==insert").unwrap(),
            Predicate::Equals("mode".into(), "insert".into())
        );
    }

    #[test]
    fn parses_not() {
        assert_eq!(
            Predicate::parse("!Editor").unwrap(),
            Predicate::Not(Box::new(ident("Editor")))
        );
        assert_eq!(
            Predicate::parse("!!Editor").unwrap(),
            Predicate::Not(Box::new(Predicate::Not(Box::new(ident("Editor")))))
        );
    }

    #[test]
    fn not_binds_tighter_than_and() {
        assert_eq!(
            Predicate::parse("!a && b").unwrap(),
            Predicate::And(
                Box::new(Predicate::Not(Box::new(ident("a")))),
                Box::new(ident("b"))
            )
        );
    }

    #[test]
    fn and_binds_tighter_than_or() {
        assert_eq!(
            Predicate::parse("a || b && c").unwrap(),
            Predicate::Or(
                Box::new(ident("a")),
                Box::new(Predicate::And(Box::new(ident("b")), Box::new(ident("c"))))
            )
        );
        assert_eq!(
            Predicate::parse("a && b || c").unwrap(),
            Predicate::Or(
                Box::new(Predicate::And(Box::new(ident("a")), Box::new(ident("b")))),
                Box::new(ident("c"))
            )
        );
    }

    #[test]
    fn parens_override_precedence() {
        assert_eq!(
            Predicate::parse("(a || b) && c").unwrap(),
            Predicate::And(
                Box::new(Predicate::Or(Box::new(ident("a")), Box::new(ident("b")))),
                Box::new(ident("c"))
            )
        );
    }

    #[test]
    fn child_binds_tighter_than_not() {
        // Per the grammar, `!a > b` is `!(a > b)`.
        assert_eq!(
            Predicate::parse("!a > b").unwrap(),
            Predicate::Not(Box::new(Predicate::Child(
                Box::new(ident("a")),
                Box::new(ident("b"))
            )))
        );
    }

    #[test]
    fn child_binds_tighter_than_and() {
        assert_eq!(
            Predicate::parse("a > b && c").unwrap(),
            Predicate::And(
                Box::new(Predicate::Child(Box::new(ident("a")), Box::new(ident("b")))),
                Box::new(ident("c"))
            )
        );
    }

    #[test]
    fn child_is_left_associative() {
        assert_eq!(
            Predicate::parse("a > b > c").unwrap(),
            Predicate::Child(
                Box::new(Predicate::Child(Box::new(ident("a")), Box::new(ident("b")))),
                Box::new(ident("c"))
            )
        );
    }

    #[test]
    fn parenthesized_child_operands() {
        assert_eq!(
            Predicate::parse("(a || b) > c").unwrap(),
            Predicate::Child(
                Box::new(Predicate::Or(Box::new(ident("a")), Box::new(ident("b")))),
                Box::new(ident("c"))
            )
        );
    }

    #[test]
    fn parse_error_on_empty_input() {
        let err = Predicate::parse("").unwrap_err();
        assert!(err.to_string().contains("end of input"));
    }

    #[test]
    fn parse_error_on_trailing_operator() {
        let err = Predicate::parse("a &&").unwrap_err();
        assert!(err.to_string().contains("end of input"));
    }

    #[test]
    fn parse_error_on_single_ampersand() {
        let err = Predicate::parse("a & b").unwrap_err();
        assert!(err.to_string().contains("&&"));
    }

    #[test]
    fn parse_error_on_single_pipe() {
        let err = Predicate::parse("a | b").unwrap_err();
        assert!(err.to_string().contains("||"));
    }

    #[test]
    fn parse_error_on_single_equals() {
        let err = Predicate::parse("a = b").unwrap_err();
        assert!(err.to_string().contains("=="));
    }

    #[test]
    fn parse_error_on_unclosed_paren() {
        let err = Predicate::parse("(a && b").unwrap_err();
        assert!(err.to_string().contains("unclosed"));
    }

    #[test]
    fn parse_error_on_missing_equals_value() {
        let err = Predicate::parse("mode ==").unwrap_err();
        assert!(err.to_string().contains("after `==`"));
    }

    #[test]
    fn parse_error_on_dangling_tokens() {
        let err = Predicate::parse("a b").unwrap_err();
        assert!(err.to_string().contains("unexpected"));
    }

    #[test]
    fn parse_error_on_invalid_character() {
        let err = Predicate::parse("a @ b").unwrap_err();
        assert!(err.to_string().contains("unexpected character"));
        assert!(err.to_string().contains("@"));
    }

    #[test]
    fn parse_error_on_lone_child_operator() {
        assert!(Predicate::parse(">").is_err());
        assert!(Predicate::parse("a >").is_err());
    }

    // -- KeyContext ---------------------------------------------------------

    #[test]
    fn context_ident_matches_name_and_attr_keys() {
        let ctx = KeyContext::new("Editor")
            .with_attr("mode", "insert")
            .with_flag("renaming");
        assert!(ctx.matches_ident("Editor"));
        assert!(ctx.matches_ident("mode"));
        assert!(ctx.matches_ident("renaming"));
        assert!(!ctx.matches_ident("Workspace"));
    }

    #[test]
    fn context_attr_lookup() {
        let ctx = KeyContext::new("Editor").with_attr("mode", "insert");
        assert_eq!(ctx.attr("mode"), Some("insert"));
        assert_eq!(ctx.attr("missing"), None);
    }

    // -- Evaluation ---------------------------------------------------------

    #[test]
    fn identifier_matches_some_context_in_stack() {
        let p = Predicate::parse("Editor").unwrap();
        assert!(p.matches(&stack(&["Workspace", "Editor"])));
        assert!(p.matches(&stack(&["Workspace", "Editor", "Menu"])));
        assert!(!p.matches(&stack(&["Workspace", "Pane"])));
    }

    #[test]
    fn identifier_does_not_match_empty_stack() {
        let p = Predicate::parse("Editor").unwrap();
        assert!(!p.matches(&ContextStack::new()));
        assert_eq!(p.match_depth(&ContextStack::new()), None);
    }

    #[test]
    fn match_depth_returns_deepest_match() {
        let p = Predicate::parse("Editor").unwrap();
        let s = stack(&["Editor", "Pane", "Editor"]);
        assert_eq!(p.match_depth(&s), Some(2));
        assert_eq!(p.match_depth(&stack(&["Editor", "Pane"])), Some(0));
    }

    #[test]
    fn equals_matches_attribute_value() {
        let s: ContextStack = vec![KeyContext::new("Editor").with_attr("mode", "insert")].into();
        assert!(Predicate::parse("mode == insert").unwrap().matches(&s));
        assert!(!Predicate::parse("mode == normal").unwrap().matches(&s));
        assert!(!Predicate::parse("missing == insert").unwrap().matches(&s));
    }

    #[test]
    fn and_requires_both_on_same_context() {
        let p = Predicate::parse("Editor && mode == insert").unwrap();
        let matching: ContextStack =
            vec![KeyContext::new("Editor").with_attr("mode", "insert")].into();
        assert!(p.matches(&matching));

        // Name and attribute live on *different* stack entries: no single
        // context satisfies the conjunction.
        let split: ContextStack = vec![
            KeyContext::new("Editor"),
            KeyContext::new("Menu").with_attr("mode", "insert"),
        ]
        .into();
        assert!(!p.matches(&split));
    }

    #[test]
    fn or_matches_either() {
        let p = Predicate::parse("Editor || Terminal").unwrap();
        assert!(p.matches(&stack(&["Workspace", "Terminal"])));
        assert!(p.matches(&stack(&["Editor"])));
        assert!(!p.matches(&stack(&["Workspace"])));
    }

    #[test]
    fn not_matches_contexts_without_ident() {
        let p = Predicate::parse("Editor && !renaming").unwrap();
        let plain: ContextStack = vec![KeyContext::new("Editor")].into();
        assert!(p.matches(&plain));
        let renaming: ContextStack = vec![KeyContext::new("Editor").with_flag("renaming")].into();
        assert!(!p.matches(&renaming));
    }

    #[test]
    fn child_requires_ancestor() {
        let p = Predicate::parse("Workspace > Editor").unwrap();
        assert!(p.matches(&stack(&["Workspace", "Editor"])));
        // Reversed order: Editor is not below Workspace.
        assert!(!p.matches(&stack(&["Editor", "Workspace"])));
        // Same context only: no ancestor.
        assert!(!p.matches(&stack(&["Editor"])));
    }

    #[test]
    fn child_matches_non_immediate_ancestor() {
        let p = Predicate::parse("Workspace > Editor").unwrap();
        assert!(p.matches(&stack(&["Workspace", "Pane", "Editor"])));
    }

    #[test]
    fn chained_child_requires_ordered_ancestors() {
        let p = Predicate::parse("Workspace > Pane > Editor").unwrap();
        assert!(p.matches(&stack(&["Workspace", "Pane", "Editor"])));
        assert!(p.matches(&stack(&["Workspace", "Other", "Pane", "Editor"])));
        assert!(!p.matches(&stack(&["Pane", "Workspace", "Editor"])));
    }

    #[test]
    fn child_with_attribute_predicates() {
        let p = Predicate::parse("Workspace > mode == insert").unwrap();
        let s: ContextStack = vec![
            KeyContext::new("Workspace"),
            KeyContext::new("Editor").with_attr("mode", "insert"),
        ]
        .into();
        assert!(p.matches(&s));
    }

    #[test]
    fn child_match_depth_is_descendant_depth() {
        let p = Predicate::parse("Workspace > Editor").unwrap();
        let s = stack(&["Workspace", "Pane", "Editor"]);
        assert_eq!(p.match_depth(&s), Some(2));
    }

    #[test]
    fn complex_predicate_evaluation() {
        let p = Predicate::parse("(Editor || Terminal) && !modal && mode == full").unwrap();
        let s: ContextStack = vec![
            KeyContext::new("Workspace"),
            KeyContext::new("Editor").with_attr("mode", "full"),
        ]
        .into();
        assert!(p.matches(&s));

        let modal: ContextStack = vec![
            KeyContext::new("Workspace"),
            KeyContext::new("Editor")
                .with_attr("mode", "full")
                .with_flag("modal"),
        ]
        .into();
        assert!(!p.matches(&modal));
    }

    #[test]
    fn stack_push_pop() {
        let mut s = ContextStack::new();
        assert!(s.is_empty());
        s.push(KeyContext::new("Workspace"));
        s.push(KeyContext::new("Editor"));
        assert_eq!(s.len(), 2);
        assert_eq!(s.pop().unwrap().name, "Editor");
        assert_eq!(s.len(), 1);
    }
}
