use crate::api::ObjectModuleId;
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FunctionIdentifier(pub BlueprintId, pub String);

impl FunctionIdentifier {
    pub fn new(blueprint: BlueprintId, ident: String) -> Self {
        Self(blueprint, ident)
    }

    pub fn size(&self) -> usize {
        self.0.len() + self.1.len()
    }
}

#[derive(Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FnIdentifier {
    pub blueprint_id: BlueprintId,
    pub ident: String,
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
