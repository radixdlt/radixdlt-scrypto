use crate::traversal::*;
use crate::*;

pub trait ValidatableCustomExtension<T>: CustomExtension {
    // Note that the current SBOR extension only supports terminal custom type,
    // i.e., no custom value can be container.

    fn apply_custom_type_validation<'de>(
        custom_type_validation: &<Self::CustomSchema as CustomSchema>::CustomTypeValidation,
        value: &TerminalValueRef<'de, Self::CustomTraversal>,
        context: &mut T,
    ) -> Result<(), ValidationError>;
}
