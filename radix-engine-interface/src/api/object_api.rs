use crate::api::node_modules::auth::ROLE_ASSIGNMENT_BLUEPRINT;
use crate::api::node_modules::metadata::METADATA_BLUEPRINT;
use crate::api::CollectionIndex;
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
use radix_engine_interface::api::FieldIndex;
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

impl From<Option<ModuleId>> for ObjectModuleId {
    fn from(value: Option<ModuleId>) -> Self {
        match value {
            None => ObjectModuleId::Main,
            Some(ModuleId::Metadata) => ObjectModuleId::Metadata,
            Some(ModuleId::Royalty) => ObjectModuleId::Royalty,
            Some(ModuleId::RoleAssignment) => ObjectModuleId::RoleAssignment,
        }
    }
}

impl Into<Option<ModuleId>> for ObjectModuleId {
    fn into(self) -> Option<ModuleId> {
        match self {
            ObjectModuleId::Main => None,
            ObjectModuleId::Metadata => Some(ModuleId::Metadata),
            ObjectModuleId::Royalty => Some(ModuleId::Royalty),
            ObjectModuleId::RoleAssignment => Some(ModuleId::RoleAssignment),
        }
    }
}

impl ObjectModuleId {
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
pub enum ModuleId {
    Metadata = 1,
    Royalty = 2,
    RoleAssignment = 3,
}

impl ModuleId {
    pub fn static_blueprint(&self) -> BlueprintId {
        match self {
            ModuleId::Metadata => BlueprintId::new(&METADATA_MODULE_PACKAGE, METADATA_BLUEPRINT),
            ModuleId::Royalty => {
                BlueprintId::new(&ROYALTY_MODULE_PACKAGE, COMPONENT_ROYALTY_BLUEPRINT)
            }
            ModuleId::RoleAssignment => {
                BlueprintId::new(&ROLE_ASSIGNMENT_MODULE_PACKAGE, ROLE_ASSIGNMENT_BLUEPRINT)
            }
        }
    }
}

impl Into<ObjectModuleId> for ModuleId {
    fn into(self) -> ObjectModuleId {
        match self {
            ModuleId::Metadata => ObjectModuleId::Metadata,
            ModuleId::Royalty => ObjectModuleId::Royalty,
            ModuleId::RoleAssignment => ObjectModuleId::RoleAssignment,
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
        fields: BTreeMap<FieldIndex, FieldValue>,
    ) -> Result<NodeId, E> {
        self.new_object(
            blueprint_ident,
            vec![],
            GenericArgs::default(),
            fields,
            btreemap![],
        )
    }

    /// Creates a new object of a given blueprint type
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        features: Vec<&str>,
        generic_args: GenericArgs,
        fields: BTreeMap<FieldIndex, FieldValue>,
        kv_entries: BTreeMap<CollectionIndex, BTreeMap<Vec<u8>, KVEntry>>,
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
        modules: BTreeMap<ModuleId, NodeId>,
        address_reservation: Option<GlobalAddressReservation>,
    ) -> Result<GlobalAddress, E>;

    fn globalize_with_address_and_create_inner_object_and_emit_event(
        &mut self,
        node_id: NodeId,
        modules: BTreeMap<ModuleId, NodeId>,
        address_reservation: GlobalAddressReservation,
        inner_object_blueprint: &str,
        inner_object_fields: BTreeMap<u8, FieldValue>,
        event_name: String,
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
        module_id: ModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;
}
