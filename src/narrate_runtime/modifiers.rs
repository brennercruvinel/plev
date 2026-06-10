//! Modifier application, body parsing, and skip handlers for the Parser.
//!
//! Split from parser.rs to keep each file under 300 lines.
//! All methods here are `impl Parser` extensions that handle:
//! - modifier chain recognition and application (style/layout attributes)
//! - body parsing (`show`, child elements)
//! - skipping dynamic blocks (`on`, `when`, `each`, `bind`)

use crate::builder::Element;

use super::keywords::{is_block_keyword, is_element_keyword, is_modifier_keyword};
use super::parser::Parser;
use super::tokenizer::Token;

impl Parser {
    // ── Modifier parsing ──

    pub(super) fn parse_modifiers(&mut self, mut element: Element) -> Element {
        loop {
            self.skip_comma();

            let Some(Token::Ident(name)) = self.tokens.get(self.pos) else {
                break;
            };

            if is_element_keyword(name) || is_block_keyword(name) {
                break;
            }
            if !is_modifier_keyword(name) {
                break;
            }

            let name = name.clone();
            self.pos += 1;

            element = self.apply_modifier(element, &name);
        }
        element
    }

    fn apply_modifier(&mut self, element: Element, name: &str) -> Element {
        match name {
            // ── Flags ──
            "flex" => element.flex(),
            "center" | "centered" => element.center(),
            "wrap" => element.wrap(),
            "bold" => element.bold(),
            "italic" => element.italic(),
            "uppercase" => element.uppercase(),

            // ── Numeric values ──
            "gap" => self.apply_f32(element, Element::gap),
            "p" => self.apply_f32(element, Element::p),
            "px" => self.apply_f32(element, Element::px),
            "py" => self.apply_f32(element, Element::py),
            "pt" => self.apply_f32(element, Element::pt),
            "pb" => self.apply_f32(element, Element::pb),
            "pl" => self.apply_f32(element, Element::pl),
            "pr" => self.apply_f32(element, Element::pr),
            "m" => self.apply_f32(element, Element::m),
            "mx" => self.apply_f32(element, Element::mx),
            "my" => self.apply_f32(element, Element::my),
            "w" => self.apply_f32(element, Element::w),
            "h" => self.apply_f32(element, Element::h),
            "min_w" => self.apply_f32(element, Element::min_w),
            "min_h" => self.apply_f32(element, Element::min_h),
            "max_w" => self.apply_f32(element, Element::max_w),
            "max_h" => self.apply_f32(element, Element::max_h),
            "grow" => self.apply_f32(element, Element::grow),
            "shrink" => self.apply_f32(element, Element::shrink),
            "basis" => self.apply_f32(element, Element::basis),
            "font_size" => self.apply_f32(element, Element::font_size),
            "opacity" => self.apply_f32(element, Element::opacity),
            "border" => self.apply_f32(element, Element::border),
            "shadow" => self.apply_f32(element, Element::shadow),
            "tracking" | "letter_spacing" => self.apply_f32(element, Element::tracking),

            // ── Color values ──
            "bg" => {
                if let Some(s) = self.take_str() {
                    element.bg(s.as_str())
                } else if self.peek_is_open_brace() {
                    self.skip_brace_block();
                    element
                } else {
                    element
                }
            }
            "text_color" => {
                if let Some(s) = self.take_str() {
                    element.text_color(s.as_str())
                } else if self.peek_is_open_brace() {
                    self.skip_brace_block();
                    element
                } else {
                    element
                }
            }
            "border_color" => {
                if let Some(s) = self.take_str() {
                    element.border_color(s.as_str())
                } else if self.peek_is_open_brace() {
                    self.skip_brace_block();
                    element
                } else {
                    element
                }
            }

            // ── Rounded (string preset or number) ──
            "rounded" => {
                if let Some(s) = self.take_str() {
                    element.rounded(s.as_str())
                } else if let Some(v) = self.take_f32() {
                    element.rounded(v)
                } else if self.peek_is_open_brace() {
                    self.skip_brace_block();
                    element
                } else {
                    element
                }
            }

            "font_weight" => {
                if let Some(v) = self.take_f32() {
                    element.font_weight(v.clamp(1.0, 1000.0) as u16)
                } else {
                    self.try_consume_value();
                    element
                }
            }

            _ => {
                log::warn!("Hot reload: unknown modifier '{}'", name);
                self.try_consume_value();
                element
            }
        }
    }

    /// Try to take an f32 value and apply it to the element via `method`.
    fn apply_f32(&mut self, element: Element, method: fn(Element, f32) -> Element) -> Element {
        if let Some(v) = self.take_f32() {
            method(element, v)
        } else if self.peek_is_open_brace() {
            self.skip_brace_block();
            element
        } else {
            element
        }
    }

    // ── Body parsing ──

    pub(super) fn parse_body(&mut self, mut element: Element) -> Element {
        while !self.at_end() && !self.peek_is(&Token::CloseBrace) {
            if let Some(Token::Ident(name)) = self.tokens.get(self.pos) {
                let name = name.clone();
                match name.as_str() {
                    "show" => {
                        self.pos += 1;
                        element = self.parse_show(element);
                    }
                    "on" => {
                        self.pos += 1;
                        self.skip_on_block();
                    }
                    "when" => {
                        self.pos += 1;
                        self.skip_when_block();
                    }
                    "each" => {
                        self.pos += 1;
                        self.skip_each_block();
                    }
                    "bind" => {
                        self.pos += 1;
                        self.skip_bind_stmt();
                    }
                    _ => {
                        if let Some(child) = self.parse_element() {
                            element = element.child(child);
                        } else {
                            break;
                        }
                    }
                }
            } else {
                break;
            }
        }
        element
    }

    fn parse_show(&mut self, element: Element) -> Element {
        if let Some(s) = self.take_str() {
            element.child(s.as_str())
        } else if self.peek_is_open_brace() {
            self.skip_brace_block();
            log::debug!("Hot reload: skipping show expression (requires recompile)");
            element
        } else {
            element
        }
    }

    fn skip_on_block(&mut self) {
        // on EVENT |params| { body }
        self.take_ident(); // event name
        if self.peek_is(&Token::Pipe) {
            self.pos += 1; // |
            self.take_ident(); // param
            if self.peek_is(&Token::Pipe) {
                self.pos += 1;
            } // |
        }
        self.skip_brace_block();
        log::debug!("Hot reload: skipping on block (requires recompile)");
    }

    fn skip_when_block(&mut self) {
        // when { condition } { body } otherwise { body }
        self.skip_brace_block(); // condition
        self.skip_brace_block(); // body
        if let Some(Token::Ident(s)) = self.tokens.get(self.pos) {
            if s == "otherwise" {
                self.pos += 1;
                self.skip_brace_block();
            }
        }
        log::debug!("Hot reload: skipping when block (requires recompile)");
    }

    fn skip_each_block(&mut self) {
        // each BINDING in { iterable } keyed by { key } { body }
        self.take_ident(); // binding
        if let Some(Token::Ident(s)) = self.tokens.get(self.pos) {
            if s == "in" {
                self.pos += 1;
            }
        }
        self.skip_brace_block(); // iterable
        if let Some(Token::Ident(s)) = self.tokens.get(self.pos) {
            if s == "keyed" {
                self.pos += 1;
                if let Some(Token::Ident(s)) = self.tokens.get(self.pos) {
                    if s == "by" {
                        self.pos += 1;
                    }
                }
                self.skip_brace_block(); // key expr
            }
        }
        self.skip_brace_block(); // body
        log::debug!("Hot reload: skipping each block (requires recompile)");
    }

    fn skip_bind_stmt(&mut self) {
        // bind TARGET to VALUE
        self.take_ident(); // target
        if let Some(Token::Ident(s)) = self.tokens.get(self.pos) {
            if s == "to" {
                self.pos += 1;
            }
        }
        self.try_consume_value();
        log::debug!("Hot reload: skipping bind (requires recompile)");
    }
}
