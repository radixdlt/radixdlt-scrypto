use radix_common::prelude::*;
use radix_engine::system::system_modules::execution_trace::{
    ApplicationFnIdentifier, ExecutionTrace, ResourceSpecifier, TraceOrigin, WorktopChange,
};
use radix_engine_tests::common::*;
use radix_transactions::{model::PreviewFlags, validation::TransactionValidator};
use scrypto_test::prelude::*;

#[test]
fn test_trace_resource_transfers_using_take() {
    run_resource_transfers_trace_test(false);
}

#[test]
fn test_trace_resource_transfers_using_take_advanced() {
    run_resource_transfers_trace_test(true);
}

fn run_resource_transfers_trace_test(use_take_advanced: bool) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("execution_trace"));
    let transfer_amount = 10u8;

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500)
        .call_function(
            package_address,
            "ExecutionTraceBp",
            "transfer_resource_between_two_components",
            manifest_args!(transfer_amount, use_take_advanced),
        )
        .build();
    let receipt = ledger.preview_manifest(
        manifest,
        vec![public_key.clone().into()],
        0,
        PreviewFlags::default(),
    );

    // Assert
    let (_resource_address, source_component, target_component): (
        ResourceAddress,
        ComponentAddress,
        ComponentAddress,
    ) = receipt.expect_commit(true).output(1);

    /* There should be three resource changes: withdrawal from the source vault,
    deposit to the target vault and withdrawal for the fee */
    println!(
        "{:?}",
        receipt
            .expect_commit_success()
            .execution_trace
            .as_ref()
            .unwrap()
            .resource_changes
    );
    assert_eq!(
        2,
        receipt
            .expect_commit_success()
            .execution_trace
            .as_ref()
            .unwrap()
            .resource_changes
            .len()
    ); // Two instructions
    assert_eq!(
        1,
        receipt
            .expect_commit_success()
            .execution_trace
            .as_ref()
            .unwrap()
            .resource_changes
            .get(&0)
            .unwrap()
            .len()
    ); // One resource change in the first instruction (lock fee)
    assert_eq!(
        2,
        receipt
            .expect_commit_success()
            .execution_trace
            .as_ref()
            .unwrap()
            .resource_changes
            .get(&1)
            .unwrap()
            .len()
    ); // One resource change in the first instruction (lock fee)

    let fee_summary = receipt.fee_summary.clone();
    let total_fee_paid = fee_summary.total_cost();

    // Source vault withdrawal
    assert!(receipt
        .expect_commit_success()
        .execution_trace
        .as_ref()
        .unwrap()
        .resource_changes
        .iter()
        .flat_map(|(_, rc)| rc)
        .any(|r| r.node_id == source_component.into()
            && r.amount == Decimal::from(transfer_amount).checked_neg().unwrap()));

    // Target vault deposit
    assert!(receipt
        .expect_commit_success()
        .execution_trace
        .as_ref()
        .unwrap()
        .resource_changes
        .iter()
        .flat_map(|(_, rc)| rc)
        .any(
            |r| r.node_id == target_component.into() && r.amount == Decimal::from(transfer_amount)
        ));

    // Fee withdrawal
    assert!(receipt
        .expect_commit_success()
        .execution_trace
        .as_ref()
        .unwrap()
        .resource_changes
        .iter()
        .flat_map(|(_, rc)| rc)
        .any(|r| r.node_id == account.into()
            && r.amount == Decimal::from(total_fee_paid).checked_neg().unwrap()));
}

#[test]
fn test_trace_fee_payments() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("execution_trace"));

    // Prepare the component that will pay the fee
    let manifest_prepare = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .get_free_xrd_from_faucet()
        .call_function(
            package_address,
            "ExecutionTraceBp",
            "create_and_fund_a_component",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .drop_auth_zone_proofs()
        .build();

    let funded_component = ledger
        .execute_manifest(manifest_prepare, vec![])
        .expect_commit(true)
        .new_component_addresses()
        .into_iter()
        .nth(0)
        .unwrap()
        .clone();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            funded_component.clone(),
            "test_lock_contingent_fee",
            manifest_args!(),
        )
        .drop_auth_zone_proofs()
        .build();

    let receipt = ledger.preview_manifest(manifest, vec![], 0, PreviewFlags::default());

    // Assert
    let resource_changes = &receipt
        .expect_commit_success()
        .execution_trace
        .as_ref()
        .unwrap()
        .resource_changes;
    let fee_summary = receipt.fee_summary.clone();
    let total_fee_paid = fee_summary.total_cost();

    assert_eq!(1, resource_changes.len());
    assert!(resource_changes
        .into_iter()
        .flat_map(|(_, rc)| rc)
        .any(|r| r.node_id == funded_component.into()
            && r.amount == total_fee_paid.checked_neg().unwrap()));
}

#[test]
fn test_instruction_traces() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("execution_trace"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .get_free_xrd_from_faucet()
        .take_all_from_worktop(XRD, "bucket")
        .create_proof_from_bucket_of_all("bucket", "proof")
        .drop_proof("proof")
        .return_to_worktop("bucket")
        .call_function(
            package_address,
            "ExecutionTraceBp",
            "create_and_fund_a_component",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();

    let receipt = ledger.preview_manifest(manifest, vec![], 0, PreviewFlags::default());

    let traces: Vec<ExecutionTrace> = receipt
        .expect_commit_success()
        .execution_trace
        .as_ref()
        .unwrap()
        .execution_traces
        .clone();

    assert_eq!(8, traces.len());

    // Check traces for the 7 manifest instructions
    {
        // LOCK_FEE
        let traces = traces_for_instruction(&traces, 0);
        assert!(traces.is_empty()); // No traces for lock_fee
    }

    {
        // CALL_METHOD: free
        let traces = traces_for_instruction(&traces, 1);
        // Expecting two traces: an output bucket from the "free" call
        // followed by a single input (auto-add to worktop) - in this order.
        assert_eq!(2, traces.len());
        let free_trace = traces.get(0).unwrap();
        if let TraceOrigin::ScryptoMethod(ApplicationFnIdentifier {
            ident: method_name, ..
        }) = &free_trace.origin
        {
            assert_eq!("free", method_name);
        } else {
            panic!(
                "Expected a scrypto method call but was {:?}",
                free_trace.origin
            );
        };
        assert!(free_trace.input.is_empty());
        assert!(free_trace.output.proofs.is_empty());
        assert_eq!(1, free_trace.output.buckets.len());
        let output_resource = free_trace.output.buckets.values().nth(0).unwrap();
        assert_eq!(XRD, output_resource.resource_address());
        assert_eq!(dec!("10000"), output_resource.amount());

        let worktop_put_trace = traces.get(1).unwrap();
        assert_eq!(
            TraceOrigin::ScryptoMethod(ApplicationFnIdentifier {
                blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, WORKTOP_BLUEPRINT),
                ident: WORKTOP_PUT_IDENT.to_string(),
            }),
            worktop_put_trace.origin
        );
        assert!(worktop_put_trace.output.is_empty());
        assert!(worktop_put_trace.input.proofs.is_empty());
        assert_eq!(1, worktop_put_trace.input.buckets.len());
        let input_resource = worktop_put_trace.input.buckets.values().nth(0).unwrap();
        assert_eq!(XRD, input_resource.resource_address());
        assert_eq!(dec!("10000"), input_resource.amount());
    }

    {
        // TAKE_ALL_FROM_WORKTOP
        let traces = traces_for_instruction(&traces, 2);
        // Take from worktop is just a single sys call with a single bucket output
        assert_eq!(1, traces.len());

        let trace = traces.get(0).unwrap();
        assert_eq!(
            TraceOrigin::ScryptoMethod(ApplicationFnIdentifier {
                blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, WORKTOP_BLUEPRINT),
                ident: WORKTOP_TAKE_ALL_IDENT.to_string(),
            }),
            trace.origin
        );

        assert!(trace.input.is_empty());
        assert!(trace.output.proofs.is_empty());
        assert_eq!(1, trace.output.buckets.len());

        let output_resource = trace.output.buckets.values().nth(0).unwrap();
        assert_eq!(XRD, output_resource.resource_address());
        assert_eq!(dec!("10000"), output_resource.amount());
    }

    {
        // CREATE_PROOF_FROM_BUCKET
        let traces = traces_for_instruction(&traces, 3);
        assert_eq!(1, traces.len());
        let trace = traces.get(0).unwrap();
        assert_eq!(
            TraceOrigin::ScryptoMethod(ApplicationFnIdentifier {
                blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, FUNGIBLE_BUCKET_BLUEPRINT),
                ident: BUCKET_CREATE_PROOF_OF_ALL_IDENT.to_string(),
            }),
            trace.origin
        );

        assert!(trace.input.is_empty());
        assert!(trace.output.buckets.is_empty());
        assert_eq!(1, trace.output.proofs.len());

        let output_proof = trace.output.proofs.values().nth(0).unwrap();
        assert_eq!(XRD, output_proof.resource_address());
        assert_eq!(dec!(10000), output_proof.amount());
    }

    {
        // DROP_PROOF
        let traces = traces_for_instruction(&traces, 4);
        assert_eq!(1, traces.len());
        let trace = traces.get(0).unwrap();
        assert_eq!(
            TraceOrigin::ScryptoFunction(ApplicationFnIdentifier {
                blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, FUNGIBLE_PROOF_BLUEPRINT),
                ident: PROOF_DROP_IDENT.to_string()
            }),
            trace.origin
        );

        assert!(trace.output.is_empty());
        assert!(trace.input.buckets.is_empty());
        assert_eq!(1, trace.input.proofs.len());

        let input_proof = trace.input.proofs.values().nth(0).unwrap();
        assert_eq!(XRD, input_proof.resource_address());
        assert_eq!(dec!(10000), input_proof.amount());
    }

    {
        // RETURN_TO_WORKTOP
        let traces = traces_for_instruction(&traces, 5);
        assert_eq!(1, traces.len());
        let trace = traces.get(0).unwrap();
        assert_eq!(
            TraceOrigin::ScryptoMethod(ApplicationFnIdentifier {
                blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, WORKTOP_BLUEPRINT),
                ident: WORKTOP_PUT_IDENT.to_string(),
            }),
            trace.origin
        );
        assert!(trace.output.is_empty());
        assert!(trace.input.proofs.is_empty());
        assert_eq!(1, trace.input.buckets.len());

        let input_resource = trace.input.buckets.values().nth(0).unwrap();
        assert_eq!(XRD, input_resource.resource_address());
        assert_eq!(dec!("10000"), input_resource.amount());
    }

    {
        // CALL_FUNCTION: create_and_fund_a_component
        let traces = traces_for_instruction(&traces, 6);
        // Expected two traces: take from worktop and call scrypto function
        assert_eq!(2, traces.len());

        let take_trace = traces.get(0).unwrap();
        assert_eq!(
            TraceOrigin::ScryptoMethod(ApplicationFnIdentifier {
                blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, WORKTOP_BLUEPRINT),
                ident: WORKTOP_DRAIN_IDENT.to_string(),
            }),
            take_trace.origin
        );

        let call_trace = traces.get(1).unwrap();
        if let TraceOrigin::ScryptoFunction(ApplicationFnIdentifier {
            ident: function_name,
            ..
        }) = &call_trace.origin
        {
            assert_eq!("create_and_fund_a_component", function_name);
        } else {
            panic!("Expected a scrypto function call");
        };
        assert!(call_trace.output.is_empty());
        assert!(call_trace.input.proofs.is_empty());
        assert_eq!(1, call_trace.input.buckets.len());
        let input_resource = call_trace.input.buckets.values().nth(0).unwrap();
        assert_eq!(XRD, input_resource.resource_address());
        assert_eq!(dec!("10000"), input_resource.amount());
    }
}

#[test]
fn test_worktop_changes() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk, _, account) = ledger.new_account(false);

    let fungible_resource = ledger.create_fungible_resource(100.into(), 18, account);
    let non_fungible_resource = ledger.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .withdraw_from_account(account, fungible_resource, 100)
        .withdraw_non_fungibles_from_account(
            account,
            non_fungible_resource,
            [
                NonFungibleLocalId::integer(1),
                NonFungibleLocalId::integer(2),
                NonFungibleLocalId::integer(3),
            ],
        )
        .take_all_from_worktop(fungible_resource, "bucket1")
        .return_to_worktop("bucket1")
        .take_from_worktop(fungible_resource, 20, "bucket2")
        .return_to_worktop("bucket2")
        .take_all_from_worktop(non_fungible_resource, "bucket3")
        .return_to_worktop("bucket3")
        .take_from_worktop(non_fungible_resource, 2, "bucket4")
        .return_to_worktop("bucket4")
        .take_non_fungibles_from_worktop(
            non_fungible_resource,
            [
                NonFungibleLocalId::integer(1),
                NonFungibleLocalId::integer(3),
            ],
            "bucket5",
        )
        .return_to_worktop("bucket5")
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.preview_manifest(
        manifest,
        vec![pk.clone().into()],
        0,
        PreviewFlags::default(),
    );

    // Assert
    {
        receipt.expect_commit_success();

        let worktop_changes = receipt
            .expect_commit_success()
            .execution_trace
            .as_ref()
            .unwrap()
            .worktop_changes();

        // Lock fee
        assert_eq!(worktop_changes.get(&0), None);

        // Withdraw fungible resource from account
        assert_eq!(
            worktop_changes.get(&1),
            Some(&vec![WorktopChange::Put(ResourceSpecifier::Amount(
                fungible_resource,
                100.into()
            ))])
        );

        // Withdraw non-fungible resource from account
        assert_eq!(
            worktop_changes.get(&2),
            Some(&vec![WorktopChange::Put(ResourceSpecifier::Ids(
                non_fungible_resource,
                indexset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2),
                    NonFungibleLocalId::integer(3),
                )
            ))])
        );

        // Take fungible resource from worktop (takes all)
        assert_eq!(
            worktop_changes.get(&3),
            Some(&vec![WorktopChange::Take(ResourceSpecifier::Amount(
                fungible_resource,
                100.into()
            ))])
        );
        assert_eq!(
            worktop_changes.get(&4),
            Some(&vec![WorktopChange::Put(ResourceSpecifier::Amount(
                fungible_resource,
                100.into()
            ))])
        );

        // Take fungible resource from worktop by amount
        assert_eq!(
            worktop_changes.get(&5),
            Some(&vec![WorktopChange::Take(ResourceSpecifier::Amount(
                fungible_resource,
                20.into()
            ))])
        );
        assert_eq!(
            worktop_changes.get(&6),
            Some(&vec![WorktopChange::Put(ResourceSpecifier::Amount(
                fungible_resource,
                20.into()
            ))])
        );

        // Take non-fungible from worktop (takes all)
        assert_eq!(
            worktop_changes.get(&7),
            Some(&vec![WorktopChange::Take(ResourceSpecifier::Ids(
                non_fungible_resource,
                indexset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2),
                    NonFungibleLocalId::integer(3),
                )
            ))])
        );
        assert_eq!(
            worktop_changes.get(&8),
            Some(&vec![WorktopChange::Put(ResourceSpecifier::Ids(
                non_fungible_resource,
                indexset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2),
                    NonFungibleLocalId::integer(3),
                )
            ))])
        );

        // Take non-fungible from worktop by amount
        assert_eq!(
            worktop_changes.get(&9),
            Some(&vec![WorktopChange::Take(ResourceSpecifier::Ids(
                non_fungible_resource,
                indexset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2),
                )
            ))])
        );
        assert_eq!(
            worktop_changes.get(&10),
            Some(&vec![WorktopChange::Put(ResourceSpecifier::Ids(
                non_fungible_resource,
                indexset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(2),
                )
            ))])
        );

        // Take non-fungible from worktop by ids
        assert_eq!(
            worktop_changes.get(&11),
            Some(&vec![WorktopChange::Take(ResourceSpecifier::Ids(
                non_fungible_resource,
                indexset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(3),
                )
            ))])
        );
        assert_eq!(
            worktop_changes.get(&12),
            Some(&vec![WorktopChange::Put(ResourceSpecifier::Ids(
                non_fungible_resource,
                indexset!(
                    NonFungibleLocalId::integer(1),
                    NonFungibleLocalId::integer(3),
                )
            ))])
        );

        // Take all from worktop and deposit
        assert_eq!(
            worktop_changes.get(&13),
            Some(&vec![
                WorktopChange::Take(ResourceSpecifier::Amount(fungible_resource, 100.into())),
                WorktopChange::Take(ResourceSpecifier::Ids(
                    non_fungible_resource,
                    indexset!(
                        NonFungibleLocalId::integer(1),
                        NonFungibleLocalId::integer(2),
                        NonFungibleLocalId::integer(3),
                    )
                )),
            ])
        );
    }
}

fn traces_for_instruction(
    traces: &Vec<ExecutionTrace>,
    instruction_index: usize,
) -> Vec<&ExecutionTrace> {
    traces
        .iter()
        .filter(|t| t.instruction_index == instruction_index)
        .collect()
}

#[test]
fn test_execution_trace_for_transaction_v2() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key1, private_key1, account1) = ledger.new_allocated_account();
    let (_public_key2, private_key2, account2) = ledger.new_allocated_account();

    // Flow:
    // 1. root sends child 10 XRD
    // 2. child deposits 7 XRD
    // 3. child yields 3 XRD to root
    // 4. root deposits all
    let start_epoch_inclusive = ledger.get_current_epoch();
    let end_epoch_exclusive = start_epoch_inclusive.after(1).unwrap();
    let transaction = TransactionV2Builder::new()
        .add_signed_child(
            "child",
            PartialTransactionV2Builder::new()
                .intent_header(IntentHeaderV2 {
                    network_id: NetworkDefinition::simulator().id,
                    start_epoch_inclusive,
                    end_epoch_exclusive,
                    min_proposer_timestamp_inclusive: None,
                    max_proposer_timestamp_exclusive: None,
                    intent_discriminator: 1,
                })
                .manifest_builder(|builder| {
                    builder
                        .take_from_worktop(XRD, 7, "bucket1")
                        .try_deposit_batch_or_abort(account2, ["bucket1"], None)
                        .take_all_from_worktop(XRD, "bucket2")
                        .yield_to_parent_with_name_lookup(|lookup| (lookup.bucket("bucket2"),))
                })
                .sign(&private_key2)
                .build_minimal(),
        )
        .intent_header(IntentHeaderV2 {
            network_id: NetworkDefinition::simulator().id,
            start_epoch_inclusive,
            end_epoch_exclusive,
            min_proposer_timestamp_inclusive: None,
            max_proposer_timestamp_exclusive: None,
            intent_discriminator: 2,
        })
        .manifest_builder(|builder| {
            builder
                .lock_fee(account1, 3)
                .withdraw_from_account(account1, XRD, 10)
                .take_all_from_worktop(XRD, "bucket")
                .yield_to_child_with_name_lookup("child", |lookup| (lookup.bucket("bucket"),))
                .deposit_entire_worktop(account1)
        })
        .transaction_header(TransactionHeaderV2 {
            notary_public_key: public_key1.into(),
            notary_is_signatory: false,
            tip_basis_points: 0,
        })
        .sign(&private_key1)
        .notarize(&private_key1)
        .build_minimal()
        .to_raw()
        .unwrap();

    let executable = transaction
        .validate(&TransactionValidator::new_for_latest_simulator())
        .expect("Expected raw transaction to be valid")
        .create_executable();
    let receipt = ledger.execute_transaction(
        executable,
        ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator())
            .with_execution_trace(Some(10)),
    );

    let trace = receipt.expect_commit_success().execution_trace.clone();
    let expected_trace = r#"Some(
    TransactionExecutionTrace {
        execution_traces: [
            ExecutionTrace {
                origin: ScryptoMethod(
                    ApplicationFnIdentifier {
                        blueprint_id: PackageAddress(0d906318c6318c6ee313598c6318c6318cf7bcaa2e954a9626318c6318c6):<Account>,
                        ident: "withdraw",
                    },
                ),
                kernel_call_depth: 0,
                current_frame_actor: Method(
                    NodeId(
                        "c1f7abd48c518b8ebdc6a35abfbe78583725a97eabdc99224571e0d11d42",
                    ),
                ),
                current_frame_depth: 1,
                instruction_index: 1,
                input: ResourceSummary {
                    buckets: {},
                    proofs: {},
                },
                output: ResourceSummary {
                    buckets: {
                        NodeId(
                            "f86a60c975ad7ef440e3a31a7926af0b51258b92ccc79a83b28dfbfb1c1e",
                        ): Fungible {
                            resource_address: ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6),
                            liquid: 10,
                        },
                    },
                    proofs: {},
                },
                children: [
                    ExecutionTrace {
                        origin: ScryptoMethod(
                            ApplicationFnIdentifier {
                                blueprint_id: PackageAddress(0d906318c6318c61e603c64c6318c6318cf7be913d63aafbc6318c6318c6):<FungibleVault>,
                                ident: "take",
                            },
                        ),
                        kernel_call_depth: 1,
                        current_frame_actor: Method(
                            NodeId(
                                "58d39b18c2cb0885ab8e1da7b25b973ed5489f64e0765696956941aa1cf5",
                            ),
                        ),
                        current_frame_depth: 2,
                        instruction_index: 1,
                        input: ResourceSummary {
                            buckets: {},
                            proofs: {},
                        },
                        output: ResourceSummary {
                            buckets: {
                                NodeId(
                                    "f86a60c975ad7ef440e3a31a7926af0b51258b92ccc79a83b28dfbfb1c1e",
                                ): Fungible {
                                    resource_address: ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6),
                                    liquid: 10,
                                },
                            },
                            proofs: {},
                        },
                        children: [
                            ExecutionTrace {
                                origin: CreateNode,
                                kernel_call_depth: 2,
                                current_frame_actor: Method(
                                    NodeId(
                                        "58d39b18c2cb0885ab8e1da7b25b973ed5489f64e0765696956941aa1cf5",
                                    ),
                                ),
                                current_frame_depth: 2,
                                instruction_index: 1,
                                input: ResourceSummary {
                                    buckets: {},
                                    proofs: {},
                                },
                                output: ResourceSummary {
                                    buckets: {
                                        NodeId(
                                            "f86a60c975ad7ef440e3a31a7926af0b51258b92ccc79a83b28dfbfb1c1e",
                                        ): Fungible {
                                            resource_address: ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6),
                                            liquid: 10,
                                        },
                                    },
                                    proofs: {},
                                },
                                children: [],
                            },
                        ],
                    },
                ],
            },
            ExecutionTrace {
                origin: ScryptoMethod(
                    ApplicationFnIdentifier {
                        blueprint_id: PackageAddress(0d906318c6318c61e603c64c6318c6318cf7be913d63aafbc6318c6318c6):<Worktop>,
                        ident: "Worktop_put",
                    },
                ),
                kernel_call_depth: 0,
                current_frame_actor: Method(
                    NodeId(
                        "f8811b53b24a9f967ff36ae876d1d275740052e56d3e3f0c5cf1f150b15a",
                    ),
                ),
                current_frame_depth: 1,
                instruction_index: 1,
                input: ResourceSummary {
                    buckets: {
                        NodeId(
                            "f86a60c975ad7ef440e3a31a7926af0b51258b92ccc79a83b28dfbfb1c1e",
                        ): Fungible {
                            resource_address: ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6),
                            liquid: 10,
                        },
                    },
                    proofs: {},
                },
                output: ResourceSummary {
                    buckets: {},
                    proofs: {},
                },
                children: [],
            },
            ExecutionTrace {
                origin: ScryptoMethod(
                    ApplicationFnIdentifier {
                        blueprint_id: PackageAddress(0d906318c6318c61e603c64c6318c6318cf7be913d63aafbc6318c6318c6):<Worktop>,
                        ident: "Worktop_take_all",
                    },
                ),
                kernel_call_depth: 0,
                current_frame_actor: Method(
                    NodeId(
                        "f8811b53b24a9f967ff36ae876d1d275740052e56d3e3f0c5cf1f150b15a",
                    ),
                ),
                current_frame_depth: 1,
                instruction_index: 2,
                input: ResourceSummary {
                    buckets: {},
                    proofs: {},
                },
                output: ResourceSummary {
                    buckets: {
                        NodeId(
                            "f86a60c975ad7ef440e3a31a7926af0b51258b92ccc79a83b28dfbfb1c1e",
                        ): Fungible {
                            resource_address: ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6),
                            liquid: 10,
                        },
                    },
                    proofs: {},
                },
                children: [],
            },
            ExecutionTrace {
                origin: ScryptoMethod(
                    ApplicationFnIdentifier {
                        blueprint_id: PackageAddress(0d906318c6318c61e603c64c6318c6318cf7be913d63aafbc6318c6318c6):<Worktop>,
                        ident: "Worktop_put",
                    },
                ),
                kernel_call_depth: 0,
                current_frame_actor: Method(
                    NodeId(
                        "f8811b53b24a9f967ff36ae876d1d275740052e56d3e3f0c5cf1f150b15a",
                    ),
                ),
                current_frame_depth: 1,
                instruction_index: 3,
                input: ResourceSummary {
                    buckets: {
                        NodeId(
                            "f86a60c975ad7ef440e3a31a7926af0b51258b92ccc79a83b28dfbfb1c1e",
                        ): Fungible {
                            resource_address: ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6),
                            liquid: 3,
                        },
                    },
                    proofs: {},
                },
                output: ResourceSummary {
                    buckets: {},
                    proofs: {},
                },
                children: [],
            },
            ExecutionTrace {
                origin: ScryptoMethod(
                    ApplicationFnIdentifier {
                        blueprint_id: PackageAddress(0d906318c6318c61e603c64c6318c6318cf7be913d63aafbc6318c6318c6):<Worktop>,
                        ident: "Worktop_drain",
                    },
                ),
                kernel_call_depth: 0,
                current_frame_actor: Method(
                    NodeId(
                        "f8811b53b24a9f967ff36ae876d1d275740052e56d3e3f0c5cf1f150b15a",
                    ),
                ),
                current_frame_depth: 1,
                instruction_index: 4,
                input: ResourceSummary {
                    buckets: {},
                    proofs: {},
                },
                output: ResourceSummary {
                    buckets: {
                        NodeId(
                            "f86a60c975ad7ef440e3a31a7926af0b51258b92ccc79a83b28dfbfb1c1e",
                        ): Fungible {
                            resource_address: ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6),
                            liquid: 3,
                        },
                    },
                    proofs: {},
                },
                children: [],
            },
            ExecutionTrace {
                origin: ScryptoMethod(
                    ApplicationFnIdentifier {
                        blueprint_id: PackageAddress(0d906318c6318c6ee313598c6318c6318cf7bcaa2e954a9626318c6318c6):<Account>,
                        ident: "deposit_batch",
                    },
                ),
                kernel_call_depth: 0,
                current_frame_actor: Method(
                    NodeId(
                        "c1f7abd48c518b8ebdc6a35abfbe78583725a97eabdc99224571e0d11d42",
                    ),
                ),
                current_frame_depth: 1,
                instruction_index: 4,
                input: ResourceSummary {
                    buckets: {
                        NodeId(
                            "f86a60c975ad7ef440e3a31a7926af0b51258b92ccc79a83b28dfbfb1c1e",
                        ): Fungible {
                            resource_address: ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6),
                            liquid: 3,
                        },
                    },
                    proofs: {},
                },
                output: ResourceSummary {
                    buckets: {},
                    proofs: {},
                },
                children: [
                    ExecutionTrace {
                        origin: ScryptoMethod(
                            ApplicationFnIdentifier {
                                blueprint_id: PackageAddress(0d906318c6318c61e603c64c6318c6318cf7be913d63aafbc6318c6318c6):<FungibleVault>,
                                ident: "put",
                            },
                        ),
                        kernel_call_depth: 1,
                        current_frame_actor: Method(
                            NodeId(
                                "58d39b18c2cb0885ab8e1da7b25b973ed5489f64e0765696956941aa1cf5",
                            ),
                        ),
                        current_frame_depth: 2,
                        instruction_index: 4,
                        input: ResourceSummary {
                            buckets: {
                                NodeId(
                                    "f86a60c975ad7ef440e3a31a7926af0b51258b92ccc79a83b28dfbfb1c1e",
                                ): Fungible {
                                    resource_address: ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6),
                                    liquid: 3,
                                },
                            },
                            proofs: {},
                        },
                        output: ResourceSummary {
                            buckets: {},
                            proofs: {},
                        },
                        children: [
                            ExecutionTrace {
                                origin: DropNode,
                                kernel_call_depth: 2,
                                current_frame_actor: Method(
                                    NodeId(
                                        "58d39b18c2cb0885ab8e1da7b25b973ed5489f64e0765696956941aa1cf5",
                                    ),
                                ),
                                current_frame_depth: 2,
                                instruction_index: 4,
                                input: ResourceSummary {
                                    buckets: {
                                        NodeId(
                                            "f86a60c975ad7ef440e3a31a7926af0b51258b92ccc79a83b28dfbfb1c1e",
                                        ): Fungible {
                                            resource_address: ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6),
                                            liquid: 3,
                                        },
                                    },
                                    proofs: {},
                                },
                                output: ResourceSummary {
                                    buckets: {},
                                    proofs: {},
                                },
                                children: [],
                            },
                        ],
                    },
                ],
            },
        ],
        resource_changes: {
            0: [
                ResourceChange {
                    node_id: NodeId(
                        "c1f7abd48c518b8ebdc6a35abfbe78583725a97eabdc99224571e0d11d42",
                    ),
                    vault_id: NodeId(
                        "58d39b18c2cb0885ab8e1da7b25b973ed5489f64e0765696956941aa1cf5",
                    ),
                    resource_address: ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6),
                    amount: -0.34813389042,
                },
            ],
            1: [
                ResourceChange {
                    node_id: NodeId(
                        "c1f7abd48c518b8ebdc6a35abfbe78583725a97eabdc99224571e0d11d42",
                    ),
                    vault_id: NodeId(
                        "58d39b18c2cb0885ab8e1da7b25b973ed5489f64e0765696956941aa1cf5",
                    ),
                    resource_address: ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6),
                    amount: -10,
                },
            ],
            4: [
                ResourceChange {
                    node_id: NodeId(
                        "c1f7abd48c518b8ebdc6a35abfbe78583725a97eabdc99224571e0d11d42",
                    ),
                    vault_id: NodeId(
                        "58d39b18c2cb0885ab8e1da7b25b973ed5489f64e0765696956941aa1cf5",
                    ),
                    resource_address: ResourceAddress(5da66318c6318c61f5a61b4c6318c6318cf794aa8d295f14e6318c6318c6),
                    amount: 3,
                },
            ],
        },
        fee_locks: FeeLocks {
            lock: 3,
            contingent_lock: 0,
        },
    },
)"#;
    assert_eq!(format!("{:#?}", trace), expected_trace);
}
