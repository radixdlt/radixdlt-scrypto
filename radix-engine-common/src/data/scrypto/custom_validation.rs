use super::*;
use crate::*;
use sbor::rust::prelude::*;
use sbor::traversal::TerminalValueRef;
use sbor::*;

impl ValidatableCustomExtension<()> for ScryptoCustomExtension {
    fn apply_custom_type_validation<'de>(
        _: &<Self::CustomSchema as CustomSchema>::CustomTypeValidation,
        _: &TerminalValueRef<'de, Self::CustomTraversal>,
        _: &mut (),
    ) -> Result<(), ValidationError> {
        Ok(())
    }
}
