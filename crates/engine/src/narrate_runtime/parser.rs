//! Recursive-descent parser for narrate DSL tokens.
//!
//! Converts a flat token stream into an `Element` tree by recognizing
//! element keywords, modifier chains, body blocks (`show`, `on`, `when`,
//! `each`, `bind`), and nested children.
//!
//! Modifier application and body/skip logic live in `modifiers.rs`
//! (an `impl Parser` extension) to keep each file under 300 lines.

use crate::builder::{self, Element};

use super::tokenizer::{Token, tokenize};

// ── Parser ──

pub(crate) struct Parser {
    pub(super) tokens: Vec<Token>,
    pub(super) pos: usize,
}

impl Parser {
    pub(crate) fn new(input: &str) -> Self {
        Self {
            tokens: tokenize(input),
            pos: 0,
        }
    }

    pub(super) fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    // ── Atomic take helpers (no borrow conflicts) ──

    pub(super) fn take_ident(&mut self) -> Option<String> {
        if let Some(Token::Ident(s)) = self.tokens.get(self.pos) {
            let s = s.clone();
            self.pos += 1;
            Some(s)
        } else {
            None
        }
    }

    pub(super) fn take_str(&mut self) -> Option<String> {
        if let Some(Token::Str(s)) = self.tokens.get(self.pos) {
            let s = s.clone();
            self.pos += 1;
            Some(s)
        } else {
            None
        }
    }

    pub(super) fn take_f32(&mut self) -> Option<f32> {
        match self.tokens.get(self.pos) {
            Some(Token::Int(n)) => {
                let v = *n as f32;
                self.pos += 1;
                Some(v)
            }
            Some(Token::Float(f)) => {
                let v = *f as f32;
                self.pos += 1;
                Some(v)
            }
            _ => None,
        }
    }

    pub(super) fn peek_is(&self, expected: &Token) -> bool {
        self.tokens.get(self.pos) == Some(expected)
    }

    pub(super) fn peek_is_open_brace(&self) -> bool {
        matches!(self.tokens.get(self.pos), Some(Token::OpenBrace))
    }

    pub(super) fn skip_comma(&mut self) {
        if self.peek_is(&Token::Comma) {
            self.pos += 1;
        }
    }

    /// Consume a brace-delimited block without interpreting it.
    pub(super) fn skip_brace_block(&mut self) {
        if !self.peek_is_open_brace() {
            return;
        }
        self.pos += 1;
        let mut depth = 1;
        while depth > 0 && self.pos < self.tokens.len() {
            match &self.tokens[self.pos] {
                Token::OpenBrace => depth += 1,
                Token::CloseBrace => depth -= 1,
                _ => {}
            }
            self.pos += 1;
        }
    }

    /// Try to consume whatever value follows (string, number, or brace block).
    pub(super) fn try_consume_value(&mut self) {
        match self.tokens.get(self.pos) {
            Some(Token::Str(_) | Token::Int(_) | Token::Float(_)) => {
                self.pos += 1;
            }
            Some(Token::OpenBrace) => self.skip_brace_block(),
            _ => {}
        }
    }

    // ── Top-level parsing ──

    pub(crate) fn parse_top_level(&mut self) -> Vec<Element> {
        let mut elements = Vec::new();
        while !self.at_end() {
            if self.peek_is(&Token::CloseBrace) {
                break;
            }
            if let Some(el) = self.parse_element() {
                elements.push(el);
            } else {
                break;
            }
        }
        elements
    }

    pub(super) fn parse_element(&mut self) -> Option<Element> {
        let name = self.take_ident()?;

        let mut element = match name.as_str() {
            "div" => builder::div(),
            "row" => builder::div().flex().row(),
            "col" => builder::div().flex().col(),
            "text" => builder::text(""),
            "button" => builder::div()
                .bg(crate::color::Color::rgba(0.25, 0.25, 0.35, 1.0))
                .rounded(4.0_f32)
                .p(8.0_f32),
            "image" => builder::image(),
            "spacer" => builder::spacer(),
            _ if name.starts_with(|c: char| c.is_ascii_uppercase()) => {
                log::warn!(
                    "Hot reload: custom component '{}' rendered as placeholder",
                    name
                );
                builder::div()
            }
            _ => {
                log::warn!("Hot reload: unknown element '{}'", name);
                return None;
            }
        };

        element = self.parse_modifiers(element);
        self.skip_comma();

        if self.peek_is_open_brace() {
            self.pos += 1; // consume {
            element = self.parse_body(element);
            if self.peek_is(&Token::CloseBrace) {
                self.pos += 1; // consume }
            }
        }

        Some(element)
    }
}
