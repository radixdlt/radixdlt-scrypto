use crate::{ResourceComponentMeta, SystemTestFuzzer};
use radix_common::data::manifest::ManifestArgs;
use radix_common::manifest_args;
use radix_common::prelude::IndexMap;
use radix_common::prelude::{
    manifest_decode, manifest_encode, scrypto_decode, scrypto_encode, ManifestValue,
    NonFungibleLocalId, ScryptoValue,
};
use radix_common::types::{ComponentAddress, ResourceAddress};
use radix_engine::errors::RuntimeError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::vm::{VmApi, VmInvoke};
use radix_engine_interface::api::{AttachedModuleId, LockFlags, SystemApi, ACTOR_STATE_SELF};
use radix_engine_interface::prelude::*;
use radix_engine_interface::prelude::{
    Bucket, FieldValue, FungibleResourceManagerMintInput,
    NonFungibleResourceManagerMintManifestInput, OwnerRole, Vault,
    FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT, NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
    NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT, VAULT_PUT_IDENT, VAULT_TAKE_ADVANCED_IDENT,
    VAULT_TAKE_IDENT,
};
use radix_engine_interface::types::IndexedScryptoValue;
use radix_native_sdk::modules::metadata::Metadata;
use radix_native_sdk::modules::role_assignment::RoleAssignment;
use radix_native_sdk::resource::{NativeBucket, NativeVault};
use radix_rust::indexmap;
use radix_transactions::builder::ManifestBuilder;

pub const BLUEPRINT_NAME: &str = "MyBlueprint";
pub const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;

#[derive(Clone)]
pub struct ResourceTestInvoke;
impl VmInvoke for ResourceTestInvoke {
    fn invoke<
        Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
        V: VmApi,
    >(
        &mut self,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
        _vm_api: &V,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match export_name {
            "call_vault" => {
                let handle = api
                    .actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::read_only())
                    .unwrap();
                let vault: Vault = api.field_read_typed(handle).unwrap();

                let input: (String, ScryptoValue) = scrypto_decode(input.as_slice()).unwrap();

                let rtn = api.call_method(
                    vault.0.as_node_id(),
                    input.0.as_str(),
                    scrypto_encode(&input.1).unwrap(),
                )?;
                return Ok(IndexedScryptoValue::from_vec(rtn).unwrap());
            }
            "combine_buckets" => {
                let input: (Bucket, Bucket) = scrypto_decode(input.as_slice()).unwrap();
                input.0.put(input.1, api)?;
                return Ok(IndexedScryptoValue::from_typed(&input.0));
            }
            "new" => {
                let resource_address: (ResourceAddress,) =
                    scrypto_decode(input.as_slice()).unwrap();
                let vault = Vault::create(resource_address.0, api).unwrap();

                let metadata = Metadata::create(api)?;
                let access_rules = RoleAssignment::create(OwnerRole::None, indexmap!(), api)?;
                let node_id = api
                    .new_simple_object(BLUEPRINT_NAME, indexmap!(0u8 => FieldValue::new(&vault)))?;

                api.globalize(
                    node_id,
                    indexmap!(
                        AttachedModuleId::Metadata => metadata.0,
                        AttachedModuleId::RoleAssignment => access_rules.0.0,
                    ),
                    None,
                )?;
            }
            "new_with_bucket" => {
                let bucket: (Bucket,) = scrypto_decode(input.as_slice()).unwrap();
                let resource_address = bucket.0.resource_address(api).unwrap();
                let mut vault = Vault::create(resource_address, api).unwrap();
                vault.put(bucket.0, api).unwrap();

                let metadata = Metadata::create(api)?;
                let access_rules = RoleAssignment::create(OwnerRole::None, indexmap!(), api)?;
                let node_id = api
                    .new_simple_object(BLUEPRINT_NAME, indexmap!(0u8 => FieldValue::new(&vault)))?;

                api.globalize(
                    node_id,
                    indexmap!(
                        AttachedModuleId::Metadata => metadata.0,
                        AttachedModuleId::RoleAssignment => access_rules.0.0,
                    ),
                    None,
                )?;
            }
            _ => {}
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
pub enum FungibleResourceFuzzGetBucketAction {
    Mint,
    VaultTake,
    VaultTakeAdvanced,
    VaultRecall,
}

impl FungibleResourceFuzzGetBucketAction {
    pub fn add_to_manifest(
        &self,
        builder: ManifestBuilder,
        fuzzer: &mut SystemTestFuzzer,
        vault_meta: &ResourceComponentMeta,
    ) -> (ManifestBuilder, bool) {
        match self {
            FungibleResourceFuzzGetBucketAction::Mint => {
                let amount = fuzzer.next_amount();
                let builder = builder.call_method(
                    vault_meta.resource_address,
                    FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
                    FungibleResourceManagerMintInput { amount },
                );
                (builder, amount.is_zero())
            }
            FungibleResourceFuzzGetBucketAction::VaultTake => {
                let amount = fuzzer.next_amount();
                let builder = builder.call_method(
                    vault_meta.component_address,
                    "call_vault",
                    manifest_args!(VAULT_TAKE_IDENT, (amount,)),
                );
                (builder, amount.is_zero())
            }
            FungibleResourceFuzzGetBucketAction::VaultTakeAdvanced => {
                let amount = fuzzer.next_amount();
                let withdraw_strategy = fuzzer.next_withdraw_strategy();
                let builder = builder.call_method(
                    vault_meta.component_address,
                    "call_vault",
                    manifest_args!(VAULT_TAKE_ADVANCED_IDENT, (amount, withdraw_strategy)),
                );
                (builder, amount.is_zero())
            }
            FungibleResourceFuzzGetBucketAction::VaultRecall => {
                let amount = fuzzer.next_amount();
                let builder = builder.recall(vault_meta.vault_address, amount);
                (builder, amount.is_zero())
            }
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
pub enum ResourceFuzzTransformBucketAction {
    Combine,
}

impl ResourceFuzzTransformBucketAction {
    pub fn add_to_manifest(
        &self,
        builder: ManifestBuilder,
        fuzzer: &mut SystemTestFuzzer,
        vault_meta: &ResourceComponentMeta,
    ) -> (ManifestBuilder, bool) {
        match self {
            ResourceFuzzTransformBucketAction::Combine => {
                let amount1 = fuzzer.next_amount();
                let amount2 = fuzzer.next_amount();
                let builder = builder
                    .take_from_worktop(vault_meta.resource_address, amount1, "bucket1")
                    .take_from_worktop(vault_meta.resource_address, amount2, "bucket2")
                    .with_name_lookup(|builder, lookup| {
                        builder.call_method(
                            vault_meta.component_address,
                            "combine_buckets",
                            manifest_args!(lookup.bucket("bucket1"), lookup.bucket("bucket2")),
                        )
                    });

                (builder, amount1.is_zero() && amount2.is_zero())
            }
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
pub enum ResourceFuzzRandomAction {
    AccountTake,
    FungibleMint,
    NonFungibleMint,
    CombineBuckets,
    VaultPut,
}

impl ResourceFuzzRandomAction {
    pub fn add_to_manifest(
        &self,
        builder: ManifestBuilder,
        fuzzer: &mut SystemTestFuzzer,
        account: ComponentAddress,
        fungible_vault_meta: &ResourceComponentMeta,
        non_fungible_vault_meta: &ResourceComponentMeta,
    ) -> (ManifestBuilder, bool) {
        match self {
            ResourceFuzzRandomAction::AccountTake => {
                let amount = fuzzer.next_amount();
                let builder =
                    builder.withdraw_from_account(account, fuzzer.next_resource(), amount);
                (builder, amount.is_zero())
            }
            ResourceFuzzRandomAction::FungibleMint => {
                let amount = fuzzer.next_amount();
                let builder = builder.call_method(
                    fuzzer.next_resource(),
                    FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
                    FungibleResourceManagerMintInput { amount },
                );
                (builder, false)
            }
            ResourceFuzzRandomAction::NonFungibleMint => {
                let entries = fuzzer
                    .next_non_fungible_id_set()
                    .into_iter()
                    .map(|id| {
                        (
                            id,
                            (manifest_decode(&manifest_encode(&()).unwrap()).unwrap(),),
                        )
                    })
                    .collect();
                let builder = builder.call_method(
                    fuzzer.next_non_fungible(),
                    NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
                    NonFungibleResourceManagerMintManifestInput { entries },
                );
                (builder, false)
            }
            ResourceFuzzRandomAction::CombineBuckets => {
                let amount1 = fuzzer.next_amount();
                let amount2 = fuzzer.next_amount();
                let builder = builder
                    .take_from_worktop(fuzzer.next_resource(), amount1, "bucket1")
                    .take_from_worktop(fuzzer.next_resource(), amount2, "bucket2")
                    .with_name_lookup(|builder, lookup| {
                        builder.call_method(
                            fungible_vault_meta.component_address,
                            "combine_buckets",
                            manifest_args!(lookup.bucket("bucket1"), lookup.bucket("bucket2")),
                        )
                    });

                (builder, amount1.is_zero() && amount2.is_zero())
            }
            ResourceFuzzRandomAction::VaultPut => {
                let amount = fuzzer.next_amount();
                let component = match fuzzer.next(0u32..=1u32) {
                    0u32 => fungible_vault_meta.component_address,
                    _ => non_fungible_vault_meta.component_address,
                };

                let builder = builder
                    .take_from_worktop(fuzzer.next_resource(), amount, "bucket")
                    .with_name_lookup(|builder, lookup| {
                        builder.call_method(
                            component,
                            "call_vault",
                            manifest_args!(VAULT_PUT_IDENT, (lookup.bucket("bucket"),)),
                        )
                    });

                (builder, amount.is_zero())
            }
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
pub enum ResourceFuzzUseBucketAction {
    Burn,
    VaultPut,
}

impl ResourceFuzzUseBucketAction {
    pub fn add_to_manifest(
        &self,
        builder: ManifestBuilder,
        fuzzer: &mut SystemTestFuzzer,
        vault_meta: &ResourceComponentMeta,
    ) -> (ManifestBuilder, bool) {
        match self {
            ResourceFuzzUseBucketAction::Burn => {
                let amount = fuzzer.next_amount();
                let builder = builder
                    .take_from_worktop(vault_meta.resource_address, amount, "bucket")
                    .burn_resource("bucket");

                (builder, amount.is_zero())
            }
            ResourceFuzzUseBucketAction::VaultPut => {
                let amount = fuzzer.next_amount();
                let builder = builder
                    .take_from_worktop(vault_meta.resource_address, amount, "bucket")
                    .with_bucket("bucket", |builder, bucket| {
                        builder.call_method(
                            vault_meta.component_address,
                            "call_vault",
                            manifest_args!(VAULT_PUT_IDENT, (bucket,)),
                        )
                    });
                (builder, amount.is_zero())
            }
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
pub enum NonFungibleResourceFuzzGetBucketAction {
    Mint,
    VaultTake,
    VaultTakeNonFungibles,
    VaultTakeAdvanced,
    VaultRecall,
    VaultRecallNonFungibles,
}

impl NonFungibleResourceFuzzGetBucketAction {
    pub fn add_to_manifest(
        &self,
        builder: ManifestBuilder,
        fuzzer: &mut SystemTestFuzzer,
        vault_meta: &ResourceComponentMeta,
    ) -> (ManifestBuilder, bool) {
        match self {
            NonFungibleResourceFuzzGetBucketAction::Mint => {
                let ids = fuzzer.next_non_fungible_id_set();
                let trivial = ids.is_empty();
                let entries: IndexMap<NonFungibleLocalId, (ManifestValue,)> = ids
                    .into_iter()
                    .map(|id| {
                        let value: ManifestValue =
                            manifest_decode(&manifest_encode(&()).unwrap()).unwrap();
                        (id, (value,))
                    })
                    .collect();

                let builder = builder.call_method(
                    vault_meta.resource_address,
                    NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
                    NonFungibleResourceManagerMintManifestInput { entries },
                );

                (builder, trivial)
            }
            NonFungibleResourceFuzzGetBucketAction::VaultTake => {
                let amount = fuzzer.next_amount();
                let builder = builder.call_method(
                    vault_meta.component_address,
                    "call_vault",
                    manifest_args!(VAULT_TAKE_IDENT, (amount,)),
                );
                (builder, amount.is_zero())
            }
            NonFungibleResourceFuzzGetBucketAction::VaultTakeNonFungibles => {
                let ids = fuzzer.next_non_fungible_id_set();
                let trivial = ids.is_empty();
                let builder = builder.call_method(
                    vault_meta.component_address,
                    "call_vault",
                    manifest_args!(NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT, (ids,)),
                );
                (builder, trivial)
            }
            NonFungibleResourceFuzzGetBucketAction::VaultTakeAdvanced => {
                let amount = fuzzer.next_amount();
                let withdraw_strategy = fuzzer.next_withdraw_strategy();
                let builder = builder.call_method(
                    vault_meta.component_address,
                    "call_vault",
                    manifest_args!(VAULT_TAKE_ADVANCED_IDENT, (amount, withdraw_strategy)),
                );
                (builder, amount.is_zero())
            }
            NonFungibleResourceFuzzGetBucketAction::VaultRecall => {
                let amount = fuzzer.next_amount();
                let builder = builder.recall(vault_meta.vault_address, amount);
                (builder, amount.is_zero())
            }
            NonFungibleResourceFuzzGetBucketAction::VaultRecallNonFungibles => {
                let ids = fuzzer.next_non_fungible_id_set();
                let trivial = ids.is_empty();
                let builder = builder.recall_non_fungibles(vault_meta.vault_address, ids);
                (builder, trivial)
            }
        }
    }
}
