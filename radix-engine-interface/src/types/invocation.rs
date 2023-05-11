use crate::api::ObjectModuleId;
use crate::blueprints::resource::MethodKey;
use crate::types::*;
use crate::*;
use core::fmt::Formatter;
use radix_engine_common::address::{AddressDisplayContext, NO_NETWORK};
use radix_engine_common::types::*;
use sbor::rust::prelude::*;
use sbor::rust::string::String;
use utils::ContextualDisplay;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct MethodIdentifier(pub NodeId, pub ObjectModuleId, pub String);

impl MethodIdentifier {
    pub fn method_key(&self) -> MethodKey {
        MethodKey::new(self.1, self.2.as_str())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FunctionIdentifier(pub Blueprint, pub String);

impl FunctionIdentifier {
    pub fn new(blueprint: Blueprint, ident: String) -> Self {
        Self(blueprint, ident)
    }

    pub fn size(&self) -> usize {
        self.0.len() + self.1.len()
    }
}

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
    pub blueprint: Blueprint,
    pub ident: FnIdent,
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for FnIdentifier {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        write!(f, "{}:{:?}", self.blueprint.display(*context), self.ident,)
    }
}

impl Debug for FnIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}
