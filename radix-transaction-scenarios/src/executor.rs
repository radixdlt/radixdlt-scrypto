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

use scenarios::account_authorized_depositors::AccountAuthorizedDepositorsScenarioCreator;
use scenarios::account_locker::AccountLockerScenarioCreator;
use scenarios::fungible_resource::FungibleResourceScenarioCreator;
use scenarios::global_n_owned::GlobalNOwnedScenarioCreator;
use scenarios::kv_store_with_remote_type::KVStoreScenarioCreator;
use scenarios::max_transaction::MaxTransactionScenarioCreator;
use scenarios::metadata::MetadataScenario;
use scenarios::non_fungible_resource::NonFungibleResourceScenarioCreator;
use scenarios::non_fungible_resource_with_remote_type::NonFungibleResourceWithRemoteTypeScenarioCreator;
use scenarios::radiswap::RadiswapScenarioCreator;
use scenarios::royalties::RoyaltiesScenarioCreator;
use scenarios::transfer_xrd::TransferXrdScenarioCreator;

type ScenarioCreatorDynFunc = dyn FnOnce(ScenarioCore) -> Box<dyn ScenarioInstance>;

pub struct TransactionScenarioExecutor<D, W, E, F1, F2, F3, F4>
where
    D: SubstateDatabase + CommittableSubstateDatabase,
    W: WasmEngine,
    E: NativeVmExtension,
    F1: FnMut(&NetworkDefinition, ProtocolUpdate, &mut D),
    F2: FnMut(&ScenarioMetadata, &NextTransaction, &TransactionReceiptV1, &D),
    F3: FnMut(&ScenarioMetadata),
    F4: FnMut(&NetworkDefinition, &mut D),
{
    /* Environment */
    /// The substate database that the scenario will be run against.
    database: D,
    /// The Scrypto VM to use in executing the scenarios.
    scrypto_vm: ScryptoVm<W>,
    /// The Native VM to use in executing the scenarios.
    native_vm_extension: E,

    /* Execution */
    /// A filter that controls which scenarios are executed and which are not. When [`None`] then
    /// all scenarios are executed and when [`Some`] then some scenarios may be filtered out.
    filter: Option<ScenarioFilter>,
    /// A map of the scenarios registered on the executor. Not all registered scenarios will be
    /// executed, it merely informs the executor of the existence of these scenarios. Execution of a
    /// scenario requires that is passes the filter specified by the client.
    registered_scenarios: BTreeMap<Option<ProtocolUpdate>, Vec<Box<ScenarioCreatorDynFunc>>>,
    /// Controls whether the bootstrap process should be performed or not.
    bootstrap: bool,
    /// The first nonce to use in the execution of the scenarios.
    starting_nonce: u32,
    /// How the executor should handle nonces and how it should get the next nonce.
    next_scenario_nonce_handling: ScenarioStartNonceHandling,
    /// The network definition to use in the execution of the scenarios.
    network_definition: NetworkDefinition,

    /* Callbacks */
    /// A callback that is called when a new protocol requirement is encountered. This can be useful
    /// for clients who wish to apply protocol updates they wish.
    on_new_protocol_requirement_encountered: F1,
    /// A callback that is called when a scenario transaction is executed.
    on_transaction_executed: F2,
    /// A callback that is called when a new scenario is started.
    on_scenario_start: F3,
    /// A callback that is called after bootstrapping if bootstrapping is enabled.
    after_bootstrap: F4,
}

pub type DefaultTransactionScenarioExecutor<D> = TransactionScenarioExecutor<
    D,
    DefaultWasmEngine,
    NoExtension,
    fn(&NetworkDefinition, ProtocolUpdate, &mut D),
    fn(&ScenarioMetadata, &NextTransaction, &TransactionReceiptV1, &D),
    fn(&ScenarioMetadata),
    fn(&NetworkDefinition, &mut D),
>;

impl<D, W, E, F1, F2, F3, F4> TransactionScenarioExecutor<D, W, E, F1, F2, F3, F4>
where
    D: SubstateDatabase + CommittableSubstateDatabase,
    W: WasmEngine,
    E: NativeVmExtension,
    F1: FnMut(&NetworkDefinition, ProtocolUpdate, &mut D),
    F2: FnMut(&ScenarioMetadata, &NextTransaction, &TransactionReceiptV1, &D),
    F3: FnMut(&ScenarioMetadata),
    F4: FnMut(&NetworkDefinition, &mut D),
{
    pub fn new(
        database: D,
        network_definition: NetworkDefinition,
    ) -> DefaultTransactionScenarioExecutor<D> {
        DefaultTransactionScenarioExecutor::<D> {
            /* Environment */
            database,
            scrypto_vm: ScryptoVm::default(),
            native_vm_extension: NoExtension,
            /* Execution */
            filter: None,
            registered_scenarios: Default::default(),
            bootstrap: true,
            starting_nonce: 0,
            next_scenario_nonce_handling:
                ScenarioStartNonceHandling::PreviousScenarioEndNoncePlusOne,
            network_definition,
            /* Callbacks */
            on_new_protocol_requirement_encountered: |network_definition, protocol_update, db| {
                protocol_update
                    .generate_state_updates(db, network_definition)
                    .into_iter()
                    .for_each(|state_updates| {
                        db.commit(&state_updates.create_database_updates::<SpreadPrefixKeyMapper>())
                    })
            },
            on_transaction_executed: |_, _, _, _| {},
            on_scenario_start: |_| {},
            after_bootstrap: |_, _| {},
        }
        // Genesis Scenarios.
        .register_scenario_with_create_fn::<TransferXrdScenarioCreator>()
        .register_scenario_with_create_fn::<RadiswapScenarioCreator>()
        .register_scenario_with_create_fn::<MetadataScenario>()
        .register_scenario_with_create_fn::<FungibleResourceScenarioCreator>()
        .register_scenario_with_create_fn::<NonFungibleResourceScenarioCreator>()
        .register_scenario_with_create_fn::<AccountAuthorizedDepositorsScenarioCreator>()
        .register_scenario_with_create_fn::<GlobalNOwnedScenarioCreator>()
        .register_scenario_with_create_fn::<NonFungibleResourceWithRemoteTypeScenarioCreator>()
        .register_scenario_with_create_fn::<KVStoreScenarioCreator>()
        .register_scenario_with_create_fn::<MaxTransactionScenarioCreator>()
        .register_scenario_with_create_fn::<RoyaltiesScenarioCreator>()
        // Anemone Scenarios - None.
        // Bottlenose Scenarios.
        .register_scenario_with_create_fn::<AccountLockerScenarioCreator>()
    }

    /// Sets the Scrypto VM to use for the scenarios execution.
    pub fn scrypto_vm<NW: WasmEngine>(
        self,
        scrypto_vm: ScryptoVm<NW>,
    ) -> TransactionScenarioExecutor<D, NW, E, F1, F2, F3, F4> {
        TransactionScenarioExecutor {
            /* Environment */
            database: self.database,
            scrypto_vm,
            native_vm_extension: self.native_vm_extension,
            /* Execution */
            filter: self.filter,
            registered_scenarios: self.registered_scenarios,
            bootstrap: self.bootstrap,
            starting_nonce: self.starting_nonce,
            next_scenario_nonce_handling: self.next_scenario_nonce_handling,
            network_definition: self.network_definition,
            /* Callbacks */
            on_new_protocol_requirement_encountered: self.on_new_protocol_requirement_encountered,
            on_transaction_executed: self.on_transaction_executed,
            on_scenario_start: self.on_scenario_start,
            after_bootstrap: self.after_bootstrap,
        }
    }

    /// Sets the Native VM to use for the scenarios execution.
    pub fn native_vm_extension<NE: NativeVmExtension>(
        self,
        native_vm_extension: NE,
    ) -> TransactionScenarioExecutor<D, W, NE, F1, F2, F3, F4> {
        TransactionScenarioExecutor {
            /* Environment */
            database: self.database,
            scrypto_vm: self.scrypto_vm,
            native_vm_extension,
            /* Execution */
            filter: self.filter,
            registered_scenarios: self.registered_scenarios,
            bootstrap: self.bootstrap,
            starting_nonce: self.starting_nonce,
            next_scenario_nonce_handling: self.next_scenario_nonce_handling,
            network_definition: self.network_definition,
            /* Callbacks */
            on_new_protocol_requirement_encountered: self.on_new_protocol_requirement_encountered,
            on_transaction_executed: self.on_transaction_executed,
            on_scenario_start: self.on_scenario_start,
            after_bootstrap: self.after_bootstrap,
        }
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
    pub fn nonce_handling(mut self, nonce_handling: ScenarioStartNonceHandling) -> Self {
        self.next_scenario_nonce_handling = nonce_handling;
        self
    }

    /// Defines the filter to use for the execution of scenarios.
    pub fn filter(mut self, filter: ScenarioFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Removes the filter used for scenarios.
    pub fn remove_filter(mut self) -> Self {
        self.filter = None;
        self
    }

    /// Sets the callback to call when a new protocol requirement is encountered.
    pub fn on_new_protocol_requirement_encountered<
        NF1: FnMut(&NetworkDefinition, ProtocolUpdate, &mut D),
    >(
        self,
        callback: NF1,
    ) -> TransactionScenarioExecutor<D, W, E, NF1, F2, F3, F4> {
        TransactionScenarioExecutor {
            /* Environment */
            database: self.database,
            scrypto_vm: self.scrypto_vm,
            native_vm_extension: self.native_vm_extension,
            /* Execution */
            filter: self.filter,
            registered_scenarios: self.registered_scenarios,
            bootstrap: self.bootstrap,
            starting_nonce: self.starting_nonce,
            next_scenario_nonce_handling: self.next_scenario_nonce_handling,
            network_definition: self.network_definition,
            /* Callbacks */
            on_new_protocol_requirement_encountered: callback,
            on_transaction_executed: self.on_transaction_executed,
            on_scenario_start: self.on_scenario_start,
            after_bootstrap: self.after_bootstrap,
        }
    }

    /// Sets the callback to call after executing a scenario transaction.
    pub fn on_transaction_executed<
        NF2: FnMut(&ScenarioMetadata, &NextTransaction, &TransactionReceiptV1, &D),
    >(
        self,
        callback: NF2,
    ) -> TransactionScenarioExecutor<D, W, E, F1, NF2, F3, F4> {
        TransactionScenarioExecutor {
            /* Environment */
            database: self.database,
            scrypto_vm: self.scrypto_vm,
            native_vm_extension: self.native_vm_extension,
            /* Execution */
            filter: self.filter,
            registered_scenarios: self.registered_scenarios,
            bootstrap: self.bootstrap,
            starting_nonce: self.starting_nonce,
            next_scenario_nonce_handling: self.next_scenario_nonce_handling,
            network_definition: self.network_definition,
            /* Callbacks */
            on_new_protocol_requirement_encountered: self.on_new_protocol_requirement_encountered,
            on_transaction_executed: callback,
            on_scenario_start: self.on_scenario_start,
            after_bootstrap: self.after_bootstrap,
        }
    }

    /// A callback that is called when a new scenario is started.
    pub fn on_scenario_start<NF3: FnMut(&ScenarioMetadata)>(
        self,
        callback: NF3,
    ) -> TransactionScenarioExecutor<D, W, E, F1, F2, NF3, F4> {
        TransactionScenarioExecutor {
            /* Environment */
            database: self.database,
            scrypto_vm: self.scrypto_vm,
            native_vm_extension: self.native_vm_extension,
            /* Execution */
            filter: self.filter,
            registered_scenarios: self.registered_scenarios,
            bootstrap: self.bootstrap,
            starting_nonce: self.starting_nonce,
            next_scenario_nonce_handling: self.next_scenario_nonce_handling,
            network_definition: self.network_definition,
            /* Callbacks */
            on_new_protocol_requirement_encountered: self.on_new_protocol_requirement_encountered,
            on_transaction_executed: self.on_transaction_executed,
            on_scenario_start: callback,
            after_bootstrap: self.after_bootstrap,
        }
    }

    pub fn without_protocol_updates(
        self,
    ) -> TransactionScenarioExecutor<
        D,
        W,
        E,
        fn(&NetworkDefinition, ProtocolUpdate, &mut D),
        F2,
        F3,
        F4,
    > {
        TransactionScenarioExecutor {
            /* Environment */
            database: self.database,
            scrypto_vm: self.scrypto_vm,
            native_vm_extension: self.native_vm_extension,
            /* Execution */
            filter: self.filter,
            registered_scenarios: self.registered_scenarios,
            bootstrap: self.bootstrap,
            starting_nonce: self.starting_nonce,
            next_scenario_nonce_handling: self.next_scenario_nonce_handling,
            network_definition: self.network_definition,
            /* Callbacks */
            on_new_protocol_requirement_encountered: |_, _, _| {},
            on_transaction_executed: self.on_transaction_executed,
            on_scenario_start: self.on_scenario_start,
            after_bootstrap: self.after_bootstrap,
        }
    }

    /// A callback that is called after bootstrapping if bootstrap is enabled.
    pub fn after_bootstrap<NF4: FnMut(&NetworkDefinition, &mut D)>(
        self,
        callback: NF4,
    ) -> TransactionScenarioExecutor<D, W, E, F1, F2, F3, NF4> {
        TransactionScenarioExecutor {
            /* Environment */
            database: self.database,
            scrypto_vm: self.scrypto_vm,
            native_vm_extension: self.native_vm_extension,
            /* Execution */
            filter: self.filter,
            registered_scenarios: self.registered_scenarios,
            bootstrap: self.bootstrap,
            starting_nonce: self.starting_nonce,
            next_scenario_nonce_handling: self.next_scenario_nonce_handling,
            network_definition: self.network_definition,
            /* Callbacks */
            on_new_protocol_requirement_encountered: self.on_new_protocol_requirement_encountered,
            on_transaction_executed: self.on_transaction_executed,
            on_scenario_start: self.on_scenario_start,
            after_bootstrap: callback,
        }
    }

    pub fn execute(mut self) -> Result<ScenarioExecutionReceipt<D>, ScenarioExecutorError> {
        // Bootstrapping if needed
        if self.bootstrap {
            Bootstrapper::new(
                self.network_definition.clone(),
                &mut self.database,
                VmInit::new(&self.scrypto_vm, self.native_vm_extension.clone()),
                false,
            )
            .bootstrap_test_default()
            .ok_or(ScenarioExecutorError::BootstrapFailed)?;
            (self.after_bootstrap)(&self.network_definition, &mut self.database);
        };

        // Getting the scenario builder functions of the scenarios that we will execute. There is a
        // canonical order to these function batches which is that the genesis functions come first,
        // then anemone, bottlenose, and so on. This order is enforced by this function and by the
        // ordering of the `ProtocolUpdate` enum variants. Within a protocol update (or requirement)
        // the canonical order is as seen in the [`new`] function.
        for protocol_requirement in std::iter::once(None).chain(ProtocolUpdate::VARIANTS.map(Some))
        {
            // When a new protocol requirement is encountered the appropriate callback is called to
            // inform the client of this event.
            if let Some(protocol_update) = protocol_requirement {
                (self.on_new_protocol_requirement_encountered)(
                    &self.network_definition,
                    protocol_update,
                    &mut self.database,
                );
            }

            // Build each scenario and execute it.
            let mut next_nonce = self.starting_nonce;
            for scenario_builder in self
                .registered_scenarios
                .remove(&protocol_requirement)
                .unwrap_or_default()
                .into_iter()
            {
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

                // Before executing the scenario determine if it's valid for the current filter that
                // the client specified.
                let passes_filter = match self.filter {
                    // Ensure that the scenario name from the metadata is in the list of exact
                    // scenarios. Otherwise continue to the next.
                    Some(ScenarioFilter::ExactScenarios(ref exact_scenarios)) => {
                        exact_scenarios.contains(metadata.logical_name)
                    }
                    // Execute only ones from a particular protocol update.
                    Some(ScenarioFilter::SpecificProtocolUpdate(protocol_update)) => {
                        protocol_requirement.is_some_and(|update| protocol_update == update)
                    }
                    // Execute only scenarios that are valid before a particular protocol update.
                    Some(ScenarioFilter::AllValidBeforeProtocolUpdate(protocol_update)) => {
                        match protocol_update {
                            Boundary::Inclusive(protocol_update) => RangeToInclusive {
                                end: Some(protocol_update),
                            }
                            .contains(&protocol_requirement),
                            Boundary::Exclusive(protocol_update) => RangeTo {
                                end: Some(protocol_update),
                            }
                            .contains(&protocol_requirement),
                        }
                    }
                    // Execute only scenarios that are valid after a particular protocol update.
                    Some(ScenarioFilter::AllValidAfterProtocolUpdate(protocol_update)) => {
                        RangeFrom {
                            start: Some(protocol_update),
                        }
                        .contains(&protocol_requirement)
                    }
                    // No filter is specified, the scenario is valid.
                    None => true,
                };
                if !passes_filter {
                    continue;
                }

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
                            (self.on_transaction_executed)(
                                &metadata,
                                &next,
                                &receipt,
                                &self.database,
                            );
                            previous = Some(receipt);
                        }
                        NextAction::Completed(end_state) => {
                            match self.next_scenario_nonce_handling {
                                ScenarioStartNonceHandling::PreviousScenarioStartNoncePlus(
                                    increment,
                                ) => next_nonce += increment,
                                ScenarioStartNonceHandling::PreviousScenarioEndNoncePlusOne => {
                                    next_nonce = end_state.next_unused_nonce
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }

        Ok(ScenarioExecutionReceipt {
            database: self.database,
        })
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

    fn register_scenario<
        S: ScenarioCreator<'static>,
        F: FnOnce(ScenarioCore) -> S::Instance + 'static,
    >(
        mut self,
        create: F,
    ) -> Self {
        let requirement = S::SCENARIO_PROTOCOL_REQUIREMENT;
        self.registered_scenarios
            .entry(requirement)
            .or_default()
            .push(
                Box::new(move |core| Box::new(create(core)) as Box<dyn ScenarioInstance>)
                    as Box<ScenarioCreatorDynFunc>,
            );
        self
    }

    fn register_scenario_with_create_fn<S: ScenarioCreator<'static> + 'static>(self) -> Self {
        self.register_scenario::<S, _>(S::create)
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

pub struct ScenarioExecutionReceipt<D: SubstateDatabase + CommittableSubstateDatabase> {
    pub database: D,
}

#[derive(Clone, Debug)]
pub enum ScenarioFilter {
    /// An exact set of scenarios to execute, specified by their scenario name. Before a scenario is
    /// executed its name is checked against this set. It is executed if it's name is a member of
    /// this set and ignored otherwise. Note that there is no check to ensure that the names in this
    /// filter are valid. If an incorrect scenario name is provided in the set then it will simply
    /// be ignored and wont match against anything.
    ExactScenarios(BTreeSet<String>),
    /// Only executes the scenarios of a particular protocol update.
    SpecificProtocolUpdate(ProtocolUpdate),
    /// Filters scenarios based on their protocol version requirements executing all scenarios up
    /// until the ones that require the specified protocol update. As an example, to execute all
    /// scenarios from Genesis to Anemone this variant could be used and  specified a protocol
    /// update of [`ProtocolUpdate::Anemone`].
    AllValidBeforeProtocolUpdate(Boundary<ProtocolUpdate>),
    /// Filters scenarios based on their protocol version requirements executing all scenarios from
    /// the specified protocol update and up until the end. As an example, to execute all scenarios
    /// from Anemone to the end then this variant could be used specified a protocol update of
    /// [`ProtocolUpdate::Anemone`]. The specified protocol update is included.
    AllValidAfterProtocolUpdate(ProtocolUpdate),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Boundary<T> {
    Inclusive(T),
    Exclusive(T),
}
