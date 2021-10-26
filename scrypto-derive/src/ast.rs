use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{ItemImpl, ItemStruct, Path, Result};

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

/// Represents the AST of allowed badges for authorization.
pub struct Auth {
    pub allowed: Punctuated<Path, Comma>,
}

impl Parse for Auth {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            allowed: Punctuated::parse_terminated(input)?,
        })
    }
}
