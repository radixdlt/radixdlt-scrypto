use crate::blueprints::package::BlueprintVersion;
use crate::ScryptoSbor;
use core::fmt;
use core::fmt::Formatter;
use radix_engine_common::address::{AddressDisplayContext, NO_NETWORK};
use radix_engine_common::types::GlobalAddress;
use radix_engine_common::types::PackageAddress;
use radix_engine_derive::ManifestSbor;
use radix_engine_interface::api::ObjectModuleId;
use sbor::rust::prelude::*;
use scrypto_schema::{InstanceSchema, KeyValueStoreSchema};
use utils::ContextualDisplay;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum OuterObjectInfo {
    Some { outer_object: GlobalAddress },
    None,
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
    pub outer_obj_info: OuterObjectInfo,
    pub features: BTreeSet<String>,
    pub instance_schema: Option<InstanceSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ObjectInfo {
    /// Whether this node is global or not, ie. true, if this node has no parent, false otherwise
    pub global: bool,
    pub module_versions: BTreeMap<ObjectModuleId, BlueprintVersion>,

    /// Blueprint Info of Object
    pub blueprint_info: BlueprintInfo,
}

impl ObjectInfo {
    pub fn get_outer_object(&self) -> GlobalAddress {
        match &self.blueprint_info.outer_obj_info {
            OuterObjectInfo::Some { outer_object } => outer_object.clone(),
            OuterObjectInfo::None { .. } => {
                panic!("Broken Application logic: Expected to be an inner object but is an outer object");
            }
        }
    }

    pub fn get_features(&self) -> BTreeSet<String> {
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
    pub schema: KeyValueStoreSchema,
}

#[derive(Clone, PartialEq, Eq, Hash, ScryptoSbor, ManifestSbor)]
pub struct BlueprintId {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
}

impl BlueprintId {
    pub fn new<S: ToString>(package_address: &PackageAddress, blueprint_name: S) -> Self {
        BlueprintId {
            package_address: *package_address,
            blueprint_name: blueprint_name.to_string(),
        }
    }

    pub fn len(&self) -> usize {
        self.package_address.as_ref().len() + self.blueprint_name.len()
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for BlueprintId {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        write!(
            f,
            "{}:<{}>",
            self.package_address.display(*context),
            self.blueprint_name,
        )
    }
}

impl core::fmt::Debug for BlueprintId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}
