use radix_common::manifest_args;
use radix_common::prelude::ManifestArgs;
use radix_common::prelude::{FromPublicKey, NonFungibleGlobalId, XRD};
use radix_engine::transaction::ExecutionConfig;
use radix_engine_interface::macros::dec;
use radix_rust::btreeset;
use radix_transactions::builder::ManifestBuilder;
use radix_transactions::model::{ManifestIntent, TestTransaction};
use scrypto_test::ledger_simulator::LedgerSimulatorBuilder;

#[test]
fn can_send_resources_to_child_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let intents = vec![
        {
            let manifest = ManifestBuilder::new_v2()
                .lock_standard_test_fee(account)
                .withdraw_from_account(account, XRD, dec!(10))
                .take_all_from_worktop(XRD, "xrd")
                .with_name_lookup(|builder, lookup| {
                    builder.yield_to_child(ManifestIntent(0), manifest_args!(lookup.bucket("xrd")))
                })
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![1],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
        },
        {
            let manifest = ManifestBuilder::new_v2()
                .assert_worktop_contains(XRD, dec!(10))
                .deposit_entire_worktop(account)
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
        },
    ];

    let receipt = ledger.execute_transaction(
        TestTransaction::new_v2_from_nonce(intents)
            .prepare()
            .expect("expected transaction to be preparable")
            .get_executable(),
        ExecutionConfig::for_test_transaction(),
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_send_resources_to_parent_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let intents = vec![
        {
            let manifest = ManifestBuilder::new_v2()
                .lock_standard_test_fee(account)
                .yield_to_child(ManifestIntent(0), ())
                .assert_worktop_contains(XRD, dec!(10))
                .deposit_entire_worktop(account)
                .yield_to_child(ManifestIntent(0), ())
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![1],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
        },
        {
            let manifest = ManifestBuilder::new_v2()
                .withdraw_from_account(account, XRD, dec!(10))
                .take_all_from_worktop(XRD, "xrd")
                .with_name_lookup(|builder, lookup| {
                    builder.yield_to_parent(manifest_args!(lookup.bucket("xrd")))
                })
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
        },
    ];

    let receipt = ledger.execute_transaction(
        TestTransaction::new_v2_from_nonce(intents)
            .prepare()
            .expect("expected transaction to be preparable")
            .get_executable(),
        ExecutionConfig::for_test_transaction(),
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_send_and_receive_resources_as_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let intents = vec![
        {
            let manifest = ManifestBuilder::new_v2()
                .lock_standard_test_fee(account)
                .withdraw_from_account(account, XRD, dec!(10))
                .take_all_from_worktop(XRD, "xrd")
                .with_name_lookup(|builder, lookup| {
                    builder.yield_to_child(ManifestIntent(0), manifest_args!(lookup.bucket("xrd")))
                })
                .assert_worktop_contains(XRD, dec!(10))
                .deposit_entire_worktop(account)
                .yield_to_child(ManifestIntent(0), ())
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![1],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
        },
        {
            let manifest = ManifestBuilder::new_v2()
                .take_all_from_worktop(XRD, "xrd")
                .with_name_lookup(|builder, lookup| {
                    builder.yield_to_parent(manifest_args!(lookup.bucket("xrd")))
                })
                .build();

            (
                manifest,
                ledger.next_transaction_nonce(),
                vec![],
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
        },
    ];

    let receipt = ledger.execute_transaction(
        TestTransaction::new_v2_from_nonce(intents)
            .prepare()
            .expect("expected transaction to be preparable")
            .get_executable(),
        ExecutionConfig::for_test_transaction(),
    );

    // Assert
    receipt.expect_commit_success();
}
