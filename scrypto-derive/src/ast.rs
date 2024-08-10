use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{As, Brace, Paren};
use syn::{
    braced, parenthesized, Attribute, Ident, ItemConst, ItemImpl, ItemMacro, ItemStruct, ItemUse,
    Path, Result, Token, Visibility,
};

/// Represents a blueprint which is a module with an optional set of attributes
pub struct Blueprint {
    pub attributes: Vec<Attribute>,
    pub module: BlueprintMod,
}

impl Parse for Blueprint {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attributes: input.call(Attribute::parse_outer)?,
            module: input.parse()?,
        })
    }
}

/// Represents a Blueprint module which consists of a struct and an implementation of said struct
#[allow(dead_code)] // Fields for tokens from parse for completeness
pub struct BlueprintMod {
    pub vis: Visibility,
    pub mod_token: Token![mod],
    pub module_ident: Ident,
    pub brace: Brace,
    pub use_statements: Vec<ItemUse>,
    pub const_statements: Vec<ItemConst>,
    pub macro_statements: Vec<ItemMacro>,
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
        let (const_statements, macro_statements) = {
            let mut const_statements = Vec::new();
            let mut macro_statements = Vec::new();
            loop {
                let is_const = content.peek(Token![const]);
                let is_macro = content.peek2(Token![!]);
                if !is_const && !is_macro {
                    break;
                }
                if is_const {
                    const_statements.push(content.call(ItemConst::parse)?)
                }
                if is_macro {
                    macro_statements.push(content.call(ItemMacro::parse)?)
                }
            }

            (const_statements, macro_statements)
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
            const_statements,
            macro_statements,
            structure,
            implementation,
            semi,
        })
    }
}

#[allow(dead_code)] // Fields for tokens from parse for completeness
pub struct EventsInner {
    pub paren_token: Paren,
    pub paths: Punctuated<Path, Token![,]>,
}

impl Parse for EventsInner {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            paren_token: parenthesized!(content in input),
            paths: content.parse_terminated(Path::parse)?,
        })
    }
}

#[allow(dead_code)] // Fields for tokens from parse for completeness
pub struct TypesInner {
    pub paren_token: Paren,
    pub aliasable_types: Punctuated<AliasableType, Token![,]>,
}

impl Parse for TypesInner {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            paren_token: parenthesized!(content in input),
            aliasable_types: content.parse_terminated(AliasableType::parse)?,
        })
    }
}

pub struct AliasableType {
    pub path: Path,
    pub alias: Option<Ident>,
}

impl Parse for AliasableType {
    fn parse(input: ParseStream) -> Result<Self> {
        let rust_type: Path = input.parse()?;
        let alias = if input.parse::<Option<As>>()?.is_some() {
            Some(input.parse::<Ident>()?)
        } else {
            None
        };

        Ok(Self {
            path: rust_type,
            alias,
        })
    }
}
