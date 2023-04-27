use super::*;
use crate::*;
use sbor::rust::prelude::*;
use sbor::traversal::TerminalValueRef;
use sbor::*;

impl ValidatableCustomTypeExtension<()> for ScryptoCustomTypeExtension {
    fn apply_custom_type_validation<'de>(
        _: &Self::CustomTypeValidation,
        _: &TerminalValueRef<'de, Self::CustomTraversal>,
        _: &mut (),
    ) -> Result<(), ValidationError> {
        Ok(())
    }
}
