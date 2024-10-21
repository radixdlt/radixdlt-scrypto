use super::*;
use crate::internal_prelude::*;
use radix_transactions::{model::*, validation::TransactionValidator};

use crate::{
    system::bootstrap::FlashReceipt,
    transaction::*,
    vm::{VmInitialize, VmModules},
};

pub struct ProtocolUpdateExecutor {
    pub network_definition: NetworkDefinition,
    pub protocol_version: ProtocolVersion,
    pub generator: Box<dyn ProtocolUpdateGenerator>,
    pub start_at_batch_group_index: usize,
    pub start_at_batch_index_in_first_group: usize,
}

impl ProtocolUpdateExecutor {
    pub fn new_for_version(protocol_version: ProtocolVersion, settings: &ProtocolSettings) -> Self {
        let network_definition = settings.network_definition.clone();
        let generator = settings.resolve_generator_for_update(&protocol_version);
        Self {
            network_definition,
            protocol_version,
            generator,
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
        let batch_generator = settings.resolve_generator_for_update(&protocol_version);
        Self {
            network_definition,
            protocol_version,
            generator: batch_generator,
            start_at_batch_group_index: from_inclusive.0,
            start_at_batch_index_in_first_group: from_inclusive.1,
        }
    }

    pub fn new<US: UpdateSettings + 'static>(
        network_definition: NetworkDefinition,
        update_settings: US,
    ) -> Self {
        let protocol_version = US::protocol_version();
        let batch_generator = Box::new(update_settings.create_generator());
        Self {
            network_definition,
            protocol_version,
            generator: batch_generator,
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
        let add_status_update = self
            .generator
            .enable_status_tracking_into_substate_database();

        let batch_groups = self.generator.batch_groups();

        for (batch_group_index, batch_group_generator) in batch_groups
            .into_iter()
            .skip(self.start_at_batch_group_index)
            .enumerate()
        {
            let batch_group_name = batch_group_generator.batch_group_name();
            let start_at_batch = if batch_group_index == self.start_at_batch_group_index {
                self.start_at_batch_index_in_first_group
            } else {
                0
            };
            let batches = batch_group_generator.generate_batches(&*store);
            for (batch_index, batch_generator) in
                batches.into_iter().skip(start_at_batch).enumerate()
            {
                let batch_name = batch_generator.batch_name().to_string();
                let batch = batch_generator.generate_batch(store);
                for (transaction_index, transaction) in batch.transactions.into_iter().enumerate() {
                    let receipt = match &transaction {
                        ProtocolUpdateTransaction::FlashTransactionV1(flash) => {
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
                        ProtocolUpdateTransaction::SystemTransactionV1(
                            ProtocolSystemTransactionV1 {
                                transaction,
                                disable_auth,
                                ..
                            },
                        ) => {
                            let execution_config = if *disable_auth {
                                ExecutionConfig::for_auth_disabled_system_transaction(
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
                            batch_group_name,
                            batch_index,
                            batch_name: &batch_name,
                            transaction_index,
                            transaction: &transaction,
                            receipt: &receipt,
                            resultant_store: store,
                        });
                    }
                }

                if add_status_update {
                    // A scrypto-only executor feature.
                    // In the node's executor, this state is tracked in the `LedgerProofOrigin`.
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
                                        batch_group_name: batch_group_name.to_string(),
                                        batch_index,
                                        batch_name: batch_name.to_string(),
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
                        batch_group_name,
                        batch_index,
                        batch_name: &batch_name,
                        status_update_committed: add_status_update,
                        resultant_store: store,
                    });
                }
            }
        }
        if add_status_update {
            // A scrypto-only executor feature.
            // In the node's executor, this state is tracked in the `LedgerProofOrigin`.
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
        if H::IS_ENABLED {
            hooks.on_protocol_update_completed(OnProtocolUpdateCompleted {
                protocol_version: self.protocol_version,
                status_update_committed: add_status_update,
                resultant_store: store,
            });
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

    fn on_protocol_update_completed(&mut self, event: OnProtocolUpdateCompleted) {}
}

/// Using a struct allows lots of parameters to be passed, without
/// having a large number of method arguments
pub struct OnProtocolTransactionExecuted<'a> {
    pub protocol_version: ProtocolVersion,
    pub batch_group_index: usize,
    pub batch_group_name: &'a str,
    pub batch_index: usize,
    pub batch_name: &'a str,
    pub transaction_index: usize,
    pub transaction: &'a ProtocolUpdateTransaction,
    pub receipt: &'a TransactionReceipt,
    pub resultant_store: &'a mut dyn SubstateDatabase,
}

pub struct OnProtocolTransactionBatchCommitted<'a> {
    pub protocol_version: ProtocolVersion,
    pub batch_group_index: usize,
    pub batch_group_name: &'a str,
    pub batch_index: usize,
    pub batch_name: &'a str,
    pub status_update_committed: bool,
    pub resultant_store: &'a mut dyn SubstateDatabase,
}

pub struct OnProtocolUpdateCompleted<'a> {
    pub protocol_version: ProtocolVersion,
    pub status_update_committed: bool,
    pub resultant_store: &'a mut dyn SubstateDatabase,
}

impl ProtocolUpdateExecutionHooks for () {
    const IS_ENABLED: bool = false;
}
