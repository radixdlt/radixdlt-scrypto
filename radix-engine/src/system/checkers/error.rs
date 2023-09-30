use radix_engine_interface::prelude::*;

#[derive(Debug, Clone)]
pub struct LocatedError<T> {
    /// The location where the error was encountered. This is the full path a field or a collection
    /// as well as the value.
    pub location: ErrorLocation,
    /// The encountered error.
    pub error: T,
}

impl<T> LocatedError<T> {
    pub fn new(location: ErrorLocation, error: T) -> Self {
        Self { location, error }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorLocation {
    Field {
        info: BlueprintInfo,
        node_id: NodeId,
        module_id: ModuleId,
        field_index: FieldIndex,
        value: Vec<u8>,
    },
    CollectionEntry {
        info: BlueprintInfo,
        node_id: NodeId,
        module_id: ModuleId,
        collection_index: CollectionIndex,
        key: Vec<u8>,
        value: Vec<u8>,
    },
}
