use radix_engine::system::bootstrap::{
    Bootstrapper, GenesisDataChunk, GenesisReceipts, GenesisStakeAllocation,
};
use radix_engine::transaction::{CommitResult, TransactionResult};
use radix_engine::types::*;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::*;
use radix_engine_queries::typed_substate_layout::{to_typed_substate_key, to_typed_substate_value};
use radix_engine_store_interface::interface::DatabaseUpdate;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use scrypto_unit::{CustomGenesis, TestRunner};
use transaction::signing::secp256k1::Secp256k1PrivateKey;
use transaction_scenarios::scenario::{NextAction, ScenarioCore};
use transaction_scenarios::scenarios::get_builder_for_every_scenario;

#[test]
fn test_bootstrap_receipt_should_have_substate_changes_which_can_be_typed() {
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
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

    let mut bootstrapper = Bootstrapper::new(&mut substate_db, &scrypto_vm, true);

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
fn test_all_scenario_commit_receipts_should_have_substate_changes_which_can_be_typed() {
    let network = NetworkDefinition::simulator();
    let mut test_runner = TestRunner::builder().build();

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
                    match &receipt.transaction_result {
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
