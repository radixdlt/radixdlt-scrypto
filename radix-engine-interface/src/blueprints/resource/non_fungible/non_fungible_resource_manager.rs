use crate::blueprints::resource::*;
use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::{Arbitrary, Result, Unstructured};
use radix_common::data::manifest::model::ManifestAddressReservation;
use radix_common::data::manifest::ManifestValue;
use radix_common::data::scrypto::{ScryptoCustomTypeKind, ScryptoValue, VersionedScryptoSchema};
use radix_common::prelude::replace_self_package_address;
use radix_common::prelude::*;
use radix_common::traits::NonFungibleData;
use radix_engine_interface::object_modules::metadata::MetadataInit;
use radix_engine_interface::object_modules::ModuleConfig;
use sbor::rust::collections::{IndexMap, IndexSet};
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::{generate_full_schema, LocalTypeId, TypeAggregator};

pub const NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT: &str = "NonFungibleResourceManager";

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Default, Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct NonFungibleResourceRoles {
    pub mint_roles: Option<MintRoles<RoleDefinition>>,
    pub burn_roles: Option<BurnRoles<RoleDefinition>>,
    pub freeze_roles: Option<FreezeRoles<RoleDefinition>>,
    pub recall_roles: Option<RecallRoles<RoleDefinition>>,
    pub withdraw_roles: Option<WithdrawRoles<RoleDefinition>>,
    pub deposit_roles: Option<DepositRoles<RoleDefinition>>,
    pub non_fungible_data_update_roles: Option<NonFungibleDataUpdateRoles<RoleDefinition>>,
}

impl NonFungibleResourceRoles {
    pub fn single_locked_rule(access_rule: AccessRule) -> Self {
        Self {
            mint_roles: mint_roles! {
                minter => access_rule.clone();
                minter_updater => rule!(deny_all);
            },
            burn_roles: burn_roles! {
                burner => access_rule.clone();
                burner_updater => rule!(deny_all);
            },
            freeze_roles: freeze_roles! {
                freezer => access_rule.clone();
                freezer_updater => rule!(deny_all);
            },
            recall_roles: recall_roles! {
                recaller => access_rule.clone();
                recaller_updater => rule!(deny_all);
            },
            non_fungible_data_update_roles: non_fungible_data_update_roles! {
                non_fungible_data_updater => access_rule.clone();
                non_fungible_data_updater_updater => rule!(deny_all);
            },
            withdraw_roles: withdraw_roles! {
                withdrawer => access_rule.clone();
                withdrawer_updater => rule!(deny_all);
            },
            deposit_roles: deposit_roles! {
                depositor => access_rule;
                depositor_updater => rule!(deny_all);
            },
        }
    }
}

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT: &str = "create";

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerCreateInput {
    pub owner_role: OwnerRole,
    pub id_type: NonFungibleIdType,
    pub track_total_supply: bool,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub resource_roles: NonFungibleResourceRoles,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<GlobalAddressReservation>,
}

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NonFungibleResourceManagerCreateManifestInput {
    pub owner_role: OwnerRole,
    pub id_type: NonFungibleIdType,
    pub track_total_supply: bool,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub resource_roles: NonFungibleResourceRoles,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<ManifestAddressReservation>,
}

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerCreateGenericInput<S> {
    pub owner_role: OwnerRole,
    pub id_type: NonFungibleIdType,
    pub track_total_supply: bool,
    pub non_fungible_schema: S,
    pub resource_roles: NonFungibleResourceRoles,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<GlobalAddressReservation>,
}

pub type NonFungibleResourceManagerCreateOutput = ResourceAddress;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_with_initial_supply";

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerCreateWithInitialSupplyInput {
    pub owner_role: OwnerRole,
    pub id_type: NonFungibleIdType,
    pub track_total_supply: bool,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub entries: IndexMap<NonFungibleLocalId, (ScryptoValue,)>,
    pub resource_roles: NonFungibleResourceRoles,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<GlobalAddressReservation>,
}

/// For manifest
#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
    pub owner_role: OwnerRole,
    pub id_type: NonFungibleIdType,
    pub track_total_supply: bool,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub entries: IndexMap<NonFungibleLocalId, (ManifestValue,)>,
    pub resource_roles: NonFungibleResourceRoles,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<ManifestAddressReservation>,
}

/// For typed value, to skip any codec
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerCreateWithInitialSupplyGenericInput<S, T> {
    pub owner_role: OwnerRole,
    pub id_type: NonFungibleIdType,
    pub track_total_supply: bool,
    pub non_fungible_schema: S,
    pub entries: IndexMap<NonFungibleLocalId, (T,)>,
    pub resource_roles: NonFungibleResourceRoles,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<GlobalAddressReservation>,
}

pub type NonFungibleResourceManagerCreateWithInitialSupplyOutput = (ResourceAddress, Bucket);

pub const NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_ruid_non_fungible_with_initial_supply";

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerCreateRuidWithInitialSupplyInput {
    pub owner_role: OwnerRole,
    pub track_total_supply: bool,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub entries: Vec<(ScryptoValue,)>,
    pub resource_roles: NonFungibleResourceRoles,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<GlobalAddressReservation>,
}

/// For manifest
#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NonFungibleResourceManagerCreateRuidWithInitialSupplyManifestInput {
    pub owner_role: OwnerRole,
    pub track_total_supply: bool,
    pub non_fungible_schema: NonFungibleDataSchema,
    pub entries: Vec<(ManifestValue,)>,
    pub resource_roles: NonFungibleResourceRoles,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<ManifestAddressReservation>,
}

/// For typed value, to skip any codec
#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerCreateRuidWithInitialSupplyGenericInput<S, T> {
    pub owner_role: OwnerRole,
    pub track_total_supply: bool,
    pub non_fungible_schema: S,
    pub entries: Vec<(T,)>,
    pub resource_roles: NonFungibleResourceRoles,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<GlobalAddressReservation>,
}

pub type NonFungibleResourceManagerCreateRuidWithInitialSupplyOutput = (ResourceAddress, Bucket);

pub const NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT: &str = "update_non_fungible_data";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerUpdateDataInput {
    pub id: NonFungibleLocalId,
    pub field_name: String,
    pub data: ScryptoValue,
}

/// For manifest
#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NonFungibleResourceManagerUpdateDataManifestInput {
    pub id: NonFungibleLocalId,
    pub field_name: String,
    pub data: ManifestValue,
}

/// For typed value, to skip any codec
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerUpdateDataGenericInput<T> {
    pub id: NonFungibleLocalId,
    pub field_name: String,
    pub data: T,
}

pub type NonFungibleResourceManagerUpdateDataOutput = ();

pub const NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT: &str = "non_fungible_exists";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct NonFungibleResourceManagerExistsInput {
    pub id: NonFungibleLocalId,
}

pub type NonFungibleResourceManagerExistsManifestInput = NonFungibleResourceManagerExistsInput;

pub type NonFungibleResourceManagerExistsOutput = bool;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT: &str = "get_non_fungible";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct NonFungibleResourceManagerGetNonFungibleInput {
    pub id: NonFungibleLocalId,
}

pub type NonFungibleResourceManagerGetNonFungibleManifestInput =
    NonFungibleResourceManagerGetNonFungibleInput;

pub type NonFungibleResourceManagerGetNonFungibleOutput = ScryptoValue;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT: &str = "mint";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMintInput {
    pub entries: IndexMap<NonFungibleLocalId, (ScryptoValue,)>,
}

/// For manifest
#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NonFungibleResourceManagerMintManifestInput {
    pub entries: IndexMap<NonFungibleLocalId, (ManifestValue,)>,
}

/// For typed value, to skip any codec
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMintGenericInput<T> {
    pub entries: IndexMap<NonFungibleLocalId, (T,)>,
}

pub type NonFungibleResourceManagerMintOutput = Bucket;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT: &str = "mint_ruid";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMintRuidInput {
    pub entries: Vec<(ScryptoValue,)>,
}

/// For manifest
#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NonFungibleResourceManagerMintRuidManifestInput {
    pub entries: Vec<(ManifestValue,)>,
}

/// For typed value, to skip any codec
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMintRuidGenericInput<T> {
    pub entries: Vec<(T,)>,
}

pub type NonFungibleResourceManagerMintRuidOutput = Bucket;

pub const NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_RUID_IDENT: &str = "mint_single_ruid";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMintSingleRuidInput {
    pub entry: ScryptoValue,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NonFungibleResourceManagerMintSingleRuidManifestInput {
    pub entry: ManifestValue,
}

/// For typed value, to skip any codec
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonFungibleResourceManagerMintSingleRuidGenericInput<T> {
    pub entry: T,
}

pub type NonFungibleResourceManagerMintSingleRuidOutput = (Bucket, NonFungibleLocalId);

pub const NON_FUNGIBLE_DATA_SCHEMA_VARIANT_LOCAL: u8 = 0;
pub const NON_FUNGIBLE_DATA_SCHEMA_VARIANT_REMOTE: u8 = 1;

pub type NonFungibleResourceManagerCreateEmptyBucketInput = ResourceManagerCreateEmptyBucketInput;
pub type NonFungibleResourceManagerCreateEmptyBucketManifestInput =
    NonFungibleResourceManagerCreateEmptyBucketInput;

pub type NonFungibleResourceManagerPackageBurnInput = ResourceManagerPackageBurnInput;
pub type NonFungibleResourceManagerPackageBurnManifestInput =
    NonFungibleResourceManagerPackageBurnInput;

pub type NonFungibleResourceManagerBurnInput = ResourceManagerBurnInput;
pub type NonFungibleResourceManagerBurnManifestInput = NonFungibleResourceManagerBurnInput;

pub type NonFungibleResourceManagerCreateEmptyVaultInput = ResourceManagerCreateEmptyVaultInput;
pub type NonFungibleResourceManagerCreateEmptyVaultManifestInput =
    NonFungibleResourceManagerCreateEmptyVaultInput;

pub type NonFungibleResourceManagerGetResourceTypeInput = ResourceManagerGetResourceTypeInput;
pub type NonFungibleResourceManagerGetResourceTypeManifestInput =
    NonFungibleResourceManagerGetResourceTypeInput;

pub type NonFungibleResourceManagerGetTotalSupplyInput = ResourceManagerGetTotalSupplyInput;
pub type NonFungibleResourceManagerGetTotalSupplyManifestInput =
    NonFungibleResourceManagerGetTotalSupplyInput;

pub type NonFungibleResourceManagerAmountForWithdrawalInput =
    ResourceManagerGetAmountForWithdrawalInput;
pub type NonFungibleResourceManagerAmountForWithdrawalManifestInput =
    NonFungibleResourceManagerAmountForWithdrawalInput;

pub type NonFungibleResourceManagerDropEmptyBucketInput = ResourceManagerDropEmptyBucketInput;
pub type NonFungibleResourceManagerDropEmptyBucketManifestInput =
    NonFungibleResourceManagerDropEmptyBucketInput;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum NonFungibleDataSchema {
    // TODO: ignore this variant in Scrypto for smaller code size
    Local(#[sbor(flatten)] LocalNonFungibleDataSchema),
    Remote(#[sbor(flatten)] RemoteNonFungibleDataSchema),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct LocalNonFungibleDataSchema {
    pub schema: VersionedScryptoSchema,
    pub type_id: LocalTypeId,
    pub mutable_fields: IndexSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct RemoteNonFungibleDataSchema {
    pub type_id: BlueprintTypeIdentifier,
    pub mutable_fields: IndexSet<String>,
}

impl LocalNonFungibleDataSchema {
    pub fn new_with_self_package_replacement<N: NonFungibleData>(
        package_address: PackageAddress,
    ) -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let type_id = aggregator.add_child_type_and_descendents::<N>();
        let mut schema = generate_full_schema(aggregator);
        replace_self_package_address(&mut schema, package_address);
        Self {
            schema,
            type_id,
            mutable_fields: N::MUTABLE_FIELDS.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn new_without_self_package_replacement<N: NonFungibleData>() -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let type_id = aggregator.add_child_type_and_descendents::<N>();
        let schema = generate_full_schema(aggregator);
        Self {
            schema,
            type_id,
            mutable_fields: N::MUTABLE_FIELDS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl RemoteNonFungibleDataSchema {
    pub fn new(type_id: BlueprintTypeIdentifier, mutable_fields: IndexSet<String>) -> Self {
        Self {
            type_id,
            mutable_fields,
        }
    }
}

impl NonFungibleDataSchema {
    pub fn new_with_self_package_replacement<N: NonFungibleData>(
        package_address: PackageAddress,
    ) -> Self {
        let schema =
            LocalNonFungibleDataSchema::new_with_self_package_replacement::<N>(package_address);
        Self::Local(schema)
    }

    pub fn new_local_without_self_package_replacement<N: NonFungibleData>() -> Self {
        let schema = LocalNonFungibleDataSchema::new_without_self_package_replacement::<N>();
        Self::Local(schema)
    }
}

#[cfg(feature = "fuzzing")]
impl<'a> Arbitrary<'a> for NonFungibleDataSchema {
    // At the moment I see no smart method to derive Arbitrary for type Schema, which is part of
    // ScryptoSchema, therefore implementing arbitrary by hand.
    // TODO: Introduce a method that genearates NonFungibleDataSchema in a truly random manner
    fn arbitrary(_u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(Self::Local(LocalNonFungibleDataSchema {
            schema: VersionedScryptoSchema::from_latest_version(SchemaV1 {
                type_kinds: vec![],
                type_metadata: vec![],
                type_validations: vec![],
            }),
            type_id: LocalTypeId::WellKnown(sbor::basic_well_known_types::UNIT_TYPE),
            mutable_fields: indexset!(),
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(ScryptoSbor)]
    pub struct SomeNonFungibleData {
        pub field: String,
    }

    impl NonFungibleData for SomeNonFungibleData {
        const MUTABLE_FIELDS: &'static [&'static str] = &[];
    }

    #[test]
    fn test_non_fungible_data_schema_with_self_package_replacement() {
        pub const SOME_ADDRESS: PackageAddress =
            PackageAddress::new_or_panic([EntityType::GlobalPackage as u8; NodeId::LENGTH]);

        let ds: NonFungibleDataSchema = NonFungibleDataSchema::new_with_self_package_replacement::<
            SomeNonFungibleData,
        >(SOME_ADDRESS);
        if let NonFungibleDataSchema::Local(LocalNonFungibleDataSchema {
            schema,
            type_id,
            mutable_fields,
        }) = ds
        {
            let s = schema.fully_update_and_into_latest_version();
            assert_eq!(s.type_kinds.len(), 1);
            assert_eq!(s.type_metadata.len(), 1);
            assert_eq!(s.type_validations.len(), 1);
            assert_matches!(type_id, LocalTypeId::SchemaLocalIndex(0));
            assert!(mutable_fields.is_empty());
        } else {
            panic!("Wrong Non Fungible Data Schema type")
        }
    }
}
