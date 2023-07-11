use crate::types::*;
use crate::*;
use core::fmt::Formatter;
use radix_engine_common::address::{AddressDisplayContext, NO_NETWORK};
use sbor::rust::prelude::*;
use sbor::rust::string::String;
use utils::ContextualDisplay;

#[derive(Clone, Eq, PartialEq, ScryptoSbor)]
pub enum FnIdent {
    Application(String),
    System(u8),
}

impl FnIdent {
    pub fn len(&self) -> usize {
        match self {
            FnIdent::System(..) => 1,
            FnIdent::Application(ident) => ident.len(),
        }
    }

    pub fn to_debug_string(&self) -> String {
        match self {
            FnIdent::Application(x) => x.clone(),
            FnIdent::System(x) => x.to_string(),
        }
    }
}

impl Debug for FnIdent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FnIdent::Application(method) => {
                write!(f, "<{}>", method)
            }
            FnIdent::System(i) => {
                write!(f, "#{}#", i)
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FnIdentifier {
    pub blueprint_id: BlueprintId,
    pub ident: FnIdent,
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for FnIdentifier {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        write!(
            f,
            "{}:{:?}",
            self.blueprint_id.display(*context),
            self.ident,
        )
    }
}

impl Debug for FnIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}
