use crate::api::node_modules::auth::ACCESS_RULES_BLUEPRINT;
use crate::api::node_modules::metadata::METADATA_BLUEPRINT;
use crate::constants::{
    ACCESS_RULES_MODULE_PACKAGE, METADATA_MODULE_PACKAGE, ROYALTY_MODULE_PACKAGE,
};
use crate::types::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_common::types::*;
use radix_engine_derive::{ManifestSbor, ScryptoSbor};
use radix_engine_interface::api::node_modules::royalty::COMPONENT_ROYALTY_BLUEPRINT;
use sbor::rust::collections::*;
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;
use scrypto_schema::InstanceSchema;

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
    AccessRules,
}

impl ObjectModuleId {
    pub fn to_u8(&self) -> u8 {
        match self {
            ObjectModuleId::Main => 0u8,
            ObjectModuleId::Metadata => 1u8,
            ObjectModuleId::Royalty => 2u8,
            ObjectModuleId::AccessRules => 3u8,
        }
    }

    pub fn base_partition_num(&self) -> PartitionNumber {
        match self {
            ObjectModuleId::Metadata => METADATA_KV_STORE_PARTITION,
            ObjectModuleId::Royalty => ROYALTY_BASE_PARTITION,
            ObjectModuleId::AccessRules => ACCESS_RULES_BASE_PARTITION,
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
            ObjectModuleId::AccessRules => Some(BlueprintId::new(
                &ACCESS_RULES_MODULE_PACKAGE,
                ACCESS_RULES_BLUEPRINT,
            )),
            ObjectModuleId::Main => None,
        }
    }
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
        fields: Vec<Vec<u8>>,
    ) -> Result<NodeId, E> {
        self.new_object(blueprint_ident, vec![], None, fields, btreemap![])
    }

    /// Creates a new object of a given blueprint type
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        features: Vec<&str>,
        schema: Option<InstanceSchema>,
        fields: Vec<Vec<u8>>,
        kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>>,
    ) -> Result<NodeId, E>;

    /// Drops an object, returns the fields of the object
    fn drop_object(&mut self, node_id: &NodeId) -> Result<Vec<Vec<u8>>, E>;

    /// Get info regarding a visible object
    fn get_object_info(&mut self, node_id: &NodeId) -> Result<ObjectInfo, E>;

    fn get_reservation_address(&mut self, node_id: &NodeId) -> Result<GlobalAddress, E>;

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

    /// Moves an object currently in the heap into the global space making
    /// it accessible to all with the provided global address.
    fn globalize(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
        address_reservation: Option<GlobalAddressReservation>,
    ) -> Result<GlobalAddress, E>;

    fn globalize_with_address_and_create_inner_object(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
        address_reservation: GlobalAddressReservation,
        inner_object_blueprint: &str,
        inner_object_fields: Vec<Vec<u8>>,
    ) -> Result<(GlobalAddress, NodeId), E>;

    /// Calls a method on an object
    fn call_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E> {
        self.call_method_advanced(receiver, false, ObjectModuleId::Main, method_name, args)
    }

    fn call_direct_access_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E> {
        self.call_method_advanced(receiver, true, ObjectModuleId::Main, method_name, args)
    }

    /// Calls a method on an object module
    fn call_method_advanced(
        &mut self,
        receiver: &NodeId,
        direct_access: bool, // May change to enum for other types of reference in future
        module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;
}
