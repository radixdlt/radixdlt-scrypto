use crate::prelude::*;

#[test]
fn subintent_should_not_be_able_to_use_proofs_from_transaction_intent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .withdraw_from_account(account, XRD, dec!(10))
            .deposit_entire_worktop(account)
            .yield_to_parent(())
            .build(),
        [],
    );

    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_standard_test_fee(account)
            .yield_to_child("child", ())
            .build(),
        [public_key.signature_proof()],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(..))
        )
    });
}

#[test]
fn subintent_should_not_be_able_to_use_proofs_from_other_subintents() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let assert_access_rule_component_address = {
        let package_address = ledger.publish_package_simple(PackageLoader::get("role_assignment"));

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package_address, "AssertAccessRule", "new", manifest_args!())
            .build();

        let receipt = ledger.execute_manifest(manifest, []);
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    // We will create a complex transaction with lots of intents.
    // Naming convention: subintent XXX will have children XXXa, XXXb, XXXc, etc.
    // Each intent will be signed by its own key.

    let keys = indexmap! {
        "aaa" => ledger.new_allocated_account().0,
        "aaba" => ledger.new_allocated_account().0,
        "aab" => ledger.new_allocated_account().0,
        "aac" => ledger.new_allocated_account().0,
        "aa" => ledger.new_allocated_account().0,
        "ab" => ledger.new_allocated_account().0,
        "a" => ledger.new_allocated_account().0,
        "ba" => ledger.new_allocated_account().0,
        "b" => ledger.new_allocated_account().0,
        "c" => ledger.new_allocated_account().0,
        "ROOT" => ledger.new_allocated_account().0,
    };

    // We will set all their manifests to be simple/trivial, except the AAB manifest
    // which will be set to call "assert access rule" with each of the keys.
    // The transaction should only pass if the AAB key is used.
    for (key_name_to_assert, key_to_assert) in keys.iter() {
        let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

        let aaa = builder.add_simple_subintent([], [keys["aaa"].signature_proof()]);
        let aaba = builder.add_simple_subintent([], [keys["aaba"].signature_proof()]);
        let aab = builder.add_tweaked_simple_subintent(
            [aaba],
            [keys["aab"].signature_proof()],
            |builder| {
                builder.call_method(
                    assert_access_rule_component_address,
                    "assert_access_rule",
                    (rule!(require(key_to_assert.signature_proof())),),
                )
            },
        );
        let aac = builder.add_simple_subintent([], [keys["aac"].signature_proof()]);
        let aa = builder.add_simple_subintent([aaa, aab, aac], [keys["aa"].signature_proof()]);
        let ab = builder.add_simple_subintent([], [keys["ab"].signature_proof()]);
        let a = builder.add_simple_subintent([aa, ab], [keys["a"].signature_proof()]);
        let ba = builder.add_simple_subintent([], [keys["ba"].signature_proof()]);
        let b = builder.add_simple_subintent([ba], [keys["b"].signature_proof()]);
        let c = builder.add_simple_subintent([], [keys["c"].signature_proof()]);
        let transaction =
            builder.finish_with_simple_root_intent([a, b, c], [keys["ROOT"].signature_proof()]);

        let receipt = ledger.execute_test_transaction(transaction);

        // ASSERT
        if *key_name_to_assert == "aab" {
            receipt.expect_commit_success();
        } else {
            receipt.expect_specific_failure(|e| {
                matches!(
                    e,
                    RuntimeError::SystemError(SystemError::AssertAccessRuleFailed)
                )
            });
        }
    }
}

#[test]
fn should_be_able_to_use_separate_auth_in_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let (public_key2, _, account2) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .withdraw_from_account(account2, XRD, dec!(10))
            .deposit_entire_worktop(account2)
            .yield_to_parent(())
            .build(),
        [public_key2.signature_proof()],
    );

    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_standard_test_fee(account)
            .yield_to_child("child", ())
            .build(),
        [public_key.signature_proof()],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn subintent_processor_uses_transaction_processor_global_caller() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let assert_access_rule_component_address = {
        let package_address = ledger.publish_package_simple(PackageLoader::get("role_assignment"));

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package_address, "AssertAccessRule", "new", manifest_args!())
            .build();

        let receipt = ledger.execute_manifest(manifest, []);
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let transaction_processor = BlueprintId {
        package_address: TRANSACTION_PROCESSOR_PACKAGE,
        blueprint_name: TRANSACTION_PROCESSOR_BLUEPRINT.to_string(),
    };

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .call_method(
                assert_access_rule_component_address,
                "assert_access_rule",
                (rule!(require(global_caller(transaction_processor))),),
            )
            .yield_to_parent(())
            .build(),
        [],
    );

    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_fee_from_faucet()
            .yield_to_child("child", ())
            .build(),
        [],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_commit_success();
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct ManifestTransactionProcessorRunInput {
    pub manifest_encoded_instructions: Vec<u8>,
    pub global_address_reservations: Vec<()>,
    pub references: Vec<()>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
}

#[test]
fn should_not_be_able_to_call_tx_processor_in_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut builder = TestTransaction::new_v2_builder(ledger.next_transaction_nonce());

    let instructions: Vec<InstructionV1> = Vec::new();
    let manifest_encoded_instructions = manifest_encode(&instructions).unwrap();

    let child = builder.add_subintent(
        ManifestBuilder::new_subintent_v2()
            .call_function(
                TRANSACTION_PROCESSOR_PACKAGE,
                TRANSACTION_PROCESSOR_BLUEPRINT,
                TRANSACTION_PROCESSOR_RUN_IDENT,
                ManifestTransactionProcessorRunInput {
                    manifest_encoded_instructions,
                    global_address_reservations: vec![],
                    references: vec![],
                    blobs: index_map_new(),
                },
            )
            .yield_to_parent(())
            .build(),
        [],
    );

    let transaction = builder.finish_with_root_intent(
        ManifestBuilder::new_v2()
            .use_child("child", child)
            .lock_standard_test_fee(account)
            .yield_to_child("child", ())
            .build(),
        [public_key.signature_proof()],
    );

    let receipt = ledger.execute_test_transaction(transaction);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}
