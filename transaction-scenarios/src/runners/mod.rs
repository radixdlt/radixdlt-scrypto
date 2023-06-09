use radix_engine::{
    system::bootstrap::Bootstrapper,
    vm::{
        wasm::{DefaultWasmEngine, WasmEngine},
        ScryptoVm,
    },
};
use radix_engine_store_interface::interface::*;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use transaction::validation::{NotarizedTransactionValidator, ValidationConfig};

use crate::{internal_prelude::*, scenarios::get_all_scenarios};

#[cfg(feature = "std")]
pub fn run_all_in_memory_and_dump_examples(
    network: NetworkDefinition,
    root_path: std::path::PathBuf,
) -> Result<(), FullScenarioError> {
    let mut substate_db = InMemorySubstateDatabase::standard();
    let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();

    let receipts = Bootstrapper::new(&mut substate_db, &scrypto_vm, false)
        .bootstrap_test_default()
        .unwrap();
    let epoch = receipts
        .wrap_up_receipt
        .expect_commit_success()
        .next_epoch()
        .expect("Wrap up ends in next epoch")
        .epoch;

    for scenario in get_all_scenarios() {
        let sub_folder = root_path.join(scenario.logical_name());
        // Clear directory before generating anew
        std::fs::remove_dir_all(&sub_folder).unwrap();
        let scenario_context =
            ScenarioContext::new(network.clone(), epoch).with_manifest_dumping(sub_folder);
        run_scenario_with_default_config(&mut substate_db, scenario, scenario_context)?;
    }
    Ok(())
}

pub fn run_scenario_with_default_config<S>(
    substate_db: &mut S,
    scenario: Box<dyn ScenarioCore>,
    context: ScenarioContext,
) -> Result<(), FullScenarioError>
where
    S: SubstateDatabase + CommittableSubstateDatabase,
{
    let fee_reserve_config = FeeReserveConfig::default();
    let execution_config = ExecutionConfig::default();
    let scrypto_interpreter = ScryptoVm::<DefaultWasmEngine>::default();
    let validator =
        NotarizedTransactionValidator::new(ValidationConfig::default(context.network().id));

    run_scenario(
        &validator,
        substate_db,
        &scrypto_interpreter,
        &fee_reserve_config,
        &execution_config,
        scenario,
        context,
    )
}

pub fn run_scenario<S, W>(
    validator: &NotarizedTransactionValidator,
    substate_db: &mut S,
    scrypto_interpreter: &ScryptoVm<W>,
    fee_reserve_config: &FeeReserveConfig,
    execution_config: &ExecutionConfig,
    mut scenario: Box<dyn ScenarioCore>,
    mut context: ScenarioContext,
) -> Result<(), FullScenarioError>
where
    S: SubstateDatabase + CommittableSubstateDatabase,
    W: WasmEngine,
{
    let mut previous = None;
    loop {
        let next = scenario
            .next(&mut context, previous.as_ref())
            .map_err(|err| err.into_full(&scenario))?;
        match next {
            Some(next) => {
                let transaction = next
                    .validate(&validator)
                    .map_err(|err| err.into_full(&scenario))?;
                previous = Some(execute_and_commit_transaction(
                    substate_db,
                    scrypto_interpreter,
                    fee_reserve_config,
                    execution_config,
                    &transaction.get_executable(),
                ));
            }
            None => break Ok(()),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod test {
    use super::*;

    #[test]
    pub fn regenerate_all() {
        let scenarios_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
        run_all_in_memory_and_dump_examples(NetworkDefinition::simulator(), scenarios_dir).unwrap()
    }
}
