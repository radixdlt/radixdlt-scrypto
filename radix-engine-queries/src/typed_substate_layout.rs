use radix_engine::types::*;

// Import and re-export these types so they are available easily with a single import
pub use radix_engine::blueprints::access_controller::*;
pub use radix_engine::blueprints::account::{AccountBlueprint, AccountError, AccountNativePackage};
use radix_engine::blueprints::account::{AccountTypedSubstateKey, AccountTypedSubstateValue};
pub use radix_engine::blueprints::consensus_manager::*;
pub use radix_engine::blueprints::package::*;
pub use radix_engine::blueprints::pool::multi_resource_pool;
use radix_engine::blueprints::pool::multi_resource_pool::{
    MultiResourcePoolTypedSubstateKey, MultiResourcePoolTypedSubstateValue,
};
pub use radix_engine::blueprints::pool::one_resource_pool;
use radix_engine::blueprints::pool::one_resource_pool::{
    OneResourcePoolTypedSubstateKey, OneResourcePoolTypedSubstateValue,
};
pub use radix_engine::blueprints::pool::two_resource_pool;
use radix_engine::blueprints::pool::two_resource_pool::{
    TwoResourcePoolTypedSubstateKey, TwoResourcePoolTypedSubstateValue,
};
pub use radix_engine::blueprints::resource::*;
pub use radix_engine::blueprints::transaction_tracker::*;
pub use radix_engine::system::node_modules::metadata::*;
pub use radix_engine::system::node_modules::role_assignment::*;
pub use radix_engine::system::node_modules::royalty::*;
pub use radix_engine::system::node_modules::type_info::*;
use radix_engine::system::system::FieldSubstate;
pub use radix_engine::system::system::KeyValueEntrySubstate;
pub use radix_engine_interface::api::node_modules::royalty::*;
use transaction::prelude::IntentHash;

//=========================================================================
// Please update REP-60 after updating types/configs defined in this file!
//
// The below defines well-known substate types which are used in the
// Core API of the node.
//
// Specifically:
// * Every (EntityType, PartitionNumber, SubstateKey) should be mappable into a `TypedSubstateKey`
// * Every (&TypedSubstateKey, Data) should be mappable into a `TypedSubstateValue`
//
// Please keep them these in-line with the well-known objects, and please don't
// remove these without talking to the Network team.
//=========================================================================

//=========================================================================
// A partition can be in one of four types:
//
// - Field
//   => Has key: TupleKey(u8) also known as an offset
//   => No iteration exposed to engine
//   => Is versioned / locked substate-by-substate
// - KeyValue ("ConcurrentMap")
//   => Has key: MapKey(Vec<u8>)
//   => No iteration exposed to engine
//   => Is versioned / locked substate-by-substate
// - Index
//   => Has key: MapKey(Vec<u8>)
//   => Iteration exposed to engine via the MapKey's database key (ie hash of the key)
//   => Is versioned / locked in its entirety
// - SortedU16Index
//   => Has key: SortedKey([u8; 2], Vec<u8>)
//   => Iteration exposed to engine via the user-controlled U16 prefix and then the MapKey's database key (ie hash of the key)
//   => Is versioned / locked in its entirety
//
// But in this file, we just handle explicitly supported/possible combinations of things.
//
// An entirely generic capturing of a substate key for a given node partition would look something like this:
//
// pub enum GenericModuleSubstateKey {
//    Field(TupleKey),
//    KeyValue(MapKey),
//    Index(MapKey),
//    SortedU16Index(SortedKey),
// }
//=========================================================================

#[derive(Debug, Clone)]
pub enum TypedSubstateKey {
    TypeInfo(TypedTypeInfoSubstateKey),
    Schema(TypedSchemaSubstateKey),
    RoleAssignmentModule(TypedRoleAssignmentSubstateKey),
    RoyaltyModule(TypedRoyaltyModuleSubstateKey),
    MetadataModule(TypedMetadataModuleSubstateKey),
    MainModule(TypedMainModuleSubstateKey),
}

impl TypedSubstateKey {
    /// This method should be used to filter out substates which we don't want to map in the Core API.
    /// (See `radix-engine-tests/tests/bootstrap.rs` for an example of how it should be used)
    /// Just a work around for now to filter out "transient" substates we shouldn't really be storing.
    pub fn value_is_mappable(&self) -> bool {
        match self {
            TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleVault(
                NonFungibleVaultTypedSubstateKey::Field(NonFungibleVaultField::LockedResource),
            )) => false,
            TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::FungibleVault(
                FungibleVaultTypedSubstateKey::Field(FungibleVaultField::LockedBalance),
            )) => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypedTypeInfoSubstateKey {
    TypeInfoField(TypeInfoField),
}

#[derive(Debug, Clone)]
pub enum TypedSchemaSubstateKey {
    SchemaKey(SchemaHash),
}

#[derive(Debug, Clone)]
pub enum TypedRoleAssignmentSubstateKey {
    RoleAssignmentField(RoleAssignmentField),
    Rule(ModuleRoleKey),
}

#[derive(Debug, Clone)]
pub enum TypedRoyaltyModuleSubstateKey {
    RoyaltyField(RoyaltyField),
    /// The key is the method ident
    RoyaltyMethodRoyaltyEntryKey(String),
}

#[derive(Debug, Clone)]
pub enum TypedMetadataModuleSubstateKey {
    MetadataEntryKey(String),
}

/// Doesn't include non-object modules, nor transient nodes.
#[derive(Debug, Clone)]
pub enum TypedMainModuleSubstateKey {
    // Objects - Native
    Package(PackageTypedSubstateKey),
    FungibleResourceManager(FungibleResourceManagerTypedSubstateKey),
    NonFungibleResourceManager(NonFungibleResourceManagerTypedSubstateKey),
    FungibleVault(FungibleVaultTypedSubstateKey),
    NonFungibleVault(NonFungibleVaultTypedSubstateKey),
    ConsensusManagerField(ConsensusManagerField),
    ConsensusManagerRegisteredValidatorsByStakeIndexKey(ValidatorByStakeKey),
    ValidatorField(ValidatorField),
    AccessControllerField(AccessControllerField),
    Account(AccountTypedSubstateKey),
    OneResourcePool(OneResourcePoolTypedSubstateKey),
    TwoResourcePool(TwoResourcePoolTypedSubstateKey),
    MultiResourcePool(MultiResourcePoolTypedSubstateKey),
    TransactionTrackerField(TransactionTrackerField),
    TransactionTrackerCollectionEntry(IntentHash),
    // Objects - Generic Scrypto Components
    GenericScryptoComponentField(ComponentField),
    // KVStores - Generic KV Stores
    GenericKeyValueStoreKey(MapKey),
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct ValidatorByStakeKey {
    pub divided_stake: u16,
    pub validator_address: ComponentAddress,
}

impl TryFrom<SortedKey> for ValidatorByStakeKey {
    type Error = DecodeError;

    fn try_from(value: SortedKey) -> Result<Self, Self::Error> {
        // See to_sorted_key in validator.rs
        Ok(Self {
            divided_stake: u16::MAX - u16::from_be_bytes(value.0),
            validator_address: scrypto_decode(&value.1)?,
        })
    }
}

fn error(descriptor: &'static str) -> String {
    format!("Could not convert {} to TypedSubstateKey", descriptor)
}

pub fn to_typed_substate_key(
    entity_type: EntityType,
    partition_num: PartitionNumber,
    substate_key: &SubstateKey,
) -> Result<TypedSubstateKey, String> {
    let substate_type = match partition_num {
        TYPE_INFO_FIELD_PARTITION => {
            TypedSubstateKey::TypeInfo(TypedTypeInfoSubstateKey::TypeInfoField(
                TypeInfoField::try_from(substate_key).map_err(|_| error("TypeInfoField"))?,
            ))
        }
        SCHEMAS_PARTITION => {
            let key = substate_key.for_map().ok_or_else(|| error("Schema key"))?;
            TypedSubstateKey::Schema(TypedSchemaSubstateKey::SchemaKey(
                scrypto_decode(key).map_err(|_| error("Schema key"))?,
            ))
        }
        METADATA_BASE_PARTITION => {
            TypedSubstateKey::MetadataModule(TypedMetadataModuleSubstateKey::MetadataEntryKey(
                scrypto_decode(
                    substate_key
                        .for_map()
                        .ok_or_else(|| error("Metadata key"))?,
                )
                .map_err(|_| error("string Metadata key"))?,
            ))
        }
        ROYALTY_FIELDS_PARTITION => {
            TypedSubstateKey::RoyaltyModule(TypedRoyaltyModuleSubstateKey::RoyaltyField(
                RoyaltyField::try_from(substate_key).map_err(|_| error("RoyaltyField"))?,
            ))
        }
        ROYALTY_CONFIG_PARTITION => TypedSubstateKey::RoyaltyModule(
            TypedRoyaltyModuleSubstateKey::RoyaltyMethodRoyaltyEntryKey(
                scrypto_decode(
                    substate_key
                        .for_map()
                        .ok_or_else(|| error("RoyaltyConfigEntryFnIdent key"))?,
                )
                .map_err(|_| error("string RoyaltyConfigEntryFnIdent key"))?,
            ),
        ),
        ROLE_ASSIGNMENT_FIELDS_PARTITION => TypedSubstateKey::RoleAssignmentModule(
            TypedRoleAssignmentSubstateKey::RoleAssignmentField(
                RoleAssignmentField::try_from(substate_key)
                    .map_err(|_| error("RoleAssignmentField"))?,
            ),
        ),
        ROLE_ASSIGNMENT_ROLE_DEF_PARTITION => {
            let key = substate_key
                .for_map()
                .ok_or_else(|| error("Access Rules key"))?;
            TypedSubstateKey::RoleAssignmentModule(TypedRoleAssignmentSubstateKey::Rule(
                scrypto_decode(&key).map_err(|_| error("Access Rules key"))?,
            ))
        }
        partition_num @ _ if partition_num >= MAIN_BASE_PARTITION => {
            TypedSubstateKey::MainModule(to_typed_object_module_substate_key(
                entity_type,
                partition_num.0 - MAIN_BASE_PARTITION.0,
                substate_key,
            )?)
        }
        _ => return Err(format!("Unknown partition {:?}", partition_num)),
    };
    Ok(substate_type)
}

pub fn to_typed_object_module_substate_key(
    entity_type: EntityType,
    partition_offset: u8,
    substate_key: &SubstateKey,
) -> Result<TypedMainModuleSubstateKey, String> {
    return to_typed_object_substate_key_internal(
        entity_type,
        PartitionOffset(partition_offset),
        substate_key,
    )
    .map_err(|_| {
        format!(
            "Could not convert {:?} (partition offset {}) {:?} key to TypedObjectSubstateKey",
            entity_type, partition_offset, substate_key
        )
    });
}

fn to_typed_object_substate_key_internal(
    entity_type: EntityType,
    partition_offset: PartitionOffset,
    substate_key: &SubstateKey,
) -> Result<TypedMainModuleSubstateKey, ()> {
    let substate_type = match entity_type {
        EntityType::InternalGenericComponent | EntityType::GlobalGenericComponent => {
            TypedMainModuleSubstateKey::GenericScryptoComponentField(ComponentField::try_from(
                substate_key,
            )?)
        }
        EntityType::GlobalPackage => TypedMainModuleSubstateKey::Package(
            PackageTypedSubstateKey::for_key_at_partition_offset(partition_offset, substate_key)?,
        ),
        EntityType::GlobalFungibleResourceManager => {
            TypedMainModuleSubstateKey::FungibleResourceManager(
                FungibleResourceManagerTypedSubstateKey::for_key_at_partition_offset(
                    partition_offset,
                    substate_key,
                )?,
            )
        }
        EntityType::GlobalNonFungibleResourceManager => {
            TypedMainModuleSubstateKey::NonFungibleResourceManager(
                NonFungibleResourceManagerTypedSubstateKey::for_key_at_partition_offset(
                    partition_offset,
                    substate_key,
                )?,
            )
        }
        EntityType::GlobalConsensusManager => {
            let partition_offset = ConsensusManagerPartitionOffset::try_from(partition_offset)?;
            match partition_offset {
                ConsensusManagerPartitionOffset::ConsensusManager => {
                    TypedMainModuleSubstateKey::ConsensusManagerField(
                        ConsensusManagerField::try_from(substate_key)?,
                    )
                }
                ConsensusManagerPartitionOffset::RegisteredValidatorsByStakeIndex => {
                    let key = substate_key.for_sorted().ok_or(())?;
                    TypedMainModuleSubstateKey::ConsensusManagerRegisteredValidatorsByStakeIndexKey(
                        key.clone().try_into().map_err(|_| ())?,
                    )
                }
            }
        }
        EntityType::GlobalValidator => {
            TypedMainModuleSubstateKey::ValidatorField(ValidatorField::try_from(substate_key)?)
        }
        EntityType::GlobalAccessController => TypedMainModuleSubstateKey::AccessControllerField(
            AccessControllerField::try_from(substate_key)?,
        ),
        EntityType::GlobalVirtualSecp256k1Account
        | EntityType::GlobalVirtualEd25519Account
        | EntityType::InternalAccount
        | EntityType::GlobalAccount => {
            TypedMainModuleSubstateKey::Account(AccountTypedSubstateKey::for_key_in_partition(
                &AccountPartitionOffset::try_from(partition_offset)?,
                substate_key,
            )?)
        }
        EntityType::GlobalVirtualSecp256k1Identity
        | EntityType::GlobalVirtualEd25519Identity
        | EntityType::GlobalIdentity => Err(())?, // Identity doesn't have any substates
        EntityType::InternalFungibleVault => TypedMainModuleSubstateKey::FungibleVault(
            FungibleVaultTypedSubstateKey::for_key_at_partition_offset(
                partition_offset,
                substate_key,
            )?,
        ),
        EntityType::InternalNonFungibleVault => TypedMainModuleSubstateKey::NonFungibleVault(
            NonFungibleVaultTypedSubstateKey::for_key_at_partition_offset(
                partition_offset,
                substate_key,
            )?,
        ),
        EntityType::GlobalOneResourcePool => TypedMainModuleSubstateKey::OneResourcePool(
            OneResourcePoolTypedSubstateKey::for_key_in_partition(
                &OneResourcePoolPartitionOffset::try_from(partition_offset)?,
                substate_key,
            )?,
        ),
        EntityType::GlobalTwoResourcePool => TypedMainModuleSubstateKey::TwoResourcePool(
            TwoResourcePoolTypedSubstateKey::for_key_in_partition(
                &TwoResourcePoolPartitionOffset::try_from(partition_offset)?,
                substate_key,
            )?,
        ),
        EntityType::GlobalMultiResourcePool => TypedMainModuleSubstateKey::MultiResourcePool(
            MultiResourcePoolTypedSubstateKey::for_key_in_partition(
                &MultiResourcePoolPartitionOffset::try_from(partition_offset)?,
                substate_key,
            )?,
        ),
        EntityType::GlobalTransactionTracker => {
            if partition_offset == PartitionOffset(0) {
                TypedMainModuleSubstateKey::TransactionTrackerField(
                    TransactionTrackerField::try_from(substate_key)?,
                )
            } else {
                if let Some(key) = substate_key.for_map() {
                    TypedMainModuleSubstateKey::TransactionTrackerCollectionEntry(
                        IntentHash::from_hash(scrypto_decode(key).map_err(|_| ())?),
                    )
                } else {
                    return Err(());
                }
            }
        }
        // These seem to be spread between Object and Virtualized SysModules
        EntityType::InternalKeyValueStore => {
            let key = substate_key.for_map().ok_or(())?;
            TypedMainModuleSubstateKey::GenericKeyValueStoreKey(key.clone())
        }
    };
    Ok(substate_type)
}

#[derive(Debug)]
pub enum TypedSubstateValue {
    TypeInfoModule(TypedTypeInfoModuleSubstateValue),
    Schema(KeyValueEntrySubstate<ScryptoSchema>),
    RoleAssignmentModule(TypedRoleAssignmentModuleSubstateValue),
    RoyaltyModule(TypedRoyaltyModuleSubstateValue),
    MetadataModule(TypedMetadataModuleSubstateValue),
    MainModule(TypedMainModuleSubstateValue),
}

#[derive(Debug)]
pub enum TypedTypeInfoModuleSubstateValue {
    TypeInfo(TypeInfoSubstate),
}

#[derive(Debug)]
pub enum TypedRoleAssignmentModuleSubstateValue {
    OwnerRole(FieldSubstate<OwnerRoleSubstate>),
    Rule(KeyValueEntrySubstate<AccessRule>),
}

#[derive(Debug)]
pub enum TypedRoyaltyModuleSubstateValue {
    ComponentRoyalty(FieldSubstate<ComponentRoyaltySubstate>),
    ComponentMethodRoyalty(ComponentMethodRoyaltySubstate),
}

#[derive(Debug)]
pub enum TypedMetadataModuleSubstateValue {
    MetadataEntry(MetadataEntrySubstate),
}

/// Contains all the main module substate values, by each known partition layout
#[derive(Debug)]
pub enum TypedMainModuleSubstateValue {
    // Objects
    Package(PackageTypedSubstateValue),
    FungibleResourceManager(FungibleResourceManagerTypedSubstateValue),
    NonFungibleResourceManager(NonFungibleResourceManagerTypedSubstateValue),
    FungibleVault(FungibleVaultTypedSubstateValue),
    NonFungibleVault(NonFungibleVaultTypedSubstateValue),
    ConsensusManagerField(TypedConsensusManagerFieldValue),
    ConsensusManagerRegisteredValidatorsByStakeIndexEntry(Validator),
    Validator(TypedValidatorFieldValue),
    AccessController(TypedAccessControllerFieldValue),
    Account(AccountTypedSubstateValue),
    OneResourcePool(OneResourcePoolTypedSubstateValue),
    TwoResourcePool(TwoResourcePoolTypedSubstateValue),
    MultiResourcePool(MultiResourcePoolTypedSubstateValue),
    TransactionTracker(TypedTransactionTrackerFieldValue),
    TransactionTrackerCollectionEntry(KeyValueEntrySubstate<TransactionStatusSubstateContents>),
    // Generic Scrypto Components and KV Stores
    GenericScryptoComponent(GenericScryptoComponentFieldValue),
    GenericKeyValueStoreEntry(KeyValueEntrySubstate<ScryptoOwnedRawValue>),
}

#[derive(Debug)]
pub enum TypedConsensusManagerFieldValue {
    Config(FieldSubstate<ConsensusManagerConfigSubstate>),
    ConsensusManager(FieldSubstate<ConsensusManagerSubstate>),
    ValidatorRewards(FieldSubstate<ValidatorRewardsSubstate>),
    CurrentValidatorSet(FieldSubstate<CurrentValidatorSetSubstate>),
    CurrentProposalStatistic(FieldSubstate<CurrentProposalStatisticSubstate>),
    CurrentTimeRoundedToMinutes(FieldSubstate<ProposerMinuteTimestampSubstate>),
    CurrentTime(FieldSubstate<ProposerMilliTimestampSubstate>),
}

#[derive(Debug)]
pub enum TypedValidatorFieldValue {
    Validator(FieldSubstate<ValidatorSubstate>),
    ProtocolUpdateReadinessSignal(FieldSubstate<ValidatorProtocolUpdateReadinessSignalSubstate>),
}

#[derive(Debug)]
pub enum TypedAccessControllerFieldValue {
    AccessController(FieldSubstate<AccessControllerSubstate>),
}

#[derive(Debug)]
pub enum GenericScryptoComponentFieldValue {
    State(FieldSubstate<ScryptoValue>),
}

#[derive(Debug)]
pub enum TypedTransactionTrackerFieldValue {
    TransactionTracker(FieldSubstate<TransactionTrackerSubstate>),
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
        TypedSubstateKey::TypeInfo(type_info_key) => {
            TypedSubstateValue::TypeInfoModule(match type_info_key {
                TypedTypeInfoSubstateKey::TypeInfoField(TypeInfoField::TypeInfo) => {
                    TypedTypeInfoModuleSubstateValue::TypeInfo(scrypto_decode(data)?)
                }
            })
        }
        TypedSubstateKey::Schema(_) => TypedSubstateValue::Schema(scrypto_decode(data)?),
        TypedSubstateKey::RoleAssignmentModule(role_assignment_key) => match role_assignment_key {
            TypedRoleAssignmentSubstateKey::RoleAssignmentField(role_assignment_field_offset) => {
                match role_assignment_field_offset {
                    RoleAssignmentField::OwnerRole => TypedSubstateValue::RoleAssignmentModule(
                        TypedRoleAssignmentModuleSubstateValue::OwnerRole(scrypto_decode(data)?),
                    ),
                }
            }
            TypedRoleAssignmentSubstateKey::Rule(_) => TypedSubstateValue::RoleAssignmentModule(
                TypedRoleAssignmentModuleSubstateValue::Rule(scrypto_decode(data)?),
            ),
        },
        TypedSubstateKey::RoyaltyModule(royalty_module_key) => {
            TypedSubstateValue::RoyaltyModule(match royalty_module_key {
                TypedRoyaltyModuleSubstateKey::RoyaltyField(RoyaltyField::RoyaltyAccumulator) => {
                    TypedRoyaltyModuleSubstateValue::ComponentRoyalty(scrypto_decode(data)?)
                }
                TypedRoyaltyModuleSubstateKey::RoyaltyMethodRoyaltyEntryKey(_) => {
                    TypedRoyaltyModuleSubstateValue::ComponentMethodRoyalty(scrypto_decode(data)?)
                }
            })
        }
        TypedSubstateKey::MetadataModule(metadata_module_key) => {
            TypedSubstateValue::MetadataModule(match metadata_module_key {
                TypedMetadataModuleSubstateKey::MetadataEntryKey(_) => {
                    TypedMetadataModuleSubstateValue::MetadataEntry(scrypto_decode(data)?)
                }
            })
        }
        TypedSubstateKey::MainModule(object_substate_key) => TypedSubstateValue::MainModule(
            to_typed_object_substate_value(object_substate_key, data)?,
        ),
    };
    Ok(substate_value)
}

fn to_typed_object_substate_value(
    substate_key: &TypedMainModuleSubstateKey,
    data: &[u8],
) -> Result<TypedMainModuleSubstateValue, DecodeError> {
    let substate_value = match substate_key {
        TypedMainModuleSubstateKey::Package(key) => TypedMainModuleSubstateValue::Package(
            PackageTypedSubstateValue::from_key_and_data(key, data)?,
        ),
        TypedMainModuleSubstateKey::FungibleResourceManager(key) => {
            TypedMainModuleSubstateValue::FungibleResourceManager(
                FungibleResourceManagerTypedSubstateValue::from_key_and_data(key, data)?,
            )
        }
        TypedMainModuleSubstateKey::NonFungibleResourceManager(key) => {
            TypedMainModuleSubstateValue::NonFungibleResourceManager(
                NonFungibleResourceManagerTypedSubstateValue::from_key_and_data(key, data)?,
            )
        }
        TypedMainModuleSubstateKey::FungibleVault(key) => {
            TypedMainModuleSubstateValue::FungibleVault(
                FungibleVaultTypedSubstateValue::from_key_and_data(key, data)?,
            )
        }
        TypedMainModuleSubstateKey::NonFungibleVault(key) => {
            TypedMainModuleSubstateValue::NonFungibleVault(
                NonFungibleVaultTypedSubstateValue::from_key_and_data(key, data)?,
            )
        }
        TypedMainModuleSubstateKey::ConsensusManagerField(offset) => {
            TypedMainModuleSubstateValue::ConsensusManagerField(match offset {
                ConsensusManagerField::Config => {
                    TypedConsensusManagerFieldValue::Config(scrypto_decode(data)?)
                }
                ConsensusManagerField::ConsensusManager => {
                    TypedConsensusManagerFieldValue::ConsensusManager(scrypto_decode(data)?)
                }
                ConsensusManagerField::ValidatorRewards => {
                    TypedConsensusManagerFieldValue::ValidatorRewards(scrypto_decode(data)?)
                }
                ConsensusManagerField::CurrentValidatorSet => {
                    TypedConsensusManagerFieldValue::CurrentValidatorSet(scrypto_decode(data)?)
                }
                ConsensusManagerField::CurrentProposalStatistic => {
                    TypedConsensusManagerFieldValue::CurrentProposalStatistic(scrypto_decode(data)?)
                }
                ConsensusManagerField::CurrentTimeRoundedToMinutes => {
                    TypedConsensusManagerFieldValue::CurrentTimeRoundedToMinutes(scrypto_decode(
                        data,
                    )?)
                }
                ConsensusManagerField::CurrentTime => {
                    TypedConsensusManagerFieldValue::CurrentTime(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::ConsensusManagerRegisteredValidatorsByStakeIndexKey(_) => {
            TypedMainModuleSubstateValue::ConsensusManagerRegisteredValidatorsByStakeIndexEntry(
                scrypto_decode(data)?,
            )
        }
        TypedMainModuleSubstateKey::ValidatorField(offset) => {
            TypedMainModuleSubstateValue::Validator(match offset {
                ValidatorField::Validator => {
                    TypedValidatorFieldValue::Validator(scrypto_decode(data)?)
                }
                ValidatorField::ProtocolUpdateReadinessSignal => {
                    TypedValidatorFieldValue::ProtocolUpdateReadinessSignal(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::Account(key) => TypedMainModuleSubstateValue::Account(
            AccountTypedSubstateValue::from_key_and_data(key, data)?,
        ),
        TypedMainModuleSubstateKey::AccessControllerField(offset) => {
            TypedMainModuleSubstateValue::AccessController(match offset {
                AccessControllerField::AccessController => {
                    TypedAccessControllerFieldValue::AccessController(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::GenericScryptoComponentField(offset) => {
            TypedMainModuleSubstateValue::GenericScryptoComponent(match offset {
                ComponentField::State0 => {
                    GenericScryptoComponentFieldValue::State(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::GenericKeyValueStoreKey(_) => {
            TypedMainModuleSubstateValue::GenericKeyValueStoreEntry(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::OneResourcePool(key) => {
            TypedMainModuleSubstateValue::OneResourcePool(
                OneResourcePoolTypedSubstateValue::from_key_and_data(key, data)?,
            )
        }
        TypedMainModuleSubstateKey::TwoResourcePool(key) => {
            TypedMainModuleSubstateValue::TwoResourcePool(
                TwoResourcePoolTypedSubstateValue::from_key_and_data(key, data)?,
            )
        }
        TypedMainModuleSubstateKey::MultiResourcePool(key) => {
            TypedMainModuleSubstateValue::MultiResourcePool(
                MultiResourcePoolTypedSubstateValue::from_key_and_data(key, data)?,
            )
        }

        TypedMainModuleSubstateKey::TransactionTrackerField(offset) => {
            TypedMainModuleSubstateValue::TransactionTracker(match offset {
                TransactionTrackerField::TransactionTracker => {
                    TypedTransactionTrackerFieldValue::TransactionTracker(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::TransactionTrackerCollectionEntry(_) => {
            TypedMainModuleSubstateValue::TransactionTrackerCollectionEntry(scrypto_decode(data)?)
        }
    };
    Ok(substate_value)
}
