#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_common::prelude::*;
use radix_engine_common::prelude::{scrypto_encode, ScryptoEncode, VersionedScryptoSchema};
use sbor::rust::vec::Vec;

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
pub struct FieldValue {
    pub value: Vec<u8>,
    pub locked: bool,
}

impl FieldValue {
    pub fn new<E: ScryptoEncode>(value: E) -> Self {
        Self {
            value: scrypto_encode(&value).unwrap(),
            locked: false,
        }
    }

    pub fn immutable<E: ScryptoEncode>(value: E) -> Self {
        Self {
            value: scrypto_encode(&value).unwrap(),
            locked: true,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct GenericArgs {
    pub additional_schema: Option<VersionedScryptoSchema>,
    pub generic_substitutions: Vec<GenericSubstitution>,
}

pub struct KVEntry {
    pub value: Option<Vec<u8>>,
    pub locked: bool,
}

/// A high level interface to manipulate objects in the actor's call frame
pub trait ClientObjectApi<E> {
    /// Creates a new simple blueprint object of a given blueprint type
    fn new_simple_object(
        &mut self,
        blueprint_ident: &str,
        fields: IndexMap<FieldIndex, FieldValue>,
    ) -> Result<NodeId, E> {
        self.new_object(
            blueprint_ident,
            vec![],
            GenericArgs::default(),
            fields,
            indexmap![],
        )
    }

    /// Creates a new object of a given blueprint type
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        features: Vec<&str>,
        generic_args: GenericArgs,
        fields: IndexMap<FieldIndex, FieldValue>,
        kv_entries: IndexMap<CollectionIndex, IndexMap<Vec<u8>, KVEntry>>,
    ) -> Result<NodeId, E>;

    /// Drops an owned object, returns the fields of the object
    fn drop_object(&mut self, node_id: &NodeId) -> Result<Vec<Vec<u8>>, E>;

    /// Get the blueprint id of a visible object
    fn get_blueprint_id(&mut self, node_id: &NodeId) -> Result<BlueprintId, E>;

    /// Get the outer object of a visible object
    fn get_outer_object(&mut self, node_id: &NodeId) -> Result<GlobalAddress, E>;

    /// Pre-allocates a global address, for a future globalization.
    fn allocate_global_address(
        &mut self,
        blueprint_id: BlueprintId,
    ) -> Result<(GlobalAddressReservation, GlobalAddress), E>;

    fn allocate_virtual_global_address(
        &mut self,
        blueprint_id: BlueprintId,
        global_address: GlobalAddress,
    ) -> Result<GlobalAddressReservation, E>;

    fn get_reservation_address(&mut self, node_id: &NodeId) -> Result<GlobalAddress, E>;

    /// Moves an object currently in the heap into the global space making
    /// it accessible to all with the provided global address.
    fn globalize(
        &mut self,
        node_id: NodeId,
        modules: IndexMap<AttachedModuleId, NodeId>,
        address_reservation: Option<GlobalAddressReservation>,
    ) -> Result<GlobalAddress, E>;

    fn globalize_with_address_and_create_inner_object_and_emit_event(
        &mut self,
        node_id: NodeId,
        modules: IndexMap<AttachedModuleId, NodeId>,
        address_reservation: GlobalAddressReservation,
        inner_object_blueprint: &str,
        inner_object_fields: IndexMap<u8, FieldValue>,
        event_name: &str,
        event_data: Vec<u8>,
    ) -> Result<(GlobalAddress, NodeId), E>;

    /// Calls a method on an object
    fn call_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;

    fn call_direct_access_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;

    /// Calls a method on an object module
    fn call_module_method(
        &mut self,
        receiver: &NodeId,
        module_id: AttachedModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;
}
