#![cfg(feature = "std")]

use std::path::PathBuf;

use radix_engine::system::system_modules::execution_trace::{ResourceSpecifier, WorktopChange};
use radix_engine_interface::prelude::MetadataValue;
use radix_engine_tests::*;
use radix_engine_toolkit_common::receipt::*;
use radix_transaction_scenarios::executor::*;
use scrypto::*;
use scrypto_test::prelude::*;

#[test]
fn test_toolkit_receipt_roundtrip_property() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);

    // Act
    let receipt = ledger.preview_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![],
        0,
        PreviewFlags::default(),
    );
    receipt.expect_commit_success();

    // Assert
    check_and_convert_receipt_to_runtime_receipt(receipt);
}

#[test]
fn test_toolkit_receipt_roundtrip_property_on_scenario_receipts() {
    struct Hooks;
    impl<S> ScenarioExecutionHooks<S> for Hooks
    where
        S: SubstateDatabase,
    {
        fn adapt_execution_config(&mut self, _: ExecutionConfig) -> ExecutionConfig {
            ExecutionConfig::for_preview(NetworkDefinition::simulator())
        }

        fn on_transaction_executed(&mut self, event: OnScenarioTransactionExecuted<S>) {
            check_and_convert_receipt_to_runtime_receipt(event.receipt.clone());
        }
    }
    TransactionScenarioExecutor::new(
        InMemorySubstateDatabase::standard(),
        NetworkDefinition::simulator(),
    )
    .execute_every_protocol_update_and_scenario(&mut Hooks)
    .expect("Must succeed!");
}

#[test]
#[ignore = "Run this test to output the transaction receipts to the file system"]
fn output_scenario_serialized_transaction_receipts_to_file_system() {
    struct Hooks;
    impl<S> ScenarioExecutionHooks<S> for Hooks
    where
        S: SubstateDatabase,
    {
        fn adapt_execution_config(&mut self, _: ExecutionConfig) -> ExecutionConfig {
            ExecutionConfig::for_preview(NetworkDefinition::simulator())
        }

        fn on_transaction_executed(
            &mut self,
            OnScenarioTransactionExecuted {
                metadata,
                transaction,
                receipt,
                ..
            }: OnScenarioTransactionExecuted<S>,
        ) {
            let runtime_toolkit_receipt =
                RuntimeToolkitTransactionReceipt::try_from(receipt.clone()).unwrap();

            // Convert to a serializable transaction receipt.
            let encoder = AddressBech32Encoder::for_simulator();
            let serializable_toolkit_receipt =
                SerializableToolkitTransactionReceipt::contextual_try_from(
                    runtime_toolkit_receipt.clone(),
                    &encoder,
                )
                .expect("Failed during runtime -> serializable conversion");

            // Serialize the serializable receipt to JSON through serde_json.
            let serialized_receipt = serde_json::to_string_pretty(&serializable_toolkit_receipt)
                .expect("Serializing through serde_json failed");

            // Create a file for this transaction receipt.
            let directory_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("assets")
                .join("serialized_receipts")
                .join(metadata.logical_name);
            std::fs::create_dir_all(&directory_path).expect("Creation failed!");
            let file_path = directory_path.join(format!("{}.json", transaction.logical_name));
            std::fs::write(file_path, serialized_receipt).expect("Writing the receipt failed");
        }
    }
    TransactionScenarioExecutor::new(
        InMemorySubstateDatabase::standard(),
        NetworkDefinition::simulator(),
    )
    .execute_every_protocol_update_and_scenario(&mut Hooks)
    .expect("Must succeed!");
}

#[test]
fn test_serialized_scenario_transaction_receipts_can_be_deserialized_and_converted_to_a_runtime_receipt(
) {
    // Arrange
    let mut receipt_strings = Vec::new();
    let directory_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("serialized_receipts");
    for entry in walkdir::WalkDir::new(directory_path) {
        let entry = entry.unwrap();
        let path = entry.into_path();
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            receipt_strings.push(std::fs::read_to_string(path).unwrap());
        }
    }

    // Act
    let converted_receipts = receipt_strings
        .into_iter()
        .map(|receipt| {
            let decoder = AddressBech32Decoder::for_simulator();
            serde_json::from_str::<SerializableToolkitTransactionReceipt>(&receipt)
                .ok()
                .and_then(|value| {
                    RuntimeToolkitTransactionReceipt::contextual_try_from(value, &decoder).ok()
                })
        })
        .collect::<Vec<_>>();

    // Assert
    assert!(converted_receipts.iter().all(|value| value.is_some()));
}

#[test]
fn commit_success_receipt_is_mapped_correctly() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let receipt = ledger.preview_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![],
        0,
        PreviewFlags::default(),
    );
    receipt.expect_commit_success();

    // Act
    let runtime = check_and_convert_receipt_to_runtime_receipt(receipt);

    // Assert
    assert_matches!(runtime, ToolkitTransactionReceipt::CommitSuccess { .. });
}

#[test]
fn commit_failure_receipt_is_mapped_correctly() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let receipt = ledger.preview_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .lock_fee(account, 100)
            .get_free_xrd_from_faucet()
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![],
        0,
        PreviewFlags {
            use_free_credit: false,
            ..Default::default()
        },
    );

    // Act
    let runtime = check_and_convert_receipt_to_runtime_receipt(receipt);

    // Assert
    assert_matches!(runtime, ToolkitTransactionReceipt::CommitFailure { .. });
}

#[test]
fn rejection_receipt_is_mapped_correctly() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let receipt = ledger.preview_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 100)
            .get_free_xrd_from_faucet()
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![],
        0,
        PreviewFlags {
            use_free_credit: false,
            ..Default::default()
        },
    );

    // Act
    let runtime = check_and_convert_receipt_to_runtime_receipt(receipt);

    // Assert
    assert_matches!(runtime, ToolkitTransactionReceipt::Reject { .. });
}

#[test]
fn newly_created_entities_are_mapped_correctly_in_receipt() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (wasm, definition) = (
        include_workspace_asset_bytes!("radix-transaction-scenarios", "radiswap.wasm").to_vec(),
        manifest_decode(include_workspace_asset_bytes!(
            "radix-transaction-scenarios",
            "radiswap.rpd"
        ))
        .unwrap(),
    );
    let (_, _, account) = ledger.new_account(false);
    let receipt = ledger.preview_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            /* Creating a new resource */
            .create_fungible_resource(
                Default::default(),
                Default::default(),
                DIVISIBILITY_MAXIMUM,
                Default::default(),
                Default::default(),
                None,
            )
            /* Creating a new account component */
            .new_account()
            /* Creating a new package */
            .publish_package(wasm, definition)
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![],
        0,
        PreviewFlags {
            use_free_credit: false,
            ..Default::default()
        },
    );
    let commit_result = receipt.expect_commit_success();

    // Act
    let ToolkitTransactionReceipt::CommitSuccess {
        state_updates_summary: StateUpdatesSummary { new_entities, .. },
        ..
    } = check_and_convert_receipt_to_runtime_receipt(receipt.clone())
    else {
        panic!("Not commit success!");
    };

    // Assert
    assert_eq!(
        commit_result
            .new_component_addresses()
            .into_iter()
            .map(|value| value.into_node_id())
            .chain(
                commit_result
                    .new_resource_addresses()
                    .into_iter()
                    .map(|value| value.into_node_id())
            )
            .chain(
                commit_result
                    .new_package_addresses()
                    .into_iter()
                    .map(|value| value.into_node_id())
            )
            .collect::<IndexSet<_>>(),
        new_entities
    )
}

#[test]
fn metadata_updates_show_in_receipt() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_account(false);
    let receipt = ledger.preview_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            /* Set */
            .set_metadata(account, "persisted_key", 1u32)
            /* Remove */
            .set_metadata(account, "removed_key", None)
            /* Transient key */
            .set_metadata(account, "transient_key", 1)
            .set_metadata(account, "transient_key", None)
            /* Transient key 2 */
            .set_metadata(account, "transient_key2", 1)
            .set_metadata(account, "transient_key2", None)
            .set_metadata(account, "transient_key2", 1)
            .set_metadata(account, "transient_key2", None)
            .set_metadata(account, "transient_key2", 2u32)
            .build(),
        vec![public_key.into()],
        0,
        PreviewFlags {
            use_free_credit: false,
            ..Default::default()
        },
    );

    // Act
    let ToolkitTransactionReceipt::CommitSuccess {
        state_updates_summary,
        ..
    } = check_and_convert_receipt_to_runtime_receipt(receipt.clone())
    else {
        panic!("Not commit success!");
    };

    // Assert
    assert_eq!(
        state_updates_summary
            .metadata_updates
            .get(account.as_node_id())
            .and_then(|value| value.get("persisted_key")),
        Some(&MetadataUpdate::Set(MetadataValue::U32(1)))
    );
    assert_eq!(
        state_updates_summary
            .metadata_updates
            .get(account.as_node_id())
            .and_then(|value| value.get("removed_key")),
        Some(&MetadataUpdate::Delete)
    );
    assert_eq!(
        state_updates_summary
            .metadata_updates
            .get(account.as_node_id())
            .and_then(|value| value.get("transient_key")),
        Some(&MetadataUpdate::Delete)
    );
    assert_eq!(
        state_updates_summary
            .metadata_updates
            .get(account.as_node_id())
            .and_then(|value| value.get("transient_key2")),
        Some(&MetadataUpdate::Set(MetadataValue::U32(2)))
    );
}

#[test]
fn non_fungible_data_updates_are_mapped_correctly_in_receipt() {
    // Arrange
    #[derive(ManifestSbor, ScryptoSbor, NonFungibleData)]
    struct Data {
        #[mutable]
        pub field: u32,
    }

    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let execution_config = ExecutionConfig::for_preview_no_auth(NetworkDefinition::simulator());

    // Act
    let receipt1 = ledger.execute_manifest_with_execution_config(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                Default::default(),
                NonFungibleIdType::Integer,
                Default::default(),
                NonFungibleResourceRoles {
                    mint_roles: Some(MintRoles {
                        minter: Some(rule!(allow_all)),
                        minter_updater: Some(rule!(allow_all)),
                    }),
                    burn_roles: Some(BurnRoles {
                        burner: Some(rule!(allow_all)),
                        burner_updater: Some(rule!(allow_all)),
                    }),
                    non_fungible_data_update_roles: Some(NonFungibleDataUpdateRoles {
                        non_fungible_data_updater: Some(rule!(allow_all)),
                        non_fungible_data_updater_updater: Some(rule!(allow_all)),
                    }),
                    ..Default::default()
                },
                Default::default(),
                Some(indexmap! {
                    NonFungibleLocalId::integer(1) => Data { field: 1 },
                }),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![],
        execution_config.clone(),
    );
    let non_fungible_resource = receipt1
        .expect_commit_success()
        .new_resource_addresses()
        .first()
        .copied()
        .unwrap();
    let receipt2 = ledger.execute_manifest_with_execution_config(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                non_fungible_resource,
                NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT,
                NonFungibleResourceManagerUpdateDataManifestInput {
                    id: NonFungibleLocalId::integer(1),
                    field_name: "field".to_owned(),
                    data: manifest_decode(&manifest_encode(&2u32).unwrap()).unwrap(),
                },
            )
            .build(),
        vec![],
        execution_config.clone(),
    );
    let receipt3 = ledger.execute_manifest_with_execution_config(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .burn_in_account(account, non_fungible_resource, 1)
            .build(),
        vec![],
        execution_config,
    );

    // Assert
    let non_fungible_global_id =
        NonFungibleGlobalId::new(non_fungible_resource, NonFungibleLocalId::integer(1));
    let [non_fungible_data_updates1, non_fungible_data_updates2, non_fungible_data_updates3] =
        [receipt1, receipt2, receipt3].map(|receipt| {
            let ToolkitTransactionReceipt::CommitSuccess {
                state_updates_summary:
                    StateUpdatesSummary {
                        non_fungible_data_updates,
                        ..
                    },
                ..
            } = check_and_convert_receipt_to_runtime_receipt(receipt.clone())
            else {
                panic!("Not commit success!");
            };
            non_fungible_data_updates
        });

    assert_eq!(
        non_fungible_data_updates1.get(&non_fungible_global_id),
        Some(&scrypto_encode(&Data { field: 1 }).unwrap())
    );
    assert_eq!(
        non_fungible_data_updates2.get(&non_fungible_global_id),
        Some(&scrypto_encode(&Data { field: 2 }).unwrap())
    );
    assert_eq!(
        non_fungible_data_updates3.get(&non_fungible_global_id),
        None
    );
}

#[test]
fn newly_minted_non_fungibles_are_mapped_correctly_in_receipt() {
    // Arrange
    #[derive(ManifestSbor, ScryptoSbor, NonFungibleData)]
    struct Data {
        #[mutable]
        pub field: u32,
    }

    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account1) = ledger.new_account(false);
    let (_, _, account2) = ledger.new_account(false);
    let execution_config = ExecutionConfig::for_preview_no_auth(NetworkDefinition::simulator());

    // Act
    let receipt = ledger.execute_manifest_with_execution_config(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .allocate_global_address(
                RESOURCE_PACKAGE,
                NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                "address_reservation",
                "named_address",
            )
            .then(|builder| {
                let reservation = builder.address_reservation("address_reservation");
                let named_address = builder.named_address("named_address");

                builder
                    .call_function(
                        RESOURCE_PACKAGE,
                        NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                        NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                        NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
                            owner_role: Default::default(),
                            id_type: NonFungibleIdType::Integer,
                            track_total_supply: Default::default(),
                            non_fungible_schema:
                                NonFungibleDataSchema::new_local_without_self_package_replacement::<
                                    Data,
                                >(),
                            entries: indexmap! {
                                NonFungibleLocalId::integer(1) => Data { field: 1 },
                            }
                            .into_iter()
                            .map(|(key, value)| {
                                (
                                    key,
                                    (manifest_decode(&manifest_encode(&value).unwrap()).unwrap(),),
                                )
                            })
                            .collect(),
                            resource_roles: NonFungibleResourceRoles {
                                mint_roles: Some(MintRoles {
                                    minter: Some(rule!(allow_all)),
                                    minter_updater: Some(rule!(allow_all)),
                                }),
                                burn_roles: Some(BurnRoles {
                                    burner: Some(rule!(allow_all)),
                                    burner_updater: Some(rule!(allow_all)),
                                }),
                                non_fungible_data_update_roles: Some(NonFungibleDataUpdateRoles {
                                    non_fungible_data_updater: Some(rule!(allow_all)),
                                    non_fungible_data_updater_updater: Some(rule!(allow_all)),
                                }),
                                ..Default::default()
                            },
                            metadata: Default::default(),
                            address_reservation: Some(reservation),
                        },
                    )
                    .try_deposit_entire_worktop_or_abort(account1, None)
                    .mint_non_fungible(
                        named_address,
                        indexmap! {
                            NonFungibleLocalId::integer(2) => Data { field: 2 },
                        },
                    )
                    .try_deposit_entire_worktop_or_abort(account2, None)
                    .call_method(
                        account2,
                        ACCOUNT_BURN_NON_FUNGIBLES_IDENT,
                        (named_address, indexset! { NonFungibleLocalId::integer(2) }),
                    )
            })
            .build(),
        vec![],
        execution_config.clone(),
    );
    let non_fungible_resource = receipt
        .expect_commit_success()
        .new_resource_addresses()
        .first()
        .copied()
        .unwrap();

    // Assert
    let ToolkitTransactionReceipt::CommitSuccess {
        state_updates_summary:
            StateUpdatesSummary {
                newly_minted_non_fungibles,
                ..
            },
        ..
    } = check_and_convert_receipt_to_runtime_receipt(receipt.clone())
    else {
        panic!("Not commit success!");
    };
    assert_eq!(
        newly_minted_non_fungibles,
        indexset! { NonFungibleGlobalId::new(non_fungible_resource, NonFungibleLocalId::integer(1)) }
    );
}

#[test]
fn worktop_changes_are_mapped_correctly_in_receipt() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let receipt = ledger.preview_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![],
        0,
        PreviewFlags {
            use_free_credit: false,
            ..Default::default()
        },
    );
    receipt.expect_commit_success();

    // Act
    let ToolkitTransactionReceipt::CommitSuccess {
        worktop_changes, ..
    } = check_and_convert_receipt_to_runtime_receipt(receipt.clone())
    else {
        panic!("Not commit success!");
    };

    // Assert
    assert_eq!(
        worktop_changes,
        indexmap! {
            1 => vec![WorktopChange::Put(ResourceSpecifier::Amount(
                XRD,
                dec!(10_000)
            ))],
            2 => vec![WorktopChange::Take(ResourceSpecifier::Amount(
                XRD,
                dec!(10_000)
            ))]
        }
    )
}

#[test]
fn fee_summary_are_mapped_correctly_in_receipt() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let receipt = ledger.preview_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![],
        0,
        PreviewFlags {
            use_free_credit: false,
            ..Default::default()
        },
    );
    receipt.expect_commit_success();
    let TransactionFeeSummary {
        total_execution_cost_in_xrd,
        total_finalization_cost_in_xrd,
        total_storage_cost_in_xrd,
        total_royalty_cost_in_xrd,
        ..
    } = receipt.fee_summary;

    // Act
    let ToolkitTransactionReceipt::CommitSuccess { fee_summary, .. } =
        check_and_convert_receipt_to_runtime_receipt(receipt.clone())
    else {
        panic!("Not commit success!");
    };

    // Assert
    assert_eq!(
        fee_summary.execution_fees_in_xrd,
        total_execution_cost_in_xrd
    );
    assert_eq!(
        fee_summary.finalization_fees_in_xrd,
        total_finalization_cost_in_xrd
    );
    assert_eq!(fee_summary.storage_fees_in_xrd, total_storage_cost_in_xrd);
    assert_eq!(fee_summary.royalty_fees_in_xrd, total_royalty_cost_in_xrd);
}

#[test]
fn locked_fees_are_mapped_correctly_in_receipt() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);
    let receipt = ledger.preview_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .lock_contingent_fee(account, 10)
            .get_free_xrd_from_faucet()
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![],
        0,
        PreviewFlags {
            use_free_credit: false,
            disable_auth: true,
            ..Default::default()
        },
    );
    receipt.expect_commit_success();

    // Act
    let ToolkitTransactionReceipt::CommitSuccess { locked_fees, .. } =
        check_and_convert_receipt_to_runtime_receipt(receipt.clone())
    else {
        panic!("Not commit success!");
    };

    // Assert
    assert_eq!(locked_fees.contingent, dec!(10));
    assert_eq!(locked_fees.non_contingent, dec!(5000));
}

#[test]
fn receipt_contains_metadata_of_newly_create_resources() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let receipt = ledger.preview_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                metadata! {
                    init {
                        "name" => "Something", locked;
                    }
                },
                Default::default(),
            )
            .build(),
        vec![],
        0,
        PreviewFlags {
            use_free_credit: false,
            disable_auth: true,
            ..Default::default()
        },
    );
    receipt.expect_commit_success();

    // Act
    let ToolkitTransactionReceipt::CommitSuccess {
        state_updates_summary:
            StateUpdatesSummary {
                ref mut metadata_updates,
                ..
            },
        ..
    } = check_and_convert_receipt_to_runtime_receipt(receipt.clone())
    else {
        panic!("Not commit success!");
    };
    let (_, metadata_updates) = metadata_updates
        .pop()
        .expect("There must be at minimum a single entry there.");

    let value = metadata_updates.get("name").unwrap();
    assert_eq!(
        value,
        &MetadataUpdate::Set(MetadataValue::String("Something".to_owned()))
    )
}

/// Converts a receipt to a runtime receipt and does the following checks:
/// * Checks that the receipt can be converted into a runtime receipt.
/// * Checks that the runtime receipt can be converted into a serializable receipt.
/// * Checks that the serializable receipt can be serialized through serde_json.
/// * Checks that the serialized receipt can be deserialized through serde_json.
/// * Checks that the deserialized receipt can be converted into a runtime receipt.
/// * Checks that the runtime receipt obtained from deserialization equals the runtime receipt
///   obtained from direct conversion (roundtrip property)
fn check_and_convert_receipt_to_runtime_receipt(
    receipt: TransactionReceipt,
) -> RuntimeToolkitTransactionReceipt {
    // Convert to a runtime receipt.
    let runtime_toolkit_receipt = RuntimeToolkitTransactionReceipt::try_from(receipt).unwrap();

    // Convert to a serializable transaction receipt.
    let encoder = AddressBech32Encoder::for_simulator();
    let decoder = AddressBech32Decoder::for_simulator();
    let serializable_toolkit_receipt = SerializableToolkitTransactionReceipt::contextual_try_from(
        runtime_toolkit_receipt.clone(),
        &encoder,
    )
    .expect("Failed during runtime -> serializable conversion");

    // Serialize the serializable receipt to JSON through serde_json.
    let serialized_receipt = serde_json::to_string_pretty(&serializable_toolkit_receipt)
        .expect("Serializing through serde_json failed");
    let deserialized_receipt =
        serde_json::from_str::<SerializableToolkitTransactionReceipt>(&serialized_receipt)
            .expect("Deserializing through serde_json failed");

    // Convert the serializable model to a runtime model and test the roundtrip property.
    let roundtrip_runtime_toolkit_receipt =
        RuntimeToolkitTransactionReceipt::contextual_try_from(deserialized_receipt, &decoder)
            .expect("Failed during serializable -> runtime conversion");
    assert_eq!(
        runtime_toolkit_receipt, roundtrip_runtime_toolkit_receipt,
        "Roundtrip property didn't hold"
    );

    // Return the runtime receipt.
    runtime_toolkit_receipt
}
