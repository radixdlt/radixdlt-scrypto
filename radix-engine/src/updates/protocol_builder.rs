use radix_substate_store_interface::db_key_mapper::SpreadPrefixKeyMapper;

use crate::{system::bootstrap::FlashReceipt, transaction::TransactionReceipt};

use super::*;

pub struct ProtocolUpdateExecutor {
    pub protocol_update: ProtocolUpdate,
    pub batch_generator: Box<dyn ProtocolUpdateBatchGenerator>,
}

impl ProtocolUpdateExecutor {
    pub fn new(protocol_update: ProtocolUpdate, settings: &ProtocolSettings) -> Self {
        let generator = settings.resolve_batch_generator_for_update(&protocol_update);
        Self {
            protocol_update,
            batch_generator: generator,
        }
    }

    pub fn run_and_commit(self, store: &mut (impl SubstateDatabase + CommittableSubstateDatabase)) {
        self.run_and_commit_with_hooks(store, &mut ());
    }

    pub fn run_and_commit_with_hooks<
        S: SubstateDatabase + CommittableSubstateDatabase,
        H: ProtocolUpdateExecutionHooks,
    >(
        self,
        store: &mut S,
        hooks: &mut H,
    ) {
        if H::IS_ENABLED {
            hooks.on_before_protocol_update(self.protocol_update.into(), &*store);
        }
        for batch_index in 0..self.batch_generator.batch_count() {
            if H::IS_ENABLED {
                hooks.on_before_protocol_update_batch(
                    self.protocol_update.into(),
                    batch_index,
                    &*store,
                );
            }
            let batch = self.batch_generator.generate_batch(store, batch_index);
            for (transaction_index, transaction) in batch.transactions.into_iter().enumerate() {
                if H::IS_ENABLED {
                    hooks.on_before_transaction_executed(
                        self.protocol_update.into(),
                        batch_index,
                        transaction_index,
                        &transaction,
                        &*store,
                    );

                    let state_updates = match transaction.clone() {
                        ProtocolUpdateTransactionDetails::FlashV1Transaction(flash) => {
                            flash.state_updates
                        }
                    };

                    let db_updates =
                        state_updates.create_database_updates::<SpreadPrefixKeyMapper>();

                    let flash_receipt = FlashReceipt::from_state_updates(state_updates, &*store);
                    let receipt = flash_receipt.into();
                    store.commit(&db_updates);

                    hooks.on_transaction_executed(
                        self.protocol_update.into(),
                        batch_index,
                        transaction_index,
                        &transaction,
                        &receipt,
                        &*store,
                    );
                } else {
                    let state_updates = match transaction {
                        ProtocolUpdateTransactionDetails::FlashV1Transaction(flash) => {
                            flash.state_updates
                        }
                    };
                    let db_updates =
                        state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
                    store.commit(&db_updates);
                }
            }
            if H::IS_ENABLED {
                hooks.on_protocol_update_batch_completed(
                    self.protocol_update.into(),
                    batch_index,
                    &*store,
                );
            }
        }
        if H::IS_ENABLED {
            hooks.on_protocol_update_completed(self.protocol_update.into(), &*store);
        }
    }
}

#[allow(unused_variables)]
pub trait ProtocolUpdateExecutionHooks {
    // If false, hooks aren't called, so opt out of constructing things like receipts.
    const IS_ENABLED: bool;

    fn on_before_protocol_update(
        &mut self,
        new_protocol_version: ProtocolVersion,
        store: &dyn SubstateDatabase,
    ) {
    }

    fn on_before_protocol_update_batch(
        &mut self,
        new_protocol_version: ProtocolVersion,
        batch_index: u32,
        store: &dyn SubstateDatabase,
    ) {
    }

    fn on_before_transaction_executed(
        &mut self,
        protocol_version: ProtocolVersion,
        batch_index: u32,
        transaction_index: usize,
        transaction: &ProtocolUpdateTransactionDetails,
        store: &dyn SubstateDatabase,
    ) {
    }

    fn on_transaction_executed(
        &mut self,
        protocol_version: ProtocolVersion,
        batch_index: u32,
        transaction_index: usize,
        transaction: &ProtocolUpdateTransactionDetails,
        receipt: &TransactionReceipt,
        resultant_store: &dyn SubstateDatabase,
    ) {
    }

    fn on_protocol_update_batch_completed(
        &mut self,
        protocol_version: ProtocolVersion,
        batch_index: u32,
        resultant_store: &dyn SubstateDatabase,
    ) {
    }

    fn on_protocol_update_completed(
        &mut self,
        new_protocol_version: ProtocolVersion,
        resultant_store: &dyn SubstateDatabase,
    ) {
    }
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
    // TODO: It would be nice to move bootstrap / Genesis into this formulation
    anemone: AnemoneSettings,
    bottlenose: BottlenoseSettings,
}

impl ProtocolSettings {
    pub fn resolve_batch_generator_for_update(
        &self,
        protocol_update: &ProtocolUpdate,
    ) -> Box<dyn ProtocolUpdateBatchGenerator> {
        match protocol_update {
            ProtocolUpdate::Anemone => Box::new(self.anemone.create_batch_generator()),
            ProtocolUpdate::Bottlenose => Box::new(self.bottlenose.create_batch_generator()),
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
                anemone: AnemoneSettings::all_enabled_as_default_for_network(network_definition),
                bottlenose: BottlenoseSettings::all_enabled_as_default_for_network(
                    network_definition,
                ),
            },
        }
    }

    pub fn with_anemone(mut self, settings: AnemoneSettings) -> Self {
        self.settings.anemone = settings;
        self
    }

    pub fn with_bottlenose(mut self, settings: BottlenoseSettings) -> Self {
        self.settings.bottlenose = settings;
        self
    }

    pub fn until_babylon(self) -> ProtocolExecutor {
        self.until(ProtocolVersion::Babylon)
    }

    pub fn until_latest_protocol_version(self) -> ProtocolExecutor {
        self.until(ProtocolVersion::LATEST)
    }

    pub fn until(self, protocol_version: ProtocolVersion) -> ProtocolExecutor {
        ProtocolExecutor::new(protocol_version, self.settings)
    }
}

pub struct ProtocolExecutor {
    update_until: ProtocolVersion,
    settings: ProtocolSettings,
}

impl ProtocolExecutor {
    pub fn new(update_until: ProtocolVersion, settings: ProtocolSettings) -> Self {
        Self {
            update_until,
            settings,
        }
    }

    pub fn commit_each_protocol_update<S: SubstateDatabase + CommittableSubstateDatabase>(
        self,
        store: &mut S,
    ) {
        for update_execution in self.each_protocol_update_executor() {
            update_execution.run_and_commit(store);
        }
    }

    pub fn commit_each_protocol_update_with_hooks(
        self,
        store: &mut (impl SubstateDatabase + CommittableSubstateDatabase),
        hooks: &mut impl ProtocolUpdateExecutionHooks,
    ) {
        for update_execution in self.each_protocol_update_executor() {
            update_execution.run_and_commit_with_hooks(store, hooks);
        }
    }

    pub fn each_protocol_update_executor(self) -> impl Iterator<Item = ProtocolUpdateExecutor> {
        let until_protocol_version = self.update_until;
        ProtocolUpdate::VARIANTS
            .into_iter()
            .take_while(move |update| ProtocolVersion::from(*update) <= until_protocol_version)
            .map(move |protocol_update| {
                ProtocolUpdateExecutor::new(protocol_update, &self.settings)
            })
    }
}
