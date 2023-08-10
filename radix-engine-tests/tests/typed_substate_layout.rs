use radix_engine::blueprints::native_schema::*;
use radix_engine::system::bootstrap::{
    Bootstrapper, GenesisDataChunk, GenesisReceipts, GenesisResource, GenesisResourceAllocation,
    GenesisStakeAllocation,
};
use radix_engine::transaction::{CommitResult, TransactionResult};
use radix_engine::types::*;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::*;
use radix_engine_queries::typed_native_events::{to_typed_native_event, TypedNativeEvent};
use radix_engine_queries::typed_substate_layout::{to_typed_substate_key, to_typed_substate_value};
use radix_engine_store_interface::interface::DatabaseUpdate;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use sbor::rust::ops::Deref;
use scrypto_unit::*;
use transaction::signing::secp256k1::Secp256k1PrivateKey;
use transaction_scenarios::scenario::{NextAction, ScenarioCore};
use transaction_scenarios::scenarios::get_builder_for_every_scenario;

#[test]
fn test_bootstrap_receipt_should_have_substate_changes_which_can_be_typed() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    let vm = Vm::new(&scrypto_vm, native_vm);
    let mut substate_db = InMemorySubstateDatabase::standard();
    let validator_key = Secp256k1PublicKey([0; 33]);
    let staker_address = ComponentAddress::virtual_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(1).unwrap().public_key(),
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

    let mut bootstrapper = Bootstrapper::new(&mut substate_db, vm, true);

    let GenesisReceipts {
        system_bootstrap_receipt,
        data_ingestion_receipts,
        wrap_up_receipt,
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

    assert_receipt_substate_changes_can_be_typed(system_bootstrap_receipt.expect_commit_success());
    for receipt in data_ingestion_receipts.into_iter() {
        assert_receipt_substate_changes_can_be_typed(receipt.expect_commit_success());
    }
    assert_receipt_substate_changes_can_be_typed(wrap_up_receipt.expect_commit_success());
}

#[test]
fn test_bootstrap_receipt_should_have_events_that_can_be_typed() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
    let native_vm = DefaultNativeVm::new();
    let mut substate_db = InMemorySubstateDatabase::standard();
    let validator_key = Secp256k1PublicKey([0; 33]);
    let staker_address = ComponentAddress::virtual_account_from_public_key(
        &Secp256k1PrivateKey::from_u64(1).unwrap().public_key(),
    );
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
    let stake = GenesisStakeAllocation {
        account_index: 0,
        xrd_amount: Decimal::one(),
    };
    let mut xrd_balances = Vec::new();
    let mut pub_key_accounts = Vec::new();

    for i in 0..20 {
        let pub_key = Secp256k1PrivateKey::from_u64((i + 1).try_into().unwrap())
            .unwrap()
            .public_key();
        let account_address = ComponentAddress::virtual_account_from_public_key(&pub_key);
        pub_key_accounts.push((pub_key, account_address));
        xrd_balances.push((account_address, dec!("10")));
    }
    let genesis_resource = GenesisResource {
        reserved_resource_address: resource_address,
        metadata: vec![(
            "symbol".to_string(),
            MetadataValue::String("TST".to_string()),
        )],
        owner: None,
    };
    let resource_allocation = GenesisResourceAllocation {
        account_index: 0,
        amount: dec!("10"),
    };
    let genesis_data_chunks = vec![
        GenesisDataChunk::Validators(vec![validator_key.clone().into()]),
        GenesisDataChunk::Stakes {
            accounts: vec![staker_address],
            allocations: vec![(validator_key, vec![stake])],
        },
        GenesisDataChunk::XrdBalances(xrd_balances),
        GenesisDataChunk::Resources(vec![genesis_resource]),
        GenesisDataChunk::ResourceBalances {
            accounts: vec![token_holder.clone()],
            allocations: vec![(resource_address.clone(), vec![resource_allocation])],
        },
    ];

    let mut bootstrapper = Bootstrapper::new(
        &mut substate_db,
        Vm {
            scrypto_vm: &scrypto_vm,
            native_vm,
        },
        true,
    );

    let GenesisReceipts {
        system_bootstrap_receipt,
        data_ingestion_receipts,
        wrap_up_receipt,
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

    assert_receipt_events_can_be_typed(system_bootstrap_receipt.expect_commit_success());
    for receipt in data_ingestion_receipts.into_iter() {
        assert_receipt_events_can_be_typed(receipt.expect_commit_success());
    }
    assert_receipt_events_can_be_typed(wrap_up_receipt.expect_commit_success());
}

#[test]
fn test_all_scenario_commit_receipts_should_have_substate_changes_which_can_be_typed() {
    let network = NetworkDefinition::simulator();
    let mut test_runner = TestRunnerBuilder::new().build();

    let mut next_nonce: u32 = 0;
    for scenario_builder in get_builder_for_every_scenario() {
        let epoch = test_runner.get_current_epoch();
        let mut scenario = scenario_builder(ScenarioCore::new(network.clone(), epoch, next_nonce));
        let mut previous = None;
        loop {
            let next = scenario
                .next(previous.as_ref())
                .map_err(|err| err.into_full(&scenario))
                .unwrap();
            match next {
                NextAction::Transaction(next) => {
                    let receipt =
                        test_runner.execute_raw_transaction(&network, &next.raw_transaction);
                    match &receipt.result {
                        TransactionResult::Commit(commit_result) => {
                            assert_receipt_substate_changes_can_be_typed(commit_result);
                        }
                        // Ignore other results - which may be valid in the context of the scenario
                        _ => {}
                    }
                    previous = Some(receipt);
                }
                NextAction::Completed(end_state) => {
                    next_nonce = end_state.next_unused_nonce;
                    break;
                }
            }
        }
    }
}

#[test]
fn test_all_scenario_commit_receipts_should_have_events_that_can_be_typed() {
    let network = NetworkDefinition::simulator();
    let mut test_runner = TestRunnerBuilder::new().build();

    let mut next_nonce: u32 = 0;
    for scenario_builder in get_builder_for_every_scenario() {
        let epoch = test_runner.get_current_epoch();
        let mut scenario = scenario_builder(ScenarioCore::new(network.clone(), epoch, next_nonce));
        let mut previous = None;
        loop {
            let next = scenario
                .next(previous.as_ref())
                .map_err(|err| err.into_full(&scenario))
                .unwrap();
            match next {
                NextAction::Transaction(next) => {
                    let receipt =
                        test_runner.execute_raw_transaction(&network, &next.raw_transaction);
                    match &receipt.result {
                        TransactionResult::Commit(commit_result) => {
                            assert_receipt_events_can_be_typed(commit_result);
                        }
                        // Ignore other results - which may be valid in the context of the scenario
                        _ => {}
                    }
                    previous = Some(receipt);
                }
                NextAction::Completed(end_state) => {
                    next_nonce = end_state.next_unused_nonce;
                    break;
                }
            }
        }
    }
}

/// We need to ensure that all of the events registered to native events are included in the typed
/// native event model. This test checks that the events in `typed_native_events.rs` module all
/// exist in the blueprint schema.
#[test]
fn typed_native_event_type_contains_all_native_events() {
    // Arrange
    let package_name_definition_mapping = hashmap! {
        "ConsensusManager" => CONSENSUS_MANAGER_PACKAGE_DEFINITION.deref(),
        "Account" => ACCOUNT_PACKAGE_DEFINITION.deref(),
        "Identity" => IDENTITY_PACKAGE_DEFINITION.deref(),
        "AccessController" => ACCESS_CONTROLLER_PACKAGE_DEFINITION.deref(),
        "Pool" => POOL_PACKAGE_DEFINITION.deref(),
        "TransactionTracker" => TRANSACTION_TRACKER_PACKAGE_DEFINITION.deref(),
        "Resource" => RESOURCE_PACKAGE_DEFINITION.deref(),
        "Package" => PACKAGE_PACKAGE_DEFINITION.deref(),
        "TransactionProcessor" => TRANSACTION_PROCESSOR_PACKAGE_DEFINITION.deref(),
        "Metadata" => METADATA_PACKAGE_DEFINITION.deref(),
        "Royalty" => ROYALTY_PACKAGE_DEFINITION.deref(),
        "RoleAssignment" => ROLE_ASSIGNMENT_PACKAGE_DEFINITION.deref(),
    };

    // Act
    let registered_events = TypedNativeEvent::registered_events();

    // Assert
    for (package_name, package_blueprints) in registered_events.into_iter() {
        let package_definition = package_name_definition_mapping
            .get(package_name.as_str())
            .expect(
                format!(
                    "No package definition found for a package with the name: \"{package_name}\""
                )
                .as_str(),
            );
        for (blueprint_name, blueprint_events) in package_blueprints.into_iter() {
            let blueprint_definition = package_definition.blueprints.get(&blueprint_name).expect(
                format!(
                    "Package named \"{package_name}\" has no blueprint named \"{blueprint_name}\""
                )
                .as_str(),
            );
            let actual_blueprint_events = blueprint_definition
                .schema
                .events
                .event_schema
                .keys()
                .cloned()
                .collect::<HashSet<_>>();

            assert_eq!(
                blueprint_events,
                actual_blueprint_events,
                "There is a difference between the actual blueprint events and the ones registered in the typed model. Package name: \"{package_name}\", Blueprint name: \"{blueprint_name}\""
            )
        }
    }
}

fn assert_receipt_substate_changes_can_be_typed(commit_result: &CommitResult) {
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

fn assert_receipt_events_can_be_typed(commit_result: &CommitResult) {
    for (event_type_identifier, event_data) in &commit_result.application_events {
        let _ = to_typed_native_event(event_type_identifier, event_data).unwrap();
    }
}
