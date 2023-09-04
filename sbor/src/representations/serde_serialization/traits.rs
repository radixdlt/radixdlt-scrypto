use super::*;
use crate::representations::*;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

pub struct CustomTypeSerialization<'a, 't, 'de, 's1, 's2, E: SerializableCustomExtension> {
    pub include_type_tag_in_simple_mode: bool,
    pub serialization: SerializableType<'a, 't, 'de, 's1, 's2, E>,
}

// Note - the Copy here is to work around the dodgy derive implementation of Copy on SerializationContext
pub trait SerializableCustomExtension: FormattableCustomExtension + Copy {
    fn map_value_for_serialization<'s, 'de, 'a, 't, 's1, 's2>(
        context: &SerializationContext<'s, 'a, Self>,
        type_id: LocalTypeId,
        value: <Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
    ) -> CustomTypeSerialization<'a, 't, 'de, 's1, 's2, Self>;
}
