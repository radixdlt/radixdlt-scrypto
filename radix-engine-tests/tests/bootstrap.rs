use radix_engine::blueprints::consensus_manager::ProposerMinuteTimestampSubstate;
use radix_engine::blueprints::resource::FungibleResourceManagerTotalSupplySubstate;
use radix_engine::system::bootstrap::{
    Bootstrapper, GenesisDataChunk, GenesisReceipts, GenesisResource, GenesisResourceAllocation,
    GenesisStakeAllocation,
};
use radix_engine::track::db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper};
use radix_engine::transaction::{BalanceChange, CommitResult};
use radix_engine::types::*;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::*;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use radix_engine_queries::typed_substate_layout::{to_typed_substate_key, to_typed_substate_value};
use radix_engine_store_interface::interface::DatabaseUpdate;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use scrypto_unit::CustomGenesis;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;

#[test]
fn test_bootstrap_receipt_should_match_constants() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = InMemorySubstateDatabase::standard();
    let validator_key = EcdsaSecp256k1PublicKey([0; 33]);
    let staker_address = ComponentAddress::virtual_account_from_public_key(
        &EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key(),
    );
    let stake = GenesisStakeAllocation {
        account_index: 0,
        xrd_amount: Decimal::one(),
    };
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![validator_key.clone().into()]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_address],
            allocations: vec![(validator_key, vec![stake])],
        },
    ];

    let mut bootstrapper = Bootstrapper::new(&mut substate_db, &scrypto_vm, true);

    let GenesisReceipts {
        system_bootstrap_receipt,
        wrap_up_receipt,
        ..
    } = bootstrapper
        .bootstrap_with_genesis_data(
            genesis_data_chunks,
            1u64,
            CustomGenesis::default_consensus_manager_config(),
            1,
        )
        .unwrap();

    assert!(system_bootstrap_receipt
        .expect_commit_success()
        .new_package_addresses()
        .contains(&PACKAGE_PACKAGE));

    assert!(system_bootstrap_receipt
        .expect_commit_success()
        .new_component_addresses()
        .contains(&GENESIS_HELPER));

    assert!(wrap_up_receipt
        .expect_commit_success()
        .new_component_addresses()
        .contains(&FAUCET));

    wrap_up_receipt
        .expect_commit_success()
        .next_epoch()
        .expect("There should be a new epoch.");
}

#[test]
fn test_bootstrap_receipt_should_have_substate_changes_which_can_be_typed() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = InMemorySubstateDatabase::standard();
    let validator_key = EcdsaSecp256k1PublicKey([0; 33]);
    let staker_address = ComponentAddress::virtual_account_from_public_key(
        &EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key(),
    );
    let stake = GenesisStakeAllocation {
        account_index: 0,
        xrd_amount: Decimal::one(),
    };
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![validator_key.clone().into()]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_address],
            allocations: vec![(validator_key, vec![stake])],
        },
    ];

    let mut bootstrapper = Bootstrapper::new(&mut substate_db, &scrypto_vm, true);

    let GenesisReceipts {
        system_bootstrap_receipt,
        data_ingestion_receipts,
        wrap_up_receipt,
    } = bootstrapper
        .bootstrap_with_genesis_data(
            genesis_data_chunks,
            1u64,
            CustomGenesis::default_consensus_manager_config(),
            1,
        )
        .unwrap();

    validate_receipt_substate_changes_which_can_be_typed(
        system_bootstrap_receipt.expect_commit_success(),
    );
    for receipt in data_ingestion_receipts.into_iter() {
        validate_receipt_substate_changes_which_can_be_typed(receipt.expect_commit_success());
    }
    validate_receipt_substate_changes_which_can_be_typed(wrap_up_receipt.expect_commit_success());
}

fn validate_receipt_substate_changes_which_can_be_typed(commit_result: &CommitResult) {
    let system_updates = &commit_result.state_updates.system_updates;
    for ((node_id, partition_num), partition_updates) in system_updates.into_iter() {
        for (substate_key, database_update) in partition_updates.into_iter() {
            let typed_substate_key =
                to_typed_substate_key(node_id.entity_type().unwrap(), *partition_num, substate_key)
                    .expect("Substate key should be typeable");
            if !typed_substate_key.value_is_mappable() {
                continue;
            }
            match database_update {
                DatabaseUpdate::Set(raw_value) => {
                    // Check that typed value mapping works
                    to_typed_substate_value(&typed_substate_key, raw_value)
                        .expect("Substate value should be typeable");
                }
                DatabaseUpdate::Delete => {}
            }
        }
    }
}

#[test]
fn test_genesis_xrd_allocation_to_accounts() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = InMemorySubstateDatabase::standard();
    let account_public_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
    let account_component_address = ComponentAddress::virtual_account_from_public_key(
        &PublicKey::EcdsaSecp256k1(account_public_key.clone()),
    );
    let allocation_amount = dec!("100");
    let genesis_data_chunks = vec![GenesisDataChunk::XrdBalances(vec![(
        account_component_address,
        allocation_amount,
    )])];

    let mut bootstrapper = Bootstrapper::new(&mut substate_db, &scrypto_vm, true);

    let GenesisReceipts {
        data_ingestion_receipts,
        ..
    } = bootstrapper
        .bootstrap_with_genesis_data(
            genesis_data_chunks,
            1u64,
            CustomGenesis::default_consensus_manager_config(),
            1,
        )
        .unwrap();

    let receipt = &data_ingestion_receipts[0];
    assert_eq!(
        receipt
            .expect_commit_success()
            .state_update_summary
            .balance_changes
            .get(&GlobalAddress::from(account_component_address))
            .unwrap()
            .get(&RADIX_TOKEN)
            .unwrap(),
        &BalanceChange::Fungible(allocation_amount)
    );
}

#[test]
fn test_genesis_resource_with_initial_allocation() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = InMemorySubstateDatabase::standard();
    let token_holder = ComponentAddress::virtual_account_from_public_key(
        &PublicKey::EcdsaSecp256k1(EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key()),
    );
    let address_bytes_without_entity_id = hash(vec![1, 2, 3]).lower_bytes();
    let resource_address = ResourceAddress::new_or_panic(
        NodeId::new(
            EntityType::GlobalFungibleResourceManager as u8,
            &address_bytes_without_entity_id,
        )
        .0,
    );

    let resource_owner = ComponentAddress::virtual_account_from_public_key(
        &EcdsaSecp256k1PrivateKey::from_u64(2).unwrap().public_key(),
    );
    let allocation_amount = dec!("105");
    let genesis_resource = GenesisResource {
        address_bytes_without_entity_id,
        initial_supply: allocation_amount,
        metadata: vec![("symbol".to_string(), "TST".to_string())],
        owner: Some(resource_owner),
    };
    let resource_allocation = GenesisResourceAllocation {
        account_index: 0,
        amount: allocation_amount,
    };
    let genesis_data_chunks = vec![
        GenesisDataChunk::Resources(vec![genesis_resource]),
        GenesisDataChunk::ResourceBalances {
            accounts: vec![token_holder.clone()],
            allocations: vec![(resource_address.clone(), vec![resource_allocation])],
        },
    ];

    let mut bootstrapper = Bootstrapper::new(&mut substate_db, &scrypto_vm, true);

    let GenesisReceipts {
        mut data_ingestion_receipts,
        ..
    } = bootstrapper
        .bootstrap_with_genesis_data(
            genesis_data_chunks,
            1u64,
            CustomGenesis::default_consensus_manager_config(),
            1,
        )
        .unwrap();

    let total_supply = substate_db
        .get_mapped::<SpreadPrefixKeyMapper, FungibleResourceManagerTotalSupplySubstate>(
            &resource_address.as_node_id(),
            OBJECT_BASE_PARTITION,
            &FungibleResourceManagerField::TotalSupply.into(),
        )
        .unwrap();
    assert_eq!(total_supply, allocation_amount);

    let key = scrypto_encode("symbol").unwrap();
    let entry = substate_db
        .get_mapped::<SpreadPrefixKeyMapper, Option<MetadataValue>>(
            &resource_address.as_node_id(),
            METADATA_KV_STORE_PARTITION,
            &SubstateKey::Map(key),
        )
        .unwrap();

    if let Some(MetadataValue::String(symbol)) = entry {
        assert_eq!(symbol, "TST");
    } else {
        panic!("Resource symbol was not a string");
    }

    let allocation_receipt = data_ingestion_receipts.pop().unwrap();
    let resource_creation_receipt = data_ingestion_receipts.pop().unwrap();

    let created_owner_badge = resource_creation_receipt
        .expect_commit_success()
        .new_resource_addresses()[0];
    let created_resource = resource_creation_receipt
        .expect_commit_success()
        .new_resource_addresses()[1];
    assert_eq!(
        resource_creation_receipt
            .expect_commit_success()
            .state_update_summary
            .balance_changes
            .get(&GlobalAddress::from(resource_owner))
            .unwrap()
            .get(&created_owner_badge)
            .unwrap(),
        &BalanceChange::Fungible(1.into())
    );
    assert_eq!(
        allocation_receipt
            .expect_commit_success()
            .state_update_summary
            .balance_changes
            .get(&GlobalAddress::from(token_holder))
            .unwrap()
            .get(&created_resource)
            .unwrap(),
        &BalanceChange::Fungible(allocation_amount)
    );
}

#[test]
fn test_genesis_stake_allocation() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = InMemorySubstateDatabase::standard();

    // There are two genesis validators
    // - one with two stakers (0 and 1)
    // - one with one staker (just 1)
    let validator_0_key = EcdsaSecp256k1PrivateKey::from_u64(10).unwrap().public_key();
    let validator_1_key = EcdsaSecp256k1PrivateKey::from_u64(11).unwrap().public_key();
    let staker_0 = ComponentAddress::virtual_account_from_public_key(
        &EcdsaSecp256k1PrivateKey::from_u64(4).unwrap().public_key(),
    );
    let staker_1 = ComponentAddress::virtual_account_from_public_key(
        &EcdsaSecp256k1PrivateKey::from_u64(5).unwrap().public_key(),
    );
    let validator_0_allocations = vec![
        GenesisStakeAllocation {
            account_index: 0,
            xrd_amount: dec!("10"),
        },
        GenesisStakeAllocation {
            account_index: 1,
            xrd_amount: dec!("50000"),
        },
    ];
    let validator_1_allocations = vec![GenesisStakeAllocation {
        account_index: 1,
        xrd_amount: dec!("1"),
    }];
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![
            validator_0_key.clone().into(),
            validator_1_key.clone().into(),
        ]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_0, staker_1],
            allocations: vec![
                (validator_0_key, validator_0_allocations),
                (validator_1_key, validator_1_allocations),
            ],
        },
    ];

    let mut bootstrapper = Bootstrapper::new(&mut substate_db, &scrypto_vm, true);

    let GenesisReceipts {
        mut data_ingestion_receipts,
        ..
    } = bootstrapper
        .bootstrap_with_genesis_data(
            genesis_data_chunks,
            1u64,
            CustomGenesis::default_consensus_manager_config(),
            1,
        )
        .unwrap();

    let allocate_stakes_receipt = data_ingestion_receipts.pop().unwrap();

    // Staker 0 should have one liquidity balance entry
    {
        let address: GlobalAddress = staker_0.into();
        let balances = allocate_stakes_receipt
            .expect_commit_success()
            .state_update_summary
            .balance_changes
            .get(&address)
            .unwrap();
        assert_eq!(balances.len(), 1);
        assert!(balances
            .values()
            .any(|bal| *bal == BalanceChange::Fungible(dec!("10"))));
    }

    // Staker 1 should have two liquidity balance entries
    {
        let address: GlobalAddress = staker_1.into();
        let balances = allocate_stakes_receipt
            .expect_commit_success()
            .state_update_summary
            .balance_changes
            .get(&address)
            .unwrap();
        assert_eq!(balances.len(), 2);
        assert!(balances
            .values()
            .any(|bal| *bal == BalanceChange::Fungible(dec!("1"))));
        assert!(balances
            .values()
            .any(|bal| *bal == BalanceChange::Fungible(dec!("50000"))));
    }
}

#[test]
fn test_genesis_time() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = InMemorySubstateDatabase::standard();

    let mut bootstrapper = Bootstrapper::new(&mut substate_db, &scrypto_vm, false);

    let _ = bootstrapper
        .bootstrap_with_genesis_data(
            vec![],
            1u64,
            CustomGenesis::default_consensus_manager_config(),
            123 * 60 * 1000 + 22, // 123 full minutes + 22 ms (which should be rounded down)
        )
        .unwrap();

    let proposer_minute_timestamp = substate_db
        .get_mapped::<SpreadPrefixKeyMapper, ProposerMinuteTimestampSubstate>(
            CONSENSUS_MANAGER.as_node_id(),
            OBJECT_BASE_PARTITION,
            &ConsensusManagerField::CurrentTimeRoundedToMinutes.into(),
        )
        .unwrap();

    assert_eq!(proposer_minute_timestamp.epoch_minute, 123);
}
