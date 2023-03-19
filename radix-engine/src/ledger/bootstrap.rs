use crate::blueprints::access_controller::*;
use crate::blueprints::account::AccountNativePackage;
use crate::blueprints::auth_zone::AuthZoneNativePackage;
use crate::blueprints::clock::ClockNativePackage;

use crate::blueprints::epoch_manager::EpochManagerNativePackage;
use crate::blueprints::identity::IdentityNativePackage;
use crate::blueprints::package::PackageNativePackage;
use crate::blueprints::resource::ResourceManagerNativePackage;
use crate::blueprints::transaction_processor::TransactionProcessorNativePackage;
use crate::kernel::interpreters::ScryptoInterpreter;
use crate::ledger::{ReadableSubstateStore, WriteableSubstateStore};
use crate::system::node_modules::access_rules::AccessRulesNativePackage;
use crate::system::node_modules::metadata::MetadataNativePackage;
use crate::system::node_modules::royalty::RoyaltyNativePackage;
use crate::transaction::{
    execute_transaction, ExecutionConfig, FeeReserveConfig, TransactionReceipt,
};
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::blueprints::clock::{
    ClockCreateInput, CLOCK_BLUEPRINT, CLOCK_CREATE_IDENT,
};
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::rule;
use transaction::model::{Instruction, SystemTransaction};
use transaction::validation::ManifestIdAllocator;

const XRD_SYMBOL: &str = "XRD";
const XRD_NAME: &str = "Radix";
const XRD_DESCRIPTION: &str = "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.";
const XRD_URL: &str = "https://tokens.radixdlt.com";
const XRD_MAX_SUPPLY: i128 = 1_000_000_000_000i128;

pub struct GenesisReceipt {
    pub faucet_component: ComponentAddress,
}

pub fn create_genesis(
    validator_set_and_stake_owners: BTreeMap<EcdsaSecp256k1PublicKey, (Decimal, ComponentAddress)>,
    account_xrd_allocations: BTreeMap<EcdsaSecp256k1PublicKey, Decimal>,
    initial_epoch: u64,
    rounds_per_epoch: u64,
    num_unstake_epochs: u64,
) -> SystemTransaction {
    // NOTES
    // * Create resources before packages to avoid circular dependencies.

    let mut id_allocator = ManifestIdAllocator::new();
    let mut instructions = Vec::new();
    let mut pre_allocated_ids = BTreeSet::new();

    // Package Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalObject(PACKAGE_PACKAGE.into()));
        let package_address = PACKAGE_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                native_package_code_id: PACKAGE_CODE_ID,
                schema: PackageNativePackage::schema(),
                dependent_resources: vec![],
                dependent_components: vec![],
                metadata: BTreeMap::new(),
                access_rules: AccessRulesConfig::new(),
                package_access_rules: PackageNativePackage::function_access_rules(),
                default_package_access_rule: AccessRule::DenyAll,
            }),
        });
    }

    // Metadata Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalObject(METADATA_PACKAGE.into()));
        let package_address = METADATA_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                native_package_code_id: METADATA_CODE_ID,
                schema: MetadataNativePackage::schema(),
                dependent_resources: vec![],
                dependent_components: vec![],
                metadata: BTreeMap::new(),
                access_rules: AccessRulesConfig::new(),
                package_access_rules: MetadataNativePackage::function_access_rules(),
                default_package_access_rule: AccessRule::DenyAll,
            }),
        });
    }

    // Royalty Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalObject(ROYALTY_PACKAGE.into()));
        let package_address = ROYALTY_PACKAGE.to_array_without_entity_id();

        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                native_package_code_id: ROYALTY_CODE_ID,
                schema: RoyaltyNativePackage::schema(),
                dependent_resources: vec![RADIX_TOKEN],
                dependent_components: vec![],
                metadata: BTreeMap::new(),
                access_rules: AccessRulesConfig::new(),
                package_access_rules: RoyaltyNativePackage::function_access_rules(),
                default_package_access_rule: AccessRule::DenyAll,
            }),
        });
    }

    // Access Rules Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalObject(ACCESS_RULES_PACKAGE.into()));
        let package_address = ACCESS_RULES_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                native_package_code_id: ACCESS_RULES_CODE_ID,
                schema: AccessRulesNativePackage::schema(),
                dependent_resources: vec![],
                dependent_components: vec![],
                metadata: BTreeMap::new(),
                access_rules: AccessRulesConfig::new(),
                package_access_rules: AccessRulesNativePackage::function_access_rules(),
                default_package_access_rule: AccessRule::DenyAll,
            }),
        });
    }

    // Resource Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalObject(RESOURCE_MANAGER_PACKAGE.into()));
        let package_address = RESOURCE_MANAGER_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                native_package_code_id: RESOURCE_MANAGER_CODE_ID,
                schema: ResourceManagerNativePackage::schema(),
                dependent_resources: vec![],
                dependent_components: vec![],
                metadata: BTreeMap::new(),
                access_rules: AccessRulesConfig::new(),
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::AllowAll,
            }),
        });
    }

    // XRD Token
    {
        let mut metadata = BTreeMap::new();
        metadata.insert("symbol".to_owned(), XRD_SYMBOL.to_owned());
        metadata.insert("name".to_owned(), XRD_NAME.to_owned());
        metadata.insert("description".to_owned(), XRD_DESCRIPTION.to_owned());
        metadata.insert("url".to_owned(), XRD_URL.to_owned());

        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let initial_supply: Decimal = XRD_MAX_SUPPLY.into();
        let resource_address = RADIX_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalObject(RADIX_TOKEN.into()));
        instructions.push(Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT
                .to_string(),
            args: to_manifest_value(
                &FungibleResourceManagerCreateWithInitialSupplyAndAddressInput {
                    divisibility: 18,
                    metadata,
                    access_rules,
                    initial_supply,
                    resource_address,
                },
            ),
        });
    }

    // Package Token
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = PACKAGE_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalObject(PACKAGE_TOKEN.into()));
        instructions.push(Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
            args: to_manifest_value(&NonFungibleResourceManagerCreateWithAddressInput {
                id_type: NonFungibleIdType::Bytes,
                non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                metadata,
                access_rules,
                resource_address,
            }),
        });
    }

    // Identity Package
    {
        // TODO: Integrate this into package instantiation to remove circular depedendency
        let mut access_rules = BTreeMap::new();
        let local_id =
            NonFungibleLocalId::bytes(scrypto_encode(&IDENTITY_PACKAGE).unwrap()).unwrap();
        let global_id = NonFungibleGlobalId::new(PACKAGE_TOKEN, local_id);
        access_rules.insert(Mint, (rule!(require(global_id)), rule!(deny_all)));
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = IDENTITY_OWNER_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalObject(IDENTITY_OWNER_TOKEN.into()));
        instructions.push(Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
            args: to_manifest_value(&NonFungibleResourceManagerCreateWithAddressInput {
                id_type: NonFungibleIdType::UUID,
                non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                metadata: btreemap!(),
                access_rules,
                resource_address,
            }),
        });

        pre_allocated_ids.insert(RENodeId::GlobalObject(IDENTITY_PACKAGE.into()));
        let package_address = IDENTITY_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                schema: IdentityNativePackage::schema(),
                dependent_resources: vec![
                    ECDSA_SECP256K1_TOKEN,
                    EDDSA_ED25519_TOKEN,
                    IDENTITY_OWNER_TOKEN,
                    PACKAGE_TOKEN,
                ],
                dependent_components: vec![],
                native_package_code_id: IDENTITY_CODE_ID,
                metadata: BTreeMap::new(),
                access_rules: AccessRulesConfig::new(),
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::AllowAll,
            }),
        });
    }

    // EpochManager Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalObject(EPOCH_MANAGER_PACKAGE.into()));
        let package_address = EPOCH_MANAGER_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                schema: EpochManagerNativePackage::schema(),
                native_package_code_id: EPOCH_MANAGER_CODE_ID,
                metadata: BTreeMap::new(),
                access_rules: AccessRulesConfig::new(),
                dependent_resources: vec![RADIX_TOKEN, PACKAGE_TOKEN, SYSTEM_TOKEN],
                dependent_components: vec![],
                package_access_rules: EpochManagerNativePackage::package_access_rules(),
                default_package_access_rule: AccessRule::DenyAll,
            }),
        });
    }

    // Clock Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalObject(CLOCK_PACKAGE.into()));
        let package_address = CLOCK_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                schema: ClockNativePackage::schema(),
                native_package_code_id: CLOCK_CODE_ID,
                metadata: BTreeMap::new(),
                access_rules: AccessRulesConfig::new(),
                dependent_resources: vec![SYSTEM_TOKEN],
                dependent_components: vec![],
                package_access_rules: ClockNativePackage::package_access_rules(),
                default_package_access_rule: AccessRule::DenyAll,
            }),
        });
    }

    // Account Package
    {
        // TODO: Integrate this into package instantiation to remove circular depedendency
        let mut access_rules = BTreeMap::new();
        let global_id = NonFungibleGlobalId::package_actor(ACCOUNT_PACKAGE);
        access_rules.insert(Mint, (rule!(require(global_id)), rule!(deny_all)));
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = ACCOUNT_OWNER_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalObject(ACCOUNT_OWNER_TOKEN.into()));
        instructions.push(Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
            args: to_manifest_value(&NonFungibleResourceManagerCreateWithAddressInput {
                id_type: NonFungibleIdType::UUID,
                non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                metadata: btreemap!(),
                access_rules,
                resource_address,
            }),
        });

        pre_allocated_ids.insert(RENodeId::GlobalObject(ACCOUNT_PACKAGE.into()));
        let package_address = ACCOUNT_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                schema: AccountNativePackage::schema(),
                native_package_code_id: ACCOUNT_CODE_ID,
                metadata: BTreeMap::new(),
                access_rules: AccessRulesConfig::new(),
                dependent_resources: vec![
                    ECDSA_SECP256K1_TOKEN,
                    EDDSA_ED25519_TOKEN,
                    ACCOUNT_OWNER_TOKEN,
                    PACKAGE_TOKEN,
                ],
                dependent_components: vec![],
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::AllowAll,
            }),
        });
    }

    // AccessController Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalObject(ACCESS_CONTROLLER_PACKAGE.into()));
        let package_address = ACCESS_CONTROLLER_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                schema: AccessControllerNativePackage::schema(),
                metadata: BTreeMap::new(),
                access_rules: AccessRulesConfig::new(),
                native_package_code_id: ACCESS_CONTROLLER_CODE_ID,
                dependent_resources: vec![PACKAGE_TOKEN],
                dependent_components: vec![CLOCK],
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::AllowAll,
            }),
        });
    }

    // TransactionProcessor Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalObject(TRANSACTION_PROCESSOR_PACKAGE.into()));
        let package_address = TRANSACTION_PROCESSOR_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                schema: TransactionProcessorNativePackage::schema(),
                metadata: BTreeMap::new(),
                access_rules: AccessRulesConfig::new(),
                native_package_code_id: TRANSACTION_PROCESSOR_CODE_ID,
                dependent_resources: vec![],
                dependent_components: vec![],
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::AllowAll,
            }),
        });
    }

    // AuthZone Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalObject(AUTH_ZONE_PACKAGE.into()));
        let package_address = AUTH_ZONE_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_NATIVE_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                schema: AuthZoneNativePackage::schema(),
                metadata: BTreeMap::new(),
                access_rules: AccessRulesConfig::new(),
                native_package_code_id: AUTH_ZONE_CODE_ID,
                dependent_resources: vec![],
                dependent_components: vec![],
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::DenyAll,
            }),
        });
    }

    // ECDSA
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = ECDSA_SECP256K1_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalObject(ECDSA_SECP256K1_TOKEN.into()));
        instructions.push(Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
            args: to_manifest_value(&NonFungibleResourceManagerCreateWithAddressInput {
                id_type: NonFungibleIdType::Bytes,
                non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                metadata,
                access_rules,
                resource_address,
            }),
        });
    }

    // EDDSA ED25519 Token
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = EDDSA_ED25519_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalObject(EDDSA_ED25519_TOKEN.into()));
        instructions.push(Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
            args: to_manifest_value(&NonFungibleResourceManagerCreateWithAddressInput {
                id_type: NonFungibleIdType::Bytes,
                non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                metadata,
                access_rules,
                resource_address,
            }),
        });
    }

    // System Token
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = SYSTEM_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalObject(SYSTEM_TOKEN.into()));
        instructions.push(Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT.to_string(),
            args: to_manifest_value(&NonFungibleResourceManagerCreateWithAddressInput {
                id_type: NonFungibleIdType::Bytes,
                non_fungible_schema: NonFungibleDataSchema::new_schema::<()>(),
                metadata,
                access_rules,
                resource_address,
            }),
        });
    }

    // Faucet Package
    {
        let faucet_code = include_bytes!("../../../assets/faucet.wasm").to_vec();
        let faucet_abi = include_bytes!("../../../assets/faucet.schema").to_vec();
        let package_address = FAUCET_PACKAGE.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalObject(FAUCET_PACKAGE.into()));
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishWasmInput {
                package_address: Some(package_address),
                code: faucet_code,
                schema: scrypto_decode(&faucet_abi).unwrap(),
                royalty_config: BTreeMap::new(),
                metadata: BTreeMap::new(),
                access_rules: AccessRulesConfig::new()
                    .default(AccessRule::DenyAll, AccessRule::DenyAll),
            }),
        });
    }

    // Clock Component
    {
        let component_address = CLOCK.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalObject(CLOCK.into()));
        instructions.push(Instruction::CallFunction {
            package_address: CLOCK_PACKAGE,
            blueprint_name: CLOCK_BLUEPRINT.to_string(),
            function_name: CLOCK_CREATE_IDENT.to_string(),
            args: to_manifest_value(&ClockCreateInput { component_address }),
        });
    }

    {
        let mut validators = BTreeMap::new();
        for (key, (amount, stake_account_address)) in validator_set_and_stake_owners {
            let initial_stake = id_allocator.new_bucket_id().unwrap();
            instructions.push(
                Instruction::TakeFromWorktopByAmount {
                    resource_address: RADIX_TOKEN,
                    amount,
                }
                .into(),
            );
            let validator_account_address = ComponentAddress::virtual_account_from_public_key(&key);
            validators.insert(
                key,
                ManifestValidatorInit {
                    validator_account_address,
                    initial_stake,
                    stake_account_address,
                },
            );
        }

        let component_address = EPOCH_MANAGER.to_array_without_entity_id();
        let olympia_validator_token_address = VALIDATOR_OWNER_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalObject(VALIDATOR_OWNER_TOKEN.into()));
        pre_allocated_ids.insert(RENodeId::GlobalObject(EPOCH_MANAGER.into()));
        instructions.push(Instruction::CallFunction {
            package_address: EPOCH_MANAGER_PACKAGE,
            blueprint_name: EPOCH_MANAGER_BLUEPRINT.to_string(),
            function_name: EPOCH_MANAGER_CREATE_IDENT.to_string(),
            args: manifest_args!(
                olympia_validator_token_address,
                component_address,
                validators,
                initial_epoch,
                rounds_per_epoch,
                num_unstake_epochs
            ),
        });
    }

    for (public_key, amount) in account_xrd_allocations.into_iter() {
        let bucket_id = id_allocator.new_bucket_id().unwrap();
        instructions.push(
            Instruction::TakeFromWorktopByAmount {
                resource_address: RADIX_TOKEN,
                amount,
            }
            .into(),
        );
        let component_address = ComponentAddress::virtual_account_from_public_key(
            &PublicKey::EcdsaSecp256k1(public_key),
        );
        instructions.push(
            Instruction::CallMethod {
                component_address: component_address,
                method_name: "deposit".to_string(),
                args: manifest_args!(bucket_id),
            }
            .into(),
        );
    }

    // Faucet
    {
        instructions.push(
            Instruction::TakeFromWorktop {
                resource_address: RADIX_TOKEN,
            }
            .into(),
        );

        let bucket = id_allocator.new_bucket_id().unwrap();
        instructions.push(Instruction::CallFunction {
            package_address: FAUCET_PACKAGE,
            blueprint_name: FAUCET_BLUEPRINT.to_string(),
            function_name: "new".to_string(),
            args: manifest_args!(bucket),
        });
    };

    SystemTransaction {
        instructions,
        blobs: Vec::new(),
        pre_allocated_ids,
        nonce: 0,
    }
}

pub fn genesis_result(receipt: &TransactionReceipt) -> GenesisReceipt {
    // TODO: Remove this when appropriate syscalls are implemented for Scrypto
    let faucet_component = receipt
        .expect_commit(true)
        .new_component_addresses()
        .last()
        .unwrap()
        .clone();
    GenesisReceipt { faucet_component }
}

pub fn bootstrap<S, W>(
    substate_store: &mut S,
    scrypto_interpreter: &ScryptoInterpreter<W>,
) -> Option<TransactionReceipt>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine,
{
    bootstrap_with_validator_set(
        substate_store,
        scrypto_interpreter,
        BTreeMap::new(),
        BTreeMap::new(),
        1u64,
        1u64,
        1u64,
    )
}

pub fn bootstrap_with_validator_set<S, W>(
    substate_store: &mut S,
    scrypto_interpreter: &ScryptoInterpreter<W>,
    validator_set_and_stake_owners: BTreeMap<EcdsaSecp256k1PublicKey, (Decimal, ComponentAddress)>,
    account_xrd_allocations: BTreeMap<EcdsaSecp256k1PublicKey, Decimal>,
    initial_epoch: u64,
    rounds_per_epoch: u64,
    num_unstake_epochs: u64,
) -> Option<TransactionReceipt>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine,
{
    if substate_store
        .get_substate(&SubstateId(
            RENodeId::GlobalObject(RADIX_TOKEN.into()),
            NodeModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .is_none()
    {
        let genesis_transaction = create_genesis(
            validator_set_and_stake_owners,
            account_xrd_allocations,
            initial_epoch,
            rounds_per_epoch,
            num_unstake_epochs,
        );

        let transaction_receipt = execute_transaction(
            substate_store,
            scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::genesis(),
            &genesis_transaction.get_executable(vec![AuthAddresses::system_role()]),
        );

        let commit_result = transaction_receipt.expect_commit(true);
        commit_result.state_updates.commit(substate_store);

        Some(transaction_receipt)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ledger::TypedInMemorySubstateStore, wasm::DefaultWasmEngine};
    use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

    #[test]
    fn bootstrap_receipt_should_match_constants() {
        let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
        let substate_store = TypedInMemorySubstateStore::new();
        let mut initial_validator_set = BTreeMap::new();
        let public_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
        let account_address = ComponentAddress::virtual_account_from_public_key(&public_key);
        initial_validator_set.insert(
            EcdsaSecp256k1PublicKey([0; 33]),
            (Decimal::one(), account_address),
        );
        let genesis_transaction =
            create_genesis(initial_validator_set, BTreeMap::new(), 1u64, 1u64, 1u64);

        let transaction_receipt = execute_transaction(
            &substate_store,
            &scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::genesis().with_trace(true),
            &genesis_transaction.get_executable(vec![AuthAddresses::system_role()]),
        );
        #[cfg(not(feature = "alloc"))]
        println!("{:?}", transaction_receipt);

        transaction_receipt
            .expect_commit(true)
            .next_epoch()
            .expect("There should be a new epoch.");

        assert!(transaction_receipt
            .expect_commit(true)
            .new_package_addresses()
            .contains(&PACKAGE_PACKAGE));
        let genesis_receipt = genesis_result(&transaction_receipt);
        assert_eq!(genesis_receipt.faucet_component, FAUCET_COMPONENT);
    }

    #[test]
    fn test_genesis_xrd_allocation_to_accounts() {
        let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
        let mut substate_store = TypedInMemorySubstateStore::new();
        let account_public_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
        let account_component_address = ComponentAddress::virtual_account_from_public_key(
            &PublicKey::EcdsaSecp256k1(account_public_key.clone()),
        );
        let allocation_amount = dec!("100");
        let mut account_xrd_allocations = BTreeMap::new();
        account_xrd_allocations.insert(account_public_key, allocation_amount);
        let genesis_transaction =
            create_genesis(BTreeMap::new(), account_xrd_allocations, 1u64, 1u64, 1u64);

        let transaction_receipt = execute_transaction(
            &substate_store,
            &scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::genesis(),
            &genesis_transaction.get_executable(vec![AuthAddresses::system_role()]),
        );

        let commit_result = transaction_receipt.expect_commit(true);
        commit_result.state_updates.commit(&mut substate_store);

        assert!(transaction_receipt
            .execution_trace
            .resource_changes
            .iter()
            .flat_map(|(_, rc)| rc)
            .any(|rc| rc.amount == allocation_amount
                && rc.node_id == RENodeId::GlobalObject(account_component_address.into())));
    }

    #[test]
    fn test_encode_and_decode_validator_init() {
        let t = ManifestValidatorInit {
            validator_account_address: ComponentAddress::AccessController([0u8; 26]),
            initial_stake: ManifestBucket(1),
            stake_account_address: ComponentAddress::AccessController([0u8; 26]),
        };

        let bytes = manifest_encode(&t).unwrap();
        let decoded: ManifestValidatorInit = manifest_decode(&bytes).unwrap();
        assert_eq!(decoded, t);
    }
}
