use crate::engine::ScryptoInterpreter;
use crate::ledger::{ReadableSubstateStore, WriteableSubstateStore};
use crate::transaction::{
    execute_transaction, ExecutionConfig, FeeReserveConfig, TransactionReceipt,
};
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::{
    GlobalAddress, RENodeId, ResourceManagerOffset, SubstateId, SubstateOffset,
};
use radix_engine_interface::crypto::hash;
use radix_engine_interface::data::*;
use radix_engine_interface::model::*;
use radix_engine_interface::modules::auth::AuthAddresses;
use radix_engine_interface::rule;
use transaction::model::{BasicInstruction, Instruction, SystemTransaction};
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
    pub clock: SystemAddress,
    pub eddsa_ed25519_token: ResourceAddress,
    pub package_token: ResourceAddress,
    pub epoch_manager: SystemAddress,
}

pub fn create_genesis(
    validator_set: BTreeSet<EcdsaSecp256k1PublicKey>,
    initial_epoch: u64,
    rounds_per_epoch: u64,
) -> SystemTransaction {
    let mocked_hash = hash([0u8; 1]);
    let mut blobs = Vec::new();
    let mut id_allocator = IdAllocator::new(IdSpace::Transaction, mocked_hash);
    let mut instructions = Vec::new();

    {
        let faucet_code = include_bytes!("../../../assets/faucet.wasm").to_vec();
        let faucet_abi = include_bytes!("../../../assets/faucet.abi").to_vec();
        instructions.push(Instruction::Basic(BasicInstruction::PublishPackage {
            code: Blob(hash(&faucet_code)),
            abi: Blob(hash(&faucet_abi)),
            royalty_config: BTreeMap::new(),
            metadata: BTreeMap::new(),
            access_rules: AccessRules::new().default(AccessRule::DenyAll, AccessRule::DenyAll),
        }));

        blobs.push(faucet_code);
        blobs.push(faucet_abi);
    }

    {
        let account_code = include_bytes!("../../../assets/account.wasm").to_vec();
        let account_abi = include_bytes!("../../../assets/account.abi").to_vec();
        instructions.push(Instruction::Basic(BasicInstruction::PublishPackage {
            code: Blob(hash(&account_code)),
            abi: Blob(hash(&account_abi)),
            royalty_config: BTreeMap::new(),
            metadata: BTreeMap::new(),
            access_rules: AccessRules::new().default(AccessRule::DenyAll, AccessRule::DenyAll),
        }));

        blobs.push(account_code);
        blobs.push(account_abi);
    }

    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceMethodAuthKey::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );

        // TODO: Create token at a specific address
        instructions.push(Instruction::Basic(
            BasicInstruction::CreateNonFungibleResource {
                id_type: NonFungibleIdType::Bytes,
                metadata,
                access_rules,
                initial_supply: None,
            },
        ));
    }

    // TODO: Perhaps combine with ecdsa token?
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceMethodAuthKey::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );
        let initial_supply = None;

        // TODO: Create token at a specific address
        instructions.push(Instruction::Basic(
            BasicInstruction::CreateNonFungibleResource {
                id_type: NonFungibleIdType::Bytes,
                metadata,
                access_rules,
                initial_supply,
            },
        ));
    }

    {
        let mut metadata = BTreeMap::new();
        metadata.insert("symbol".to_owned(), XRD_SYMBOL.to_owned());
        metadata.insert("name".to_owned(), XRD_NAME.to_owned());
        metadata.insert("description".to_owned(), XRD_DESCRIPTION.to_owned());
        metadata.insert("url".to_owned(), XRD_URL.to_owned());

        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceMethodAuthKey::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );

        let initial_supply: Option<Decimal> = Some(XRD_MAX_SUPPLY.into());
        instructions.push(Instruction::Basic(
            BasicInstruction::CreateFungibleResource {
                divisibility: 18,
                metadata,
                access_rules,
                initial_supply,
            },
        ));
    }

    instructions.push(
        BasicInstruction::TakeFromWorktop {
            resource_address: RADIX_TOKEN,
        }
        .into(),
    );

    {
        let bucket = Bucket(id_allocator.new_bucket_id().unwrap());
        instructions.push(Instruction::Basic(BasicInstruction::CallFunction {
            package_address: FAUCET_PACKAGE,
            blueprint_name: FAUCET_BLUEPRINT.to_string(),
            function_name: "new".to_string(),
            args: args!(bucket),
        }));
    };

    instructions.push(Instruction::System(NativeInvocation::Clock(
        ClockInvocation::Create(ClockCreateInvocation {}),
    )));

    // EDDSA ED25519 Token
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let initial_supply = None;

        // TODO: Create token at a specific address
        instructions.push(Instruction::Basic(
            BasicInstruction::CreateNonFungibleResource {
                id_type: NonFungibleIdType::Bytes,
                metadata,
                access_rules,
                initial_supply,
            },
        ));
    };

    // Package Token
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let initial_supply = None;

        // TODO: Create token at a specific address
        instructions.push(Instruction::Basic(
            BasicInstruction::CreateNonFungibleResource {
                id_type: NonFungibleIdType::Bytes,
                metadata,
                access_rules,
                initial_supply,
            },
        ));
    }

    instructions.push(Instruction::System(NativeInvocation::EpochManager(
        EpochManagerInvocation::Create(EpochManagerCreateInvocation {
            validator_set: validator_set.clone(),
            initial_epoch,
            rounds_per_epoch,
        }),
    )));

    SystemTransaction {
        instructions,
        blobs,
        nonce: 0,
    }
}

pub fn genesis_result(receipt: &TransactionReceipt) -> GenesisReceipt {
    let faucet_package: PackageAddress = receipt.output(0);
    let account_package: PackageAddress = receipt.output(1);
    let (ecdsa_secp256k1_token, _bucket): (ResourceAddress, Option<Bucket>) = receipt.output(2);
    let (system_token, _bucket): (ResourceAddress, Option<Bucket>) = receipt.output(3);
    let (xrd_token, _bucket): (ResourceAddress, Option<Bucket>) = receipt.output(4);
    let faucet_component: ComponentAddress = receipt.output(6);
    let clock: SystemAddress = receipt.output(7);
    let (eddsa_ed25519_token, _bucket): (ResourceAddress, Option<Bucket>) = receipt.output(8);
    let (package_token, _bucket): (ResourceAddress, Option<Bucket>) = receipt.output(9);
    let epoch_manager: SystemAddress = receipt.output(10);

    GenesisReceipt {
        faucet_package,
        account_package,
        ecdsa_secp256k1_token,
        system_token,
        xrd_token,
        faucet_component,
        clock,
        eddsa_ed25519_token,
        package_token,
        epoch_manager,
    }
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
        BTreeSet::new(),
        1u64,
        1u64,
    )
}

pub fn bootstrap_with_validator_set<S, W>(
    substate_store: &mut S,
    scrypto_interpreter: &ScryptoInterpreter<W>,
    validator_set: BTreeSet<EcdsaSecp256k1PublicKey>,
    initial_epoch: u64,
    rounds_per_epoch: u64,
) -> Option<TransactionReceipt>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine,
{
    if substate_store
        .get_substate(&SubstateId(
            RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)),
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
        ))
        .is_none()
    {
        let genesis_transaction = create_genesis(validator_set, initial_epoch, rounds_per_epoch);

        let transaction_receipt = execute_transaction(
            substate_store,
            scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::default(),
            &genesis_transaction.get_executable(vec![AuthAddresses::system_role()]),
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
    use crate::{ledger::TypedInMemorySubstateStore, wasm::DefaultWasmEngine};

    use super::*;

    #[test]
    fn bootstrap_receipt_should_match_constants() {
        let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
        let substate_store = TypedInMemorySubstateStore::new();
        let mut initial_validator_set = BTreeSet::new();
        initial_validator_set.insert(EcdsaSecp256k1PublicKey([0; 33]));
        let genesis_transaction = create_genesis(initial_validator_set.clone(), 1u64, 1u64);

        let transaction_receipt = execute_transaction(
            &substate_store,
            &scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::default(),
            &genesis_transaction.get_executable(vec![AuthAddresses::system_role()]),
        );

        let genesis_receipt = genesis_result(&transaction_receipt);

        assert_eq!(genesis_receipt.faucet_package, FAUCET_PACKAGE);
        assert_eq!(genesis_receipt.account_package, ACCOUNT_PACKAGE);
        assert_eq!(genesis_receipt.ecdsa_secp256k1_token, ECDSA_SECP256K1_TOKEN);
        assert_eq!(genesis_receipt.system_token, SYSTEM_TOKEN);
        assert_eq!(genesis_receipt.xrd_token, RADIX_TOKEN);
        assert_eq!(genesis_receipt.faucet_component, FAUCET_COMPONENT);
        assert_eq!(genesis_receipt.clock, CLOCK);
        assert_eq!(genesis_receipt.eddsa_ed25519_token, EDDSA_ED25519_TOKEN);
        assert_eq!(genesis_receipt.package_token, PACKAGE_TOKEN);
        assert_eq!(genesis_receipt.epoch_manager, EPOCH_MANAGER);
    }
}
