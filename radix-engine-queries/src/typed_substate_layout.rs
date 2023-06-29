use radix_engine::types::*;

// Import and re-export these types so they are available easily with a single import
pub use radix_engine::blueprints::access_controller::*;
pub use radix_engine::blueprints::account::*;
pub use radix_engine::blueprints::consensus_manager::*;
pub use radix_engine::blueprints::package::*;
pub use radix_engine::blueprints::pool::multi_resource_pool;
pub use radix_engine::blueprints::pool::one_resource_pool;
pub use radix_engine::blueprints::pool::two_resource_pool;
pub use radix_engine::blueprints::resource::*;
pub use radix_engine::blueprints::transaction_tracker::*;
pub use radix_engine::system::node_modules::access_rules::*;
pub use radix_engine::system::node_modules::metadata::*;
pub use radix_engine::system::node_modules::royalty::*;
pub use radix_engine::system::node_modules::type_info::*;
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
//   => Has key: SortedU16Key(U16, Vec<u8>)
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
//    SortedU16Index(SortedU16Key),
// }
//=========================================================================

#[derive(Debug, Clone)]
pub enum TypedSubstateKey {
    TypeInfoModule(TypedTypeInfoModuleSubstateKey),
    AccessRulesModule(TypedAccessRulesSubstateKey),
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
            TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleVaultField(
                NonFungibleVaultField::LockedNonFungible,
            )) => false,
            TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::FungibleVaultField(
                FungibleVaultField::LockedFungible,
            )) => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypedTypeInfoModuleSubstateKey {
    TypeInfoField(TypeInfoField),
}

#[derive(Debug, Clone)]
pub enum TypedAccessRulesSubstateKey {
    AccessRulesField(AccessRulesField),
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
    // Objects
    PackageField(PackageField),
    PackageBlueprintKey(BlueprintVersionKey),
    PackageBlueprintDependenciesKey(BlueprintVersionKey),
    PackageSchemaKey(Hash),
    PackageRoyaltyKey(BlueprintVersionKey),
    PackageAuthTemplateKey(BlueprintVersionKey),
    PackageVmTypeKey(Hash),
    PackageOriginalCodeKey(Hash),
    PackageInstrumentedCodeKey(Hash),
    FungibleResourceField(FungibleResourceManagerField),
    NonFungibleResourceField(NonFungibleResourceManagerField),
    NonFungibleResourceData(NonFungibleLocalId),
    FungibleVaultField(FungibleVaultField),
    NonFungibleVaultField(NonFungibleVaultField),
    NonFungibleVaultContentsIndexKey(NonFungibleLocalId),
    ConsensusManagerField(ConsensusManagerField),
    ConsensusManagerRegisteredValidatorsByStakeIndexKey(ValidatorByStakeKey),
    ValidatorField(ValidatorField),
    AccessControllerField(AccessControllerField),
    AccountField(AccountField),
    AccountVaultIndexKey(ResourceAddress),
    AccountResourceDepositRuleIndexKey(ResourceAddress),
    OneResourcePoolField(OneResourcePoolField),
    TwoResourcePoolField(TwoResourcePoolField),
    MultiResourcePoolField(MultiResourcePoolField),
    TransactionTrackerField(TransactionTrackerField),
    TransactionTrackerCollectionEntry(IntentHash),
    // Generic Scrypto Components
    GenericScryptoComponentField(ComponentField),
    // Substates for Generic KV Stores
    GenericKeyValueStoreKey(MapKey),
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct ValidatorByStakeKey {
    pub divided_stake: u16,
    pub validator_address: ComponentAddress,
}

impl TryFrom<SortedU16Key> for ValidatorByStakeKey {
    type Error = DecodeError;

    fn try_from(value: SortedU16Key) -> Result<Self, Self::Error> {
        // See to_sorted_key in validator.rs
        Ok(Self {
            divided_stake: u16::MAX - value.0,
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
            TypedSubstateKey::TypeInfoModule(TypedTypeInfoModuleSubstateKey::TypeInfoField(
                TypeInfoField::try_from(substate_key).map_err(|_| error("TypeInfoField"))?,
            ))
        }
        METADATA_KV_STORE_PARTITION => {
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
        ACCESS_RULES_FIELDS_PARTITION => {
            TypedSubstateKey::AccessRulesModule(TypedAccessRulesSubstateKey::AccessRulesField(
                AccessRulesField::try_from(substate_key).map_err(|_| error("AccessRulesField"))?,
            ))
        }
        ACCESS_RULES_ROLE_DEF_PARTITION => {
            let key = substate_key
                .for_map()
                .ok_or_else(|| error("Access Rules key"))?;
            TypedSubstateKey::AccessRulesModule(TypedAccessRulesSubstateKey::Rule(
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
    return to_typed_object_substate_key_internal(entity_type, partition_offset, substate_key)
        .map_err(|_| {
            format!(
                "Could not convert {:?} {:?} key to TypedObjectSubstateKey",
                entity_type, substate_key
            )
        });
}

fn to_typed_object_substate_key_internal(
    entity_type: EntityType,
    partition_offset: u8,
    substate_key: &SubstateKey,
) -> Result<TypedMainModuleSubstateKey, ()> {
    let substate_type = match entity_type {
        EntityType::InternalGenericComponent | EntityType::GlobalGenericComponent => {
            TypedMainModuleSubstateKey::GenericScryptoComponentField(ComponentField::try_from(
                substate_key,
            )?)
        }
        EntityType::GlobalPackage => {
            let partition_offset = PackagePartitionOffset::try_from(partition_offset)?;
            match partition_offset {
                PackagePartitionOffset::Fields => {
                    TypedMainModuleSubstateKey::PackageField(PackageField::try_from(substate_key)?)
                }
                PackagePartitionOffset::Blueprints => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedMainModuleSubstateKey::PackageBlueprintKey(
                        scrypto_decode(&key).map_err(|_| ())?,
                    )
                }
                PackagePartitionOffset::BlueprintDependencies => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedMainModuleSubstateKey::PackageBlueprintDependenciesKey(
                        scrypto_decode(&key).map_err(|_| ())?,
                    )
                }
                PackagePartitionOffset::Schemas => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedMainModuleSubstateKey::PackageSchemaKey(
                        scrypto_decode(&key).map_err(|_| ())?,
                    )
                }
                PackagePartitionOffset::RoyaltyConfig => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedMainModuleSubstateKey::PackageRoyaltyKey(
                        scrypto_decode(&key).map_err(|_| ())?,
                    )
                }
                PackagePartitionOffset::AuthConfig => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedMainModuleSubstateKey::PackageAuthTemplateKey(
                        scrypto_decode(&key).map_err(|_| ())?,
                    )
                }
                PackagePartitionOffset::VmType => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedMainModuleSubstateKey::PackageVmTypeKey(
                        scrypto_decode(&key).map_err(|_| ())?,
                    )
                }
                PackagePartitionOffset::OriginalCode => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedMainModuleSubstateKey::PackageOriginalCodeKey(
                        scrypto_decode(&key).map_err(|_| ())?,
                    )
                }
                PackagePartitionOffset::InstrumentedCode => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedMainModuleSubstateKey::PackageInstrumentedCodeKey(
                        scrypto_decode(&key).map_err(|_| ())?,
                    )
                }
            }
        }
        EntityType::GlobalFungibleResourceManager => {
            TypedMainModuleSubstateKey::FungibleResourceField(
                FungibleResourceManagerField::try_from(substate_key)?,
            )
        }
        EntityType::GlobalNonFungibleResourceManager => {
            let partition_offset =
                NonFungibleResourceManagerPartitionOffset::try_from(partition_offset)?;
            match partition_offset {
                NonFungibleResourceManagerPartitionOffset::ResourceManager => {
                    TypedMainModuleSubstateKey::NonFungibleResourceField(
                        NonFungibleResourceManagerField::try_from(substate_key)?,
                    )
                }
                NonFungibleResourceManagerPartitionOffset::NonFungibleData => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedMainModuleSubstateKey::NonFungibleResourceData(
                        scrypto_decode(&key).map_err(|_| ())?,
                    )
                }
            }
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
            let partition_offset = AccountPartitionOffset::try_from(partition_offset)?;

            match partition_offset {
                AccountPartitionOffset::AccountVaultsByResourceAddress => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedMainModuleSubstateKey::AccountVaultIndexKey(
                        scrypto_decode(&key).map_err(|_| ())?,
                    )
                }
                AccountPartitionOffset::AccountResourceDepositRuleByAddress => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedMainModuleSubstateKey::AccountResourceDepositRuleIndexKey(
                        scrypto_decode(&key).map_err(|_| ())?,
                    )
                }
                AccountPartitionOffset::Account => {
                    TypedMainModuleSubstateKey::AccountField(AccountField::try_from(substate_key)?)
                }
            }
        }
        EntityType::GlobalVirtualSecp256k1Identity
        | EntityType::GlobalVirtualEd25519Identity
        | EntityType::GlobalIdentity => Err(())?, // Identity doesn't have any substates
        EntityType::InternalFungibleVault => TypedMainModuleSubstateKey::FungibleVaultField(
            FungibleVaultField::try_from(substate_key)?,
        ),
        EntityType::InternalNonFungibleVault => {
            let partition_offset = NonFungibleVaultPartitionOffset::try_from(partition_offset)?;

            match partition_offset {
                NonFungibleVaultPartitionOffset::Balance => {
                    TypedMainModuleSubstateKey::NonFungibleVaultField(
                        NonFungibleVaultField::try_from(substate_key)?,
                    )
                }
                NonFungibleVaultPartitionOffset::NonFungibles => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedMainModuleSubstateKey::NonFungibleVaultContentsIndexKey(
                        scrypto_decode(&key).map_err(|_| ())?,
                    )
                }
            }
        }
        EntityType::GlobalOneResourcePool => TypedMainModuleSubstateKey::OneResourcePoolField(
            OneResourcePoolField::try_from(substate_key)?,
        ),
        EntityType::GlobalTwoResourcePool => TypedMainModuleSubstateKey::TwoResourcePoolField(
            TwoResourcePoolField::try_from(substate_key)?,
        ),
        EntityType::GlobalMultiResourcePool => TypedMainModuleSubstateKey::MultiResourcePoolField(
            MultiResourcePoolField::try_from(substate_key)?,
        ),
        EntityType::GlobalTransactionTracker => {
            if partition_offset == 0 {
                TypedMainModuleSubstateKey::TransactionTrackerField(
                    TransactionTrackerField::try_from(substate_key)?,
                )
            } else {
                if let Some(key) = substate_key.for_map() {
                    TypedMainModuleSubstateKey::TransactionTrackerCollectionEntry(
                        IntentHash::from_hash(Hash(key.clone().try_into().map_err(|_| ())?)),
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

#[derive(Debug, Clone)]
pub enum TypedSubstateValue {
    TypeInfoModule(TypedTypeInfoModuleSubstateValue),
    AccessRulesModule(TypedAccessRulesModuleSubstateValue),
    RoyaltyModule(TypedRoyaltyModuleSubstateValue),
    MetadataModule(TypedMetadataModuleSubstateValue),
    MainModule(TypedMainModuleSubstateValue),
}

#[derive(Debug, Clone)]
pub enum TypedTypeInfoModuleSubstateValue {
    TypeInfo(TypeInfoSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedAccessRulesModuleSubstateValue {
    OwnerRole(OwnerRoleSubstate),
    Rule(KeyValueEntrySubstate<AccessRule>),
}

#[derive(Debug, Clone)]
pub enum TypedRoyaltyModuleSubstateValue {
    ComponentRoyalty(ComponentRoyaltySubstate),
    ComponentMethodRoyalty(ComponentMethodRoyaltySubstate),
}

#[derive(Debug, Clone)]
pub enum TypedMetadataModuleSubstateValue {
    MetadataEntry(MetadataEntrySubstate),
}

/// Contains all the main module substate values, by each known partition layout
#[derive(Debug, Clone)]
pub enum TypedMainModuleSubstateValue {
    // Objects
    Package(TypedPackageFieldValue),
    PackageBlueprint(KeyValueEntrySubstate<BlueprintDefinition>),
    PackageBlueprintDependencies(KeyValueEntrySubstate<BlueprintDependencies>),
    PackageSchema(KeyValueEntrySubstate<ScryptoSchema>),
    PackageAuthTemplate(KeyValueEntrySubstate<AuthConfig>),
    PackageRoyalty(KeyValueEntrySubstate<PackageRoyaltyConfig>),
    PackageVmType(KeyValueEntrySubstate<PackageVmTypeSubstate>),
    PackageOriginalCode(KeyValueEntrySubstate<PackageOriginalCodeSubstate>),
    PackageInstrumentedCode(KeyValueEntrySubstate<PackageInstrumentedCodeSubstate>),
    FungibleResource(TypedFungibleResourceManagerFieldValue),
    NonFungibleResource(TypedNonFungibleResourceManagerFieldValue),
    NonFungibleResourceData(KeyValueEntrySubstate<ScryptoOwnedRawValue>),
    FungibleVault(TypedFungibleVaultFieldValue),
    NonFungibleVaultField(TypedNonFungibleVaultFieldValue),
    NonFungibleVaultContentsIndexEntry(NonFungibleVaultContentsEntry),
    ConsensusManagerField(TypedConsensusManagerFieldValue),
    ConsensusManagerRegisteredValidatorsByStakeIndexEntry(EpochRegisteredValidatorByStakeEntry),
    Validator(TypedValidatorFieldValue),
    AccessController(TypedAccessControllerFieldValue),
    Account(TypedAccountFieldValue),
    AccountVaultIndex(KeyValueEntrySubstate<Own>),
    AccountResourceDepositRuleIndex(KeyValueEntrySubstate<AccountResourceDepositRuleEntry>),
    OneResourcePool(TypedOneResourcePoolFieldValue),
    TwoResourcePool(TypedTwoResourcePoolFieldValue),
    MultiResourcePool(TypedMultiResourcePoolFieldValue),
    TransactionTracker(TypedTransactionTrackerFieldValue),
    TransactionTrackerCollectionEntry(KeyValueEntrySubstate<TransactionStatusSubstateContents>),
    // Generic Scrypto Components and KV Stores
    GenericScryptoComponent(GenericScryptoComponentFieldValue),
    GenericKeyValueStore(KeyValueEntrySubstate<ScryptoOwnedRawValue>),
}

#[derive(Debug, Clone)]
pub enum TypedPackageFieldValue {
    Royalty(PackageRoyaltyAccumulatorSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedFungibleResourceManagerFieldValue {
    Divisibility(FungibleResourceManagerDivisibilitySubstate),
    TotalSupply(FungibleResourceManagerTotalSupplySubstate),
}

#[derive(Debug, Clone)]
pub enum TypedNonFungibleResourceManagerFieldValue {
    IdType(NonFungibleResourceManagerIdTypeSubstate),
    MutableFields(NonFungibleResourceManagerMutableFieldsSubstate),
    TotalSupply(NonFungibleResourceManagerTotalSupplySubstate),
}

#[derive(Debug, Clone)]
pub enum TypedFungibleVaultFieldValue {
    Balance(FungibleVaultBalanceSubstate),
    VaultFrozenFlag(VaultFrozenFlag),
}

#[derive(Debug, Clone)]
pub enum TypedNonFungibleVaultFieldValue {
    Balance(NonFungibleVaultBalanceSubstate),
    VaultFrozenFlag(VaultFrozenFlag),
}

#[derive(Debug, Clone)]
pub enum TypedConsensusManagerFieldValue {
    Config(ConsensusManagerConfigSubstate),
    ConsensusManager(ConsensusManagerSubstate),
    ValidatorRewards(ValidatorRewardsSubstate),
    CurrentValidatorSet(CurrentValidatorSetSubstate),
    CurrentProposalStatistic(CurrentProposalStatisticSubstate),
    CurrentTimeRoundedToMinutes(ProposerMinuteTimestampSubstate),
    CurrentTime(ProposerMilliTimestampSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedValidatorFieldValue {
    Validator(ValidatorSubstate),
    ProtocolUpdateReadinessSignal(ValidatorProtocolUpdateReadinessSignalSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedAccessControllerFieldValue {
    AccessController(AccessControllerSubstate),
}

#[derive(Debug, Clone)]
pub enum GenericScryptoComponentFieldValue {
    State(GenericScryptoSborPayload),
}

#[derive(Debug, Clone)]
pub enum TypedAccountFieldValue {
    Account(AccountSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedOneResourcePoolFieldValue {
    OneResourcePool(one_resource_pool::OneResourcePoolSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedTwoResourcePoolFieldValue {
    TwoResourcePool(two_resource_pool::TwoResourcePoolSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedMultiResourcePoolFieldValue {
    MultiResourcePool(multi_resource_pool::MultiResourcePoolSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedTransactionTrackerFieldValue {
    TransactionTracker(TransactionTrackerSubstate),
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
        TypedSubstateKey::TypeInfoModule(type_info_key) => {
            TypedSubstateValue::TypeInfoModule(match type_info_key {
                TypedTypeInfoModuleSubstateKey::TypeInfoField(TypeInfoField::TypeInfo) => {
                    TypedTypeInfoModuleSubstateValue::TypeInfo(scrypto_decode(data)?)
                }
            })
        }
        TypedSubstateKey::AccessRulesModule(access_rules_key) => match access_rules_key {
            TypedAccessRulesSubstateKey::AccessRulesField(access_rules_field_offset) => {
                match access_rules_field_offset {
                    AccessRulesField::OwnerRole => TypedSubstateValue::AccessRulesModule(
                        TypedAccessRulesModuleSubstateValue::OwnerRole(scrypto_decode(data)?),
                    ),
                }
            }
            TypedAccessRulesSubstateKey::Rule(_) => TypedSubstateValue::AccessRulesModule(
                TypedAccessRulesModuleSubstateValue::Rule(scrypto_decode(data)?),
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
        TypedMainModuleSubstateKey::PackageField(offset) => {
            TypedMainModuleSubstateValue::Package(match offset {
                PackageField::Royalty => TypedPackageFieldValue::Royalty(scrypto_decode(data)?),
            })
        }
        TypedMainModuleSubstateKey::PackageBlueprintKey(_key) => {
            TypedMainModuleSubstateValue::PackageBlueprint(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::PackageBlueprintDependenciesKey(..) => {
            TypedMainModuleSubstateValue::PackageBlueprintDependencies(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::PackageSchemaKey(..) => {
            TypedMainModuleSubstateValue::PackageSchema(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::PackageRoyaltyKey(_fn_key) => {
            TypedMainModuleSubstateValue::PackageRoyalty(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::PackageAuthTemplateKey(_fn_key) => {
            TypedMainModuleSubstateValue::PackageAuthTemplate(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::PackageVmTypeKey(..) => {
            TypedMainModuleSubstateValue::PackageVmType(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::PackageOriginalCodeKey(..) => {
            TypedMainModuleSubstateValue::PackageOriginalCode(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::PackageInstrumentedCodeKey(..) => {
            TypedMainModuleSubstateValue::PackageInstrumentedCode(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::FungibleResourceField(offset) => {
            TypedMainModuleSubstateValue::FungibleResource(match offset {
                FungibleResourceManagerField::Divisibility => {
                    TypedFungibleResourceManagerFieldValue::Divisibility(scrypto_decode(data)?)
                }
                FungibleResourceManagerField::TotalSupply => {
                    TypedFungibleResourceManagerFieldValue::TotalSupply(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::NonFungibleResourceField(offset) => {
            TypedMainModuleSubstateValue::NonFungibleResource(match offset {
                NonFungibleResourceManagerField::IdType => {
                    TypedNonFungibleResourceManagerFieldValue::IdType(scrypto_decode(data)?)
                }
                NonFungibleResourceManagerField::MutableFields => {
                    TypedNonFungibleResourceManagerFieldValue::MutableFields(scrypto_decode(data)?)
                }
                NonFungibleResourceManagerField::TotalSupply => {
                    TypedNonFungibleResourceManagerFieldValue::TotalSupply(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::NonFungibleResourceData(_) => {
            TypedMainModuleSubstateValue::NonFungibleResourceData(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::FungibleVaultField(offset) => {
            TypedMainModuleSubstateValue::FungibleVault(match offset {
                FungibleVaultField::LiquidFungible => {
                    TypedFungibleVaultFieldValue::Balance(scrypto_decode(data)?)
                }
                // This shouldn't be persistable - so use a bizarre (but temporary!) placeholder error code here!
                FungibleVaultField::LockedFungible => Err(DecodeError::InvalidCustomValue)?,
                FungibleVaultField::VaultFrozenFlag => {
                    TypedFungibleVaultFieldValue::VaultFrozenFlag(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::NonFungibleVaultField(offset) => {
            TypedMainModuleSubstateValue::NonFungibleVaultField(match offset {
                NonFungibleVaultField::LiquidNonFungible => {
                    TypedNonFungibleVaultFieldValue::Balance(scrypto_decode(data)?)
                }
                // This shouldn't be persistable - so use a bizarre (but temporary!) placeholder error code here!
                NonFungibleVaultField::LockedNonFungible => Err(DecodeError::InvalidCustomValue)?,
                NonFungibleVaultField::VaultFrozenFlag => {
                    TypedNonFungibleVaultFieldValue::VaultFrozenFlag(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::NonFungibleVaultContentsIndexKey(_) => {
            TypedMainModuleSubstateValue::NonFungibleVaultContentsIndexEntry(scrypto_decode(data)?)
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
        TypedMainModuleSubstateKey::AccountField(offset) => {
            TypedMainModuleSubstateValue::Account(match offset {
                AccountField::Account => TypedAccountFieldValue::Account(scrypto_decode(data)?),
            })
        }
        TypedMainModuleSubstateKey::AccountVaultIndexKey(_) => {
            TypedMainModuleSubstateValue::AccountVaultIndex(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::AccountResourceDepositRuleIndexKey(_) => {
            TypedMainModuleSubstateValue::AccountResourceDepositRuleIndex(scrypto_decode(data)?)
        }
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
                    GenericScryptoComponentFieldValue::State(GenericScryptoSborPayload {
                        data: data.to_vec(),
                    })
                }
            })
        }
        TypedMainModuleSubstateKey::GenericKeyValueStoreKey(_) => {
            TypedMainModuleSubstateValue::GenericKeyValueStore(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::OneResourcePoolField(offset) => {
            TypedMainModuleSubstateValue::OneResourcePool(match offset {
                OneResourcePoolField::OneResourcePool => {
                    TypedOneResourcePoolFieldValue::OneResourcePool(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::TwoResourcePoolField(offset) => {
            TypedMainModuleSubstateValue::TwoResourcePool(match offset {
                TwoResourcePoolField::TwoResourcePool => {
                    TypedTwoResourcePoolFieldValue::TwoResourcePool(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::MultiResourcePoolField(offset) => {
            TypedMainModuleSubstateValue::MultiResourcePool(match offset {
                MultiResourcePoolField::MultiResourcePool => {
                    TypedMultiResourcePoolFieldValue::MultiResourcePool(scrypto_decode(data)?)
                }
            })
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
