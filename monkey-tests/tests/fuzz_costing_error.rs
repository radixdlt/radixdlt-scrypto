use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use monkey_tests::costing_err::RandomCallbackError;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::kernel::call_frame::{CallFrameMessage, NodeVisibility};
use radix_engine::kernel::kernel_api::{
    DroppedNode, KernelApi, KernelInternalApi, KernelInvocation, KernelInvokeApi, KernelNodeApi,
    KernelSubstateApi, SystemState,
};
use radix_engine::kernel::kernel_callback_api::{
    CloseSubstateEvent, CreateNodeEvent, DrainSubstatesEvent, DropNodeEvent, KernelCallbackObject,
    MoveModuleEvent, OpenSubstateEvent, ReadSubstateEvent, RemoveSubstateEvent, ScanKeysEvent,
    ScanSortedSubstatesEvent, SetSubstateEvent, WriteSubstateEvent,
};
use radix_engine::system::system_callback::SystemConfig;
use radix_engine::system::system_callback_api::SystemCallbackObject;
use radix_engine::system::system_modules::costing::{CostingError, FeeReserveError};
use radix_engine::system::system_modules::execution_trace::{BucketSnapshot, ProofSnapshot};
use radix_engine::track::NodeSubstates;
use radix_engine::transaction::{TransactionResult, WrappedSystem};
use radix_engine::types::*;
use radix_engine::vm::wasm::DefaultWasmEngine;
use radix_engine::vm::*;
use radix_engine_store_interface::db_key_mapper::SubstateKeyContent;
use scrypto_unit::*;
use transaction::prelude::PreAllocatedAddress;
use transaction_scenarios::scenario::{NextAction, ScenarioCore};
use transaction_scenarios::scenarios::get_builder_for_every_scenario;

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
                        test_runner.execute_raw_transaction_with_wrapper::<RandomCallbackError<
                            SystemConfig<Vm<'_, DefaultWasmEngine, NoExtension>>,
                        >>(&network, &next.raw_transaction);
                    match &receipt.result {
                        TransactionResult::Commit(..) => {
                            previous = Some(receipt);
                        }
                        // Ignore other results - which may be valid in the context of the scenario
                        _ => {}
                    }
                }
                NextAction::Completed(end_state) => {
                    next_nonce = end_state.next_unused_nonce;
                    break;
                }
            }
        }
    }
}

