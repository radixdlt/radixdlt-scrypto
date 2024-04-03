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

use account_authorized_depositors::AccountAuthorizedDepositorsScenarioCreator;
use account_locker::AccountLockerScenarioCreator;
use fungible_resource::FungibleResourceScenarioCreator;
use global_n_owned::GlobalNOwnedScenarioCreator;
use kv_store_with_remote_type::KVStoreScenarioCreator;
use max_transaction::MaxTransactionScenarioCreator;
use metadata::MetadataScenario;
use non_fungible_resource::NonFungibleResourceScenarioCreator;
use non_fungible_resource_with_remote_type::NonFungibleResourceWithRemoteTypeScenarioCreator;
use radiswap::RadiswapScenarioCreator;
use royalties::RoyaltiesScenarioCreator;
use transfer_xrd::TransferXrdScenarioCreator;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScenarioRequirements {
    /// These scenarios do not require any protocol updates and are expected to be run at genesis
    /// and be compatible with it.
    Genesis,
    /// These scenarios require one or more protocol updates and this enum specifies the protocol
    /// update that they require.
    ProtocolUpdateUpTo(ProtocolUpdate),
}

impl ScenarioRequirements {
    pub fn all() -> &'static [ScenarioRequirements] {
        &[
            ScenarioRequirements::Genesis,
            ScenarioRequirements::ProtocolUpdateUpTo(ProtocolUpdate::Anemone),
            ScenarioRequirements::ProtocolUpdateUpTo(ProtocolUpdate::Bottlenose),
        ]
    }
}

macro_rules! define_scenario_builders {
    (
        $(
            $requirement: expr => [
                $($func: expr),* $(,)?
            ]
        ),* $(,)?
    ) => {
        pub fn scenario_builders() -> IndexMap<
            ScenarioRequirements,
            Vec<Box<dyn FnOnce(ScenarioCore) -> Box<dyn ScenarioInstance>>>
        > {
            ::sbor::prelude::indexmap! {
                $(
                    $requirement => vec![
                        $(
                            Box::new($func)
                                as Box<dyn FnOnce(ScenarioCore) -> Box<dyn ScenarioInstance>>
                        ),*
                    ]
                ),*
            }
        }
    };
}

define_scenario_builders! {
    // The set of scenarios that should be run after genesis.
    ScenarioRequirements::Genesis => [
        TransferXrdScenarioCreator::create,
        RadiswapScenarioCreator::create,
        MetadataScenario::create,
        FungibleResourceScenarioCreator::create,
        NonFungibleResourceScenarioCreator::create,
        AccountAuthorizedDepositorsScenarioCreator::create,
        GlobalNOwnedScenarioCreator::create,
        NonFungibleResourceWithRemoteTypeScenarioCreator::create,
        KVStoreScenarioCreator::create,
        MaxTransactionScenarioCreator::create,
        RoyaltiesScenarioCreator::create,
    ],
    // The set of scenarios that should be run after the anemone protocol update.
    ScenarioRequirements::ProtocolUpdateUpTo(ProtocolUpdate::Anemone) => [],
    // The set of scenarios that should be run after the bottlenose protocol update.
    ScenarioRequirements::ProtocolUpdateUpTo(ProtocolUpdate::Bottlenose) => [
        AccountLockerScenarioCreator::create
    ],
}

pub struct TransactionScenarioExecutor<D, W, E, F1, F2>
where
    D: SubstateDatabase + CommittableSubstateDatabase,
    W: WasmEngine,
    E: NativeVmExtension,
    F1: FnMut(&ScenarioMetadata, &NextTransaction, &TransactionReceiptV1, &D),
    F2: FnMut(&ScenarioMetadata),
{
    /* Environment */
    /// The substate database that the scenario will be run against.
    database: D,
    /// The Scrypto VM to use in executing the scenarios.
    scrypto_vm: ScryptoVm<W>,
    /// The Native VM to use in executing the scenarios.
    native_vm: NativeVm<E>,

    /* Execution */
    /// The scenarios that the executor should execute. They are not specified per scenario but per
    /// requirement. Meaning that through this field a client may opt-in or opt-out of the scenarios
    /// of an entire protocol update but not individual ones.
    scenarios_to_execute: BTreeSet<ScenarioRequirements>,
    /// Controls whether the bootstrap process should be performed or not.
    bootstrap: bool,
    /// The first nonce to use in the execution of the scenarios.
    starting_nonce: u32,
    /// How the executor should handle nonces and how it should get the next nonce.
    nonce_handling: NonceHandling,
    /// The network definition to use in the execution of the scenarios.
    network_definition: NetworkDefinition,

    /* Callbacks */
    /// A callback that is called when a scenario transaction is executed.
    on_transaction_execution: F1,
    /// A callback that is called when a new scenario is started.
    on_scenario_start: F2,
}

pub type DefaultTransactionScenarioExecutor<D> = TransactionScenarioExecutor<
    D,
    DefaultWasmEngine,
    NoExtension,
    fn(&ScenarioMetadata, &NextTransaction, &TransactionReceiptV1, &D),
    fn(&ScenarioMetadata),
>;

impl<D, W, E, F1, F2> TransactionScenarioExecutor<D, W, E, F1, F2>
where
    D: SubstateDatabase + CommittableSubstateDatabase,
    W: WasmEngine,
    E: NativeVmExtension,
    F1: FnMut(&ScenarioMetadata, &NextTransaction, &TransactionReceiptV1, &D),
    F2: FnMut(&ScenarioMetadata),
{
    pub fn new(database: D) -> DefaultTransactionScenarioExecutor<D> {
        TransactionScenarioExecutor {
            /* Environment */
            database,
            scrypto_vm: ScryptoVm::default(),
            native_vm: NativeVm::new(),
            /* Execution */
            scenarios_to_execute: ScenarioRequirements::all().iter().copied().collect(),
            bootstrap: true,
            starting_nonce: 0,
            nonce_handling: NonceHandling::Increment(1),
            network_definition: NetworkDefinition::simulator(),
            /* Callbacks */
            on_transaction_execution: |_, _, _, _| {},
            on_scenario_start: |_| {},
        }
    }

    /// Sets the Scrypto VM to use for the scenarios execution.
    pub fn scrypto_vm<NW: WasmEngine>(
        self,
        scrypto_vm: ScryptoVm<NW>,
    ) -> TransactionScenarioExecutor<D, NW, E, F1, F2> {
        TransactionScenarioExecutor {
            /* Environment */
            database: self.database,
            scrypto_vm,
            native_vm: self.native_vm,
            /* Execution */
            scenarios_to_execute: self.scenarios_to_execute,
            bootstrap: self.bootstrap,
            starting_nonce: self.starting_nonce,
            nonce_handling: self.nonce_handling,
            network_definition: self.network_definition,
            /* Callbacks */
            on_transaction_execution: self.on_transaction_execution,
            on_scenario_start: self.on_scenario_start,
        }
    }

    /// Sets the Native VM to use for the scenarios execution.
    pub fn native_vm<NE: NativeVmExtension>(
        self,
        native_vm: NativeVm<NE>,
    ) -> TransactionScenarioExecutor<D, W, NE, F1, F2> {
        TransactionScenarioExecutor {
            /* Environment */
            database: self.database,
            scrypto_vm: self.scrypto_vm,
            native_vm,
            /* Execution */
            scenarios_to_execute: self.scenarios_to_execute,
            bootstrap: self.bootstrap,
            starting_nonce: self.starting_nonce,
            nonce_handling: self.nonce_handling,
            network_definition: self.network_definition,
            /* Callbacks */
            on_transaction_execution: self.on_transaction_execution,
            on_scenario_start: self.on_scenario_start,
        }
    }

    /// The scenarios that the executor should execute. They are not specified per scenario but per
    /// requirement. Meaning that through this field a client may opt-in or opt-out of the scenarios
    /// of an entire protocol update but not individual ones.
    pub fn scenarios_to_execute(
        mut self,
        scenario_requirements: impl IntoIterator<Item = ScenarioRequirements>,
    ) -> Self {
        self.scenarios_to_execute = scenario_requirements.into_iter().collect();
        self
    }

    /// Controls whether the bootstrap process should be performed or not.
    pub fn bootstrap(mut self, bootstrap: bool) -> Self {
        self.bootstrap = bootstrap;
        self
    }

    /// Sets the starting nonce for executing scenarios.
    pub fn starting_nonce(mut self, starting_nonce: u32) -> Self {
        self.starting_nonce = starting_nonce;
        self
    }

    /// Defines how the executor should handle nonces.
    pub fn nonce_handling(mut self, nonce_handling: NonceHandling) -> Self {
        self.nonce_handling = nonce_handling;
        self
    }

    /// Sets the network definition to use for the scenario execution.
    pub fn network_definition(mut self, network_definition: NetworkDefinition) -> Self {
        self.network_definition = network_definition;
        self
    }

    /// Sets the callback to call after executing a scenario transaction.
    pub fn on_transaction_execution<
        NF1: FnMut(&ScenarioMetadata, &NextTransaction, &TransactionReceiptV1, &D),
    >(
        self,
        callback: NF1,
    ) -> TransactionScenarioExecutor<D, W, E, NF1, F2> {
        TransactionScenarioExecutor {
            /* Environment */
            database: self.database,
            scrypto_vm: self.scrypto_vm,
            native_vm: self.native_vm,
            /* Execution */
            scenarios_to_execute: self.scenarios_to_execute,
            bootstrap: self.bootstrap,
            starting_nonce: self.starting_nonce,
            nonce_handling: self.nonce_handling,
            network_definition: self.network_definition,
            /* Callbacks */
            on_transaction_execution: callback,
            on_scenario_start: self.on_scenario_start,
        }
    }

    /// A callback that is called when a new scenario is started.
    pub fn on_scenario_start<NF2: FnMut(&ScenarioMetadata)>(
        self,
        callback: NF2,
    ) -> TransactionScenarioExecutor<D, W, E, F1, NF2> {
        TransactionScenarioExecutor {
            /* Environment */
            database: self.database,
            scrypto_vm: self.scrypto_vm,
            native_vm: self.native_vm,
            /* Execution */
            scenarios_to_execute: self.scenarios_to_execute,
            bootstrap: self.bootstrap,
            starting_nonce: self.starting_nonce,
            nonce_handling: self.nonce_handling,
            network_definition: self.network_definition,
            /* Callbacks */
            on_transaction_execution: self.on_transaction_execution,
            on_scenario_start: callback,
        }
    }

    pub fn execute(mut self) -> Result<CompletedExecutionReceipt<D>, ScenarioExecutorError> {
        // Bootstrapping if needed
        if self.bootstrap {
            Bootstrapper::new(
                self.network_definition.clone(),
                &mut self.database,
                Vm::new(&self.scrypto_vm, self.native_vm.clone()),
                false,
            )
            .bootstrap_test_default()
            .ok_or(ScenarioExecutorError::BootstrapFailed)?;
        };

        // Running the scenarios.
        let mut next_nonce = self.starting_nonce;
        let mut scenario_builders = scenario_builders();
        for requirement in self.scenarios_to_execute.clone() {
            if let ScenarioRequirements::ProtocolUpdateUpTo(protocol_update) = requirement {
                protocol_update
                    .generate_state_updates(&self.database, &self.network_definition)
                    .into_iter()
                    .for_each(|state_updates| {
                        self.database.commit(
                            &state_updates.create_database_updates::<SpreadPrefixKeyMapper>(),
                        )
                    })
            };

            let scenario_builders = scenario_builders
                .shift_remove(&requirement)
                .unwrap_or_default();
            for scenario_builder in scenario_builders {
                let epoch = SystemDatabaseReader::new(&self.database)
                    .read_object_field(
                        CONSENSUS_MANAGER.as_node_id(),
                        ModuleId::Main,
                        ConsensusManagerField::State.field_index(),
                    )
                    .map_err(|_| ScenarioExecutorError::FailedToGetEpoch)?
                    .as_typed::<VersionedConsensusManagerState>()
                    .unwrap()
                    .into_latest()
                    .epoch;
                let mut scenario = scenario_builder(ScenarioCore::new(
                    self.network_definition.clone(),
                    epoch,
                    next_nonce,
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
                            (self.on_transaction_execution)(
                                &metadata,
                                &next,
                                &receipt,
                                &self.database,
                            );
                            previous = Some(receipt);
                        }
                        NextAction::Completed(end_state) => {
                            match self.nonce_handling {
                                NonceHandling::Increment(increment) => next_nonce += increment,
                                NonceHandling::StartAtNextAvailable => {
                                    next_nonce = end_state.next_unused_nonce
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }

        Ok(CompletedExecutionReceipt {
            database: self.database,
        })
    }

    fn execute_transaction(
        &mut self,
        transaction: &RawNotarizedTransaction,
    ) -> Result<TransactionReceiptV1, ScenarioExecutorError> {
        let network = NetworkDefinition::simulator();
        let validator = NotarizedTransactionValidator::new(ValidationConfig::default(network.id));
        let validated = validator
            .validate_from_raw(transaction)
            .map_err(ScenarioExecutorError::TransactionValidationError)?;

        let receipt = execute_transaction(
            &self.database,
            Vm::new(&self.scrypto_vm, self.native_vm.clone()),
            &CostingParameters::default(),
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
pub enum NonceHandling {
    Increment(u32),
    StartAtNextAvailable,
}

pub struct CompletedExecutionReceipt<D: SubstateDatabase + CommittableSubstateDatabase> {
    pub database: D,
}
