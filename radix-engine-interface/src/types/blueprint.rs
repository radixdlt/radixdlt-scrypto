use crate::blueprints::package::{BlueprintVersion, BlueprintVersionKey};
use crate::ScryptoSbor;
use core::fmt;
use core::fmt::Formatter;
use radix_engine_common::address::{AddressDisplayContext, NO_NETWORK};
use radix_engine_common::types::GlobalAddress;
use radix_engine_common::types::PackageAddress;
use radix_engine_derive::ManifestSbor;
use sbor::rust::prelude::*;
use scrypto_schema::{InstanceSchema, KeyValueStoreSchema};
use utils::ContextualDisplay;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ObjectInfo {
    pub global: bool,

    pub blueprint_id: BlueprintId,
    pub version: BlueprintVersion,

    // Blueprint parameters
    pub outer_object: Option<GlobalAddress>,
    pub instance_schema: Option<InstanceSchema>,
    pub features: BTreeSet<String>,
}

impl ObjectInfo {
    pub fn blueprint_version_key(&self) -> BlueprintVersionKey {
        BlueprintVersionKey {
            blueprint: self.blueprint_id.blueprint_name.clone(),
            version: self.version,
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
