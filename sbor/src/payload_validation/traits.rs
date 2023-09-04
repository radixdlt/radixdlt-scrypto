use crate::traversal::*;
use crate::*;

pub trait ValidatableCustomExtension<T>: CustomExtension {
    // Note that the current SBOR extension only supports terminal custom type,
    // i.e., no custom value can be container.

    fn apply_validation_for_custom_value<'de>(
        schema: &Schema<Self::CustomSchema>,
        custom_value: &<Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
        type_id: LocalTypeId,
        context: &T,
    ) -> Result<(), PayloadValidationError<Self>>;

    fn apply_custom_type_validation_for_non_custom_value<'de>(
        schema: &Schema<Self::CustomSchema>,
        custom_type_validation: &<Self::CustomSchema as CustomSchema>::CustomTypeValidation,
        non_custom_value: &TerminalValueRef<'de, Self::CustomTraversal>,
        context: &T,
    ) -> Result<(), PayloadValidationError<Self>>;
}
