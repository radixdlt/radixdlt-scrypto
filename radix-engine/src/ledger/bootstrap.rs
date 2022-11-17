use crate::engine::ScryptoInterpreter;
use crate::fee::SystemLoanFeeReserve;
use crate::ledger::{ReadableSubstateStore, WriteableSubstateStore};
use crate::transaction::{
    execute_transaction_with_fee_reserve, ExecutionConfig, TransactionReceipt,
};
use crate::types::ResourceMethodAuthKey::Withdraw;
use crate::types::*;
use crate::wasm::{DefaultWasmEngine, InstructionCostRules, WasmInstrumenter, WasmMeteringConfig};
use radix_engine_constants::GENESIS_CREATION_CREDIT;
use radix_engine_lib::crypto::hash;
use radix_engine_lib::engine::types::{
    EpochManagerFunction, GlobalAddress, NativeFunctionIdent, RENodeId, ResourceManagerFunction,
    ResourceManagerOffset, ScryptoFunctionIdent, ScryptoPackage, SubstateId, SubstateOffset,
};
use radix_engine_lib::model::*;
use scrypto::rule;

use transaction::model::{Instruction, SystemTransaction, TransactionManifest};
use transaction::validation::{IdAllocator, IdSpace};

const XRD_SYMBOL: &str = "XRD";
const XRD_NAME: &str = "Radix";
const XRD_DESCRIPTION: &str = "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.";
const XRD_URL: &str = "https://tokens.radixdlt.com";
const XRD_MAX_SUPPLY: i128 = 1_000_000_000_000i128;

pub struct GenesisReceipt {
    pub faucet_package: PackageAddress,
    pub account_package: PackageAddress,
    pub ecdsa_secp256k1_token: ResourceAddress,
    pub system_token: ResourceAddress,
    pub xrd_token: ResourceAddress,
    pub faucet_component: ComponentAddress,
    pub epoch_manager: SystemAddress,
    pub eddsa_ed25519_token: ResourceAddress,
}

pub fn create_genesis() -> SystemTransaction {
    let mut blobs = Vec::new();
    let mut id_allocator = IdAllocator::new(IdSpace::Transaction);
    let create_faucet_package = {
        let faucet_code = include_bytes!("../../../assets/faucet.wasm").to_vec();
        let faucet_abi = include_bytes!("../../../assets/faucet.abi").to_vec();
        let inst = Instruction::PublishPackage {
            code: Blob(hash(&faucet_code)),
            abi: Blob(hash(&faucet_abi)),
        };

        blobs.push(faucet_code);
        blobs.push(faucet_abi);

        inst
    };
    let create_account_package = {
        let account_code = include_bytes!("../../../assets/account.wasm").to_vec();
        let account_abi = include_bytes!("../../../assets/account.abi").to_vec();
        let inst = Instruction::PublishPackage {
            code: Blob(hash(&account_code)),
            abi: Blob(hash(&account_abi)),
        };

        blobs.push(account_code);
        blobs.push(account_abi);

        inst
    };
    let create_ecdsa_secp256k1_token = {
        let metadata: HashMap<String, String> = HashMap::new();
        let mut access_rules = HashMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), LOCKED));
        let initial_supply: Option<MintParams> = None;

        // TODO: Create token at a specific address
        Instruction::CallNativeFunction {
            function_ident: NativeFunctionIdent {
                blueprint_name: RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: ResourceManagerFunction::Create.to_string(),
            },
            args: args!(
                ResourceType::NonFungible,
                metadata,
                access_rules,
                initial_supply
            ),
        }
    };

    // TODO: Perhaps combine with ecdsa token?
    let create_system_token = {
        let metadata: HashMap<String, String> = HashMap::new();
        let mut access_rules: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)> =
            HashMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), LOCKED));
        let initial_supply: Option<MintParams> = None;

        // TODO: Create token at a specific address
        Instruction::CallNativeFunction {
            function_ident: NativeFunctionIdent {
                blueprint_name: RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: ResourceManagerFunction::Create.to_string(),
            },
            args: args!(
                ResourceType::NonFungible,
                metadata,
                access_rules,
                initial_supply
            ),
        }
    };

    let create_xrd_token = {
        let mut metadata = HashMap::new();
        metadata.insert("symbol".to_owned(), XRD_SYMBOL.to_owned());
        metadata.insert("name".to_owned(), XRD_NAME.to_owned());
        metadata.insert("description".to_owned(), XRD_DESCRIPTION.to_owned());
        metadata.insert("url".to_owned(), XRD_URL.to_owned());

        let mut access_rules = HashMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), LOCKED));

        let initial_supply: Option<MintParams> = Option::Some(MintParams::Fungible {
            amount: XRD_MAX_SUPPLY.into(),
        });

        Instruction::CallNativeFunction {
            function_ident: NativeFunctionIdent {
                blueprint_name: RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: ResourceManagerFunction::Create.to_string(),
            },
            args: args!(
                ResourceType::Fungible { divisibility: 18 },
                metadata,
                access_rules,
                initial_supply
            ),
        }
    };

    let take_xrd = Instruction::TakeFromWorktop {
        resource_address: RADIX_TOKEN,
    };

    let create_xrd_faucet = {
        let bucket = Bucket(id_allocator.new_bucket_id().unwrap());
        Instruction::CallFunction {
            function_ident: ScryptoFunctionIdent {
                package: ScryptoPackage::Global(SYS_FAUCET_PACKAGE),
                blueprint_name: FAUCET_BLUEPRINT.to_string(),
                function_name: "new".to_string(),
            },
            args: args!(bucket),
        }
    };

    let create_epoch_manager = {
        Instruction::CallNativeFunction {
            function_ident: NativeFunctionIdent {
                blueprint_name: EPOCH_MANAGER_BLUEPRINT.to_string(),
                function_name: EpochManagerFunction::Create.to_string(),
            },
            args: args!(),
        }
    };

    let create_eddsa_ed25519_token = {
        let metadata: HashMap<String, String> = HashMap::new();
        let mut access_rules = HashMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), LOCKED));
        let initial_supply: Option<MintParams> = None;

        // TODO: Create token at a specific address
        Instruction::CallNativeFunction {
            function_ident: NativeFunctionIdent {
                blueprint_name: RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: ResourceManagerFunction::Create.to_string(),
            },
            args: args!(
                ResourceType::NonFungible,
                metadata,
                access_rules,
                initial_supply
            ),
        }
    };

    let manifest = TransactionManifest {
        instructions: vec![
            create_faucet_package,
            create_account_package,
            create_ecdsa_secp256k1_token,
            create_system_token,
            create_xrd_token,
            take_xrd,
            create_xrd_faucet,
            create_epoch_manager,
            create_eddsa_ed25519_token,
        ],
        blobs,
    };

    SystemTransaction { manifest }
}

pub fn genesis_result(invoke_result: &Vec<Vec<u8>>) -> GenesisReceipt {
    let faucet_package: PackageAddress = scrypto_decode(&invoke_result[0]).unwrap();
    let account_package: PackageAddress = scrypto_decode(&invoke_result[1]).unwrap();
    let (ecdsa_secp256k1_token, _bucket): (ResourceAddress, Option<Bucket>) =
        scrypto_decode(&invoke_result[2]).unwrap();
    let (system_token, _bucket): (ResourceAddress, Option<Bucket>) =
        scrypto_decode(&invoke_result[3]).unwrap();
    let (xrd_token, _bucket): (ResourceAddress, Option<Bucket>) =
        scrypto_decode(&invoke_result[4]).unwrap();
    let faucet_component: ComponentAddress = scrypto_decode(&invoke_result[6]).unwrap();
    let epoch_manager: SystemAddress = scrypto_decode(&invoke_result[7]).unwrap();
    let (eddsa_ed25519_token, _bucket): (ResourceAddress, Option<Bucket>) =
        scrypto_decode(&invoke_result[8]).unwrap();

    GenesisReceipt {
        faucet_package,
        account_package,
        ecdsa_secp256k1_token,
        system_token,
        xrd_token,
        faucet_component,
        epoch_manager,
        eddsa_ed25519_token,
    }
}

pub fn bootstrap<S>(substate_store: &mut S) -> Option<TransactionReceipt>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
{
    if substate_store
        .get_substate(&SubstateId(
            RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)),
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
        ))
        .is_none()
    {
        let scrypto_interpreter = ScryptoInterpreter {
            wasm_engine: DefaultWasmEngine::default(),
            wasm_instrumenter: WasmInstrumenter::default(),
            wasm_metering_config: WasmMeteringConfig::new(
                InstructionCostRules::tiered(1, 5, 10, 5000),
                512,
            ),
        };

        let genesis_transaction = create_genesis();
        let mut fee_reserve = SystemLoanFeeReserve::default();
        fee_reserve.credit(GENESIS_CREATION_CREDIT);

        let transaction_receipt = execute_transaction_with_fee_reserve(
            substate_store,
            &scrypto_interpreter,
            fee_reserve,
            &ExecutionConfig::standard(),
            &genesis_transaction.get_executable(),
        );

        let commit_result = transaction_receipt.expect_commit();
        commit_result.outcome.expect_success();
        commit_result.state_updates.commit(substate_store);

        Some(transaction_receipt)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::TypedInMemorySubstateStore;

    use super::*;

    #[test]
    fn bootstrap_receipt_should_match_constants() {
        let wasm_engine = DefaultWasmEngine::default();
        let wasm_instrumenter = WasmInstrumenter::default();
        let wasm_metering_config =
            WasmMeteringConfig::new(InstructionCostRules::tiered(1, 5, 10, 5000), 512);
        let scrypto_interpreter = ScryptoInterpreter {
            wasm_engine,
            wasm_instrumenter,
            wasm_metering_config,
        };
        let substate_store = TypedInMemorySubstateStore::new();
        let genesis_transaction = create_genesis();
        let mut fee_reserve = SystemLoanFeeReserve::default();
        fee_reserve.credit(GENESIS_CREATION_CREDIT);

        let transaction_receipt = execute_transaction_with_fee_reserve(
            &substate_store,
            &scrypto_interpreter,
            fee_reserve,
            &ExecutionConfig::standard(),
            &genesis_transaction.get_executable(),
        );

        let commit_result = transaction_receipt.expect_commit();
        let invoke_result = commit_result.outcome.expect_success();
        let genesis_receipt = genesis_result(&invoke_result);

        assert_eq!(genesis_receipt.faucet_package, SYS_FAUCET_PACKAGE);
        assert_eq!(genesis_receipt.account_package, ACCOUNT_PACKAGE);
        assert_eq!(genesis_receipt.ecdsa_secp256k1_token, ECDSA_SECP256K1_TOKEN);
        assert_eq!(genesis_receipt.system_token, SYSTEM_TOKEN);
        assert_eq!(genesis_receipt.xrd_token, RADIX_TOKEN);
        assert_eq!(genesis_receipt.faucet_component, FAUCET_COMPONENT);
        assert_eq!(genesis_receipt.epoch_manager, EPOCH_MANAGER);
        assert_eq!(genesis_receipt.eddsa_ed25519_token, EDDSA_ED25519_TOKEN);
    }
}
