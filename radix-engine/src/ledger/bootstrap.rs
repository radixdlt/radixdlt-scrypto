use crate::blueprints::access_controller::*;
use crate::blueprints::account::AccountNativePackage;
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct GenesisData {
    validators: Vec<GenesisValidator>,
    resources: Vec<GenesisResource>,
    accounts: Vec<ComponentAddress>,
    resource_balances: Vec<NonXrdResourceBalance>,
    xrd_balances: Vec<XrdBalance>,
    stakes: Vec<Stake>,
}

impl GenesisData {
    pub fn empty() -> GenesisData {
        GenesisData {
            validators: vec![],
            resources: vec![],
            accounts: vec![],
            resource_balances: vec![],
            xrd_balances: vec![],
            stakes: vec![],
        }
    }

    pub fn single_validator_and_staker(
        validator_key: EcdsaSecp256k1PublicKey,
        stake_amount_xrd: Decimal,
        account_address: ComponentAddress,
    ) -> GenesisData {
        let stakes = vec![
            Stake { validator_index: 0, account_index: 0, xrd_amount: stake_amount_xrd },
        ];
        GenesisData {
            validators: vec![validator_key.into()],
            resources: vec![],
            accounts: vec![account_address],
            resource_balances: vec![],
            xrd_balances: vec![],
            stakes,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct GenesisValidator {
    key: EcdsaSecp256k1PublicKey,
    component_address: ComponentAddress,
    allows_delegation: bool,
    is_registered: bool,
    metadata: Vec<(String, String)>,
}

impl From<EcdsaSecp256k1PublicKey> for GenesisValidator {
    fn from(key: EcdsaSecp256k1PublicKey) -> Self {
        let component_address = ComponentAddress::virtual_account_from_public_key(&key);
        GenesisValidator {
            key,
            component_address,
            allows_delegation: true,
            is_registered: true,
            metadata: vec![],
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct GenesisResource {
    address_bytes: [u8; 26],
    metadata: Vec<(String, String)>,
    owner_account_index: Option<usize>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct NonXrdResourceBalance {
    resource_index: usize,
    account_index: usize,
    amount: Decimal,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct XrdBalance {
    account_index: usize,
    amount: Decimal,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct Stake {
    validator_index: usize,
    account_index: usize,
    xrd_amount: Decimal,
}

pub fn create_genesis(
    genesis_data: GenesisData,
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
                dependent_resources: vec![PACKAGE_TOKEN, PACKAGE_OWNER_TOKEN],
                dependent_components: vec![],
                metadata: BTreeMap::new(),
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
        access_rules.insert(Withdraw, (rule!(deny_all), rule!(deny_all)));
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

    // Object Token
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(deny_all), rule!(deny_all)));
        let resource_address = GLOBAL_OBJECT_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalObject(GLOBAL_OBJECT_TOKEN.into()));
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

    // Package Owner Token
    {
        // TODO: Integrate this into package instantiation to remove circular depedendency
        let mut access_rules = BTreeMap::new();
        let local_id =
            NonFungibleLocalId::bytes(scrypto_encode(&PACKAGE_PACKAGE).unwrap()).unwrap();
        let global_id = NonFungibleGlobalId::new(PACKAGE_TOKEN, local_id);
        access_rules.insert(Mint, (rule!(require(global_id)), rule!(deny_all)));
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = PACKAGE_OWNER_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalObject(PACKAGE_OWNER_TOKEN.into()));
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
                native_package_code_id: TRANSACTION_PROCESSOR_CODE_ID,
                dependent_resources: vec![],
                dependent_components: vec![],
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::AllowAll,
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
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishWasmAdvancedInput {
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

    // Genesis helper package
    {
        let genesis_helper_code = include_bytes!("../../../assets/genesis_helper.wasm").to_vec();
        let genesis_helper_abi = include_bytes!("../../../assets/genesis_helper.schema").to_vec();
        let package_address = GENESIS_HELPER_PACKAGE.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalObject(GENESIS_HELPER_PACKAGE.into()));
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishWasmAdvancedInput {
                package_address: Some(package_address),
                code: genesis_helper_code,
                schema: scrypto_decode(&genesis_helper_abi).unwrap(),
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

    // Call the GenesisHelper to init the epoch manager/validators/resources
    {
        for resource in genesis_data.resources.iter() {
            let address_bytes = resource.address_bytes;
            let resource_address = ResourceAddress::Fungible(address_bytes.clone());
            pre_allocated_ids.insert(RENodeId::GlobalObject(Address::Resource(resource_address)));
        }
        let epoch_manager_component_address = EPOCH_MANAGER.to_array_without_entity_id();
        let olympia_validator_token_address = VALIDATOR_OWNER_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalObject(EPOCH_MANAGER.into()));
        pre_allocated_ids.insert(RENodeId::GlobalObject(VALIDATOR_OWNER_TOKEN.into()));

        let whole_lotta_xrd = id_allocator.new_bucket_id().unwrap();
        instructions.push(
            Instruction::TakeFromWorktop {
                resource_address: RADIX_TOKEN,
            }
            .into(),
        );
        instructions.push(Instruction::CallFunction {
            package_address: GENESIS_HELPER_PACKAGE,
            blueprint_name: "GenesisHelper".to_string(),
            function_name: "init".to_string(),
            args: manifest_args!(
                genesis_data,
                whole_lotta_xrd,
                olympia_validator_token_address,
                epoch_manager_component_address,
                initial_epoch,
                rounds_per_epoch,
                num_unstake_epochs
            ),
        });
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
    }

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
    bootstrap_with_genesis_data(
        substate_store,
        scrypto_interpreter,
        GenesisData::empty(),
        1u64,
        1u64,
        1u64,
    )
}

pub fn bootstrap_with_genesis_data<S, W>(
    substate_store: &mut S,
    scrypto_interpreter: &ScryptoInterpreter<W>,
    genesis_data: GenesisData,
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
            genesis_data,
            initial_epoch,
            rounds_per_epoch,
            num_unstake_epochs,
        );

        let transaction_receipt = execute_transaction(
            substate_store,
            scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::genesis(),
            &genesis_transaction.get_executable(btreeset![AuthAddresses::system_role()]),
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
    use crate::system::node_substates::PersistedSubstate;
    use crate::transaction::BalanceChange;
    use crate::types::ResourceAddress;
    use crate::{ledger::TypedInMemorySubstateStore, wasm::DefaultWasmEngine};
    use radix_engine_interface::api::node_modules::metadata::{MetadataEntry, MetadataValue};
    use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

    #[test]
    fn test_bootstrap_receipt_should_match_constants() {
        let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
        let substate_store = TypedInMemorySubstateStore::new();
        let validator_key = EcdsaSecp256k1PublicKey([0; 33]);
        let validator_address = ComponentAddress::virtual_account_from_public_key(&validator_key);
        let staker_address = ComponentAddress::virtual_account_from_public_key(
            &EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key(),
        );
        let stakes = vec![Stake { validator_index: 0, account_index: 0, xrd_amount: Decimal::one() }];
        let genesis_data = GenesisData {
            validators: vec![GenesisValidator {
                key: validator_key,
                component_address: validator_address,
                allows_delegation: true,
                is_registered: true,
                metadata: vec![],
            }],
            resources: vec![],
            accounts: vec![staker_address],
            resource_balances: vec![],
            xrd_balances: vec![],
            stakes,
        };
        let genesis_transaction = create_genesis(genesis_data, 1u64, 1u64, 1u64);

        let transaction_receipt = execute_transaction(
            &substate_store,
            &scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::genesis(),
            &genesis_transaction.get_executable(btreeset![AuthAddresses::system_role()]),
        );

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
        let xrd_balances = vec![XrdBalance { account_index: 0, amount: allocation_amount }];
        let genesis_data = GenesisData {
            validators: vec![],
            resources: vec![],
            accounts: vec![account_component_address],
            resource_balances: vec![],
            xrd_balances,
            stakes: vec![],
        };
        let genesis_transaction = create_genesis(genesis_data, 1u64, 1u64, 1u64);

        let transaction_receipt = execute_transaction(
            &substate_store,
            &scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::genesis(),
            &genesis_transaction.get_executable(btreeset![AuthAddresses::system_role()]),
        );

        let commit_result = transaction_receipt.expect_commit(true);
        commit_result.state_updates.commit(&mut substate_store);

        assert!(transaction_receipt
            .execution_trace
            .resource_changes
            .iter()
            .flat_map(|(_, rc)| rc)
            .any(|rc| rc.amount == allocation_amount
                && rc.node_id == RENodeId::GlobalObject(account_component_address.into())
                && rc.resource_address == RADIX_TOKEN));
    }

    #[test]
    fn test_genesis_resource_with_initial_allocation() {
        let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
        let mut substate_store = TypedInMemorySubstateStore::new();
        let tokenholder = ComponentAddress::virtual_account_from_public_key(
            &PublicKey::EcdsaSecp256k1(EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key()),
        );
        let allocation_amount = dec!("105");
        let address_bytes = hash(vec![1, 2, 3]).lower_26_bytes();
        let resource_address = ResourceAddress::Fungible(address_bytes);

        let owner = ComponentAddress::virtual_account_from_public_key(
            &EcdsaSecp256k1PrivateKey::from_u64(2).unwrap().public_key(),
        );

        let metadata = vec![("symbol".to_string(), "TST".to_string())];
        let genesis_resource = GenesisResource {
            address_bytes,
            metadata,
            owner_account_index: Some(1),
        };
        let resource_balances = vec![NonXrdResourceBalance { resource_index: 0, account_index: 0, amount: allocation_amount }];
        let genesis_data = GenesisData {
            resources: vec![genesis_resource],
            validators: vec![],
            accounts: vec![tokenholder.clone(), owner],
            resource_balances,
            xrd_balances: vec![],
            stakes: vec![],
        };

        let genesis_transaction = create_genesis(genesis_data, 1u64, 1u64, 1u64);

        let transaction_receipt = execute_transaction(
            &substate_store,
            &scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::genesis(),
            &genesis_transaction.get_executable(btreeset![AuthAddresses::system_role()]),
        );

        let commit_result = transaction_receipt.expect_commit(true);
        commit_result.state_updates.commit(&mut substate_store);

        let persisted_resource_manager_substate: PersistedSubstate = substate_store
            .get_substate(&SubstateId(
                RENodeId::GlobalObject(Address::Resource(resource_address)),
                NodeModuleId::SELF,
                SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
            ))
            .map(|o| o.substate)
            .unwrap();

        if let PersistedSubstate::ResourceManager(resource_manager_substate) =
            persisted_resource_manager_substate
        {
            assert_eq!(resource_manager_substate.total_supply, dec!("105"));
        } else {
            panic!("Failed to get a resource manager substate")
        }

        let persisted_symbol_metadata_entry: PersistedSubstate = substate_store
            .get_substate(&SubstateId(
                RENodeId::GlobalObject(Address::Resource(resource_address)),
                NodeModuleId::Metadata,
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(
                    scrypto_encode("symbol").unwrap(),
                )),
            ))
            .map(|o| o.substate)
            .unwrap();

        if let PersistedSubstate::KeyValueStoreEntry(Some(value)) = persisted_symbol_metadata_entry
        {
            let entry: MetadataEntry = scrypto_decode(&scrypto_encode(&value).unwrap()).unwrap();
            if let MetadataEntry::Value(MetadataValue::String(symbol)) = entry {
                assert_eq!(symbol, "TST");
            } else {
                panic!("Resource symbol was not a string");
            }
        } else {
            panic!("Failed to get resource symbol metadata")
        }

        assert!(transaction_receipt
            .execution_trace
            .resource_changes
            .iter()
            .flat_map(|(_, rc)| rc)
            .any(|rc| rc.amount == allocation_amount
                && rc.node_id == RENodeId::GlobalObject(tokenholder.into())
                && rc.resource_address == resource_address));

        assert!(transaction_receipt
            .execution_trace
            .resource_changes
            .iter()
            .flat_map(|(_, rc)| rc)
            .any(|rc|
                // Not an ideal condition, but assuming this is the owner badge
                rc.amount == dec!("1")
                && rc.node_id == RENodeId::GlobalObject(owner.into())));
    }

    #[test]
    fn test_genesis_stake_allocation() {
        let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
        let mut substate_store = TypedInMemorySubstateStore::new();

        // There are two genesis validators
        // - one with two stakers (0 and 1)
        // - one with one staker (just 1)
        let validator_0: GenesisValidator = EcdsaSecp256k1PrivateKey::from_u64(10)
            .unwrap()
            .public_key()
            .into();
        let validator_1: GenesisValidator = EcdsaSecp256k1PrivateKey::from_u64(11)
            .unwrap()
            .public_key()
            .into();

        let staker_0 = ComponentAddress::virtual_account_from_public_key(
            &EcdsaSecp256k1PrivateKey::from_u64(4).unwrap().public_key(),
        );

        let staker_1 = ComponentAddress::virtual_account_from_public_key(
            &EcdsaSecp256k1PrivateKey::from_u64(5).unwrap().public_key(),
        );

        let stakes = vec![
            Stake { validator_index: 0, account_index: 0, xrd_amount: dec!("10") },
            Stake { validator_index: 0, account_index: 1, xrd_amount: dec!("50000") },
            Stake { validator_index: 1, account_index: 1, xrd_amount: dec!("1") },
        ];
        let genesis_data = GenesisData {
            resources: vec![],
            validators: vec![validator_0, validator_1],
            accounts: vec![staker_0.clone(), staker_1.clone()],
            resource_balances: vec![],
            xrd_balances: vec![],
            stakes,
        };

        let genesis_transaction = create_genesis(genesis_data, 1u64, 1u64, 1u64);

        let transaction_receipt = execute_transaction(
            &substate_store,
            &scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::genesis(),
            &genesis_transaction.get_executable(btreeset![AuthAddresses::system_role()]),
        );

        let commit_result = transaction_receipt.expect_commit(true);
        commit_result.state_updates.commit(&mut substate_store);

        // Staker 0 should have one liquidity balance entry
        {
            let balances = commit_result
                .state_update_summary
                .balance_changes
                .get(&Address::Component(staker_0))
                .unwrap();
            assert!(balances.len() == 1);
            assert!(balances
                .values()
                .any(|bal| *bal == BalanceChange::Fungible(dec!("10"))));
        }

        // Staker 1 should have two liquidity balance entries
        {
            let balances = commit_result
                .state_update_summary
                .balance_changes
                .get(&Address::Component(staker_1))
                .unwrap();
            assert!(balances.len() == 2);
            assert!(balances
                .values()
                .any(|bal| *bal == BalanceChange::Fungible(dec!("1"))));
            assert!(balances
                .values()
                .any(|bal| *bal == BalanceChange::Fungible(dec!("50000"))));
        }
    }
}
