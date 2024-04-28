#![allow(clippy::type_complexity)]

use core::ops::*;

use self::internal_prelude::*;
use super::*;
use radix_engine::blueprints::consensus_manager::*;
use radix_engine::system::bootstrap::*;
use radix_engine::system::system_db_reader::*;
use radix_engine::updates::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::*;
use radix_substate_store_interface::db_key_mapper::*;
use radix_substate_store_interface::interface::*;
use radix_transactions::errors::*;
use radix_transactions::validation::*;
use sbor::prelude::*;

use scenarios::ALL_SCENARIOS;

#[derive(Clone, Debug)]
pub enum ScenarioTrigger {
    AtStartOfProtocolVersions(BTreeSet<ProtocolVersion>),
    AtStartOfEveryProtocolVersion,
    AfterCompletionOfAllProtocolUpdates,
}

#[derive(Clone, Debug)]
pub enum ScenarioFilter {
    ValidScenariosFrom(BTreeSet<String>),
    AllScenariosValidAtProtocolVersion,
    AllScenariosFirstValidAtProtocolVersion,
}

pub struct TransactionScenarioExecutor<'a, D, W, E>
where
    D: SubstateDatabase + CommittableSubstateDatabase,
    W: WasmEngine,
    E: NativeVmExtension,
{
    /* Environment */
    /// The substate database that the scenario will be run against.
    database: D,
    /// The Scrypto VM to use in executing the scenarios.
    scrypto_vm: ScryptoVm<W>,
    /// The Native VM to use in executing the scenarios.
    native_vm_extension: E,

    /* Execution */
    /// The first nonce to use in the execution of the scenarios.
    starting_nonce: u32,
    /// How the executor should handle nonces and how it should get the next nonce.
    next_scenario_nonce_handling: ScenarioStartNonceHandling,
    /// The network definition to use in the execution of the scenarios.
    network_definition: NetworkDefinition,

    /* Callbacks */
    /// A callback that is called when a scenario transaction is executed.
    on_transaction_executed:
        Box<dyn FnMut(&ScenarioMetadata, &NextTransaction, &TransactionReceiptV1, &D) + 'a>,
    /// A callback that is called when a new scenario is started.
    on_scenario_start: Box<dyn FnMut(&ScenarioMetadata) + 'a>,
    on_before_execute_protocol_update: Box<dyn FnMut(&ProtocolUpdateExecutor) + 'a>,

    /* Phantom */
    /// The lifetime of the callbacks used in the executor.
    callback_lifetime: PhantomData<&'a ()>,
}

pub type DefaultTransactionScenarioExecutor<'a, D> =
    TransactionScenarioExecutor<'a, D, DefaultWasmEngine, NoExtension>;

impl<'a, D, W, E> TransactionScenarioExecutor<'a, D, W, E>
where
    D: SubstateDatabase + CommittableSubstateDatabase,
    W: WasmEngine,
    E: NativeVmExtension,
{
    pub fn new(
        database: D,
        network_definition: &NetworkDefinition,
    ) -> DefaultTransactionScenarioExecutor<'a, D> {
        DefaultTransactionScenarioExecutor::<'a, D> {
            /* Environment */
            database,
            scrypto_vm: ScryptoVm::default(),
            native_vm_extension: NoExtension,
            /* Execution */
            starting_nonce: 0,
            next_scenario_nonce_handling:
                ScenarioStartNonceHandling::PreviousScenarioEndNoncePlusOne,
            network_definition: network_definition.clone(),
            /* Callbacks */
            on_transaction_executed: Box::new(|_, _, _, _| {}),
            on_scenario_start: Box::new(|_| {}),
            on_before_execute_protocol_update: Box::new(|_| {}),
            /* Phantom */
            callback_lifetime: Default::default(),
        }
    }

    /// Sets the Scrypto VM to use for the scenarios execution.
    pub fn scrypto_vm<NW: WasmEngine>(
        self,
        scrypto_vm: ScryptoVm<NW>,
    ) -> TransactionScenarioExecutor<'a, D, NW, E> {
        TransactionScenarioExecutor {
            /* Environment */
            database: self.database,
            scrypto_vm,
            native_vm_extension: self.native_vm_extension,
            /* Execution */
            starting_nonce: self.starting_nonce,
            next_scenario_nonce_handling: self.next_scenario_nonce_handling,
            network_definition: self.network_definition,
            /* Callbacks */
            on_transaction_executed: self.on_transaction_executed,
            on_scenario_start: self.on_scenario_start,
            on_before_execute_protocol_update: self.on_before_execute_protocol_update,
            /* Phantom */
            callback_lifetime: self.callback_lifetime,
        }
    }

    /// Sets the Native VM to use for the scenarios execution.
    pub fn native_vm_extension<NE: NativeVmExtension>(
        self,
        native_vm_extension: NE,
    ) -> TransactionScenarioExecutor<'a, D, W, NE> {
        TransactionScenarioExecutor {
            /* Environment */
            database: self.database,
            scrypto_vm: self.scrypto_vm,
            native_vm_extension,
            /* Execution */
            starting_nonce: self.starting_nonce,
            next_scenario_nonce_handling: self.next_scenario_nonce_handling,
            network_definition: self.network_definition,
            /* Callbacks */
            on_transaction_executed: self.on_transaction_executed,
            on_scenario_start: self.on_scenario_start,
            on_before_execute_protocol_update: self.on_before_execute_protocol_update,
            /* Phantom */
            callback_lifetime: self.callback_lifetime,
        }
    }

    /// Sets the starting nonce for executing scenarios.
    pub fn starting_nonce(mut self, starting_nonce: u32) -> Self {
        self.starting_nonce = starting_nonce;
        self
    }

    /// Defines how the executor should handle nonces.
    pub fn nonce_handling(mut self, nonce_handling: ScenarioStartNonceHandling) -> Self {
        self.next_scenario_nonce_handling = nonce_handling;
        self
    }

    /// Sets the callback to call after executing a scenario transaction.
    pub fn on_transaction_executed<
        F: FnMut(&ScenarioMetadata, &NextTransaction, &TransactionReceiptV1, &D) + 'a,
    >(
        mut self,
        callback: F,
    ) -> Self {
        self.on_transaction_executed = Box::new(callback);
        self
    }

    /// A callback that is called when a new scenario is started.
    pub fn on_scenario_start<F: FnMut(&ScenarioMetadata) + 'a>(mut self, callback: F) -> Self {
        self.on_scenario_start = Box::new(callback);
        self
    }

    /// A callback that is called before a protocol update is executed.
    pub fn on_before_execute_protocol_update<F: FnMut(&ProtocolUpdateExecutor) + 'a>(
        mut self,
        callback: F,
    ) -> Self {
        self.on_before_execute_protocol_update = Box::new(callback);
        self
    }

    pub fn into_database(self) -> D {
        self.database
    }

    /// Each scenario is executed once, when it first becomes valid.
    pub fn execute_every_protocol_update_and_scenario(
        &mut self,
    ) -> Result<(), ScenarioExecutorError> {
        self.execute_protocol_updates_and_scenarios(
            ProtocolBuilder::for_network(&self.network_definition).until_latest_protocol_version(),
            ScenarioTrigger::AtStartOfEveryProtocolVersion,
            ScenarioFilter::AllScenariosFirstValidAtProtocolVersion,
        )
    }

    pub fn execute_protocol_updates_and_scenarios(
        &mut self,
        protocol_executor: ProtocolExecutor,
        trigger: ScenarioTrigger,
        filter: ScenarioFilter,
    ) -> Result<(), ScenarioExecutorError> {
        Bootstrapper::new(
            self.network_definition.clone(),
            &mut self.database,
            VmInit::new(&self.scrypto_vm, self.native_vm_extension.clone()),
            false,
        )
        .bootstrap_test_default()
        .ok_or(ScenarioExecutorError::BootstrapFailed)?;

        let mut current_protocol_version = ProtocolVersion::Genesis;

        for protocol_update_executor in protocol_executor.each_protocol_update_executor() {
            self.execute_scenarios_at_new_protocol_version(
                current_protocol_version,
                &trigger,
                &filter,
                false,
            )?;
            current_protocol_version = protocol_update_executor.protocol_update.into();
            (self.on_before_execute_protocol_update)(&protocol_update_executor);
            protocol_update_executor.run_and_commit(&mut self.database);
        }

        self.execute_scenarios_at_new_protocol_version(
            current_protocol_version,
            &trigger,
            &filter,
            true,
        )?;

        Ok(())
    }

    fn execute_scenarios_at_new_protocol_version(
        &mut self,
        at_version: ProtocolVersion,
        trigger: &ScenarioTrigger,
        filter: &ScenarioFilter,
        is_last: bool,
    ) -> Result<(), ScenarioExecutorError> {
        let trigger_applies = match trigger {
            ScenarioTrigger::AtStartOfProtocolVersions(set) => set.contains(&at_version),
            ScenarioTrigger::AtStartOfEveryProtocolVersion => true,
            ScenarioTrigger::AfterCompletionOfAllProtocolUpdates => is_last,
        };

        if !trigger_applies {
            return Ok(());
        }

        let matching_scenarios = ALL_SCENARIOS.iter().filter(|(logical_name, creator)| {
            let metadata = creator.metadata();
            let is_valid = metadata.protocol_min_requirement >= at_version;
            if !is_valid {
                return false;
            }
            match filter {
                ScenarioFilter::ValidScenariosFrom(scenario_names) => {
                    scenario_names.contains(&**logical_name)
                }
                ScenarioFilter::AllScenariosValidAtProtocolVersion => true,
                ScenarioFilter::AllScenariosFirstValidAtProtocolVersion => {
                    metadata.protocol_min_requirement == at_version
                }
            }
        });

        for (_, scenario_creator) in matching_scenarios {
            self.execute_scenario(scenario_creator.as_ref())?;
        }

        Ok(())
    }

    pub fn execute_scenario(
        &mut self,
        scenario_creator: &dyn ScenarioCreatorObjectSafe,
    ) -> Result<(), ScenarioExecutorError> {
        let epoch = SystemDatabaseReader::new(&self.database)
            .read_object_field(
                CONSENSUS_MANAGER.as_node_id(),
                ModuleId::Main,
                ConsensusManagerField::State.field_index(),
            )
            .map_err(|_| ScenarioExecutorError::FailedToGetEpoch)?
            .as_typed::<VersionedConsensusManagerState>()
            .unwrap()
            .fully_update_and_into_latest_version()
            .epoch;

        let mut scenario = scenario_creator.create(ScenarioCore::new(
            self.network_definition.clone(),
            epoch,
            self.starting_nonce,
        ));
        let metadata = scenario.metadata().clone();

        (self.on_scenario_start)(&metadata);
        let mut previous = None;
        loop {
            let next = scenario
                .next(previous.as_ref())
                .map_err(|err| err.into_full(&scenario))
                .unwrap();
            match next {
                NextAction::Transaction(next) => {
                    let receipt = self.execute_transaction(&next.raw_transaction)?;
                    (self.on_transaction_executed)(&metadata, &next, &receipt, &self.database);
                    previous = Some(receipt);
                }
                NextAction::Completed(end_state) => {
                    match self.next_scenario_nonce_handling {
                        ScenarioStartNonceHandling::PreviousScenarioStartNoncePlus(increment) => {
                            self.starting_nonce += increment
                        }
                        ScenarioStartNonceHandling::PreviousScenarioEndNoncePlusOne => {
                            self.starting_nonce = end_state.next_unused_nonce
                        }
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    fn execute_transaction(
        &mut self,
        transaction: &RawNotarizedTransaction,
    ) -> Result<TransactionReceiptV1, ScenarioExecutorError> {
        let validator = NotarizedTransactionValidator::new(ValidationConfig::default(
            self.network_definition.id,
        ));
        let validated = validator
            .validate_from_raw(transaction)
            .map_err(ScenarioExecutorError::TransactionValidationError)?;

        let receipt = execute_transaction(
            &self.database,
            VmInit::new(&self.scrypto_vm, self.native_vm_extension.clone()),
            &ExecutionConfig::for_notarized_transaction(self.network_definition.clone()),
            &validated.get_executable(),
        );

        if let TransactionResult::Commit(commit) = &receipt.result {
            let database_updates = commit
                .state_updates
                .create_database_updates::<SpreadPrefixKeyMapper>();
            self.database.commit(&database_updates);
        };

        Ok(receipt)
    }
}

#[derive(Clone, Debug)]
pub enum ScenarioExecutorError {
    BootstrapFailed,
    FailedToGetEpoch,
    TransactionValidationError(TransactionValidationError),
}

#[derive(Clone, Debug)]
pub enum ScenarioStartNonceHandling {
    PreviousScenarioStartNoncePlus(u32),
    PreviousScenarioEndNoncePlusOne,
}
