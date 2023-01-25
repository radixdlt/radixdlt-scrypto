use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{braced, Ident, ItemImpl, ItemStruct, Result, Token, Visibility};

pub struct BlueprintMod {
    pub vis: Visibility,
    pub mod_token: Token![mod],
    pub ident: Ident,
    pub brace: Brace,
    pub blueprint: Blueprint,
    pub semi: Option<Token![;]>,
}

impl Parse for BlueprintMod {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            vis: input.parse()?,
            mod_token: input.parse()?,
            ident: input.parse()?,
            brace: braced!(content in input),
            blueprint: content.parse()?,
            semi: input.parse()?,
        })
    }
}

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
