use crate::ScryptoSbor;
use core::fmt;
use core::fmt::Formatter;
use radix_engine_common::address::{AddressDisplayContext, NO_NETWORK};
use radix_engine_common::native_addresses::*;
use radix_engine_common::types::{EntityType, GlobalAddress};
use radix_engine_common::types::{NodeId, PackageAddress};
use radix_engine_derive::ManifestSbor;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use scrypto_schema::{InstanceSchema, KeyValueStoreSchema};
use utils::ContextualDisplay;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum IDAllocationRequest {
    Object {
        blueprint_id: BlueprintId,
        global: bool,
        virtual_node_id: Option<NodeId>,
    },
    KeyValueStore,
    GlobalAddressOwnership,
    GlobalObjectPhantom {
        blueprint_id: BlueprintId,
    },
}

impl IDAllocationRequest {
    pub fn is_global(&self) -> bool {
        match self {
            IDAllocationRequest::Object { global, .. } => *global,
            IDAllocationRequest::KeyValueStore => false,
            IDAllocationRequest::GlobalAddressOwnership => false,
            IDAllocationRequest::GlobalObjectPhantom { .. } => true,
        }
    }

    pub fn entity_type(&self) -> EntityType {
        match self {
            IDAllocationRequest::Object {
                blueprint_id,
                global,
                virtual_node_id: _,
            } => {
                // FIXME final check before Babylon release!
                if *global {
                    match (
                        blueprint_id.package_address,
                        blueprint_id.blueprint_name.as_str(),
                    ) {
                        (ACCOUNT_PACKAGE, PACKAGE_BLUEPRINT) => EntityType::GlobalPackage,
                        (RESOURCE_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT) => {
                            EntityType::GlobalFungibleResourceManager
                        }
                        (RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT) => {
                            EntityType::GlobalNonFungibleResourceManager
                        }
                        (CONSENSUS_MANAGER_PACKAGE, CONSENSUS_MANAGER_BLUEPRINT) => {
                            EntityType::GlobalConsensusManager
                        }
                        (CONSENSUS_MANAGER_PACKAGE, VALIDATOR_BLUEPRINT) => {
                            EntityType::GlobalValidator
                        }
                        (ACCESS_CONTROLLER_PACKAGE, ACCESS_CONTROLLER_BLUEPRINT) => {
                            EntityType::GlobalAccessController
                        }
                        (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT) => EntityType::GlobalAccount,
                        (IDENTITY_PACKAGE, IDENTITY_BLUEPRINT) => EntityType::GlobalIdentity,
                        _ => EntityType::GlobalGenericComponent,
                    }
                } else {
                    match (
                        blueprint_id.package_address,
                        blueprint_id.blueprint_name.as_str(),
                    ) {
                        (RESOURCE_PACKAGE, FUNGIBLE_VAULT_BLUEPRINT) => {
                            EntityType::InternalFungibleVault
                        }
                        (RESOURCE_PACKAGE, NON_FUNGIBLE_VAULT_BLUEPRINT) => {
                            EntityType::InternalNonFungibleVault
                        }
                        (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT) => EntityType::InternalAccount,
                        _ => EntityType::InternalGenericComponent,
                    }
                }
            }
            IDAllocationRequest::KeyValueStore => EntityType::InternalKeyValueStore,
            IDAllocationRequest::GlobalAddressOwnership => EntityType::InternalGenericComponent,
            IDAllocationRequest::GlobalObjectPhantom { blueprint_id } => match (
                blueprint_id.package_address,
                blueprint_id.blueprint_name.as_str(),
            ) {
                (ACCOUNT_PACKAGE, PACKAGE_BLUEPRINT) => EntityType::GlobalPackage,
                (RESOURCE_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT) => {
                    EntityType::GlobalFungibleResourceManager
                }
                (RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT) => {
                    EntityType::GlobalNonFungibleResourceManager
                }
                (CONSENSUS_MANAGER_PACKAGE, CONSENSUS_MANAGER_BLUEPRINT) => {
                    EntityType::GlobalConsensusManager
                }
                (CONSENSUS_MANAGER_PACKAGE, VALIDATOR_BLUEPRINT) => EntityType::GlobalValidator,
                (ACCESS_CONTROLLER_PACKAGE, ACCESS_CONTROLLER_BLUEPRINT) => {
                    EntityType::GlobalAccessController
                }
                (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT) => EntityType::GlobalAccount,
                (IDENTITY_PACKAGE, IDENTITY_BLUEPRINT) => EntityType::GlobalIdentity,
                _ => EntityType::GlobalGenericComponent,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ObjectInfo {
    pub blueprint: BlueprintId,
    pub global: bool,
    pub outer_object: Option<GlobalAddress>,
    pub instance_schema: Option<InstanceSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct GlobalObjectPhantomInfo {
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
