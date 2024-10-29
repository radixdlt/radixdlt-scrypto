use radix_engine_tests::common::PackageLoader;
use radix_transactions::{
    errors::*,
    manifest::{BuildableManifest, ManifestValidationError},
};
use scrypto_test::prelude::*;

// Some of the tests in this file are to demonstrate the current behavior.

#[test]
fn can_transfer_locked_bucket_between_threads() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, sk1, account1) = ledger.new_allocated_account();

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
    let transaction = ledger
        .v2_transaction_builder()
        .add_signed_child(
            "child",
            ledger
                .v2_partial_transaction_builder()
                .manifest_builder(|builder| {
                    builder
                        // EntireWorktop will ensure the buckets are passed as-is.
                        .yield_to_parent((ManifestExpression::EntireWorktop,))
                })
                .build(),
        )
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
        .sign(&sk1)
        .notarize(&ledger.default_notary())
        .build();

    let receipt = ledger.execute_notarized_transaction(transaction);
    receipt.expect_commit_success();
}

// Arguably, we may disallow transferring references
#[test]
fn can_pass_global_and_direct_access_references() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, sk1, account1) = ledger.new_allocated_account();
    let (_, _, account) = ledger.new_allocated_account();
    let vault = ledger.get_component_vaults(account, XRD).pop().unwrap();

    let transaction = ledger
        .v2_transaction_builder()
        .add_signed_child(
            "child",
            ledger
                .v2_partial_transaction_builder()
                .manifest_builder(|builder| {
                    builder
                        // Unfortunately, there is no way to grab the received references
                        .yield_to_parent(())
                })
                .build(),
        )
        .manifest_builder(|builder| {
            builder
                .lock_fee(account1, 3)
                .yield_to_child("child", (account, ManifestAddress::Static(vault)))
        })
        .sign(&sk1)
        .notarize(&ledger.default_notary())
        .build();

    let receipt = ledger.execute_notarized_transaction(transaction);
    receipt.expect_commit_success();
}

#[test]
fn can_not_pass_address_reservation() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, sk1, account1) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("threading"));

    let transaction = ledger
        .v2_transaction_builder()
        .add_signed_child(
            "child",
            ledger
                .v2_partial_transaction_builder()
                .manifest_builder(|builder| builder.yield_to_parent(()))
                .build(),
        )
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
        .sign(&sk1)
        .notarize(&ledger.default_notary())
        .build();

    let receipt = ledger.execute_notarized_transaction(transaction);
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::SystemError(SystemError::NotAnObject))
    });
}

#[test]
fn can_pass_named_address() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, sk1, account1) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("threading"));

    let transaction = ledger
        .v2_transaction_builder()
        .add_signed_child(
            "child",
            ledger
                .v2_partial_transaction_builder()
                .manifest_builder(|builder| builder.yield_to_parent(()))
                .build(),
        )
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
        .sign(&sk1)
        .notarize(&ledger.default_notary())
        .build();

    let receipt = ledger.execute_notarized_transaction(transaction);
    receipt.expect_commit_success();
}

#[test]
fn can_not_pass_proof_between_threads() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, sk1, account1) = ledger.new_allocated_account();

    // First, try constructing an actual transaction...
    let transaction = ledger
        .v2_transaction_builder()
        .add_signed_child(
            "child",
            ledger
                .v2_partial_transaction_builder()
                .manifest_builder(|builder| builder.yield_to_parent(()))
                .build(),
        )
        .manifest_builder(|builder| {
            builder
                .lock_fee(account1, 3)
                .create_proof_from_account_of_amount(account1, XRD, 10)
                .create_proof_from_auth_zone_of_amount(XRD, 10, "proof")
                .yield_to_child_with_name_lookup("child", |lookup| (lookup.proof("proof"),))
        })
        .sign(&sk1)
        .notarize(&ledger.default_notary())
        .build_no_validate();

    // Which fails with a validation error
    assert_matches!(
        transaction
            .transaction
            .prepare_and_validate(ledger.transaction_validator()),
        Err(TransactionValidationError::IntentValidationError(
            _,
            IntentValidationError::ManifestValidationError(
                ManifestValidationError::ProofCannotBePassedToAnotherIntent
            )
        ))
    );

    // Now we create a test transaction - which avoids validation, and lets us directly probe the engine...
    // And we observe that this is also prevented at the engine layer (defense in depth!)
    let mut test_builder = TestTransactionV2Builder::new(ledger.next_transaction_nonce());
    let subintent_manifest = SubintentManifestV2::builder().yield_to_parent(()).build();
    let subintent_hash = test_builder.add_subintent(subintent_manifest, []);
    let transaction_manifest = TransactionManifestV2::builder()
        .use_child("child", subintent_hash)
        .lock_fee(account1, 3)
        .create_proof_from_account_of_amount(account1, XRD, 10)
        .create_proof_from_auth_zone_of_amount(XRD, 10, "proof")
        .yield_to_child_with_name_lookup("child", |lookup| (lookup.proof("proof"),))
        .build_no_validate();
    let test_transaction = test_builder
        .finish_with_root_intent(transaction_manifest, [sk1.public_key().signature_proof()]);
    let receipt = ledger.execute_test_transaction(test_transaction);

    assert_matches!(
        receipt.expect_failure(),
        RuntimeError::SystemError(SystemError::IntentError(IntentError::CannotYieldProof))
    );
}
