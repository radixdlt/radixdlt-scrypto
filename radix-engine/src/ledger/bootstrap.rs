use crate::engine::ScryptoInterpreter;
use crate::ledger::{ReadableSubstateStore, WriteableSubstateStore};
use crate::transaction::{
    execute_transaction, ExecutionConfig, FeeReserveConfig, TransactionReceipt,
};
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::{GlobalAddress, RENodeId, SubstateId, SubstateOffset};
use radix_engine_interface::data::*;
use radix_engine_interface::model::*;
use radix_engine_interface::modules::auth::AuthAddresses;
use radix_engine_interface::rule;
use transaction::model::{BasicInstruction, Instruction, SystemTransaction};
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

    // XRD
    {
        let mut metadata = BTreeMap::new();
        metadata.insert("symbol".to_owned(), XRD_SYMBOL.to_owned());
        metadata.insert("name".to_owned(), XRD_NAME.to_owned());
        metadata.insert("description".to_owned(), XRD_DESCRIPTION.to_owned());
        metadata.insert("url".to_owned(), XRD_URL.to_owned());

        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let initial_supply: Decimal = XRD_MAX_SUPPLY.into();
        let resource_address = RADIX_TOKEN.raw();
        pre_allocated_ids.insert(RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)));
        instructions.push(Instruction::System(NativeInvocation::ResourceManager(
            ResourceInvocation::CreateFungibleWithInitialSupply(
                ResourceManagerCreateFungibleWithInitialSupplyInvocation {
                    resource_address: Some(resource_address),
                    divisibility: 18,
                    metadata,
                    access_rules,
                    initial_supply,
                },
            ),
        )));
    }

    // ECDSA
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = ECDSA_SECP256K1_TOKEN.raw();
        pre_allocated_ids.insert(RENodeId::Global(GlobalAddress::Resource(
            ECDSA_SECP256K1_TOKEN,
        )));
        instructions.push(Instruction::System(NativeInvocation::ResourceManager(
            ResourceInvocation::CreateNonFungible(ResourceManagerCreateNonFungibleInvocation {
                resource_address: Some(resource_address),
                id_type: NonFungibleIdType::Bytes,
                metadata,
                access_rules,
            }),
        )));
    }

    // TODO: Perhaps combine with ecdsa token?
    // EDDSA ED25519 Token
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = EDDSA_ED25519_TOKEN.raw();
        pre_allocated_ids.insert(RENodeId::Global(GlobalAddress::Resource(
            EDDSA_ED25519_TOKEN,
        )));
        instructions.push(Instruction::System(NativeInvocation::ResourceManager(
            ResourceInvocation::CreateNonFungible(ResourceManagerCreateNonFungibleInvocation {
                resource_address: Some(resource_address),
                id_type: NonFungibleIdType::Bytes,
                metadata,
                access_rules,
            }),
        )));
    }

    // TODO: Perhaps combine with ecdsa token?
    // System Token
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = SYSTEM_TOKEN.raw();
        pre_allocated_ids.insert(RENodeId::Global(GlobalAddress::Resource(SYSTEM_TOKEN)));
        instructions.push(Instruction::System(NativeInvocation::ResourceManager(
            ResourceInvocation::CreateNonFungible(ResourceManagerCreateNonFungibleInvocation {
                resource_address: Some(resource_address),
                id_type: NonFungibleIdType::Bytes,
                metadata,
                access_rules,
            }),
        )));
    }

    // Package Token
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = PACKAGE_TOKEN.raw();
        pre_allocated_ids.insert(RENodeId::Global(GlobalAddress::Resource(PACKAGE_TOKEN)));
        instructions.push(Instruction::System(NativeInvocation::ResourceManager(
            ResourceInvocation::CreateNonFungible(ResourceManagerCreateNonFungibleInvocation {
                resource_address: Some(resource_address),
                id_type: NonFungibleIdType::Bytes,
                metadata,
                access_rules,
            }),
        )));
    }

    {
        let faucet_code = include_bytes!("../../../assets/faucet.wasm").to_vec();
        let faucet_abi = include_bytes!("../../../assets/faucet.abi").to_vec();
        let package_address = FAUCET_PACKAGE.raw();
        pre_allocated_ids.insert(RENodeId::Global(GlobalAddress::Package(FAUCET_PACKAGE)));
        instructions.push(Instruction::System(NativeInvocation::Package(
            PackageInvocation::Publish(PackagePublishInvocation {
                package_address: Some(package_address),
                code: faucet_code, // TODO: Use blob here instead?
                abi: faucet_abi,   // TODO: Use blob here instead?
                royalty_config: BTreeMap::new(),
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new().default(AccessRule::DenyAll, AccessRule::DenyAll),
            }),
        )));
    }

    {
        let account_code = include_bytes!("../../../assets/account.wasm").to_vec();
        let account_abi = include_bytes!("../../../assets/account.abi").to_vec();
        let package_address = ACCOUNT_PACKAGE.raw();
        pre_allocated_ids.insert(RENodeId::Global(GlobalAddress::Package(ACCOUNT_PACKAGE)));
        instructions.push(Instruction::System(NativeInvocation::Package(
            PackageInvocation::Publish(PackagePublishInvocation {
                package_address: Some(package_address),
                code: account_code, // TODO: Use blob here instead?
                abi: account_abi,   // TODO: Use blob here instead?
                royalty_config: BTreeMap::new(),
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new().default(AccessRule::DenyAll, AccessRule::DenyAll),
            }),
        )));
    }

    {
        let component_address = CLOCK.raw();
        pre_allocated_ids.insert(RENodeId::Global(GlobalAddress::Component(CLOCK)));
        instructions.push(Instruction::System(NativeInvocation::Clock(
            ClockInvocation::Create(ClockCreateInvocation { component_address }),
        )));
    }

    {
        let mut validators = BTreeMap::new();
        for (key, (amount, stake_account_address)) in validator_set_and_stake_owners {
            let bucket = Bucket(id_allocator.new_bucket_id().unwrap().0);
            instructions.push(
                BasicInstruction::TakeFromWorktopByAmount {
                    resource_address: RADIX_TOKEN,
                    amount,
                }
                .into(),
            );
            let validator_account_address = ComponentAddress::virtual_account_from_public_key(&key);
            validators.insert(
                key,
                ValidatorInit {
                    validator_account_address,
                    initial_stake: bucket,
                    stake_account_address,
                },
            );
        }

        let component_address = EPOCH_MANAGER.raw();
        let olympia_validator_token_address = OLYMPIA_VALIDATOR_TOKEN.raw();
        pre_allocated_ids.insert(RENodeId::Global(GlobalAddress::Resource(
            OLYMPIA_VALIDATOR_TOKEN,
        )));
        pre_allocated_ids.insert(RENodeId::Global(GlobalAddress::Component(EPOCH_MANAGER)));
        instructions.push(Instruction::System(NativeInvocation::EpochManager(
            EpochManagerInvocation::Create(EpochManagerCreateInvocation {
                olympia_validator_token_address,
                component_address,
                validator_set: validators,
                initial_epoch,
                rounds_per_epoch,
                num_unstake_epochs,
            }),
        )));
    }

    for (public_key, amount) in account_xrd_allocations.into_iter() {
        let bucket_id = id_allocator.new_bucket_id().unwrap();
        instructions.push(
            BasicInstruction::TakeFromWorktopByAmount {
                resource_address: RADIX_TOKEN,
                amount,
            }
            .into(),
        );
        let component_address = ComponentAddress::virtual_account_from_public_key(
            &PublicKey::EcdsaSecp256k1(public_key),
        );
        instructions.push(
            BasicInstruction::CallMethod {
                component_address: component_address,
                method_name: "deposit".to_string(),
                args: args!(bucket_id),
            }
            .into(),
        );
    }

    // Faucet
    {
        instructions.push(
            BasicInstruction::TakeFromWorktop {
                resource_address: RADIX_TOKEN,
            }
            .into(),
        );

        let bucket = id_allocator.new_bucket_id().unwrap();
        instructions.push(Instruction::Basic(BasicInstruction::CallFunction {
            package_address: FAUCET_PACKAGE,
            blueprint_name: FAUCET_BLUEPRINT.to_string(),
            function_name: "new".to_string(),
            args: args!(bucket),
        }));
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
    let faucet_component = receipt.new_component_addresses().last().unwrap().clone();
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
            RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)),
            SubstateOffset::Global(GlobalOffset::Global),
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
    use super::*;
    use crate::{ledger::TypedInMemorySubstateStore, wasm::DefaultWasmEngine};
    use transaction::signing::EcdsaSecp256k1PrivateKey;

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
            &ExecutionConfig::default(),
            &genesis_transaction.get_executable(vec![AuthAddresses::system_role()]),
        );
        #[cfg(not(feature = "alloc"))]
        println!("{:?}", transaction_receipt);

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
            &ExecutionConfig::default(),
            &genesis_transaction.get_executable(vec![AuthAddresses::system_role()]),
        );

        let commit_result = transaction_receipt.result.expect_commit();
        commit_result.state_updates.commit(&mut substate_store);

        let global_substate = substate_store
            .get_substate(&SubstateId(
                RENodeId::Global(GlobalAddress::Component(account_component_address)),
                SubstateOffset::Global(GlobalOffset::Global),
            ))
            .map(|s| s.substate.to_runtime())
            .unwrap();
        let derefed_component_id: ComponentId = global_substate.global().node_deref().into();

        assert!(commit_result
            .resource_changes
            .iter()
            .any(|rc| rc.resource_address == RADIX_TOKEN
                && rc.amount == allocation_amount
                && rc.component_id == derefed_component_id));
    }
}
