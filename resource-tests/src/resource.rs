use crate::{TestFuzzer, VaultComponentMeta};
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::role_assignment::RoleAssignment;
use native_sdk::resource::{NativeBucket, NativeVault};
use radix_engine::errors::RuntimeError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::types::FromRepr;
use radix_engine::vm::VmInvoke;
use radix_engine_common::manifest_args;
use radix_engine_common::prelude::{
    manifest_decode, manifest_encode, scrypto_decode, scrypto_encode, ManifestValue,
    NonFungibleLocalId, ScryptoValue,
};
use radix_engine_common::types::ResourceAddress;
use radix_engine_interface::api::{ClientApi, LockFlags, ModuleId, ACTOR_STATE_SELF};
use radix_engine_interface::data::manifest::ManifestArgs;
use radix_engine_interface::prelude::{
    Bucket, FieldValue, FungibleResourceManagerMintInput,
    NonFungibleResourceManagerMintManifestInput, OwnerRole, Vault,
    FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT, NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
    NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT, VAULT_PUT_IDENT, VAULT_TAKE_ADVANCED_IDENT,
    VAULT_TAKE_IDENT,
};
use radix_engine_interface::types::IndexedScryptoValue;
use scrypto::prelude::indexmap::IndexMap;
use transaction::builder::ManifestBuilder;
use utils::indexmap;

pub const BLUEPRINT_NAME: &str = "MyBlueprint";
pub const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;

#[derive(Clone)]
pub struct VaultTestInvoke;
impl VmInvoke for VaultTestInvoke {
    fn invoke<Y>(
        &mut self,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    {
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
                        ModuleId::Metadata => metadata.0,
                        ModuleId::RoleAssignment => access_rules.0.0,
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
                        ModuleId::Metadata => metadata.0,
                        ModuleId::RoleAssignment => access_rules.0.0,
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
        fuzzer: &mut TestFuzzer,
        vault_meta: &VaultComponentMeta,
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
pub enum ResourceFuzzUseBucketAction {
    Burn,
    VaultPut,
}

impl ResourceFuzzUseBucketAction {
    pub fn add_to_manifest(
        &self,
        builder: ManifestBuilder,
        fuzzer: &mut TestFuzzer,
        vault_meta: &VaultComponentMeta,
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
        fuzzer: &mut TestFuzzer,
        vault_meta: &VaultComponentMeta,
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
