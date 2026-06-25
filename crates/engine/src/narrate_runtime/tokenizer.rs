//! Tokenizer for narrate DSL text.
//!
//! Converts raw DSL input into a flat `Vec<Token>`, handling string
//! literals with escape sequences, integers, floats, identifiers,
//! braces, commas, pipes, and `//` line comments.

// ── Token type ──

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    Ident(String),
    Str(String),
    Int(i64),
    Float(f64),
    OpenBrace,
    CloseBrace,
    Comma,
    Pipe,
}

// ── Tokenizer ──

pub(crate) fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '{' => {
                tokens.push(Token::OpenBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::CloseBrace);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '|' => {
                tokens.push(Token::Pipe);
                chars.next();
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        chars.next();
                        break;
                    }
                    if ch == '\\' {
                        chars.next();
                        if let Some(&esc) = chars.peek() {
                            match esc {
                                'n' => s.push('\n'),
                                't' => s.push('\t'),
                                '\\' => s.push('\\'),
                                '"' => s.push('"'),
                                _ => {
                                    s.push('\\');
                                    s.push(esc);
                                }
                            }
                            chars.next();
                        }
                    } else {
                        s.push(ch);
                        chars.next();
                    }
                }
                tokens.push(Token::Str(s));
            }
            c if c.is_ascii_digit() => {
                let mut num = String::new();
                let mut is_float = false;
                while let Some(&ch) = chars.peek() {
                    if ch == '.' && !is_float {
                        is_float = true;
                        num.push(ch);
                        chars.next();
                    } else if ch.is_ascii_digit() {
                        num.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if is_float {
                    tokens.push(Token::Float(num.parse().unwrap_or(0.0)));
                } else {
                    tokens.push(Token::Int(num.parse().unwrap_or(0)));
                }
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut ident = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(ident));
            }
            '/' if chars.clone().nth(1) == Some('/') => {
                while let Some(&ch) = chars.peek() {
                    chars.next();
                    if ch == '\n' {
                        break;
                    }
                }
            }
            _ => {
                chars.next();
            }
        }
    }

    tokens
}
