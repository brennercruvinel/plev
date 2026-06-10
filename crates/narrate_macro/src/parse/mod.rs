// AST nodes — fields parsed but not yet fully consumed by codegen.
#[allow(dead_code)]
pub mod block_item;
#[allow(dead_code)]
pub mod element;
pub mod keywords;
#[allow(dead_code)]
pub mod modifier;
pub mod value;

#[cfg(test)]
mod tests;

use element::NarrateElement;
use syn::parse::{Parse, ParseStream};

#[derive(Debug, Clone)]
pub struct NarrateBlock {
    pub elements: Vec<NarrateElement>,
}

impl Parse for NarrateBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut elements = Vec::new();
        while !input.is_empty() {
            elements.push(input.parse()?);
        }
        Ok(NarrateBlock { elements })
    }
}
