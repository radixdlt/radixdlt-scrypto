use crate::internal_prelude::*;
use crate::types::*;
use core::fmt::Formatter;
use radix_common::address::{AddressDisplayContext, NO_NETWORK};
use radix_rust::ContextualDisplay;
use sbor::rust::prelude::*;
use sbor::rust::string::String;

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
