use radix_substate_store_interface::db_key_mapper::SpreadPrefixKeyMapper;
use radix_transactions::model::TransactionPayload;

use crate::{system::bootstrap::FlashReceipt, transaction::*, vm::wasm::*, vm::*};

use super::*;

pub struct ProtocolUpdateExecutor {
    pub network_definition: NetworkDefinition,
    pub protocol_version: ProtocolVersion,
    pub batch_generator: Box<dyn ProtocolUpdateBatchGenerator>,
}

impl ProtocolUpdateExecutor {
    pub fn new_for_version(protocol_version: ProtocolVersion, settings: &ProtocolSettings) -> Self {
        let network_definition = settings.network_definition.clone();
        let batch_generator = settings.resolve_batch_generator_for_update(&protocol_version);
        Self {
            network_definition,
            protocol_version,
            batch_generator,
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
        let scrypto_vm = hooks.create_scrypto_vm();
        if H::IS_ENABLED {
            hooks.on_before_protocol_update(self.protocol_version, &*store);
        }
        for (batch_group_index, batch_group_name) in self
            .batch_generator
            .batch_group_descriptors()
            .into_iter()
            .enumerate()
        {
            for batch_index in 0..self.batch_generator.batch_count(batch_group_index) {
                let batch =
                    self.batch_generator
                        .generate_batch(store, batch_group_index, batch_index);
                for (transaction_index, transaction) in batch.transactions.into_iter().enumerate() {
                    if H::IS_ENABLED {
                        hooks.on_before_transaction_executed(
                            self.protocol_version,
                            batch_group_index,
                            &batch_group_name,
                            batch_index,
                            transaction_index,
                            &transaction,
                            &*store,
                        );
                    }

                    let receipt = match &transaction {
                        ProtocolUpdateTransactionDetails::FlashV1Transaction(flash) => {
                            let db_updates = flash
                                .state_updates
                                .create_database_updates::<SpreadPrefixKeyMapper>();
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
                            let receipt = execute_and_commit_transaction(
                                store,
                                hooks.create_vm_init(&scrypto_vm),
                                &execution_config,
                                transaction
                                    .prepare()
                                    .expect("Expected protocol update transaction to be preparable")
                                    .get_executable(btreeset![system_execution(
                                        SystemExecution::Protocol
                                    )]),
                            );
                            receipt.expect_commit_success();
                            receipt
                        }
                    };

                    if H::IS_ENABLED {
                        hooks.on_transaction_executed(
                            self.protocol_version,
                            batch_group_index,
                            &batch_group_name,
                            batch_index,
                            transaction_index,
                            &transaction,
                            &receipt,
                            &*store,
                        );
                    }
                }
            }
        }
        if H::IS_ENABLED {
            hooks.on_protocol_update_completed(self.protocol_version, &*store);
        }
    }
}

#[allow(unused_variables)]
pub trait ProtocolUpdateExecutionHooks {
    // If false, hooks aren't called, so opt out of constructing things like receipts.
    const IS_ENABLED: bool;
    type WasmEngine: WasmEngine + Default;
    type NativeVmExtension: NativeVmExtension;

    fn get_vm_extension(&mut self) -> Self::NativeVmExtension;

    fn create_scrypto_vm(&mut self) -> ScryptoVm<Self::WasmEngine> {
        ScryptoVm::default()
    }

    fn create_vm_init<'g>(
        &mut self,
        scrypto_vm: &'g ScryptoVm<Self::WasmEngine>,
    ) -> VmInit<'g, Self::WasmEngine, Self::NativeVmExtension> {
        VmInit::new(scrypto_vm, self.get_vm_extension())
    }

    fn on_before_protocol_update(
        &mut self,
        new_protocol_version: ProtocolVersion,
        store: &dyn SubstateDatabase,
    ) {
    }

    fn on_before_transaction_executed(
        &mut self,
        protocol_version: ProtocolVersion,
        batch_group_index: usize,
        batch_group_name: &str,
        batch_index: usize,
        transaction_index: usize,
        transaction: &ProtocolUpdateTransactionDetails,
        store: &dyn SubstateDatabase,
    ) {
    }

    fn adapt_execution_config(&mut self, config: ExecutionConfig) -> ExecutionConfig {
        config
    }

    fn on_transaction_executed(
        &mut self,
        protocol_version: ProtocolVersion,
        batch_group_index: usize,
        batch_group_name: &str,
        batch_index: usize,
        transaction_index: usize,
        transaction: &ProtocolUpdateTransactionDetails,
        receipt: &TransactionReceipt,
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
    type WasmEngine = DefaultWasmEngine;
    type NativeVmExtension = NoExtension;

    fn get_vm_extension(&mut self) -> NoExtension {
        NoExtension
    }
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
}

impl ProtocolSettings {
    pub fn resolve_batch_generator_for_update(
        &self,
        protocol_version: &ProtocolVersion,
    ) -> Box<dyn ProtocolUpdateBatchGenerator> {
        match protocol_version {
            ProtocolVersion::Babylon => Box::new(self.babylon.create_batch_generator()),
            ProtocolVersion::Anemone => Box::new(self.anemone.create_batch_generator()),
            ProtocolVersion::Bottlenose => Box::new(self.bottlenose.create_batch_generator()),
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
            },
        }
    }

    pub fn with_babylon(mut self, settings: BabylonSettings) -> Self {
        self.settings.babylon = settings;
        self
    }

    pub fn with_anemone(mut self, settings: AnemoneSettings) -> Self {
        self.settings.anemone = settings;
        self
    }

    pub fn with_bottlenose(mut self, settings: BottlenoseSettings) -> Self {
        self.settings.bottlenose = settings;
        self
    }

    pub fn only_bootstrap(self) -> ProtocolExecutor {
        self.bootstrap_then_until(ProtocolVersion::EARLIEST)
    }

    pub fn bootstrap_then_until(self, protocol_version: ProtocolVersion) -> ProtocolExecutor {
        ProtocolExecutor::new(None, protocol_version, self.settings)
    }

    pub fn post_bootstrap_until(self, protocol_version: ProtocolVersion) -> ProtocolExecutor {
        ProtocolExecutor::new(
            Some(ProtocolVersion::EARLIEST),
            protocol_version,
            self.settings,
        )
    }

    pub fn from_until(
        self,
        start_protocol_verison: ProtocolVersion,
        end_protocol_version: ProtocolVersion,
    ) -> ProtocolExecutor {
        ProtocolExecutor::new(
            Some(start_protocol_verison),
            end_protocol_version,
            self.settings,
        )
    }
}

pub struct ProtocolExecutor {
    starting_at: Option<ProtocolVersion>,
    update_until: ProtocolVersion,
    settings: ProtocolSettings,
}

impl ProtocolExecutor {
    pub fn new(
        starting_at: Option<ProtocolVersion>,
        update_until: ProtocolVersion,
        settings: ProtocolSettings,
    ) -> Self {
        Self {
            starting_at,
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

    pub fn each_target_protocol_version(&self) -> impl Iterator<Item = ProtocolVersion> {
        let starting_at = self.starting_at;
        let until_protocol_version = self.update_until;
        ProtocolVersion::VARIANTS
            .into_iter()
            .filter(move |version| {
                let satisfies_lower_bound = starting_at.is_none()
                    || starting_at.is_some_and(|start_version| start_version < *version);
                let satisfies_upper_bound = *version <= until_protocol_version;
                satisfies_lower_bound && satisfies_upper_bound
            })
    }

    pub fn each_protocol_update_executor(self) -> impl Iterator<Item = ProtocolUpdateExecutor> {
        self.each_target_protocol_version()
            .map(move |version| ProtocolUpdateExecutor::new_for_version(version, &self.settings))
    }
}
