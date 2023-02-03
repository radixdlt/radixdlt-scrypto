use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{braced, Ident, ItemImpl, ItemStruct, Result, Token, Visibility};

/// Represents a Blueprint module which consists of a struct and an implementation of said struct
pub struct BlueprintMod {
    pub vis: Visibility,
    pub mod_token: Token![mod],
    pub ident: Ident,
    pub brace: Brace,
    pub structure: ItemStruct,
    pub implementation: ItemImpl,
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
            structure: content.parse()?,
            implementation: content.parse()?,
            semi: input.parse()?,
        })
    }
}
