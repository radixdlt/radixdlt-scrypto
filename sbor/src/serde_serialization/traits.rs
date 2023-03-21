use super::*;
use crate::traversal::*;
use crate::*;

pub trait CustomSerializationContext<'a>: Default + Copy {
    type CustomTypeExtension: SerializableCustomTypeExtension<CustomSerializationContext<'a> = Self>;
}

pub struct CustomTypeSerialization<'a, 't, 'de, 's1, 's2, E: SerializableCustomTypeExtension> {
    pub include_type_tag_in_simple_mode: bool,
    pub serialization: SerializableType<'a, 't, 'de, 's1, 's2, E>,
}

// Note - the Copy here is to work around the dodgy implementation of deriving Copy for SerializationContext
pub trait SerializableCustomTypeExtension: CustomTypeExtension + Copy {
    type CustomSerializationContext<'a>: CustomSerializationContext<'a, CustomTypeExtension = Self>;

    fn serialize_value<'s, 'de, 'a, 't, 's1, 's2>(
        context: &SerializationContext<'s, 'a, Self>,
        type_index: LocalTypeIndex,
        value: <Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
    ) -> CustomTypeSerialization<'a, 't, 'de, 's1, 's2, Self>;
}
