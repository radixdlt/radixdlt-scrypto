use crate::api::node_modules::auth::ROLE_ASSIGNMENT_BLUEPRINT;
use crate::api::node_modules::metadata::METADATA_BLUEPRINT;
use crate::constants::{
    METADATA_MODULE_PACKAGE, ROLE_ASSIGNMENT_MODULE_PACKAGE, ROYALTY_MODULE_PACKAGE,
};
use crate::types::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_common::prelude::{scrypto_encode, ScryptoEncode, ScryptoSchema};
use radix_engine_common::types::*;
use radix_engine_derive::{ManifestSbor, ScryptoSbor};
use radix_engine_interface::api::node_modules::royalty::COMPONENT_ROYALTY_BLUEPRINT;
use sbor::rust::collections::*;
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

#[repr(u8)]
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    ScryptoSbor,
    ManifestSbor,
    FromRepr,
    EnumIter,
)]
pub enum ObjectModuleId {
    Main,
    Metadata,
    Royalty,
    RoleAssignment,
}

impl ObjectModuleId {
    pub fn to_u8(&self) -> u8 {
        match self {
            ObjectModuleId::Main => 0u8,
            ObjectModuleId::Metadata => 1u8,
            ObjectModuleId::Royalty => 2u8,
            ObjectModuleId::RoleAssignment => 3u8,
        }
    }

    pub fn base_partition_num(&self) -> PartitionNumber {
        match self {
            ObjectModuleId::Metadata => METADATA_BASE_PARTITION,
            ObjectModuleId::Royalty => ROYALTY_BASE_PARTITION,
            ObjectModuleId::RoleAssignment => ROLE_ASSIGNMENT_BASE_PARTITION,
            ObjectModuleId::Main => MAIN_BASE_PARTITION,
        }
    }

    pub fn static_blueprint(&self) -> Option<BlueprintId> {
        match self {
            ObjectModuleId::Metadata => Some(BlueprintId::new(
                &METADATA_MODULE_PACKAGE,
                METADATA_BLUEPRINT,
            )),
            ObjectModuleId::Royalty => Some(BlueprintId::new(
                &ROYALTY_MODULE_PACKAGE,
                COMPONENT_ROYALTY_BLUEPRINT,
            )),
            ObjectModuleId::RoleAssignment => Some(BlueprintId::new(
                &ROLE_ASSIGNMENT_MODULE_PACKAGE,
                ROLE_ASSIGNMENT_BLUEPRINT,
            )),
            ObjectModuleId::Main => None,
        }
    }
}

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
    pub additional_schema: Option<ScryptoSchema>,
    pub type_substitution_refs: Vec<TypeSubstitutionRef>,
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
        fields: Vec<FieldValue>,
    ) -> Result<NodeId, E> {
        self.new_object(blueprint_ident, vec![], GenericArgs::default(), fields, btreemap![])
    }

    /// Creates a new object of a given blueprint type
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        features: Vec<&str>,
        generic_args: GenericArgs,
        fields: Vec<FieldValue>,
        kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>>,
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
        modules: BTreeMap<ObjectModuleId, NodeId>,
        address_reservation: Option<GlobalAddressReservation>,
    ) -> Result<GlobalAddress, E>;

    fn globalize_with_address_and_create_inner_object_and_emit_event(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
        address_reservation: GlobalAddressReservation,
        inner_object_blueprint: &str,
        inner_object_fields: Vec<FieldValue>,
        event_name: String,
        event_data: Vec<u8>,
    ) -> Result<(GlobalAddress, NodeId), E>;

    /// Calls a method on an object
    fn call_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E> {
        self.call_method_advanced(receiver, ObjectModuleId::Main, false, method_name, args)
    }

    fn call_direct_access_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E> {
        self.call_method_advanced(receiver, ObjectModuleId::Main, true, method_name, args)
    }

    /// Calls a method on an object module
    fn call_method_advanced(
        &mut self,
        receiver: &NodeId,
        module_id: ObjectModuleId,
        direct_access: bool, // May change to enum for other types of reference in future
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;
}
