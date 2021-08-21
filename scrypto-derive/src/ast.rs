use syn::parse::{Parse, ParseStream};
use syn::{ItemImpl, ItemStruct, Result};

/// Represents the AST of blueprint.
pub struct Blueprint {
    pub structure: ItemStruct,
    pub implementation: ItemImpl,
}

impl Parse for Blueprint {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            structure: input.parse()?,
            implementation: input.parse()?,
        })
    }
}
