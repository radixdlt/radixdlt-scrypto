use super::*;
use crate::*;
use sbor::rust::prelude::*;
use sbor::*;

impl ValidatableCustomTypeExtension for ScryptoCustomTypeExtension {
    type ValidationContext = ();

    fn validate_custom_value<'de, L: SchemaTypeLink>(
        _custom_value_ref: &<Self::CustomTraversal as traversal::CustomTraversal>::CustomTerminalValueRef<'de>,
        _custom_type_kind: &Self::CustomTypeKind<L>,
        _context: &Self::ValidationContext,
    ) -> Result<(), ValidationError> {
        Ok(())
    }
}
