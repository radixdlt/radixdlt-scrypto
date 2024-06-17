use radix_substate_store_interface::db_key_mapper::SpreadPrefixKeyMapper;

use super::*;

#[derive(Clone)]
pub struct ProtocolUpdateExecutor {
    pub protocol_update: ProtocolUpdate,
    pub batch_generator: Box<dyn ProtocolUpdateBatchGenerator>,
}

impl ProtocolUpdateExecutor {
    pub fn run_and_commit<S: SubstateDatabase + CommittableSubstateDatabase>(self, store: &mut S) {
        for batch_index in 0..self.batch_generator.batch_count() {
            let batch = self.batch_generator.generate_batch(store, batch_index);
            for transaction in batch.transactions {
                let state_updates = match transaction {
                    ProtocolUpdateTransactionDetails::FlashV1Transaction(flash) => {
                        flash.state_updates
                    }
                };
                let db_updates = state_updates.create_database_updates::<SpreadPrefixKeyMapper>();
                store.commit(&db_updates);
            }
        }
    }
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
        &self,
        store: &mut S,
    ) {
        for update_execution in self.each_protocol_update_executor() {
            update_execution.run_and_commit(store);
        }
    }

    pub fn each_protocol_update_executor(
        &self,
    ) -> impl Iterator<Item = ProtocolUpdateExecutor> + '_ {
        let until_protocol_version = self.update_until;
        ProtocolUpdate::VARIANTS
            .into_iter()
            .take_while(move |update| ProtocolVersion::from(*update) <= until_protocol_version)
            .map(move |protocol_update| self.create_executor_for_update(protocol_update))
    }

    pub fn create_executor_for_update(
        &self,
        protocol_update: ProtocolUpdate,
    ) -> ProtocolUpdateExecutor {
        let generator: Box<dyn ProtocolUpdateBatchGenerator> = match protocol_update {
            ProtocolUpdate::Anemone => Box::new(self.settings.anemone.create_batch_generator()),
            ProtocolUpdate::Bottlenose => {
                Box::new(self.settings.bottlenose.create_batch_generator())
            }
        };
        ProtocolUpdateExecutor {
            protocol_update,
            batch_generator: generator,
        }
    }
}
