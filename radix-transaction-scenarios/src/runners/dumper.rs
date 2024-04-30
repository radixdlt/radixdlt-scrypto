// NOTE: This file is only conditionally included in std mode, so std imports are allowed
use crate::internal_prelude::DefaultTransactionScenarioExecutor;
use crate::internal_prelude::ALL_SCENARIOS;
use crate::internal_prelude::*;

#[cfg(test)]
mod test {
    use super::*;
    use fmt::Write;
    use itertools::Itertools;
    use radix_engine::utils::CostingTaskMode;
    use radix_engine::{updates::*, utils::*, vm::*};
    use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
    use radix_transactions::manifest::decompiler::decompile_with_known_naming;
    use radix_transactions::manifest::*;
    use std::{ffi::OsString, fs, path::*};

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
        folders_touched: Rc<RefCell<IndexMap<OsString, FolderAligner>>>,
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
                folders_touched: Rc::new(RefCell::new(indexmap!())),
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
            self.folders_touched
                .borrow_mut()
                .insert(child_folder.into(), folder_aligner.clone());
            folder_aligner
        }

        pub fn verify_complete_recursive(&self) {
            match self.mode {
                DumperMode::Write => {}
                DumperMode::Assert => {
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
                                if !self.folders_touched.borrow().contains_key(file_name) {
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
                    for (_, child_folder) in self.folders_touched.borrow().iter() {
                        child_folder.verify_complete_recursive();
                    }
                }
            }
        }
    }

    pub fn run_all(mode: DumperMode) {
        let network_definition = NetworkDefinition::simulator();
        let address_encoder = AddressBech32Encoder::new(&network_definition);
        let execution_config = {
            let mut config = ExecutionConfig::for_notarized_transaction(network_definition.clone());
            config.enable_cost_breakdown = true;
            config
        };
        let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("generated-examples");
        for (scenario_logical_name, scenario_creator) in ALL_SCENARIOS.iter() {
            let min_requirement = scenario_creator.metadata().protocol_min_requirement;
            let valid_versions = ProtocolVersion::all_iterator().filter(|p| *p >= min_requirement);
            for protocol_version in valid_versions {
                let protocol_version_display_name = protocol_version.display_name();

                let mut db = InMemorySubstateDatabase::standard();
                let mut event_hasher = RefCell::new(Some(HashAccumulator::new()));
                let mut state_change_hasher = RefCell::new(Some(HashAccumulator::new()));

                let mut scenario_folder = FolderAligner::new(
                    root_path
                        .join(protocol_version.logical_name())
                        .join(&*scenario_logical_name),
                    mode,
                );
                let mut manifests_folder = scenario_folder.register_child_folder("manifests");
                let mut transactions_folder = scenario_folder.register_child_folder("transactions");
                let mut receipts_folder = scenario_folder.register_child_folder("receipts");
                let mut costings_folder = scenario_folder.register_child_folder("costings");

                let mut executor = DefaultTransactionScenarioExecutor::new(db, &network_definition)
                    .scenario_execution_config(execution_config.clone())
                    .on_transaction_executed(|_, transaction_details, receipt, db| {
                        match &receipt.result {
                            TransactionResult::Commit(c) => {
                                event_hasher
                                    .borrow_mut()
                                    .as_mut()
                                    .unwrap()
                                    .update_no_chain(scrypto_encode(&c.application_events).unwrap());
                                state_change_hasher
                                    .borrow_mut()
                                    .as_mut()
                                    .unwrap()
                                    .update_no_chain(scrypto_encode(&c.state_updates).unwrap())
                            }
                            TransactionResult::Reject(_) | TransactionResult::Abort(_) => {}
                        }

                        let transaction_file_prefix = format!(
                            "{:03}--{}",
                            transaction_details.stage_counter,
                            transaction_details.logical_name,
                        );

                        transactions_folder.put_file(
                            format!("{transaction_file_prefix}.bin"),
                            &transaction_details.raw_transaction.0,
                        );

                        // Write manifest
                        // NB: We purposefully don't write the blobs as they're contained in the raw transactions
                        let manifest_string = decompile_with_known_naming(
                            &transaction_details.manifest.instructions,
                            &network_definition,
                            transaction_details.naming.clone()
                        ).unwrap();
                        // Whilst we're here, let's validate that the manifest can be recompiled
                        compile(
                            &manifest_string,
                            &network_definition,
                            BlobProvider::new_with_blobs(
                                transaction_details.manifest.blobs.values().cloned().collect()
                            ),
                        )
                        .unwrap();
                        manifests_folder.put_file(
                            format!("{transaction_file_prefix}.rtm"),
                            &manifest_string,
                        );

                        // Write receipt
                        let receipt_display_context = TransactionReceiptDisplayContextBuilder::new()
                            .encoder(&address_encoder)
                            .schema_lookup_from_db(db)
                            .display_state_updates(true)
                            .use_ansi_colors(false)
                            .build();
                        receipts_folder.put_file(
                            format!("{transaction_file_prefix}.txt"),
                            receipt.to_string(receipt_display_context),
                        );

                        let cost_breakdown = format_cost_breakdown(
                            &receipt.fee_summary,
                            receipt.fee_details.as_ref().unwrap(),
                        );
                        costings_folder.put_file(
                            format!("{transaction_file_prefix}.txt"),
                            cost_breakdown,
                        );
                    })
                    .on_scenario_ended(|_, end_state, _| {
                        let mut summary = String::new();
                        writeln!(&mut summary, "Name: {scenario_logical_name}").unwrap();
                        writeln!(&mut summary).unwrap();

                        let state_change_digest = state_change_hasher
                            .borrow_mut().take().unwrap().finalize().to_string()[0..16].to_string();
                        let event_digest = event_hasher
                            .borrow_mut().take().unwrap().finalize().to_string()[0..16].to_string();

                        writeln!(&mut summary, "== SUMMARY HASHES ==").unwrap();
                        if protocol_version == ProtocolVersion::latest() {
                            writeln!(&mut summary, "These {protocol_version_display_name} hashes are permitted to change only until the scenario is deployed to a permanent network, else it can cause divergence.").unwrap();
                            writeln!(&mut summary, "State changes: {state_change_digest} (allowed to change if not deployed to any network)").unwrap();
                            writeln!(&mut summary, "Events       : {event_digest} (allowed to change if not deployed to any network)").unwrap();
                        } else {
                            writeln!(&mut summary, "These {protocol_version_display_name} hashes should NEVER change, else they will cause divergence when run historically.").unwrap();
                            writeln!(&mut summary, "State changes: {state_change_digest} (should never change)").unwrap();
                            writeln!(&mut summary, "Events       : {event_digest} (should never change)").unwrap();
                        };

                        writeln!(&mut summary).unwrap();
                        writeln!(&mut summary, "== INTERESTING ADDRESSES ==").unwrap();

                        for (name, address) in end_state.output.interesting_addresses.0.iter() {
                            writeln!(&mut summary, "- {name}: {}", address.display(&address_encoder)).unwrap();
                        }
                        writeln!(&mut summary).unwrap();

                        scenario_folder.put_file(
                            "scenario_summary.txt",
                            summary
                        );
                    });

                // Now execute just the single scenario, after executing protocol updates up to
                // the given protocol version
                executor.execute_protocol_updates_and_scenarios(
                    ProtocolBuilder::for_network(&network_definition).until(protocol_version),
                    ScenarioTrigger::AfterCompletionOfAllProtocolUpdates,
                    ScenarioFilter::ValidScenariosFrom(
                        btreeset!(scenario_logical_name.to_string()),
                    ),
                );

                scenario_folder.verify_complete_recursive();
            }
        }
    }

    #[test]
    #[ignore = "Run this test to update the generated scenarios"]
    pub fn update_all_generated_scenarios() {
        run_all(DumperMode::Write)
    }

    #[test]
    pub fn validate_all_generated_scenarios() {
        run_all(DumperMode::Assert)
    }
}
