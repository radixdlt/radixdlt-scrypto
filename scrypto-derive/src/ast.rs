use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{braced, Ident, ItemImpl, ItemStruct, ItemUse, Result, Token, Visibility};

/// Represents a Blueprint module which consists of a struct and an implementation of said struct
pub struct BlueprintMod {
    pub vis: Visibility,
    pub mod_token: Token![mod],
    pub module_ident: Ident,
    pub brace: Brace,
    pub use_statements: Vec<ItemUse>,
    pub structure: ItemStruct,
    pub implementation: ItemImpl,
    pub semi: Option<Token![;]>,
}

impl Parse for BlueprintMod {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        let vis = input.parse()?;
        let mod_token = input.parse()?;
        let module_ident = input.parse()?;
        let brace = braced!(content in input);
        let use_statements = {
            let mut use_statements = Vec::new();
            while content.peek(Token![use]) {
                use_statements.push(content.call(ItemUse::parse)?)
            }
            use_statements
        };
        let structure = content.parse()?;
        let implementation = content.parse()?;
        let semi = input.parse()?;

        Ok(Self {
            vis,
            mod_token,
            module_ident,
            brace,
            use_statements,
            structure,
            implementation,
            semi,
        })
    }
}
