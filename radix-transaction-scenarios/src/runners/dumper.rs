use crate::internal_prelude::*;
use crate::scenarios::*;
use radix_engine::system::system_callback_api::SystemCallbackObject;
use radix_engine::updates::*;
use radix_engine::vm::{DefaultNativeVm, NativeVm, NoExtension, Vm};
use radix_engine::{
    system::bootstrap::Bootstrapper,
    vm::{
        wasm::{DefaultWasmEngine, WasmEngine},
        ScryptoVm,
    },
};
use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
use radix_substate_store_impls::state_tree_support::StateTreeUpdatingDatabase;
use radix_substate_store_interface::db_key_mapper::*;
use radix_substate_store_interface::interface::*;
use radix_transactions::validation::{NotarizedTransactionValidator, ValidationConfig};

pub struct RunnerContext {
    #[cfg(feature = "std")]
    pub dump_manifest_root: Option<std::path::PathBuf>,
    pub network: NetworkDefinition,
}

#[cfg(feature = "std")]
pub fn run_all_in_memory_and_dump_examples(
    network: NetworkDefinition,
    root_path: std::path::PathBuf,
) -> Result<(), FullScenarioError> {
    let mut event_hasher = HashAccumulator::new();
    let mut substate_db = StateTreeUpdatingDatabase::new(InMemorySubstateDatabase::standard());

    let ScenarioExecutionReceipt {
        database: mut substate_db,
    } = DefaultTransactionScenarioExecutor::new(substate_db, NetworkDefinition::simulator())
        .on_scenario_start(|scenario_metadata| {
            let sub_folder = root_path.join(scenario_metadata.logical_name);
            if sub_folder.exists() {
                std::fs::remove_dir_all(&sub_folder).unwrap();
            }
        })
        .on_transaction_executed(|scenario_metadata, transaction, receipt, _| {
            transaction.dump_manifest(
                &Some(root_path.join(scenario_metadata.logical_name)),
                &NetworkDefinition::simulator(),
            );

            let intent_hash =
                PreparedNotarizedTransactionV1::prepare_from_raw(&transaction.raw_transaction)
                    .unwrap()
                    .intent_hash();

            match &receipt.result {
                TransactionResult::Commit(c) => {
                    event_hasher.update_no_chain(intent_hash.as_hash().as_bytes());
                    event_hasher.update_no_chain(scrypto_encode(&c.application_events).unwrap());
                }
                TransactionResult::Reject(_) | TransactionResult::Abort(_) => {}
            }
        })
        .nonce_handling(ScenarioStartNonceHandling::PreviousScenarioStartNoncePlus(
            1000,
        ))
        .execute()
        .expect("Must succeed");

    assert_eq!(
        substate_db.get_current_root_hash().to_string(),
        "95aa414f165c90293f8256f7e556c4bdebf5d991e6ff09bbae70bab337fdfb81"
    );
    assert_eq!(
        event_hasher.finalize().to_string(),
        "d72641b99823b65e238b392420ac1712b089bf3469d3475f37bb16e4f62268be"
    );

    Ok(())
}

#[cfg(test)]
#[cfg(feature = "std")]
mod test {
    use radix_transactions::manifest::{compile, MockBlobProvider};

    use super::*;

    #[test]
    pub fn update_expected_scenario_output() {
        let network_definition = NetworkDefinition::simulator();
        let scenarios_dir =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("generated-examples");
        run_all_in_memory_and_dump_examples(network_definition.clone(), scenarios_dir.clone())
            .unwrap();

        // Ensure that they can all be compiled back again
        for entry in walkdir::WalkDir::new(&scenarios_dir) {
            let path = entry.unwrap().path().canonicalize().unwrap();
            if path.extension().and_then(|str| str.to_str()) != Some("rtm") {
                continue;
            }

            let manifest_string = std::fs::read_to_string(path).unwrap();
            compile(
                &manifest_string,
                &network_definition,
                MockBlobProvider::new(),
            )
            .unwrap();
        }
    }
}
