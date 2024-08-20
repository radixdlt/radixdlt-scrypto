// NOTE: This file is only conditionally included in std mode, so std imports are allowed
#[cfg(test)]
mod test {
    use crate::executor::*;
    use crate::internal_prelude::*;
    use crate::scenarios::ALL_SCENARIOS;
    use fmt::Write;
    use itertools::Itertools;
    use radix_engine::utils::CostingTaskMode;
    use radix_engine::{updates::*, utils::*, vm::*};
    use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
    use radix_substate_store_interface::interface::SubstateDatabase;
    use radix_transactions::manifest::decompiler::decompile_with_known_naming;
    use radix_transactions::manifest::*;
    use std::{ffi::OsString, fs, path::*};
    use wasm::DefaultWasmEngine;

    #[derive(Copy, Clone)]
    pub enum DumperMode {
        Write,
        Assert,
    }

    pub struct FileAligner {
        folder: PathBuf,
        mode: DumperMode,
    }

    #[derive(Clone)]
    pub struct FolderAligner {
        folder: PathBuf,
        mode: DumperMode,
        files_touched: Rc<RefCell<IndexSet<OsString>>>,
        registered_folders: Rc<RefCell<IndexMap<OsString, FolderAligner>>>,
    }

    impl FolderAligner {
        pub fn new(folder: PathBuf, mode: DumperMode) -> Self {
            // NOTE: In future, we could improve this behaviour to avoid creating churn if files don't need to change
            match mode {
                DumperMode::Write => {
                    if folder.exists() {
                        std::fs::remove_dir_all(&folder).unwrap();
                    }
                    std::fs::create_dir_all(&folder).unwrap();
                }
                DumperMode::Assert => {}
            }
            Self {
                folder,
                mode,
                files_touched: Rc::new(RefCell::new(indexset!())),
                registered_folders: Rc::new(RefCell::new(indexmap!())),
            }
        }

        pub fn put_file<F: AsRef<str>, C: AsRef<[u8]>>(&self, file: F, contents: C) {
            let file = file.as_ref();
            let path = self.folder.join(file);
            self.files_touched.borrow_mut().insert(file.into());
            match self.mode {
                DumperMode::Write => fs::write(path, contents).unwrap(),
                DumperMode::Assert => {
                    let actual_contents = fs::read(&path).unwrap_or_else(|err| {
                        panic!(
                            "File {} could not be read: {:?}",
                            path.to_string_lossy(),
                            err
                        );
                    });
                    if &actual_contents != contents.as_ref() {
                        panic!(
                            "File {} did not match the expected contents",
                            path.to_string_lossy()
                        )
                    }
                }
            }
        }

        pub fn register_child_folder<F: AsRef<str>>(&self, child_folder: F) -> FolderAligner {
            let child_folder = child_folder.as_ref();
            let path = self.folder.join(child_folder);
            let folder_aligner = FolderAligner::new(path, self.mode);
            self.registered_folders
                .borrow_mut()
                .insert(child_folder.into(), folder_aligner.clone());
            folder_aligner
        }

        pub fn verify_no_extra_content_exists(&self) {
            match self.mode {
                DumperMode::Write => {}
                DumperMode::Assert => {
                    // If the folder doesn't exist, then it can't contain any contents, so this check is fine
                    if !self.folder.exists() {
                        return;
                    }
                    for entry in walkdir::WalkDir::new(&self.folder)
                        .min_depth(1)
                        .max_depth(1)
                    {
                        let entry = entry.unwrap();
                        let file_name = entry.file_name();
                        let is_file = entry.file_type().is_file();
                        let is_folder = entry.file_type().is_dir();
                        match (is_file, is_folder) {
                            (true, false) => {
                                if !self.files_touched.borrow().contains(file_name) {
                                    panic!(
                                        "File {} should not exist",
                                        entry.path().to_string_lossy()
                                    )
                                }
                            }
                            (false, true) => {
                                if !self.registered_folders.borrow().contains_key(file_name) {
                                    panic!(
                                        "Folder {} should not exist",
                                        entry.path().to_string_lossy()
                                    )
                                }
                            }
                            (true, true) => {
                                panic!(
                                    "Path {} was unexpectedly both a file and a folder",
                                    entry.path().to_string_lossy()
                                )
                            }
                            (false, false) => {
                                panic!(
                                    "Path {} was unexpectedly neither a file nor a folder",
                                    entry.path().to_string_lossy()
                                )
                            }
                        }
                    }
                    for (_, child_folder) in self.registered_folders.borrow().iter() {
                        child_folder.verify_no_extra_content_exists();
                    }
                }
            }
        }
    }

    pub fn run_all_protocol_updates(mode: DumperMode) {
        let network_definition = NetworkDefinition::simulator();
        let address_encoder = AddressBech32Encoder::new(&network_definition);
        let root_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("generated-protocol-updates");

        let scrypto_vm = ScryptoVm::<DefaultWasmEngine>::default();
        let vm_init = VmInit::new(&scrypto_vm, NoExtension);
        let mut db = InMemorySubstateDatabase::standard();

        struct ProtocolUpdateHooks<'a> {
            network_definition: &'a NetworkDefinition,
            address_encoder: &'a AddressBech32Encoder,
            manifests_folder: FolderAligner,
            receipts_folder: FolderAligner,
            event_hasher: HashAccumulator,
            state_change_hasher: HashAccumulator,
        }

        impl<'a> ProtocolUpdateExecutionHooks for ProtocolUpdateHooks<'a> {
            fn on_transaction_executed(&mut self, event: OnProtocolTransactionExecuted) {
                let OnProtocolTransactionExecuted {
                    batch_group_index,
                    batch_index,
                    transaction_index,
                    transaction,
                    receipt,
                    resultant_store,
                    ..
                } = event;
                let transaction_file_prefix = if let Some(name) = transaction.name() {
                    format!(
                        "{batch_group_index:02}-{batch_index:02}-{transaction_index:02}--{name}"
                    )
                } else {
                    format!("{batch_group_index:02}-{batch_index:02}-{transaction_index:02}")
                };

                match &receipt.result {
                    TransactionResult::Commit(c) => {
                        self.event_hasher
                            .update_no_chain(scrypto_encode(&c.application_events).unwrap());
                        self.state_change_hasher
                            .update_no_chain(scrypto_encode(&c.state_updates).unwrap())
                    }
                    TransactionResult::Reject(_) | TransactionResult::Abort(_) => {}
                }

                match transaction {
                    ProtocolUpdateTransactionDetails::FlashV1Transaction(_) => {}
                    ProtocolUpdateTransactionDetails::SystemTransactionV1 {
                        transaction, ..
                    } => {
                        // Write manifest
                        let manifest_string =
                            decompile(&transaction.instructions.0, &self.network_definition)
                                .unwrap();
                        self.manifests_folder
                            .put_file(format!("{transaction_file_prefix}.rtm"), &manifest_string);
                    }
                }

                let receipt_display_context = TransactionReceiptDisplayContextBuilder::new()
                    .encoder(&self.address_encoder)
                    .schema_lookup_from_db(resultant_store)
                    .display_state_updates(true)
                    .use_ansi_colors(false)
                    .set_max_substate_length_to_display(10 * 1024)
                    .build();
                self.receipts_folder.put_file(
                    format!("{transaction_file_prefix}.txt"),
                    receipt.to_string(receipt_display_context),
                );
            }
        }

        let protocol_executor = ProtocolBuilder::for_network(&network_definition)
            .with_babylon(BabylonSettings::test_complex())
            .from_bootstrap_to_latest();
        for protocol_update_exector in protocol_executor.each_protocol_update_executor() {
            let protocol_version = protocol_update_exector.protocol_version;
            let mut version_folder =
                FolderAligner::new(root_path.join(protocol_version.logical_name()), mode);
            let mut hooks = ProtocolUpdateHooks {
                network_definition: &network_definition,
                address_encoder: &address_encoder,
                manifests_folder: version_folder.register_child_folder("manifests"),
                receipts_folder: version_folder.register_child_folder("receipts"),
                event_hasher: HashAccumulator::new(),
                state_change_hasher: HashAccumulator::new(),
            };

            protocol_update_exector.run_and_commit_advanced(
                &mut db,
                &mut hooks,
                &mut VmModules::default(),
            );

            let mut summary = String::new();
            let protocol_version_display_name = protocol_version.display_name();
            writeln!(&mut summary, "Name: {protocol_version_display_name}").unwrap();
            writeln!(&mut summary).unwrap();

            let state_change_digest =
                hooks.state_change_hasher.finalize().to_string()[0..16].to_string();
            let event_digest = hooks.event_hasher.finalize().to_string()[0..16].to_string();

            writeln!(&mut summary, "== SUMMARY HASHES ==").unwrap();

            if protocol_version == ProtocolVersion::LATEST {
                writeln!(&mut summary, "These {protocol_version_display_name} hashes are permitted to change only until the protocol update is deployed to a permanent network, else it can cause divergence.").unwrap();
                writeln!(&mut summary, "State changes: {state_change_digest} (allowed to change if not deployed to any network)").unwrap();
                writeln!(&mut summary, "Events       : {event_digest} (allowed to change if not deployed to any network)").unwrap();
            } else {
                writeln!(&mut summary, "These {protocol_version_display_name} hashes should NEVER change, else they will cause divergence when run historically.").unwrap();
                writeln!(
                    &mut summary,
                    "State changes: {state_change_digest} (should never change)"
                )
                .unwrap();
                writeln!(
                    &mut summary,
                    "Events       : {event_digest} (should never change)"
                )
                .unwrap();
            };

            writeln!(&mut summary).unwrap();

            version_folder.put_file("protocol_update_summary.txt", summary);

            version_folder.verify_no_extra_content_exists();
        }
    }

    #[test]
    #[ignore = "Run this test to update the generated protocol update receipts"]
    pub fn update_all_generated_protocol_update_receipts() {
        run_all_protocol_updates(DumperMode::Write)
    }

    #[test]
    pub fn validate_all_generated_protocol_update_receipts() {
        run_all_protocol_updates(DumperMode::Assert)
    }

    struct ScenarioDumpingHooks {
        event_hasher: HashAccumulator,
        state_update_hasher: HashAccumulator,
        scenario_folder: FolderAligner,
        manifests_folder: FolderAligner,
        transactions_folder: FolderAligner,
        receipts_folder: FolderAligner,
        costings_folder: FolderAligner,
    }

    impl ScenarioDumpingHooks {
        fn new(scenario_folder: FolderAligner) -> Self {
            let mut manifests_folder = scenario_folder.register_child_folder("manifests");
            let mut transactions_folder = scenario_folder.register_child_folder("transactions");
            let mut receipts_folder = scenario_folder.register_child_folder("receipts");
            let mut costings_folder = scenario_folder.register_child_folder("costings");

            Self {
                event_hasher: HashAccumulator::new(),
                state_update_hasher: HashAccumulator::new(),
                scenario_folder,
                manifests_folder,
                transactions_folder,
                receipts_folder,
                costings_folder,
            }
        }
    }

    impl<S: SubstateDatabase> ScenarioExecutionHooks<S> for ScenarioDumpingHooks {
        fn adapt_execution_config(&mut self, mut config: ExecutionConfig) -> ExecutionConfig {
            config.enable_cost_breakdown = true;
            config
        }

        fn on_transaction_executed(&mut self, event: OnScenarioTransactionExecuted<S>) {
            let OnScenarioTransactionExecuted {
                transaction,
                receipt,
                database,
                network_definition,
                ..
            } = event;
            match &receipt.result {
                TransactionResult::Commit(c) => {
                    self.event_hasher
                        .update_no_chain(scrypto_encode(&c.application_events).unwrap());
                    self.state_update_hasher
                        .update_no_chain(scrypto_encode(&c.state_updates).unwrap())
                }
                TransactionResult::Reject(_) | TransactionResult::Abort(_) => {}
            }

            let transaction_file_prefix = format!(
                "{:03}--{}",
                transaction.stage_counter, transaction.logical_name,
            );

            self.transactions_folder.put_file(
                format!("{transaction_file_prefix}.bin"),
                &transaction.raw_transaction.0,
            );

            // Write manifest
            // NB: We purposefully don't write the blobs as they're contained in the raw transactions
            let manifest_string = decompile_with_known_naming(
                &transaction.manifest.instructions,
                network_definition,
                transaction.naming.clone(),
            )
            .unwrap();
            // Whilst we're here, let's validate that the manifest can be recompiled
            compile(
                &manifest_string,
                network_definition,
                BlobProvider::new_with_blobs(
                    transaction.manifest.blobs.values().cloned().collect(),
                ),
            )
            .unwrap();
            self.manifests_folder
                .put_file(format!("{transaction_file_prefix}.rtm"), &manifest_string);

            // Write receipt
            let address_encoder = AddressBech32Encoder::new(network_definition);
            let receipt_display_context = TransactionReceiptDisplayContextBuilder::new()
                .encoder(&address_encoder)
                .schema_lookup_from_db(&*database)
                .display_state_updates(true)
                .use_ansi_colors(false)
                .build();
            self.receipts_folder.put_file(
                format!("{transaction_file_prefix}.txt"),
                receipt.to_string(receipt_display_context),
            );

            let cost_breakdown =
                format_cost_breakdown(&receipt.fee_summary, receipt.fee_details.as_ref().unwrap());
            self.costings_folder
                .put_file(format!("{transaction_file_prefix}.txt"), cost_breakdown);
        }

        fn on_scenario_ended(&mut self, event: OnScenarioEnded<S>) {
            let OnScenarioEnded {
                metadata,
                end_state,
                current_protocol_version,
                network_definition,
                ..
            } = event;
            let scenario_logical_name = metadata.logical_name;

            let mut summary = String::new();
            writeln!(&mut summary, "Name: {scenario_logical_name}").unwrap();
            writeln!(&mut summary).unwrap();

            let state_change_digest = mem::take(&mut self.state_update_hasher)
                .finalize()
                .to_string()[0..16]
                .to_string();
            let event_digest =
                mem::take(&mut self.event_hasher).finalize().to_string()[0..16].to_string();

            writeln!(&mut summary, "== SUMMARY HASHES ==").unwrap();

            let protocol_version_display_name = current_protocol_version.display_name();
            if current_protocol_version == ProtocolVersion::LATEST {
                writeln!(&mut summary, "These {protocol_version_display_name} hashes are permitted to change only until the scenario is deployed to a permanent network, else it can cause divergence.").unwrap();
                writeln!(&mut summary, "State changes: {state_change_digest} (allowed to change if not deployed to any network)").unwrap();
                writeln!(&mut summary, "Events       : {event_digest} (allowed to change if not deployed to any network)").unwrap();
            } else {
                writeln!(&mut summary, "These {protocol_version_display_name} hashes should NEVER change, else they will cause divergence when run historically.").unwrap();
                writeln!(
                    &mut summary,
                    "State changes: {state_change_digest} (should never change)"
                )
                .unwrap();
                writeln!(
                    &mut summary,
                    "Events       : {event_digest} (should never change)"
                )
                .unwrap();
            };

            writeln!(&mut summary).unwrap();
            writeln!(&mut summary, "== INTERESTING ADDRESSES ==").unwrap();

            let address_encoder = AddressBech32Encoder::new(network_definition);
            for (name, address) in end_state.output.interesting_addresses.0.iter() {
                writeln!(
                    &mut summary,
                    "- {name}: {}",
                    address.display(&address_encoder)
                )
                .unwrap();
            }
            writeln!(&mut summary).unwrap();

            self.scenario_folder
                .put_file("scenario_summary.txt", summary);
            self.scenario_folder.verify_no_extra_content_exists()
        }
    }

    pub fn run_all_scenarios(mode: DumperMode) {
        let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("generated-examples");
        for (scenario_logical_name, scenario_creator) in ALL_SCENARIOS.iter() {
            let min_requirement = scenario_creator.metadata().protocol_min_requirement;
            let valid_versions = ProtocolVersion::VARIANTS
                .into_iter()
                .filter(|p| *p >= min_requirement);
            for protocol_version in valid_versions {
                let mut db = InMemorySubstateDatabase::standard();

                let scenario_folder = FolderAligner::new(
                    root_path
                        .join(protocol_version.logical_name())
                        .join(&*scenario_logical_name),
                    mode,
                );

                // Now execute just the single scenario, after executing protocol updates up to
                // the given protocol version
                let mut executor =
                    TransactionScenarioExecutor::new(db, NetworkDefinition::simulator())
                        .execute_protocol_updates_and_scenarios(
                            |builder| builder.from_bootstrap_to(protocol_version),
                            ScenarioTrigger::AfterCompletionOfAllProtocolUpdates,
                            ScenarioFilter::SpecificScenariosByName(btreeset!(
                                scenario_logical_name.to_string()
                            )),
                            &mut ScenarioDumpingHooks::new(scenario_folder),
                            &mut (),
                            &VmModules::default(),
                        );
            }
        }
    }

    #[test]
    #[ignore = "Run this test to update the generated scenarios"]
    pub fn update_all_generated_scenarios() {
        run_all_scenarios(DumperMode::Write)
    }

    #[test]
    pub fn validate_all_generated_scenarios() {
        run_all_scenarios(DumperMode::Assert)
    }

    #[test]
    pub fn validate_the_metadata_of_all_scenarios() {
        struct Hooks;
        impl<S: SubstateDatabase> ScenarioExecutionHooks<S> for Hooks {
            fn on_scenario_started(&mut self, event: OnScenarioStarted<S>) {
                let OnScenarioStarted { metadata, .. } = event;
                if let Some(testnet_run_at) = metadata.testnet_run_at {
                    assert!(
                        testnet_run_at >= metadata.protocol_min_requirement,
                        "Scenario is set to run on a testnet of an earlier version than the scenario's minimum version: {}",
                        metadata.logical_name
                    );
                }

                assert!(
                    !metadata.logical_name.contains(' '),
                    "Scenario logical name contains a space: {}",
                    metadata.logical_name
                );
            }
        }
        TransactionScenarioExecutor::new(
            InMemorySubstateDatabase::standard(),
            NetworkDefinition::simulator(),
        )
        .execute_every_protocol_update_and_scenario(&mut Hooks);
    }
}
