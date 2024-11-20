#![allow(clippy::type_complexity)]

use self::internal_prelude::*;
use super::*;
use radix_engine::blueprints::consensus_manager::*;
use radix_engine::system::system_db_reader::*;
use radix_engine::updates::*;
use radix_engine::vm::*;
use radix_substate_store_interface::interface::*;
use radix_transactions::errors::*;
use radix_transactions::validation::*;
use sbor::prelude::*;

use scenarios::all_scenarios_iter;

#[derive(Clone, Debug)]
pub enum ScenarioTrigger {
    AtStartOfProtocolVersions(BTreeSet<ProtocolVersion>),
    AtStartOfEveryProtocolVersion,
    AfterCompletionOfAllProtocolUpdates,
}

#[derive(Clone, Debug)]
pub enum ScenarioFilter {
    SpecificScenariosByName(BTreeSet<String>),
    AllScenariosValidAtProtocolVersion,
    AllScenariosFirstValidAtProtocolVersion,
}

#[allow(unused_variables)]
pub trait ScenarioExecutionHooks<S: SubstateDatabase> {
    // If false, hooks aren't called, so opt out of constructing things like receipts.
    const IS_ENABLED: bool = true;

    fn adapt_execution_config(&mut self, config: ExecutionConfig) -> ExecutionConfig {
        config
    }

    fn on_transaction_executed(&mut self, event: OnScenarioTransactionExecuted<S>) {}
    fn on_scenario_started(&mut self, event: OnScenarioStarted<S>) {}
    fn on_scenario_ended(&mut self, event: OnScenarioEnded<S>) {}
}

impl<S: SubstateDatabase> ScenarioExecutionHooks<S> for () {
    const IS_ENABLED: bool = false;
}

pub struct OnScenarioTransactionExecuted<'a, S: SubstateDatabase> {
    pub metadata: &'a ScenarioMetadata,
    pub transaction: &'a NextTransaction,
    pub receipt: &'a TransactionReceipt,
    pub database: &'a mut S,
    pub network_definition: &'a NetworkDefinition,
    pub current_protocol_version: ProtocolVersion,
}

pub struct OnScenarioStarted<'a, S: SubstateDatabase> {
    pub metadata: &'a ScenarioMetadata,
    pub database: &'a mut S,
    pub network_definition: &'a NetworkDefinition,
    pub current_protocol_version: ProtocolVersion,
}

pub struct OnScenarioEnded<'a, S: SubstateDatabase> {
    pub metadata: &'a ScenarioMetadata,
    pub end_state: &'a EndState,
    pub database: &'a mut S,
    pub network_definition: &'a NetworkDefinition,
    pub current_protocol_version: ProtocolVersion,
}

pub struct TransactionScenarioExecutor<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase,
{
    /* Environment */
    /// The substate database that the scenario will be run against.
    database: D,
    validator: TransactionValidator,

    /* Execution */
    /// The first nonce to use in the execution of the scenarios.
    starting_nonce: u32,
    /// How the executor should handle nonces and how it should get the next nonce.
    next_scenario_nonce_handling: ScenarioStartNonceHandling,
    /// The network definition to use in the execution of the scenarios.
    network_definition: NetworkDefinition,
}

impl<D> TransactionScenarioExecutor<D>
where
    D: SubstateDatabase + CommittableSubstateDatabase,
{
    pub fn new(database: D, network_definition: NetworkDefinition) -> Self {
        let validator = TransactionValidator::new(&database, &network_definition);
        Self {
            /* Environment */
            database,
            validator,
            /* Execution */
            starting_nonce: 0,
            next_scenario_nonce_handling:
                ScenarioStartNonceHandling::PreviousScenarioEndNoncePlusOne,
            network_definition: network_definition.clone(),
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

    pub fn into_database(self) -> D {
        self.database
    }

    /// Each scenario is executed once, when it first becomes valid.
    /// If you don't need any hooks, use `&mut ()`.
    pub fn execute_every_protocol_update_and_scenario(
        &mut self,
        scenario_hooks: &mut impl ScenarioExecutionHooks<D>,
    ) -> Result<(), ScenarioExecutorError> {
        self.execute_protocol_updates_and_scenarios(
            |builder| builder.from_bootstrap_to_latest(),
            ScenarioTrigger::AtStartOfEveryProtocolVersion,
            ScenarioFilter::AllScenariosFirstValidAtProtocolVersion,
            scenario_hooks,
            &mut (),
            &VmModules::default(),
        )
    }

    pub fn execute_protocol_updates_and_scenarios(
        &mut self,
        protocol: impl FnOnce(ProtocolBuilder) -> ProtocolExecutor,
        trigger: ScenarioTrigger,
        filter: ScenarioFilter,
        scenario_hooks: &mut impl ScenarioExecutionHooks<D>,
        protocol_update_hooks: &mut impl ProtocolUpdateExecutionHooks,
        modules: &impl VmInitialize,
    ) -> Result<(), ScenarioExecutorError> {
        let protocol_executor = protocol(ProtocolBuilder::for_network(&self.network_definition));
        let last_version = protocol_executor
            .each_target_protocol_version(&self.database)
            .last()
            .map(|(version, _)| version);

        for protocol_update_executor in
            protocol_executor.each_protocol_update_executor(&self.database)
        {
            let new_protocol_version = protocol_update_executor.protocol_version;
            protocol_update_executor.run_and_commit_advanced(
                &mut self.database,
                protocol_update_hooks,
                modules,
            );

            // Update the validator in case the settings have changed due to the protocol update
            self.validator = TransactionValidator::new(&self.database, &self.network_definition);

            self.execute_scenarios_at_new_protocol_version(
                new_protocol_version,
                &trigger,
                &filter,
                Some(new_protocol_version) == last_version,
                scenario_hooks,
                modules,
            )?;
        }

        Ok(())
    }

    fn execute_scenarios_at_new_protocol_version(
        &mut self,
        at_version: ProtocolVersion,
        trigger: &ScenarioTrigger,
        filter: &ScenarioFilter,
        is_last: bool,
        scenario_hooks: &mut impl ScenarioExecutionHooks<D>,
        modules: &impl VmInitialize,
    ) -> Result<(), ScenarioExecutorError> {
        let trigger_applies = match trigger {
            ScenarioTrigger::AtStartOfProtocolVersions(set) => set.contains(&at_version),
            ScenarioTrigger::AtStartOfEveryProtocolVersion => true,
            ScenarioTrigger::AfterCompletionOfAllProtocolUpdates => is_last,
        };

        if !trigger_applies {
            return Ok(());
        }

        let matching_scenarios = all_scenarios_iter().filter(|creator| {
            let metadata = creator.metadata();
            let is_valid = at_version >= metadata.protocol_min_requirement
                && at_version <= metadata.protocol_max_requirement;
            if !is_valid {
                return false;
            }
            match filter {
                ScenarioFilter::SpecificScenariosByName(scenario_names) => {
                    scenario_names.contains(metadata.logical_name)
                }
                ScenarioFilter::AllScenariosValidAtProtocolVersion => true,
                ScenarioFilter::AllScenariosFirstValidAtProtocolVersion => {
                    metadata.protocol_min_requirement == at_version
                }
            }
        });

        for scenario_creator in matching_scenarios {
            self.execute_scenario(scenario_creator, scenario_hooks, modules, at_version)?;
        }

        Ok(())
    }

    pub fn execute_scenario(
        &mut self,
        scenario_creator: &dyn ScenarioCreatorObjectSafe,
        scenario_hooks: &mut impl ScenarioExecutionHooks<D>,
        modules: &impl VmInitialize,
        current_protocol_version: ProtocolVersion,
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

        scenario_hooks.on_scenario_started(OnScenarioStarted {
            metadata: &metadata,
            current_protocol_version,
            database: &mut self.database,
            network_definition: &self.network_definition,
        });
        let mut previous = None;
        loop {
            let next = scenario
                .next(previous.as_ref())
                .map_err(|err| err.into_full(&scenario))
                .unwrap();
            match next {
                NextAction::Transaction(next) => {
                    let receipt = self.execute_transaction(
                        &next.raw_transaction,
                        &scenario_hooks.adapt_execution_config(
                            ExecutionConfig::for_notarized_transaction(
                                self.network_definition.clone(),
                            ),
                        ),
                        modules,
                    )?;
                    scenario_hooks.on_transaction_executed(OnScenarioTransactionExecuted {
                        metadata: &metadata,
                        transaction: &next,
                        receipt: &receipt,
                        database: &mut self.database,
                        current_protocol_version,
                        network_definition: &self.network_definition,
                    });
                    previous = Some(receipt);
                }
                NextAction::Completed(end_state) => {
                    scenario_hooks.on_scenario_ended(OnScenarioEnded {
                        metadata: &metadata,
                        end_state: &end_state,
                        database: &mut self.database,
                        current_protocol_version,
                        network_definition: &self.network_definition,
                    });
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
        execution_config: &ExecutionConfig,
        modules: &impl VmInitialize,
    ) -> Result<TransactionReceipt, ScenarioExecutorError> {
        let validated = transaction
            .validate(&self.validator)
            .map_err(ScenarioExecutorError::TransactionValidationError)?;

        let receipt = execute_transaction(
            &self.database,
            modules,
            execution_config,
            validated.create_executable(),
        );

        if let TransactionResult::Commit(commit) = &receipt.result {
            let database_updates = commit.state_updates.create_database_updates();
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
