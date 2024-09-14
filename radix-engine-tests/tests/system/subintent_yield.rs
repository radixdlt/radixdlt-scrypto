use radix_common::prelude::ManifestArgs;
use radix_common::prelude::{FromPublicKey, NonFungibleGlobalId, XRD};
use radix_common::{manifest_args, to_manifest_value_and_unwrap};
use radix_engine::errors::{RuntimeError, SystemError, YieldError};
use radix_engine::transaction::ExecutionConfig;
use radix_engine_interface::macros::dec;
use radix_rust::btreeset;
use radix_transactions::builder::{ManifestBuilder, ResolvableArguments};
use radix_transactions::manifest::YieldToChild;
use radix_transactions::model::{ManifestNamedIntentIndex, TestTransaction};
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
                    builder
                        .add_instruction_advanced(YieldToChild {
                            child_index: ManifestNamedIntentIndex(0),
                            args: to_manifest_value_and_unwrap!(&lookup.bucket("xrd")),
                        })
                        .0
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
                .add_instruction_advanced(YieldToChild {
                    child_index: ManifestNamedIntentIndex(0),
                    args: ().resolve(),
                })
                .0
                .assert_worktop_contains(XRD, dec!(10))
                .deposit_entire_worktop(account)
                .add_instruction_advanced(YieldToChild {
                    child_index: ManifestNamedIntentIndex(0),
                    args: ().resolve(),
                })
                .0
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
                    builder
                        .add_instruction_advanced(YieldToChild {
                            child_index: ManifestNamedIntentIndex(0),
                            args: to_manifest_value_and_unwrap!(&lookup.bucket("xrd")),
                        })
                        .0
                })
                .assert_worktop_contains(XRD, dec!(10))
                .deposit_entire_worktop(account)
                .add_instruction_advanced(YieldToChild {
                    child_index: ManifestNamedIntentIndex(0),
                    args: to_manifest_value_and_unwrap!(&()),
                })
                .0
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

#[test]
fn cannot_send_proof_to_child_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let intents = vec![
        {
            let manifest = ManifestBuilder::new_v2()
                .lock_standard_test_fee(account)
                .create_proof_from_account_of_amount(account, XRD, dec!(10))
                .create_proof_from_auth_zone_of_amount(XRD, dec!(10), "proof")
                .with_name_lookup(|builder, lookup| {
                    builder
                        .add_instruction_advanced(YieldToChild {
                            child_index: ManifestNamedIntentIndex(0),
                            args: to_manifest_value_and_unwrap!(&lookup.proof("proof")),
                        })
                        .0
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
            let manifest = ManifestBuilder::new_v2().build();

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
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::YieldError(YieldError::CannotYieldProof))
        )
    });
}

#[test]
fn cannot_send_proof_to_parent_subintent() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let intents = vec![
        {
            let manifest = ManifestBuilder::new_v2()
                .lock_standard_test_fee(account)
                .add_instruction_advanced(YieldToChild {
                    child_index: ManifestNamedIntentIndex(0),
                    args: ().resolve(),
                })
                .0
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
                .create_proof_from_account_of_amount(account, XRD, dec!(10))
                .create_proof_from_auth_zone_of_amount(XRD, dec!(10), "proof")
                .with_name_lookup(|builder, lookup| {
                    builder.yield_to_parent(manifest_args!(lookup.proof("proof")))
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
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::YieldError(YieldError::CannotYieldProof))
        )
    });
}
