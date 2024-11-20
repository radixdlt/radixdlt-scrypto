// NOTE: This file is only conditionally included in std mode, so std imports are allowed
#[cfg(test)]
mod test {
    use crate::executor::*;
    use crate::internal_prelude::*;
    use crate::scenarios::*;
    use fmt::Write;
    use itertools::Itertools;
    use radix_engine::{updates::*, utils::*, vm::*};
    use radix_substate_store_impls::memory_db::InMemorySubstateDatabase;
    use radix_substate_store_interface::interface::SubstateDatabase;
    use radix_transactions::manifest::*;
    use std::path::*;

    //=========================
    // PROTOCOL UPDATE TRIGGERS
    //=========================

    #[test]
    #[ignore = "Run this test to update the generated protocol update receipts"]
    pub fn update_all_generated_protocol_update_receipts() {
        run_all_protocol_updates(AlignerExecutionMode::Write)
    }

    #[test]
    pub fn validate_all_generated_protocol_update_receipts() {
        run_all_protocol_updates(AlignerExecutionMode::Assert)
    }

    //==================
    // SCENARIO TRIGGERS
    //==================

    #[test]
    #[ignore = "Run this test to update the generated scenarios"]
    pub fn update_all_generated_scenarios() {
        run_all_scenarios(AlignerExecutionMode::Write, all_scenarios_iter())
    }

    #[test]
    #[ignore = "Run this test manually to update a single scenario"]
    pub fn update_single_scenario() {
        run_all_scenarios(
            AlignerExecutionMode::Write,
            [get_scenario("basic_subintents")],
        )
    }

    #[test]
    pub fn validate_all_generated_scenarios() {
        run_all_scenarios(AlignerExecutionMode::Assert, all_scenarios_iter())
    }

    #[test]
    #[ignore = "Run this test manually to validate a single scenario"]
    pub fn validate_single_scenario() {
        run_all_scenarios(
            AlignerExecutionMode::Assert,
            [get_scenario("basic_subintents")],
        )
    }

    //=============================

    pub fn run_all_protocol_updates(mode: AlignerExecutionMode) {
        let network_definition = NetworkDefinition::simulator();
        let address_encoder = AddressBech32Encoder::new(&network_definition);
        let root_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("generated-protocol-updates");

        let mut vm_modules = VmModules::default();
        let mut db = InMemorySubstateDatabase::standard();

        struct ProtocolUpdateHooks<'a> {
            network_definition: &'a NetworkDefinition,
            address_encoder: &'a AddressBech32Encoder,
            manifests_folder: FolderContentAligner,
            receipts_folder: FolderContentAligner,
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
                            .concat_mut(scrypto_encode(&c.application_events).unwrap());
                        self.state_change_hasher
                            .concat_mut(scrypto_encode(&c.state_updates).unwrap());
                    }
                    TransactionResult::Reject(_) | TransactionResult::Abort(_) => {}
                }

                match transaction {
                    ProtocolUpdateTransaction::FlashTransactionV1(_) => {}
                    ProtocolUpdateTransaction::SystemTransactionV1(
                        ProtocolSystemTransactionV1 { transaction, .. },
                    ) => {
                        // Write manifest
                        let manifest_string = decompile(
                            &SystemTransactionManifestV1::from_transaction(transaction),
                            &self.network_definition,
                        )
                        .unwrap();
                        // Whilst we're here, let's validate that the manifest can be recompiled
                        compile_manifest::<SystemTransactionManifestV1>(
                            &manifest_string,
                            &self.network_definition,
                            BlobProvider::new_with_blobs(
                                transaction
                                    .blobs
                                    .blobs
                                    .iter()
                                    .map(|b| b.0.clone())
                                    .collect(),
                            ),
                        )
                        .expect("Expected the system manifest to be re-compilable");
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
            .configure_babylon(|_| BabylonSettings::test_complex())
            .from_bootstrap_to_latest();
        for protocol_update_exector in protocol_executor.each_protocol_update_executor(&db) {
            let protocol_version = protocol_update_exector.protocol_version;
            let mut version_folder = FolderContentAligner::new(
                root_path.join(protocol_version.logical_name()),
                mode,
                AlignerFolderMode::ExpectNoOtherContent,
            );
            let mut hooks = ProtocolUpdateHooks {
                network_definition: &network_definition,
                address_encoder: &address_encoder,
                manifests_folder: version_folder
                    .register_child_folder("manifests", AlignerFolderMode::ExpectNoOtherContent),
                receipts_folder: version_folder
                    .register_child_folder("receipts", AlignerFolderMode::ExpectNoOtherContent),
                event_hasher: HashAccumulator::new(),
                state_change_hasher: HashAccumulator::new(),
            };

            protocol_update_exector.run_and_commit_advanced(&mut db, &mut hooks, &vm_modules);

            let mut summary = String::new();
            let protocol_version_display_name = protocol_version.display_name();
            writeln!(&mut summary, "Name: {protocol_version_display_name}").unwrap();
            writeln!(&mut summary).unwrap();

            let state_change_digest =
                hooks.state_change_hasher.finalize().to_string()[0..16].to_string();
            let event_digest = hooks.event_hasher.finalize().to_string()[0..16].to_string();

            writeln!(&mut summary, "========= SUMMARY HASHES =========").unwrap();

            let testnet_scenario_names = default_testnet_scenarios_at_version(protocol_version)
                .map(|v| v.metadata().logical_name)
                .collect::<Vec<_>>();

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

            if testnet_scenario_names.len() > 0 {
                writeln!(&mut summary).unwrap();
                writeln!(&mut summary, "==== POST ENACTMENT SCENARIOS ====").unwrap();
                writeln!(&mut summary, "The following scenarios are set by default to run on testnets after this protocol update.").unwrap();
                if protocol_version == ProtocolVersion::LATEST {
                    writeln!(&mut summary, "If there are any changes to this list after deploying a testnet, the testnet's ledger will need to be wiped (*not allowed on stokenet*) or the network's protocol update configuration will need to be customised in the node to override the scenarios run on it.").unwrap();
                } else {
                    writeln!(&mut summary, "This list should NEVER change, else it will cause divergence for any nodes applying the protocol update.").unwrap();
                }
                for scenario_name in testnet_scenario_names {
                    writeln!(&mut summary, "=> {scenario_name}").unwrap();
                }
            }

            version_folder.put_file("protocol_update_summary.txt", summary);
        }
    }

    struct ScenarioDumpingHooks {
        event_hasher: HashAccumulator,
        state_update_hasher: HashAccumulator,
        scenario_folder: FolderContentAligner,
        manifests_folder: FolderContentAligner,
        transactions_folder: FolderContentAligner,
        receipts_folder: FolderContentAligner,
        costings_folder: FolderContentAligner,
    }

    impl ScenarioDumpingHooks {
        fn new(mut scenario_folder: FolderContentAligner) -> Self {
            let mut manifests_folder = scenario_folder
                .register_child_folder("manifests", AlignerFolderMode::ExpectNoOtherContent);
            let mut transactions_folder = scenario_folder
                .register_child_folder("transactions", AlignerFolderMode::ExpectNoOtherContent);
            let mut receipts_folder = scenario_folder
                .register_child_folder("receipts", AlignerFolderMode::ExpectNoOtherContent);
            let mut costings_folder = scenario_folder
                .register_child_folder("costings", AlignerFolderMode::ExpectNoOtherContent);

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
                        .concat_mut(scrypto_encode(&c.application_events).unwrap());
                    self.state_update_hasher
                        .concat_mut(scrypto_encode(&c.state_updates).unwrap());
                }
                TransactionResult::Reject(_) | TransactionResult::Abort(_) => {}
            }

            let transaction_file_prefix = format!(
                "{:03}--{}",
                transaction.stage_counter, transaction.logical_name,
            );

            self.transactions_folder.put_file(
                format!("{transaction_file_prefix}.bin"),
                transaction.raw_transaction.as_slice(),
            );

            // Check transaction manifest
            {
                // NB: We purposefully don't write the blobs as they're contained in the raw transactions
                let manifest_string = match &transaction.transaction_manifest {
                    UserTransactionManifest::V1(m) => decompile(m, network_definition),
                    UserTransactionManifest::V2(m) => decompile(m, network_definition),
                }
                .expect("Manifest should be decompilable to a string representation");

                // Whilst we're here, let's validate that the manifest can be recompiled
                let blob_provider = BlobProvider::new_with_blobs(
                    transaction
                        .transaction_manifest
                        .get_blobs()
                        .map(|(_, value)| value.clone())
                        .collect(),
                );
                match &transaction.transaction_manifest {
                    UserTransactionManifest::V1(_) => {
                        compile_manifest_v1(&manifest_string, network_definition, blob_provider)
                            .map(|_| ())
                    }
                    UserTransactionManifest::V2(_) => compile_manifest::<TransactionManifestV2>(
                        &manifest_string,
                        network_definition,
                        blob_provider,
                    )
                    .map(|_| ()),
                }
                .expect("Decompiled manifest should be recompilable");

                self.manifests_folder
                    .put_file(format!("{transaction_file_prefix}.rtm"), &manifest_string);
            }

            // Check subintent manifests
            for (subintent_index, subintent_manifest) in
                transaction.subintent_manifests.iter().enumerate()
            {
                // NB: We purposefully don't write the blobs as they're contained in the raw transactions
                let manifest_string = match &subintent_manifest {
                    UserSubintentManifest::V2(m) => decompile(m, network_definition),
                }
                .expect("Subintent manifest should be decompilable to a string representation");

                // Whilst we're here, let's validate that the manifest can be recompiled
                let blob_provider = BlobProvider::new_with_blobs(
                    subintent_manifest
                        .get_blobs()
                        .map(|(_, value)| value.clone())
                        .collect(),
                );
                match subintent_manifest {
                    UserSubintentManifest::V2(_) => compile_manifest::<SubintentManifestV2>(
                        &manifest_string,
                        network_definition,
                        blob_provider,
                    )
                    .map(|_| ()),
                }
                .expect("Decompiled manifest should be recompilable");

                self.manifests_folder.put_file(
                    format!("{transaction_file_prefix}--sub-{subintent_index}.rtm"),
                    &manifest_string,
                );
            }

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
        }
    }

    pub fn run_all_scenarios<'a>(
        mode: AlignerExecutionMode,
        scenarios: impl IntoIterator<Item = &'a dyn ScenarioCreatorObjectSafe>,
    ) {
        let root_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("generated-examples");
        let vm_modules = VmModules::default();
        for scenario_creator in scenarios.into_iter() {
            let scenario_logical_name = scenario_creator.metadata().logical_name;
            let min_requirement = scenario_creator.metadata().protocol_min_requirement;
            let max_requirement = scenario_creator.metadata().protocol_max_requirement;
            for protocol_version in
                ProtocolVersion::all_between_inclusive(min_requirement, max_requirement)
            {
                let mut db = InMemorySubstateDatabase::standard();

                let scenario_folder = FolderContentAligner::new(
                    root_path
                        .join(protocol_version.logical_name())
                        .join(scenario_logical_name),
                    mode,
                    AlignerFolderMode::ExpectNoOtherContent,
                );

                // Now execute just the single scenario, after executing protocol updates up to
                // the given protocol version
                TransactionScenarioExecutor::new(db, NetworkDefinition::simulator())
                    .execute_protocol_updates_and_scenarios(
                        |builder| builder.from_bootstrap_to(protocol_version),
                        ScenarioTrigger::AfterCompletionOfAllProtocolUpdates,
                        ScenarioFilter::SpecificScenariosByName(btreeset!(
                            scenario_logical_name.to_string()
                        )),
                        &mut ScenarioDumpingHooks::new(scenario_folder),
                        &mut (),
                        &vm_modules,
                    )
                    .unwrap_or_else(|err| {
                        Err(err).expect(&format!(
                            "Scenario {scenario_logical_name} should execute without error"
                        ))
                    });
            }
        }
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

            fn on_scenario_ended(&mut self, event: OnScenarioEnded<S>) {
                let OnScenarioEnded {
                    metadata,
                    end_state,
                    ..
                } = event;
                if let Some(testnet_run_at) = metadata.testnet_run_at {
                    if testnet_run_at > ProtocolVersion::GENESIS {
                        assert!(
                            metadata.safe_to_run_on_used_ledger,
                            "Scenario \"{}\" is set to run on non-Babylon testnets, but is not marked as `safe_to_run_on_used_ledger`. This could break stokenet. Change the scenario to not use pre-allocated addresses, and set `safe_to_run_on_used_ledger` to `true`.",
                            metadata.logical_name
                        );
                    }
                }
                if metadata.safe_to_run_on_used_ledger {
                    for (address_name, address) in end_state.output.interesting_addresses.0.iter() {
                        if let DescribedAddress::Global(address) = address {
                            let entity_type = address.as_node_id().entity_type().unwrap();
                            assert!(
                                !entity_type.is_global_preallocated(),
                                "Scenario \"{}\" is marked as `safe_to_run_on_used_ledger`, but its interesting address {} is pre-allocated - which suggests the scenario can be broken by someone messing with this address before the scenario runs. Change the scenario to explicitly create accounts/identities (see e.g. `maya-router.rs`).",
                                metadata.logical_name,
                                address_name,
                            );
                        }
                    }
                }
            }
        }
        TransactionScenarioExecutor::new(
            InMemorySubstateDatabase::standard(),
            NetworkDefinition::simulator(),
        )
        .execute_every_protocol_update_and_scenario(&mut Hooks);
    }
}
