use syn::parse::{Parse, ParseStream};
use syn::{ItemImpl, ItemStruct, Result};

/// Represents the AST of Component definition
pub struct Component {
    pub structure: ItemStruct,
    pub implementation: ItemImpl,
}

impl Parse for Component {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Component {
            structure: input.parse()?,
            implementation: input.parse()?,
        })
    }
}
