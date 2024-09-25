use crate::internal_prelude::*;

mod event_stream_traverser;
mod events;
mod untyped_traverser;
mod utility_visitors;

pub use event_stream_traverser::*;
pub use events::*;
pub use untyped_traverser::*;
pub use utility_visitors::*;

/// Returns the length of the value at the start of the partial payload.
pub fn calculate_value_tree_body_byte_length<'de, 's, E: CustomExtension>(
    partial_payload: &'de [u8],
    value_kind: ValueKind<E::CustomValueKind>,
    current_depth: usize,
    depth_limit: usize,
) -> Result<usize, DecodeError> {
    let mut traverser = UntypedTraverser::<E::CustomTraversal>::new(
        partial_payload,
        UntypedTraverserConfig {
            max_depth: depth_limit - current_depth,
            check_exact_end: false,
        },
    );
    traverser.run_from_start(ExpectedStart::ValueBody(value_kind), &mut ValidatingVisitor)
}
