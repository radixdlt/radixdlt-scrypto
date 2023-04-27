use radix_engine::types::*;
use sbor::rust::prelude::*;

// Import and re-export these types so they are available easily with a single import
pub use radix_engine::blueprints::access_controller::*;
pub use radix_engine::blueprints::account::*;
pub use radix_engine::blueprints::clock::*;
pub use radix_engine::blueprints::epoch_manager::*;
pub use radix_engine::blueprints::package::*;
pub use radix_engine::blueprints::resource::*;
pub use radix_engine::system::node_modules::access_rules::*;
pub use radix_engine::system::node_modules::metadata::*;
pub use radix_engine::system::node_modules::royalty::*;
pub use radix_engine::system::node_modules::type_info::*;

//=========================================================================
// Please update REP-60 after updating types/configs defined in this file!
//
// The below defines well-known substate types which are used in the
// Core API of the node.
//
// Specifically:
// * Every (EntityType, SysModuleId, SubstateKey) should be mappable into a `TypedSubstateKey`
// * Every (&TypedSubstateKey, Data) should be mappable into a `WellKnownSubstateData`
//
// Please keep them these in-line with the well-known objects, and please don't
// remove these without talking to the Network team.
//=========================================================================

//=========================================================================
// TODO - move this to a relevant REP when it's been created
//
// BACKGROUND:
// A generic Object SysModule consists of 0 or more DbModules, each with a u8 "ModuleId".
//
// These modules are one of four types:
// - Tuple
//   => Has key: TupleKey(u8) also known as an offset
//   => No iteration exposed to engine
//   => Is versioned / locked substate-by-substate
// - ConcurrentMap
//   => Has key: MapKey(Vec<u8>)
//   => No iteration exposed to engine
//   => Is versioned / locked substate-by-substate
// - Index
//   => Has key: MapKey(Vec<u8>)
//   => Iteration exposed to engine via the MapKey's database key (ie hash of the key)
//   => Is versioned / locked in its entirety
// - SortedU16Index(SortedU16Key(U16, Vec<u8>))
//   => Has key: MapKey(Vec<u8>)
//   => Iteration exposed to engine via the user-controlled U16 prefix and then the MapKey's database key (ie hash of the key)
//   => Is versioned / locked in its entirety
//
// But in this file, we just handle explicitly supported/possible combinations of things.
//
// An entirely generic capturing of a substate type would look something like this:
//
// pub enum GenericObjectModuleSubstateType {
//    Tuple(ModuleId, TupleKey),
//    ConcurrentMap(ModuleId, MapKey),
//    Index(ModuleId, MapKey),
//    SortedU16Index(ModuleId, SortedU16Key),
// }
//=========================================================================

/// By node module (roughly SysModule)
#[derive(Debug, Clone)]
pub enum TypedSubstateKey {
    TypeInfoModule(TypeInfoOffset),
    AccessRulesModule(AccessRulesOffset),
    RoyaltyModule(RoyaltyOffset),
    MetadataModule(String),
    ObjectModule(TypedObjectModuleSubstateKey),
}

/// Doesn't include non-object modules, nor transient nodes.
#[derive(Debug, Clone)]
pub enum TypedObjectModuleSubstateKey {
    // Objects
    Package(PackageOffset),
    FungibleResource(ResourceManagerOffset),
    NonFungibleResource(ResourceManagerOffset),
    FungibleVault(FungibleVaultOffset),
    NonFungibleVault(NonFungibleVaultOffset),
    EpochManager(EpochManagerOffset),
    Clock(ClockOffset),
    Validator(ValidatorOffset),
    Account(AccountOffset),
    AccessController(AccessControllerOffset),
    // Generic Scrypto Components
    GenericScryptoComponent(ComponentOffset),
    // Substates for Generic KV Stores
    GenericKeyValueStore(MapKey), // Is an entity type with a single ConcurrentMap
    GenericIndex(MapKey),         // Is an entity type with a single Index
    GenericSortedU16Index(SortedU16Key), // Is an entity type with a single u16 index
}

fn error(descriptor: &'static str) -> String {
    format!("Could not convert {} to TypedSubstateKey", descriptor)
}

pub fn to_typed_substate_key(
    entity_type: EntityType,
    module_id: ModuleId,
    substate_key: &SubstateKey,
) -> Result<TypedSubstateKey, String> {
    let sys_module_id = SysModuleId::try_from(module_id)
        .map_err(|_| format!("Could not convert ModuleId {:?}", module_id))?;
    let substate_type = match sys_module_id {
        SysModuleId::TypeInfo => TypedSubstateKey::TypeInfoModule(
            TypeInfoOffset::try_from(substate_key).map_err(|_| error("TypeInfoOffset"))?,
        ),
        SysModuleId::Metadata => TypedSubstateKey::MetadataModule(
            scrypto_decode(
                substate_key
                    .for_map()
                    .ok_or_else(|| error("Metadata key"))?,
            )
            .map_err(|_| error("string Metadata key"))?,
        ),
        SysModuleId::Royalty => TypedSubstateKey::RoyaltyModule(
            RoyaltyOffset::try_from(substate_key).map_err(|_| error("RoyaltyOffset"))?,
        ),
        SysModuleId::AccessRules => TypedSubstateKey::AccessRulesModule(
            AccessRulesOffset::try_from(substate_key).map_err(|_| error("AccessRulesOffset"))?,
        ),
        SysModuleId::Object => TypedSubstateKey::ObjectModule(to_typed_object_module_substate_key(
            entity_type,
            substate_key,
        )?),
        // SysModuleId::Virtualized is just a very ugly workaround at the moment
        SysModuleId::Virtualized => {
            to_typed_virtualized_module_substate_key(entity_type, substate_key)?
        }
    };
    Ok(substate_type)
}

pub fn to_typed_object_module_substate_key(
    entity_type: EntityType,
    substate_key: &SubstateKey,
) -> Result<TypedObjectModuleSubstateKey, String> {
    return to_typed_object_substate_key_no_error(entity_type, substate_key).map_err(|_| {
        format!(
            "Could not convert {:?} {:?} key to TypedObjectSubstateKey",
            entity_type, substate_key
        )
    });
}

fn to_typed_object_substate_key_no_error(
    entity_type: EntityType,
    substate_key: &SubstateKey,
) -> Result<TypedObjectModuleSubstateKey, ()> {
    let substate_type = match entity_type {
        EntityType::InternalGenericComponent | EntityType::GlobalGenericComponent => {
            TypedObjectModuleSubstateKey::GenericScryptoComponent(ComponentOffset::try_from(
                substate_key,
            )?)
        }
        EntityType::GlobalPackage => {
            TypedObjectModuleSubstateKey::Package(PackageOffset::try_from(substate_key)?)
        }
        EntityType::GlobalFungibleResource => TypedObjectModuleSubstateKey::FungibleResource(
            ResourceManagerOffset::try_from(substate_key)?,
        ),
        EntityType::GlobalNonFungibleResource => TypedObjectModuleSubstateKey::NonFungibleResource(
            ResourceManagerOffset::try_from(substate_key)?,
        ),
        EntityType::GlobalEpochManager => {
            TypedObjectModuleSubstateKey::EpochManager(EpochManagerOffset::try_from(substate_key)?)
        }
        EntityType::GlobalValidator => {
            TypedObjectModuleSubstateKey::Validator(ValidatorOffset::try_from(substate_key)?)
        }
        EntityType::GlobalClock => {
            TypedObjectModuleSubstateKey::Clock(ClockOffset::try_from(substate_key)?)
        }
        EntityType::GlobalAccessController => TypedObjectModuleSubstateKey::AccessController(
            AccessControllerOffset::try_from(substate_key)?,
        ),
        EntityType::GlobalVirtualEcdsaAccount
        | EntityType::GlobalVirtualEddsaAccount
        | EntityType::InternalAccount
        | EntityType::GlobalAccount => {
            TypedObjectModuleSubstateKey::Account(AccountOffset::try_from(substate_key)?)
        }
        EntityType::GlobalVirtualEcdsaIdentity
        | EntityType::GlobalVirtualEddsaIdentity
        | EntityType::GlobalIdentity => Err(())?, // Identity doesn't have any substates
        EntityType::InternalFungibleVault => TypedObjectModuleSubstateKey::FungibleVault(
            FungibleVaultOffset::try_from(substate_key)?,
        ),
        EntityType::InternalNonFungibleVault => TypedObjectModuleSubstateKey::NonFungibleVault(
            NonFungibleVaultOffset::try_from(substate_key)?,
        ),
        EntityType::InternalKeyValueStore
        | EntityType::InternalIndex
        | EntityType::InternalSortedIndex => Err(())?, // KVStore, Index and SortedIndex currently use Virtualized module
    };
    Ok(substate_type)
}

// SysModuleId::Virtualized is currently a messy workaround / hodge-podge of different ideas and will be removed soon.
pub fn to_typed_virtualized_module_substate_key(
    entity_type: EntityType,
    substate_key: &SubstateKey,
) -> Result<TypedSubstateKey, String> {
    let substate_type = match (entity_type, substate_key) {
        (EntityType::InternalKeyValueStore, SubstateKey::Map(key)) => {
            TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::GenericKeyValueStore(
                key.clone(),
            ))
        }
        (EntityType::InternalIndex, SubstateKey::Map(key)) => {
            TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::GenericIndex(key.clone()))
        }
        (EntityType::InternalSortedIndex, SubstateKey::Sorted(key)) => {
            TypedSubstateKey::ObjectModule(TypedObjectModuleSubstateKey::GenericSortedU16Index(
                key.clone(),
            ))
        }
        (_, SubstateKey::Map(key)) => {
            // For some reason, Metadata gets mapped under Virtualized SysModuleId on any entity type
            // But the good thing is that it's the only thing which is mapped under Virtualized SysModuleId for global components
            TypedSubstateKey::MetadataModule(
                scrypto_decode(key).map_err(|_| error("string Metadata key"))?,
            )
        }
        // Everything else is should be on the object substate
        _ => Err(format!(
            "Could not convert {:?} {:?} key on Virtualized module to TypedObjectSubstateKey",
            entity_type, substate_key
        ))?,
    };
    Ok(substate_type)
}

#[derive(Debug, Clone)]
pub enum TypedSubstateValue {
    TypeInfoModule(TypedTypeInfoModuleSubstateValue),
    AccessRulesModule(TypedAccessRulesModuleSubstateValue),
    RoyaltyModule(TypedRoyaltyModuleSubstateValue),
    MetadataModule(TypedMetadataModuleSubstateValue),
    ObjectModule(TypedObjectModuleSubstateValue),
}

#[derive(Debug, Clone)]
pub enum TypedTypeInfoModuleSubstateValue {
    TypeInfo(TypeInfoSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedAccessRulesModuleSubstateValue {
    MethodAccessRules(MethodAccessRulesSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedRoyaltyModuleSubstateValue {
    ComponentRoyaltyConfig(ComponentRoyaltyConfigSubstate),
    ComponentRoyaltyAccumulator(ComponentRoyaltyAccumulatorSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedMetadataModuleSubstateValue {
    Metadata(MetadataValueSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedObjectModuleSubstateValue {
    // Objects
    Package(TypedPackageSubstateValue),
    FungibleResource(TypedFungibleResourceManagerSubstateValue),
    NonFungibleResource(TypedNonFungibleResourceManagerSubstateValue),
    FungibleVault(TypedFungibleVaultSubstateValue),
    NonFungibleVault(TypedNonFungibleVaultSubstateValue),
    EpochManager(TypedEpochManagerSubstateValue),
    Clock(TypedClockSubstateValue),
    Validator(TypedValidatorSubstateValue),
    Account(TypedAccountSubstateValue),
    AccessController(TypedAccessControllerSubstateValue),
    // Generic Scrypto Components
    GenericScryptoComponent(GenericScryptoComponentSubstateValue),
    // Substates for Generic KV Stores
    GenericKeyValueStore(GenericScryptoSborPayload),
    GenericIndex(GenericScryptoSborPayload),
    GenericSortedU16Index(GenericScryptoSborPayload),
}

#[derive(Debug, Clone)]
pub enum TypedPackageSubstateValue {
    Info(PackageInfoSubstate),
    CodeType(PackageCodeTypeSubstate),
    Code(PackageCodeSubstate),
    Royalty(PackageRoyaltySubstate),
    FunctionAccessRules(PackageFunctionAccessRulesSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedFungibleResourceManagerSubstateValue {
    ResourceManager(FungibleResourceManagerSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedNonFungibleResourceManagerSubstateValue {
    ResourceManager(NonFungibleResourceManagerSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedFungibleVaultSubstateValue {
    Divisibility(FungibleVaultDivisibilitySubstate),
    Balance(FungibleVaultBalanceSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedNonFungibleVaultSubstateValue {
    IdType(NonFungibleVaultIdTypeSubstate),
    Balance(NonFungibleVaultBalanceSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedEpochManagerSubstateValue {
    EpochManager(EpochManagerSubstate),
    CurrentValidatorSet(CurrentValidatorSetSubstate),
    RegisteredValidatorSet(SecondaryIndexSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedClockSubstateValue {
    CurrentTimeRoundedToMinutes(ClockSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedValidatorSubstateValue {
    Validator(ValidatorSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedAccountSubstateValue {
    Account(AccountSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedAccessControllerSubstateValue {
    AccessController(AccessControllerSubstate),
}

#[derive(Debug, Clone)]
pub enum GenericScryptoComponentSubstateValue {
    State(GenericScryptoSborPayload),
}

#[derive(Debug, Clone)]
pub struct GenericScryptoSborPayload {
    pub data: Vec<u8>,
}

pub fn to_typed_substate_value(
    substate_key: &TypedSubstateKey,
    data: &[u8],
) -> Result<TypedSubstateValue, String> {
    to_typed_substate_value_internal(substate_key, data).map_err(|err| {
        format!(
            "Error decoding substate data for key {:?} - {:?}",
            substate_key, err
        )
    })
}

fn to_typed_substate_value_internal(
    substate_key: &TypedSubstateKey,
    data: &[u8],
) -> Result<TypedSubstateValue, DecodeError> {
    let substate_value = match substate_key {
        TypedSubstateKey::TypeInfoModule(type_info_offset) => {
            TypedSubstateValue::TypeInfoModule(match type_info_offset {
                TypeInfoOffset::TypeInfo => {
                    TypedTypeInfoModuleSubstateValue::TypeInfo(scrypto_decode(data)?)
                }
            })
        }
        TypedSubstateKey::AccessRulesModule(access_rules_offset) => {
            TypedSubstateValue::AccessRulesModule(match access_rules_offset {
                AccessRulesOffset::AccessRules => {
                    TypedAccessRulesModuleSubstateValue::MethodAccessRules(scrypto_decode(data)?)
                }
            })
        }
        TypedSubstateKey::RoyaltyModule(royalty_offset) => {
            TypedSubstateValue::RoyaltyModule(match royalty_offset {
                RoyaltyOffset::RoyaltyConfig => {
                    TypedRoyaltyModuleSubstateValue::ComponentRoyaltyConfig(scrypto_decode(data)?)
                }
                RoyaltyOffset::RoyaltyAccumulator => {
                    TypedRoyaltyModuleSubstateValue::ComponentRoyaltyAccumulator(scrypto_decode(
                        data,
                    )?)
                }
            })
        }
        TypedSubstateKey::MetadataModule(_) => TypedSubstateValue::MetadataModule(
            TypedMetadataModuleSubstateValue::Metadata(scrypto_decode(data)?),
        ),
        TypedSubstateKey::ObjectModule(object_substate_key) => TypedSubstateValue::ObjectModule(
            to_typed_object_substate_value(object_substate_key, data)?,
        ),
    };
    Ok(substate_value)
}

fn to_typed_object_substate_value(
    substate_key: &TypedObjectModuleSubstateKey,
    data: &[u8],
) -> Result<TypedObjectModuleSubstateValue, DecodeError> {
    let substate_value = match substate_key {
        TypedObjectModuleSubstateKey::Package(offset) => {
            TypedObjectModuleSubstateValue::Package(match offset {
                PackageOffset::Info => TypedPackageSubstateValue::Info(scrypto_decode(data)?),
                PackageOffset::CodeType => {
                    TypedPackageSubstateValue::CodeType(scrypto_decode(data)?)
                }
                PackageOffset::Code => TypedPackageSubstateValue::Code(scrypto_decode(data)?),
                PackageOffset::Royalty => TypedPackageSubstateValue::Royalty(scrypto_decode(data)?),
                PackageOffset::FunctionAccessRules => {
                    TypedPackageSubstateValue::FunctionAccessRules(scrypto_decode(data)?)
                }
            })
        }
        TypedObjectModuleSubstateKey::FungibleResource(offset) => {
            TypedObjectModuleSubstateValue::FungibleResource(match offset {
                ResourceManagerOffset::ResourceManager => {
                    TypedFungibleResourceManagerSubstateValue::ResourceManager(scrypto_decode(
                        data,
                    )?)
                }
            })
        }
        TypedObjectModuleSubstateKey::NonFungibleResource(offset) => {
            TypedObjectModuleSubstateValue::NonFungibleResource(match offset {
                ResourceManagerOffset::ResourceManager => {
                    TypedNonFungibleResourceManagerSubstateValue::ResourceManager(scrypto_decode(
                        data,
                    )?)
                }
            })
        }
        TypedObjectModuleSubstateKey::FungibleVault(offset) => {
            TypedObjectModuleSubstateValue::FungibleVault(match offset {
                FungibleVaultOffset::Divisibility => {
                    TypedFungibleVaultSubstateValue::Divisibility(scrypto_decode(data)?)
                }
                FungibleVaultOffset::LiquidFungible => {
                    TypedFungibleVaultSubstateValue::Balance(scrypto_decode(data)?)
                }
                // This shouldn't be persistable - so use a bizarre (but temporary!) placeholder error code here!
                FungibleVaultOffset::LockedFungible => Err(DecodeError::InvalidCustomValue)?,
            })
        }
        TypedObjectModuleSubstateKey::NonFungibleVault(offset) => {
            TypedObjectModuleSubstateValue::NonFungibleVault(match offset {
                NonFungibleVaultOffset::IdType => {
                    TypedNonFungibleVaultSubstateValue::IdType(scrypto_decode(data)?)
                }
                NonFungibleVaultOffset::LiquidNonFungible => {
                    TypedNonFungibleVaultSubstateValue::Balance(scrypto_decode(data)?)
                }
                // This shouldn't be persistable - so use a bizarre (but temporary!) placeholder error code here!
                NonFungibleVaultOffset::LockedNonFungible => Err(DecodeError::InvalidCustomValue)?,
            })
        }
        TypedObjectModuleSubstateKey::EpochManager(offset) => {
            TypedObjectModuleSubstateValue::EpochManager(match offset {
                EpochManagerOffset::EpochManager => {
                    TypedEpochManagerSubstateValue::EpochManager(scrypto_decode(data)?)
                }
                EpochManagerOffset::CurrentValidatorSet => {
                    TypedEpochManagerSubstateValue::CurrentValidatorSet(scrypto_decode(data)?)
                }
                EpochManagerOffset::RegisteredValidatorSet => {
                    TypedEpochManagerSubstateValue::RegisteredValidatorSet(scrypto_decode(data)?)
                }
            })
        }
        TypedObjectModuleSubstateKey::Clock(offset) => {
            TypedObjectModuleSubstateValue::Clock(match offset {
                ClockOffset::CurrentTimeRoundedToMinutes => {
                    TypedClockSubstateValue::CurrentTimeRoundedToMinutes(scrypto_decode(data)?)
                }
            })
        }
        TypedObjectModuleSubstateKey::Validator(offset) => {
            TypedObjectModuleSubstateValue::Validator(match offset {
                ValidatorOffset::Validator => {
                    TypedValidatorSubstateValue::Validator(scrypto_decode(data)?)
                }
            })
        }
        TypedObjectModuleSubstateKey::Account(offset) => {
            TypedObjectModuleSubstateValue::Account(match offset {
                AccountOffset::Account => TypedAccountSubstateValue::Account(scrypto_decode(data)?),
            })
        }
        TypedObjectModuleSubstateKey::AccessController(offset) => {
            TypedObjectModuleSubstateValue::AccessController(match offset {
                AccessControllerOffset::AccessController => {
                    TypedAccessControllerSubstateValue::AccessController(scrypto_decode(data)?)
                }
            })
        }
        TypedObjectModuleSubstateKey::GenericScryptoComponent(offset) => {
            TypedObjectModuleSubstateValue::GenericScryptoComponent(match offset {
                ComponentOffset::State0 => {
                    GenericScryptoComponentSubstateValue::State(GenericScryptoSborPayload {
                        data: data.to_vec(),
                    })
                }
            })
        }
        TypedObjectModuleSubstateKey::GenericKeyValueStore(_) => {
            TypedObjectModuleSubstateValue::GenericKeyValueStore(GenericScryptoSborPayload {
                data: data.to_vec(),
            })
        }
        TypedObjectModuleSubstateKey::GenericIndex(_) => {
            TypedObjectModuleSubstateValue::GenericIndex(GenericScryptoSborPayload {
                data: data.to_vec(),
            })
        }
        TypedObjectModuleSubstateKey::GenericSortedU16Index(_) => {
            TypedObjectModuleSubstateValue::GenericSortedU16Index(GenericScryptoSborPayload {
                data: data.to_vec(),
            })
        }
    };
    Ok(substate_value)
}
