use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, Token};

use super::element::NarrateElement;
use super::keywords::kw;
use super::keywords::{BLOCK_KEYWORDS, EVENT_NAMES, EventKind, suggest_similar};
use super::value::ModifierValue;

#[derive(Debug, Clone)]
pub enum BlockItem {
    Element(NarrateElement),
    Show(ShowStmt),
    On(OnStmt),
    Bind(BindStmt),
    When(WhenStmt),
    Each(EachStmt),
}

// ── show ──

#[derive(Debug, Clone)]
pub struct ShowStmt {
    pub value: ModifierValue,
    pub span: Span,
}

impl Parse for ShowStmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kw: kw::show = input.parse()?;
        let value: ModifierValue = input.parse()?;
        Ok(Self {
            value,
            span: kw.span,
        })
    }
}

// ── on ──

#[derive(Debug, Clone)]
pub struct OnStmt {
    pub event: EventKind,
    pub params: Option<Ident>,
    pub body: proc_macro2::TokenStream,
    pub span: Span,
}

impl Parse for OnStmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kw: kw::on = input.parse()?;

        // Event name
        let event_ident: Ident = input.parse()?;
        let event = EventKind::from_ident(&event_ident).ok_or_else(|| {
            let name = event_ident.to_string();
            let mut msg = format!("unknown event `{}`", name);

            if let Some(suggestion) = suggest_similar(&name, EVENT_NAMES) {
                msg.push_str(&format!(". Did you mean `{}`?", suggestion));
            } else {
                msg.push_str(". Expected one of: click, hover, key, focus, blur, scroll");
            }

            // Common mistake: using JS-style "on_click" or "onclick" instead of "on click"
            if name.starts_with("on") {
                let without_prefix = name
                    .strip_prefix("on_")
                    .or_else(|| name.strip_prefix("on"))
                    .unwrap_or(&name);
                if let Some(suggestion) = suggest_similar(without_prefix, EVENT_NAMES) {
                    msg = format!(
                        "unknown event `{}`. In plev_narrate!, use `on {}` (two words) \
                         instead of `{}`",
                        name, suggestion, name,
                    );
                }
            }

            syn::Error::new(event_ident.span(), msg)
        })?;

        // Optional closure params: |param|
        let params = if input.peek(Token![|]) {
            input.parse::<Token![|]>()?;
            let param: Ident = input.parse()?;
            input.parse::<Token![|]>()?;
            Some(param)
        } else {
            None
        };

        // Body in braces
        let content;
        syn::braced!(content in input);
        let body: proc_macro2::TokenStream = content.parse()?;

        Ok(Self {
            event,
            params,
            body,
            span: kw.span,
        })
    }
}

// ── bind ──

#[derive(Debug, Clone)]
pub struct BindStmt {
    pub target: Ident,
    pub value: ModifierValue,
    pub span: Span,
}

impl Parse for BindStmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kw: kw::bind = input.parse()?;
        let target: Ident = input.parse()?;
        input.parse::<kw::to>()?;
        let value: ModifierValue = input.parse()?;
        Ok(Self {
            target,
            value,
            span: kw.span,
        })
    }
}

// ── when ──

#[derive(Debug, Clone)]
pub struct WhenStmt {
    pub condition: Expr,
    pub body: Vec<BlockItem>,
    pub otherwise: Option<Vec<BlockItem>>,
    pub span: Span,
}

impl Parse for WhenStmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kw: kw::when = input.parse()?;

        // Condition in braces: when {expr} { ... }
        let cond_content;
        syn::braced!(cond_content in input);
        let condition: Expr = cond_content.parse()?;

        // Body block
        let body_content;
        syn::braced!(body_content in input);
        let body = parse_block_items(&body_content)?;

        // Optional otherwise clause
        let otherwise = if input.peek(kw::otherwise) {
            input.parse::<kw::otherwise>()?;
            let otherwise_content;
            syn::braced!(otherwise_content in input);
            Some(parse_block_items(&otherwise_content)?)
        } else {
            None
        };

        Ok(Self {
            condition,
            body,
            otherwise,
            span: kw.span,
        })
    }
}

// ── each ──

#[derive(Debug, Clone)]
pub struct EachStmt {
    pub binding: Ident,
    pub iterable: Expr,
    pub key: Option<Expr>,
    pub body: Vec<BlockItem>,
    pub span: Span,
}

impl Parse for EachStmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kw: kw::each = input.parse()?;

        let binding: Ident = input.parse()?;
        input.parse::<Token![in]>()?;

        // Iterable in braces
        let iter_content;
        syn::braced!(iter_content in input);
        let iterable: Expr = iter_content.parse()?;

        // Optional keyed by {expr}
        let key = if input.peek(kw::keyed) {
            input.parse::<kw::keyed>()?;
            input.parse::<kw::by>()?;
            let key_content;
            syn::braced!(key_content in input);
            Some(key_content.parse::<Expr>()?)
        } else {
            None
        };

        // Body block
        let body_content;
        syn::braced!(body_content in input);
        let body = parse_block_items(&body_content)?;

        Ok(Self {
            binding,
            iterable,
            key,
            body,
            span: kw.span,
        })
    }
}

// ── Block items parser ──

pub fn parse_block_items(input: ParseStream) -> syn::Result<Vec<BlockItem>> {
    let mut items = Vec::new();
    while !input.is_empty() {
        items.push(parse_block_item(input)?);
    }
    Ok(items)
}

fn parse_block_item(input: ParseStream) -> syn::Result<BlockItem> {
    if input.peek(kw::show) {
        Ok(BlockItem::Show(input.parse()?))
    } else if input.peek(kw::on) {
        Ok(BlockItem::On(input.parse()?))
    } else if input.peek(kw::when) {
        Ok(BlockItem::When(input.parse()?))
    } else if input.peek(kw::each) {
        Ok(BlockItem::Each(input.parse()?))
    } else if input.peek(kw::bind) {
        Ok(BlockItem::Bind(input.parse()?))
    } else if input.peek(Ident) {
        // Before trying to parse as element, check if it's a typo of a
        // block keyword so we give a better error than "unknown element"
        let fork = input.fork();
        if let Ok(ident) = fork.parse::<Ident>() {
            let name = ident.to_string();
            if let Some(suggestion) = suggest_similar(&name, BLOCK_KEYWORDS) {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("unknown keyword `{}`. Did you mean `{}`?", name, suggestion,),
                ));
            }
        }
        Ok(BlockItem::Element(input.parse()?))
    } else {
        Err(input.error(
            "expected a block item: `show`, `on`, `bind`, `when`, `each`, \
             or a child element (div, text, row, col, ...)",
        ))
    }
}
