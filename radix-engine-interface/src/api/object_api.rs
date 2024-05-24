use crate::api::CollectionIndex;
use crate::internal_prelude::*;
use crate::object_modules::metadata::METADATA_BLUEPRINT;
use crate::object_modules::role_assignment::ROLE_ASSIGNMENT_BLUEPRINT;
use crate::types::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use radix_common::constants::{
    METADATA_MODULE_PACKAGE, ROLE_ASSIGNMENT_MODULE_PACKAGE, ROYALTY_MODULE_PACKAGE,
};
use radix_common::prelude::{scrypto_encode, ScryptoEncode, VersionedScryptoSchema};
use radix_common::types::*;
use radix_common::{ManifestSbor, ScryptoSbor};
use radix_engine_interface::api::FieldIndex;
use radix_engine_interface::object_modules::royalty::COMPONENT_ROYALTY_BLUEPRINT;
use sbor::rust::collections::*;
use sbor::rust::prelude::*;
use sbor::rust::vec::Vec;

#[repr(u8)]
#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    FromRepr,
    EnumIter,
    ManifestSbor,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
pub enum ModuleId {
    Main,
    Metadata,
    Royalty,
    RoleAssignment,
}

/// Notes: This is to be deprecated, please use `ModuleId` instead
pub type ObjectModuleId = ModuleId;

impl Describe<ScryptoCustomTypeKind> for ModuleId {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::MODULE_ID_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::module_id_type_data()
    }
}

impl From<Option<AttachedModuleId>> for ModuleId {
    fn from(value: Option<AttachedModuleId>) -> Self {
        match value {
            None => ModuleId::Main,
            Some(AttachedModuleId::Metadata) => ModuleId::Metadata,
            Some(AttachedModuleId::Royalty) => ModuleId::Royalty,
            Some(AttachedModuleId::RoleAssignment) => ModuleId::RoleAssignment,
        }
    }
}

impl Into<Option<AttachedModuleId>> for ModuleId {
    fn into(self) -> Option<AttachedModuleId> {
        match self {
            ModuleId::Main => None,
            ModuleId::Metadata => Some(AttachedModuleId::Metadata),
            ModuleId::Royalty => Some(AttachedModuleId::Royalty),
            ModuleId::RoleAssignment => Some(AttachedModuleId::RoleAssignment),
        }
    }
}

impl ModuleId {
    pub fn base_partition_num(&self) -> PartitionNumber {
        match self {
            ModuleId::Metadata => METADATA_BASE_PARTITION,
            ModuleId::Royalty => ROYALTY_BASE_PARTITION,
            ModuleId::RoleAssignment => ROLE_ASSIGNMENT_BASE_PARTITION,
            ModuleId::Main => MAIN_BASE_PARTITION,
        }
    }

    pub fn static_blueprint(&self) -> Option<BlueprintId> {
        match self {
            ModuleId::Metadata => Some(BlueprintId::new(
                &METADATA_MODULE_PACKAGE,
                METADATA_BLUEPRINT,
            )),
            ModuleId::Royalty => Some(BlueprintId::new(
                &ROYALTY_MODULE_PACKAGE,
                COMPONENT_ROYALTY_BLUEPRINT,
            )),
            ModuleId::RoleAssignment => Some(BlueprintId::new(
                &ROLE_ASSIGNMENT_MODULE_PACKAGE,
                ROLE_ASSIGNMENT_BLUEPRINT,
            )),
            ModuleId::Main => None,
        }
    }
}

#[repr(u8)]
#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    FromRepr,
    EnumIter,
    ManifestSbor,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
#[sbor(use_repr_discriminators)]
pub enum AttachedModuleId {
    Metadata = 1,
    Royalty = 2,
    RoleAssignment = 3,
}

impl Describe<ScryptoCustomTypeKind> for AttachedModuleId {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::ATTACHED_MODULE_ID_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::attached_module_id_type_data()
    }
}

impl AttachedModuleId {
    pub fn static_blueprint(&self) -> BlueprintId {
        match self {
            AttachedModuleId::Metadata => {
                BlueprintId::new(&METADATA_MODULE_PACKAGE, METADATA_BLUEPRINT)
            }
            AttachedModuleId::Royalty => {
                BlueprintId::new(&ROYALTY_MODULE_PACKAGE, COMPONENT_ROYALTY_BLUEPRINT)
            }
            AttachedModuleId::RoleAssignment => {
                BlueprintId::new(&ROLE_ASSIGNMENT_MODULE_PACKAGE, ROLE_ASSIGNMENT_BLUEPRINT)
            }
        }
    }
}

impl Into<ModuleId> for AttachedModuleId {
    fn into(self) -> ModuleId {
        match self {
            AttachedModuleId::Metadata => ModuleId::Metadata,
            AttachedModuleId::Royalty => ModuleId::Royalty,
            AttachedModuleId::RoleAssignment => ModuleId::RoleAssignment,
        }
    }
}

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
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
pub trait SystemObjectApi<E> {
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

    /// Allocates a global address, for a future globalization.
    fn allocate_global_address(
        &mut self,
        blueprint_id: BlueprintId,
    ) -> Result<(GlobalAddressReservation, GlobalAddress), E>;

    /// Allocates a specific virtual global address
    fn allocate_virtual_global_address(
        &mut self,
        blueprint_id: BlueprintId,
        global_address: GlobalAddress,
    ) -> Result<GlobalAddressReservation, E>;

    /// Retrieve the global address of a given address reservation
    fn get_reservation_address(&mut self, node_id: &NodeId) -> Result<GlobalAddress, E>;

    /// Moves an object currently in the heap into the global space making
    /// it accessible to all with the provided global address.
    fn globalize(
        &mut self,
        node_id: NodeId,
        modules: IndexMap<AttachedModuleId, NodeId>,
        address_reservation: Option<GlobalAddressReservation>,
    ) -> Result<GlobalAddress, E>;

    /// Globalizes with a new inner object and emits an event
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

    /// Calls a direct access method on an object
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
