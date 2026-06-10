use syn::parse::{Parse, ParseStream};
use syn::{Expr, LitFloat, LitInt, LitStr};

#[derive(Debug, Clone)]
pub enum ModifierValue {
    Str(LitStr),
    Int(LitInt),
    Float(LitFloat),
    Expr(Expr),
}

impl Parse for ModifierValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            Ok(Self::Str(input.parse()?))
        } else if input.peek(LitFloat) {
            Ok(Self::Float(input.parse()?))
        } else if input.peek(LitInt) {
            Ok(Self::Int(input.parse()?))
        } else if input.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);
            Ok(Self::Expr(content.parse()?))
        } else {
            Err(input.error("expected a value (string, number, or {expression})"))
        }
    }
}

/// Check if the next token could be a value (literal or braced expression).
pub fn peek_value(input: ParseStream) -> bool {
    input.peek(LitStr)
        || input.peek(LitFloat)
        || input.peek(LitInt)
        || input.peek(syn::token::Brace)
}
