use radix_transactions::{model::*, validation::TransactionValidator};

use crate::{
    system::bootstrap::FlashReceipt,
    transaction::*,
    vm::{VmInitialize, VmModules},
};

use super::*;

pub struct ProtocolUpdateExecutor {
    pub network_definition: NetworkDefinition,
    pub protocol_version: ProtocolVersion,
    pub batch_generator: Box<dyn ProtocolUpdateBatchGenerator>,
    pub start_at_batch_group_index: usize,
    pub start_at_batch_index_in_first_group: usize,
}

impl ProtocolUpdateExecutor {
    pub fn new_for_version(protocol_version: ProtocolVersion, settings: &ProtocolSettings) -> Self {
        let network_definition = settings.network_definition.clone();
        let batch_generator = settings.resolve_batch_generator_for_update(&protocol_version);
        Self {
            network_definition,
            protocol_version,
            batch_generator,
            start_at_batch_group_index: 0,
            start_at_batch_index_in_first_group: 0,
        }
    }

    pub fn continue_for_version(
        protocol_version: ProtocolVersion,
        settings: &ProtocolSettings,
        from_inclusive: (usize, usize),
    ) -> Self {
        let network_definition = settings.network_definition.clone();
        let batch_generator = settings.resolve_batch_generator_for_update(&protocol_version);
        Self {
            network_definition,
            protocol_version,
            batch_generator,
            start_at_batch_group_index: from_inclusive.0,
            start_at_batch_index_in_first_group: from_inclusive.1,
        }
    }

    pub fn new<US: UpdateSettings + 'static>(
        network_definition: NetworkDefinition,
        update_settings: US,
    ) -> Self {
        let protocol_version = US::protocol_version();
        let batch_generator = Box::new(update_settings.create_batch_generator());
        Self {
            network_definition,
            protocol_version,
            batch_generator,
            start_at_batch_group_index: 0,
            start_at_batch_index_in_first_group: 0,
        }
    }

    pub fn run_and_commit(self, store: &mut (impl SubstateDatabase + CommittableSubstateDatabase)) {
        self.run_and_commit_advanced(store, &mut (), &VmModules::default());
    }

    pub fn run_and_commit_advanced<
        S: SubstateDatabase + CommittableSubstateDatabase,
        H: ProtocolUpdateExecutionHooks,
        M: VmInitialize,
    >(
        self,
        store: &mut S,
        hooks: &mut H,
        vm_modules: &M,
    ) {
        let add_status_update = self.batch_generator.status_tracking_enabled();

        for (batch_group_index, batch_group_name) in self
            .batch_generator
            .batch_group_descriptors()
            .into_iter()
            .enumerate()
            .skip(self.start_at_batch_group_index)
        {
            let start_at_batch = if batch_group_index == self.start_at_batch_group_index {
                self.start_at_batch_index_in_first_group
            } else {
                0
            };
            for batch_index in start_at_batch..self.batch_generator.batch_count(batch_group_index) {
                let batch =
                    self.batch_generator
                        .generate_batch(store, batch_group_index, batch_index);
                for (transaction_index, transaction) in batch.transactions.into_iter().enumerate() {
                    let receipt = match &transaction {
                        ProtocolUpdateTransactionDetails::FlashV1Transaction(flash) => {
                            let db_updates = flash.state_updates.create_database_updates();
                            let receipt = if H::IS_ENABLED {
                                let before_store = &*store;
                                FlashReceipt::from_state_updates(
                                    flash.state_updates.clone(),
                                    before_store,
                                )
                                .into()
                            } else {
                                // Cheap fallback
                                TransactionReceipt::empty_commit_success()
                            };

                            store.commit(&db_updates);
                            receipt
                        }
                        ProtocolUpdateTransactionDetails::SystemTransactionV1 {
                            transaction,
                            is_genesis,
                            ..
                        } => {
                            let execution_config = if *is_genesis {
                                ExecutionConfig::for_genesis_transaction(
                                    self.network_definition.clone(),
                                )
                            } else {
                                ExecutionConfig::for_system_transaction(
                                    self.network_definition.clone(),
                                )
                            };
                            let execution_config = hooks.adapt_execution_config(execution_config);
                            let validator =
                                TransactionValidator::new(store, &self.network_definition);

                            let receipt = execute_and_commit_transaction(
                                store,
                                vm_modules,
                                &execution_config,
                                transaction
                                    .with_proofs_ref(btreeset![system_execution(
                                        SystemExecution::Protocol
                                    )])
                                    .into_executable(&validator)
                                    .expect(
                                        "Expected protocol update transaction to be preparable",
                                    ),
                            );
                            receipt.expect_commit_success();
                            receipt
                        }
                    };

                    if H::IS_ENABLED {
                        hooks.on_transaction_executed(OnProtocolTransactionExecuted {
                            protocol_version: self.protocol_version,
                            batch_group_index,
                            batch_group_name: &batch_group_name,
                            batch_index,
                            transaction_index,
                            transaction: &transaction,
                            receipt: &receipt,
                            resultant_store: store,
                        });
                    }
                }

                if add_status_update {
                    // In the node's executor, this will likely need to be in a separate transaction,
                    // so it gets tracked properly by the merkle tree etc
                    store.update_substate(
                        TRANSACTION_TRACKER,
                        PROTOCOL_UPDATE_STATUS_PARTITION,
                        ProtocolUpdateStatusField::Summary,
                        ProtocolUpdateStatusSummarySubstate::from_latest_version(
                            ProtocolUpdateStatusSummaryV1 {
                                protocol_version: self.protocol_version,
                                update_status: ProtocolUpdateStatus::InProgress {
                                    latest_commit: LatestProtocolUpdateCommitBatch {
                                        batch_group_index,
                                        batch_index,
                                    },
                                },
                            },
                        ),
                    );
                }
                if H::IS_ENABLED {
                    hooks.on_transaction_batch_committed(OnProtocolTransactionBatchCommitted {
                        protocol_version: self.protocol_version,
                        batch_group_index,
                        batch_group_name: &batch_group_name,
                        batch_index,
                        resultant_store: store,
                    });
                }
            }
        }
        if add_status_update {
            // In the node's executor, this will likely need to be in a separate transaction,
            // so it gets tracked properly by the merkle tree etc
            store.update_substate(
                TRANSACTION_TRACKER,
                PROTOCOL_UPDATE_STATUS_PARTITION,
                ProtocolUpdateStatusField::Summary,
                ProtocolUpdateStatusSummarySubstate::from_latest_version(
                    ProtocolUpdateStatusSummaryV1 {
                        protocol_version: self.protocol_version,
                        update_status: ProtocolUpdateStatus::Complete,
                    },
                ),
            );
        }
    }
}

#[allow(unused_variables)]
pub trait ProtocolUpdateExecutionHooks {
    // If false, hooks aren't called, so opt out of constructing things like receipts.
    const IS_ENABLED: bool = true;

    fn adapt_execution_config(&mut self, config: ExecutionConfig) -> ExecutionConfig {
        config
    }

    fn on_transaction_executed(&mut self, event: OnProtocolTransactionExecuted) {}

    fn on_transaction_batch_committed(&mut self, event: OnProtocolTransactionBatchCommitted) {}
}

/// Using a struct allows lots of parameters to be passed, without
/// having a large number of method arguments
pub struct OnProtocolTransactionExecuted<'a> {
    pub protocol_version: ProtocolVersion,
    pub batch_group_index: usize,
    pub batch_group_name: &'a str,
    pub batch_index: usize,
    pub transaction_index: usize,
    pub transaction: &'a ProtocolUpdateTransactionDetails,
    pub receipt: &'a TransactionReceipt,
    pub resultant_store: &'a mut dyn SubstateDatabase,
}

pub struct OnProtocolTransactionBatchCommitted<'a> {
    pub protocol_version: ProtocolVersion,
    pub batch_group_index: usize,
    pub batch_group_name: &'a str,
    pub batch_index: usize,
    pub resultant_store: &'a mut dyn SubstateDatabase,
}

impl ProtocolUpdateExecutionHooks for () {
    const IS_ENABLED: bool = false;
}

#[derive(Clone)]
pub struct ProtocolBuilder {
    settings: ProtocolSettings,
}

#[derive(Clone)]
pub struct ProtocolSettings {
    network_definition: NetworkDefinition,
    babylon: BabylonSettings,
    anemone: AnemoneSettings,
    bottlenose: BottlenoseSettings,
    cuttlefish: CuttlefishSettings,
}

impl ProtocolSettings {
    pub fn resolve_batch_generator_for_update(
        &self,
        protocol_version: &ProtocolVersion,
    ) -> Box<dyn ProtocolUpdateBatchGenerator> {
        match protocol_version {
            ProtocolVersion::Unbootstrapped => Box::new(NoOpBatchGenerator),
            ProtocolVersion::Babylon => Box::new(self.babylon.create_batch_generator()),
            ProtocolVersion::Anemone => Box::new(self.anemone.create_batch_generator()),
            ProtocolVersion::Bottlenose => Box::new(self.bottlenose.create_batch_generator()),
            ProtocolVersion::Cuttlefish => Box::new(self.cuttlefish.create_batch_generator()),
        }
    }
}

impl ProtocolBuilder {
    pub fn for_simulator() -> Self {
        Self::for_network(&NetworkDefinition::simulator())
    }

    pub fn for_network(network_definition: &NetworkDefinition) -> Self {
        Self {
            settings: ProtocolSettings {
                network_definition: network_definition.clone(),
                babylon: BabylonSettings::all_enabled_as_default_for_network(network_definition),
                anemone: AnemoneSettings::all_enabled_as_default_for_network(network_definition),
                bottlenose: BottlenoseSettings::all_enabled_as_default_for_network(
                    network_definition,
                ),
                cuttlefish: CuttlefishSettings::all_enabled_as_default_for_network(
                    network_definition,
                ),
            },
        }
    }

    pub fn configure_babylon(
        mut self,
        creator: impl FnOnce(BabylonSettings) -> BabylonSettings,
    ) -> Self {
        self.settings.babylon = creator(self.settings.babylon);
        self
    }

    pub fn configure_anemone(
        mut self,
        creator: impl FnOnce(AnemoneSettings) -> AnemoneSettings,
    ) -> Self {
        self.settings.anemone = creator(self.settings.anemone);
        self
    }

    pub fn configure_bottlenose(
        mut self,
        creator: impl FnOnce(BottlenoseSettings) -> BottlenoseSettings,
    ) -> Self {
        self.settings.bottlenose = creator(self.settings.bottlenose);
        self
    }

    pub fn configure_cuttlefish(
        mut self,
        creator: impl FnOnce(CuttlefishSettings) -> CuttlefishSettings,
    ) -> Self {
        self.settings.cuttlefish = creator(self.settings.cuttlefish);
        self
    }

    pub fn unbootstrapped(self) -> ProtocolExecutor {
        self.from_to(
            ProtocolVersion::Unbootstrapped,
            ProtocolVersion::Unbootstrapped,
        )
    }

    pub fn from_bootstrap_to(self, protocol_version: ProtocolVersion) -> ProtocolExecutor {
        self.from_to(ProtocolVersion::Unbootstrapped, protocol_version)
    }

    pub fn from_bootstrap_to_latest(self) -> ProtocolExecutor {
        self.from_bootstrap_to(ProtocolVersion::LATEST)
    }

    pub fn only_babylon(self) -> ProtocolExecutor {
        self.from_bootstrap_to(ProtocolVersion::Babylon)
    }

    /// The `start_protocol_version` is assumed to be currently active.
    /// If you want to also run bootstrap (i.e. enact `ProtocolVersion::Babylon`), use the `from_bootstrap_to` method.
    pub fn from_to(
        self,
        start_protocol_version: ProtocolVersion,
        end_protocol_version: ProtocolVersion,
    ) -> ProtocolExecutor {
        ProtocolExecutor::new(
            ProtocolExecutorStart::FromCompleted(start_protocol_version),
            end_protocol_version,
            self.settings,
        )
    }

    /// Discovers the start point from the database
    pub fn from_current_to_latest(self) -> ProtocolExecutor {
        self.from_current_to(ProtocolVersion::LATEST)
    }

    /// Discovers the start point from the database
    pub fn from_current_to(self, end_protocol_version: ProtocolVersion) -> ProtocolExecutor {
        ProtocolExecutor::new(
            ProtocolExecutorStart::ResumeFromCurrent,
            end_protocol_version,
            self.settings,
        )
    }
}

enum ProtocolExecutorStart {
    FromCompleted(ProtocolVersion),
    ResumeFromCurrent,
}

pub struct ProtocolExecutor {
    starting_at: ProtocolExecutorStart,
    update_until: ProtocolVersion,
    settings: ProtocolSettings,
}

impl ProtocolExecutor {
    fn new(
        starting_at: ProtocolExecutorStart,
        update_until: ProtocolVersion,
        settings: ProtocolSettings,
    ) -> Self {
        Self {
            starting_at,
            update_until,
            settings,
        }
    }

    pub fn commit_each_protocol_update(
        self,
        store: &mut (impl SubstateDatabase + CommittableSubstateDatabase),
    ) {
        for update_execution in self.each_protocol_update_executor(&*store) {
            update_execution.run_and_commit(store);
        }
    }

    /// For defaults:
    /// * For the hooks, you can use `&mut ()`
    /// * For the modules you can use `&mut VmModules::default()`
    pub fn commit_each_protocol_update_advanced(
        self,
        store: &mut (impl SubstateDatabase + CommittableSubstateDatabase),
        hooks: &mut impl ProtocolUpdateExecutionHooks,
        modules: &impl VmInitialize,
    ) {
        for update_execution in self.each_protocol_update_executor(&*store) {
            update_execution.run_and_commit_advanced(store, hooks, modules);
        }
    }

    pub fn each_target_protocol_version(
        &self,
        store: &impl SubstateDatabase,
    ) -> impl Iterator<Item = (ProtocolVersion, (usize, usize))> {
        let starting_at = match self.starting_at {
            ProtocolExecutorStart::FromCompleted(protocol_version) => ProtocolUpdateStatusSummary {
                protocol_version,
                update_status: ProtocolUpdateStatus::Complete,
            },
            ProtocolExecutorStart::ResumeFromCurrent => {
                ProtocolUpdateStatusSummarySubstate::load(store).into_unique_version()
            }
        };
        let from_protocol_version = starting_at.protocol_version;
        let until_protocol_version = self.update_until;
        ProtocolVersion::VARIANTS
            .into_iter()
            .filter_map(move |version| {
                if from_protocol_version == version && version <= until_protocol_version {
                    match &starting_at.update_status {
                        ProtocolUpdateStatus::Complete => None,
                        ProtocolUpdateStatus::InProgress { latest_commit } => Some((
                            version,
                            (
                                latest_commit.batch_group_index,
                                latest_commit.batch_index + 1,
                            ),
                        )),
                    }
                } else if from_protocol_version < version && version <= until_protocol_version {
                    Some((version, (0, 0)))
                } else {
                    None
                }
            })
    }

    pub fn each_protocol_update_executor(
        self,
        store: &impl SubstateDatabase,
    ) -> impl Iterator<Item = ProtocolUpdateExecutor> {
        self.each_target_protocol_version(store)
            .map(move |(version, start_from_inclusive)| {
                ProtocolUpdateExecutor::continue_for_version(
                    version,
                    &self.settings,
                    start_from_inclusive,
                )
            })
    }
}
