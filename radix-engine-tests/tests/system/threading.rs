use radix_engine_tests::common::PackageLoader;
use scrypto_test::prelude::*;

#[test]
fn can_transfer_locked_bucket_between_threads() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, sk1, account1) = ledger.new_allocated_account();

    // Prepares a component that can return a locked bucket (and a proof).
    //
    // When a bucket is inserted into the worktop, it's added as-is if there is no corresponding bucket
    // allocated for the resource address, otherwise it's "merged" into the existing bucket, which will check
    // lock status.
    let package_address = ledger.publish_package_simple(PackageLoader::get("threading"));
    let component_address = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(FAUCET, "free", ())
                .take_all_from_worktop(XRD, "bucket")
                .call_function_with_name_lookup(package_address, "Threading", "new", |lookup| {
                    (lookup.bucket("bucket"),)
                })
                .build(),
            [],
        )
        .expect_commit_success()
        .new_component_addresses()[0];

    // Flow:
    // 1. root creates a locked bucket
    // 2. root sends child the bucket
    // 3. child returns the bucket
    // 4. root frees the bucket
    // 5. root deposit the bucket into an account
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
                        // EntireWorktop will ensure the buckets are passed as-is.
                        .yield_to_parent((ManifestExpression::EntireWorktop,))
                })
                .sign(&sk1)
                .build(),
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
                .call_method(component_address, "create_locked_bucket", (dec!(1),))
                // EntireWorktop will ensure the buckets are passed as-is.
                .yield_to_child("child", (ManifestExpression::EntireWorktop,))
                // Free the bucket
                .drop_all_proofs()
                .try_deposit_entire_worktop_or_abort(account1, None)
        })
        .transaction_header(TransactionHeaderV2 {
            notary_public_key: pk1.into(),
            notary_is_signatory: false,
            tip_basis_points: 0,
        })
        .sign(&sk1)
        .notarize(&sk1)
        .build();

    let receipt = ledger.execute_transaction(&transaction, ExecutionConfig::for_test_transaction());
    receipt.expect_commit_success();
}

// Arguably, we may disallow transferring references
#[test]
fn can_pass_global_and_direct_access_references() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pk1, sk1, account1) = ledger.new_allocated_account();
    let (_, _, account) = ledger.new_allocated_account();
    let vault = ledger.get_component_vaults(account, XRD).pop().unwrap();

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
                        // Unfortunately, there is no way to grab the received references
                        .yield_to_parent(())
                })
                .sign(&sk1)
                .build(),
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
            builder.lock_fee(account1, 3).yield_to_child(
                "child",
                (
                    ManifestAddress::Static(account.into_node_id()),
                    ManifestAddress::Static(vault),
                ),
            )
        })
        .transaction_header(TransactionHeaderV2 {
            notary_public_key: pk1.into(),
            notary_is_signatory: false,
            tip_basis_points: 0,
        })
        .sign(&sk1)
        .notarize(&sk1)
        .build();

    let receipt = ledger.execute_transaction(&transaction, ExecutionConfig::for_test_transaction());
    receipt.expect_commit_success();
}

#[test]
fn can_not_pass_address_reservation() {
    let mut ledger = LedgerSimulatorBuilder::new().with_kernel_trace().build();
    let (pk1, sk1, account1) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("threading"));

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
                .manifest_builder(|builder| builder.yield_to_parent(()))
                .sign(&sk1)
                .build(),
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
                .allocate_global_address(
                    package_address,
                    "Threading",
                    "address_reservation",
                    "address",
                )
                .yield_to_child_with_name_lookup("child", |lookup| {
                    (lookup.address_reservation("address_reservation"),)
                })
        })
        .transaction_header(TransactionHeaderV2 {
            notary_public_key: pk1.into(),
            notary_is_signatory: false,
            tip_basis_points: 0,
        })
        .sign(&sk1)
        .notarize(&sk1)
        .build();

    let receipt = ledger.execute_transaction(&transaction, ExecutionConfig::for_test_transaction());
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::SystemError(SystemError::NotAnObject))
    });
}

#[test]
fn can_pass_named_address() {
    let mut ledger = LedgerSimulatorBuilder::new().with_kernel_trace().build();
    let (pk1, sk1, account1) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("threading"));

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
                .manifest_builder(|builder| builder.yield_to_parent(()))
                .sign(&sk1)
                .build(),
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
                .allocate_global_address(
                    package_address,
                    "Threading",
                    "address_reservation",
                    "address",
                )
                .yield_to_child_with_name_lookup("child", |lookup| {
                    (lookup.named_address("address"),)
                })
                .call_function_with_name_lookup(package_address, "Threading", "new2", |lookup| {
                    (lookup.address_reservation("address_reservation"),)
                })
        })
        .transaction_header(TransactionHeaderV2 {
            notary_public_key: pk1.into(),
            notary_is_signatory: false,
            tip_basis_points: 0,
        })
        .sign(&sk1)
        .notarize(&sk1)
        .build();

    let receipt = ledger.execute_transaction(&transaction, ExecutionConfig::for_test_transaction());
    receipt.expect_commit_success();
}
