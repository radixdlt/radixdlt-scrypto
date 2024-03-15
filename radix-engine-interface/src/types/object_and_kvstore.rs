use crate::blueprints::package::BlueprintVersion;
use crate::internal_prelude::*;
use radix_blueprint_schema_init::KeyValueStoreGenericSubstitutions;
use radix_common::types::BlueprintId;
use radix_common::types::{GenericSubstitution, GlobalAddress};
use radix_engine_interface::api::AttachedModuleId;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum OuterObjectInfo {
    Some { outer_object: GlobalAddress },
    None,
}

impl OuterObjectInfo {
    pub fn expect(&self) -> GlobalAddress {
        match self {
            OuterObjectInfo::Some { outer_object } => *outer_object,
            OuterObjectInfo::None => panic!("Object has no outer object"),
        }
    }
}

impl Default for OuterObjectInfo {
    fn default() -> Self {
        OuterObjectInfo::None
    }
}

/// Core object state, persisted in `TypeInfoSubstate`.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct BlueprintInfo {
    pub blueprint_id: BlueprintId,
    pub blueprint_version: BlueprintVersion,
    pub outer_obj_info: OuterObjectInfo,
    pub features: IndexSet<String>,
    pub generic_substitutions: Vec<GenericSubstitution>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ObjectType {
    Global {
        modules: IndexMap<AttachedModuleId, BlueprintVersion>,
    },
    Owned,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ObjectInfo {
    /// Blueprint Info of Object
    pub blueprint_info: BlueprintInfo,
    pub object_type: ObjectType,
}

impl ObjectInfo {
    pub fn is_global(&self) -> bool {
        match self.object_type {
            ObjectType::Global { .. } => true,
            ObjectType::Owned => false,
        }
    }

    pub fn get_outer_object(&self) -> GlobalAddress {
        match &self.blueprint_info.outer_obj_info {
            OuterObjectInfo::Some { outer_object } => outer_object.clone(),
            OuterObjectInfo::None { .. } => {
                panic!("Broken Application logic: Expected to be an inner object but is an outer object");
            }
        }
    }

    pub fn get_features(&self) -> IndexSet<String> {
        self.blueprint_info.features.clone()
    }

    pub fn try_get_outer_object(&self) -> Option<GlobalAddress> {
        match &self.blueprint_info.outer_obj_info {
            OuterObjectInfo::Some { outer_object } => Some(outer_object.clone()),
            OuterObjectInfo::None { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct GlobalAddressPhantom {
    pub blueprint_id: BlueprintId,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct KeyValueStoreInfo {
    pub generic_substitutions: KeyValueStoreGenericSubstitutions,
}
