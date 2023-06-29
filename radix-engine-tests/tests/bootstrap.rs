use radix_engine::blueprints::consensus_manager::ProposerMinuteTimestampSubstate;
use radix_engine::blueprints::resource::FungibleResourceManagerTotalSupplySubstate;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::bootstrap::{
    Bootstrapper, GenesisDataChunk, GenesisReceipts, GenesisResource, GenesisResourceAllocation,
    GenesisStakeAllocation,
};
use radix_engine::system::system::KeyValueEntrySubstate;
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::transaction::BalanceChange;
use radix_engine::types::*;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::*;
use radix_engine_interface::api::node_modules::metadata::{MetadataValue, Url};
use radix_engine_store_interface::db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper};
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use scrypto_unit::{CustomGenesis, TestRunner};
use transaction::prelude::ManifestBuilder;
use transaction::signing::secp256k1::Secp256k1PrivateKey;

#[test]
fn test_bootstrap_receipt_should_match_constants() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = InMemorySubstateDatabase::standard();
    let validator_key = Secp256k1PublicKey([0; 33]);
    let staker_address = ComponentAddress::virtual_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(1).unwrap().public_key(),
    );
    let genesis_epoch = Epoch::of(1);
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
            genesis_epoch,
            CustomGenesis::default_consensus_manager_config(),
            1,
            Some(0),
            Decimal::zero(),
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

    assert!(system_bootstrap_receipt
        .expect_commit_success()
        .new_package_addresses()
        .contains(&TRANSACTION_TRACKER_PACKAGE));

    assert!(system_bootstrap_receipt
        .expect_commit_success()
        .new_component_addresses()
        .contains(&TRANSACTION_TRACKER));

    assert!(system_bootstrap_receipt
        .expect_commit_success()
        .new_component_addresses()
        .contains(&FAUCET));

    let wrap_up_epoch_change = wrap_up_receipt
        .expect_commit_success()
        .next_epoch()
        .expect("There should be a new epoch.");

    assert_eq!(wrap_up_epoch_change.epoch, genesis_epoch.next());
}

#[test]
fn test_genesis_resource_with_initial_allocation() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let mut substate_db = InMemorySubstateDatabase::standard();
    let token_holder = ComponentAddress::virtual_account_from_public_key(&PublicKey::Secp256k1(
        Secp256k1PrivateKey::from_u64(1).unwrap().public_key(),
    ));
    let resource_address = ResourceAddress::new_or_panic(
        NodeId::new(
            EntityType::GlobalFungibleResourceManager as u8,
            &hash(vec![1, 2, 3]).lower_bytes(),
        )
        .0,
    );
    let resource_owner = ComponentAddress::virtual_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(2).unwrap().public_key(),
    );
    let allocation_amount = dec!("105");
    let genesis_resource = GenesisResource {
        reserved_resource_address: resource_address,
        metadata: vec![(
            "symbol".to_string(),
            MetadataValue::String("TST".to_string()),
        )],
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
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
            1,
            Some(0),
            Decimal::zero(),
        )
        .unwrap();

    let total_supply = substate_db
        .get_mapped::<SpreadPrefixKeyMapper, FungibleResourceManagerTotalSupplySubstate>(
            &resource_address.as_node_id(),
            MAIN_BASE_PARTITION,
            &FungibleResourceManagerField::TotalSupply.into(),
        )
        .unwrap();
    assert_eq!(total_supply, allocation_amount);

    let key = scrypto_encode("symbol").unwrap();
    let entry = substate_db
        .get_mapped::<SpreadPrefixKeyMapper, KeyValueEntrySubstate<MetadataValue>>(
            &resource_address.as_node_id(),
            METADATA_KV_STORE_PARTITION,
            &SubstateKey::Map(key),
        )
        .unwrap()
        .value;

    if let Some(MetadataValue::String(symbol)) = entry {
        assert_eq!(symbol, "TST");
    } else {
        panic!("Resource symbol was not a string");
    }

    let allocation_receipt = data_ingestion_receipts.pop().unwrap();
    let resource_creation_receipt = data_ingestion_receipts.pop().unwrap();

    println!("{:?}", resource_creation_receipt);

    let created_owner_badge = resource_creation_receipt
        .expect_commit_success()
        .new_resource_addresses()[1];
    let created_resource = resource_creation_receipt
        .expect_commit_success()
        .new_resource_addresses()[0]; // The resource address is preallocated, thus [0]
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
    let validator_0_key = Secp256k1PrivateKey::from_u64(10).unwrap().public_key();
    let validator_1_key = Secp256k1PrivateKey::from_u64(11).unwrap().public_key();
    let staker_0 = ComponentAddress::virtual_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(4).unwrap().public_key(),
    );
    let staker_1 = ComponentAddress::virtual_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(5).unwrap().public_key(),
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
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
            1,
            Some(0),
            Decimal::zero(),
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

    let create_validators_receipt = data_ingestion_receipts.pop().unwrap();
    {
        let new_validators: Vec<ComponentAddress> = create_validators_receipt
            .expect_commit_success()
            .state_update_summary
            .new_components
            .iter()
            .filter(|c| c.as_node_id().entity_type() == Some(EntityType::GlobalValidator))
            .cloned()
            .collect();

        for (index, validator_key) in vec![validator_0_key, validator_1_key]
            .into_iter()
            .enumerate()
        {
            let validator_url_entry = substate_db
                .get_mapped::<SpreadPrefixKeyMapper, KeyValueEntrySubstate<MetadataValue>>(
                    &new_validators[index].as_node_id(),
                    METADATA_KV_STORE_PARTITION,
                    &SubstateKey::Map(scrypto_encode("url").unwrap()),
                )
                .unwrap();
            if let Some(MetadataValue::Url(url)) = validator_url_entry.value {
                assert_eq!(
                    url,
                    Url(format!("http://test.local?validator={:?}", validator_key))
                );
            } else {
                panic!("Validator url was not a Url");
            }
        }
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
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
            123 * 60 * 1000 + 22, // 123 full minutes + 22 ms (which should be rounded down)
            Some(0),
            Decimal::zero(),
        )
        .unwrap();

    let proposer_minute_timestamp = substate_db
        .get_mapped::<SpreadPrefixKeyMapper, ProposerMinuteTimestampSubstate>(
            CONSENSUS_MANAGER.as_node_id(),
            MAIN_BASE_PARTITION,
            &ConsensusManagerField::CurrentTimeRoundedToMinutes.into(),
        )
        .unwrap();

    assert_eq!(proposer_minute_timestamp.epoch_minute, 123);
}

#[test]
fn should_not_be_able_to_create_genesis_helper() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            GENESIS_HELPER_PACKAGE,
            GENESIS_HELPER_BLUEPRINT,
            "new",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn should_not_be_able_to_call_genesis_helper() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_method(GENESIS_HELPER, "wrap_up", manifest_args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}
