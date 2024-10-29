use radix_common::prelude::*;
use radix_engine::blueprints::resource::WorktopError;
use radix_engine::errors::RuntimeError;
use radix_engine::errors::{ApplicationError, CallFrameError, KernelError};
use radix_engine::kernel::call_frame::OpenSubstateError;
use radix_engine::transaction::{FeeLocks, TransactionReceipt};
use radix_engine_interface::types::FromPublicKey;
use radix_engine_tests::common::*;
use radix_transactions::prelude::PreviewFlags;
use scrypto_test::prelude::*;

fn run_manifest<F>(f: F) -> TransactionReceipt
where
    F: FnOnce(ComponentAddress) -> TransactionManifestV1,
{
    let (mut ledger, component_address) = setup_ledger();

    // Run the provided manifest
    let manifest = f(component_address);
    ledger.execute_manifest(manifest, vec![])
}

fn setup_ledger() -> (DefaultLedgerSimulator, ComponentAddress) {
    // Basic setup
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Publish package and instantiate component
    let package_address = ledger.publish_package_simple(PackageLoader::get("fee"));
    let receipt1 = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .withdraw_from_account(account, XRD, 1000)
            .take_all_from_worktop(XRD, "bucket")
            .with_name_lookup(|builder, lookup| {
                builder.call_function(
                    package_address,
                    "Fee",
                    "new",
                    manifest_args!(lookup.bucket("bucket")),
                )
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let commit_result = receipt1.expect_commit(true);
    let component_address = commit_result.new_component_addresses()[0];

    (ledger, component_address)
}

#[test]
fn should_be_aborted_when_loan_repaid() {
    let (mut ledger, component_address) = setup_ledger();

    let manifest = ManifestBuilder::new()
        // First, lock the fee so that the loan will be repaid
        .lock_fee_from_faucet()
        // Now spin-loop to wait for the fee loan to burn through
        .call_method(component_address, "spin_loop", manifest_args!())
        .build();

    let start = std::time::Instant::now();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    let duration = start.elapsed();
    println!("Time elapsed is: {:?}", duration);
    println!(
        "{}",
        receipt.display(&AddressBech32Encoder::for_simulator())
    );
    receipt.expect_commit_failure();
}

#[test]
fn should_succeed_when_fee_is_paid() {
    let receipt =
        run_manifest(|_component_address| ManifestBuilder::new().lock_fee_from_faucet().build());

    receipt.expect_commit_success();
}

#[test]
fn should_be_rejected_when_no_fee_is_paid() {
    let receipt = run_manifest(|_| ManifestBuilder::new().build());

    receipt.expect_rejection();
}

#[test]
fn should_be_rejected_when_insufficient_balance() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new()
            .call_method(
                component_address,
                "lock_fee_with_empty_vault",
                manifest_args!(Decimal::from(10)),
            )
            .build()
    });

    receipt.expect_rejection();
}

#[test]
fn should_be_rejected_when_non_xrd() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new()
            .call_method(
                component_address,
                "lock_fee_with_doge",
                manifest_args!(Decimal::from(10)),
            )
            .build()
    });

    receipt.expect_rejection();
}

#[test]
fn should_be_rejected_when_system_loan_is_not_fully_repaid() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new()
            .call_method(
                component_address,
                "lock_fee",
                manifest_args!(Decimal::from_str("0.001").unwrap()), // = 1000 cost units
            )
            .build()
    });

    receipt.expect_rejection();
}

#[test]
fn should_be_rejected_when_lock_fee_with_temp_vault() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                component_address,
                "lock_fee_with_temp_vault",
                manifest_args!(Decimal::from(10)),
            )
            .build()
    });

    receipt.expect_specific_failure(|e| match e {
        RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::OpenSubstateError(OpenSubstateError::LockUnmodifiedBaseOnHeapNode),
        )) => true,
        _ => false,
    });
}

#[test]
fn should_be_success_when_query_vault_and_lock_fee() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                component_address,
                "query_vault_and_lock_fee",
                manifest_args!(Decimal::from(500)),
            )
            .build()
    });

    receipt.expect_commit_success();
}

#[test]
fn should_be_rejected_when_mutate_vault_and_lock_fee() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                component_address,
                "update_vault_and_lock_fee",
                manifest_args!(Decimal::from(10)),
            )
            .build()
    });

    receipt.expect_specific_failure(|e| match e {
        RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::OpenSubstateError(
                OpenSubstateError::LockUnmodifiedBaseOnOnUpdatedSubstate(..),
            ),
        )) => true,
        _ => false,
    });
}

#[test]
fn should_succeed_when_lock_fee_and_query_vault() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                component_address,
                "lock_fee_and_query_vault",
                manifest_args!(Decimal::from(500)),
            )
            .build()
    });

    receipt.expect_commit_success();
}

#[test]
fn test_fee_accounting_success() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account1) = ledger.new_allocated_account();
    let (_, _, account2) = ledger.new_allocated_account();
    let account1_balance = ledger
        .get_component_resources(account1)
        .get(&XRD)
        .cloned()
        .unwrap();
    let account2_balance = ledger
        .get_component_resources(account2)
        .get(&XRD)
        .cloned()
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, 500)
        .withdraw_from_account(account1, XRD, 66)
        .try_deposit_entire_worktop_or_abort(account2, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit(true);
    let account1_new_balance = ledger
        .get_component_resources(account1)
        .get(&XRD)
        .cloned()
        .unwrap();
    let account2_new_balance = ledger
        .get_component_resources(account2)
        .get(&XRD)
        .cloned()
        .unwrap();
    assert_eq!(
        account1_new_balance,
        account1_balance
            .checked_sub(Decimal::from(66))
            .unwrap()
            .checked_sub(receipt.fee_summary.total_cost())
            .unwrap()
    );
    assert_eq!(
        account2_new_balance,
        account2_balance.checked_add(66).unwrap()
    );
}

#[test]
fn test_fee_accounting_failure() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account1) = ledger.new_allocated_account();
    let (_, _, account2) = ledger.new_allocated_account();
    let account1_balance = ledger
        .get_component_resources(account1)
        .get(&XRD)
        .cloned()
        .unwrap();
    let account2_balance = ledger
        .get_component_resources(account2)
        .get(&XRD)
        .cloned()
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, 500)
        .withdraw_from_account(account1, XRD, 66)
        .try_deposit_entire_worktop_or_abort(account2, None)
        .assert_worktop_contains(XRD, 1)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::WorktopError(
                WorktopError::AssertionFailed(ResourceConstraintsError::ResourceConstraintFailed {
                    resource_address: XRD,
                    error: ResourceConstraintError::ExpectedAtLeastAmount {
                        expected_at_least_amount: Decimal::ONE,
                        actual_amount: Decimal::ZERO,
                    },
                })
            ))
        )
    });
    receipt.expect_commit(false);
    let account1_new_balance = ledger
        .get_component_resources(account1)
        .get(&XRD)
        .cloned()
        .unwrap();
    let account2_new_balance = ledger
        .get_component_resources(account2)
        .get(&XRD)
        .cloned()
        .unwrap();
    assert_eq!(
        account1_new_balance,
        account1_balance
            .checked_sub(receipt.fee_summary.total_cost())
            .unwrap()
    );
    assert_eq!(account2_new_balance, account2_balance);
}

#[test]
fn test_fee_accounting_rejection() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account1) = ledger.new_allocated_account();
    let account1_balance = ledger
        .get_component_resources(account1)
        .get(&XRD)
        .cloned()
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, Decimal::from_str("0.000000000000000001").unwrap())
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_rejection();
    let account1_new_balance = ledger
        .get_component_resources(account1)
        .get(&XRD)
        .cloned()
        .unwrap();
    assert_eq!(account1_new_balance, account1_balance);
}

#[test]
fn test_contingent_fee_accounting_success() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key1, _, account1) = ledger.new_allocated_account();
    let (public_key2, _, account2) = ledger.new_allocated_account();
    let account1_balance = ledger
        .get_component_resources(account1)
        .get(&XRD)
        .cloned()
        .unwrap();
    let account2_balance = ledger
        .get_component_resources(account2)
        .get(&XRD)
        .cloned()
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, 500)
        .lock_contingent_fee(account2, dec!("0.001"))
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![
            NonFungibleGlobalId::from_public_key(&public_key1),
            NonFungibleGlobalId::from_public_key(&public_key2),
        ],
    );

    // Assert
    receipt.expect_commit(true);
    let account1_new_balance = ledger
        .get_component_resources(account1)
        .get(&XRD)
        .cloned()
        .unwrap();
    let account2_new_balance = ledger
        .get_component_resources(account2)
        .get(&XRD)
        .cloned()
        .unwrap();
    let contingent_fee = dec!("0.001");
    assert_eq!(
        account1_new_balance,
        account1_balance
            .checked_sub(
                receipt
                    .effective_execution_cost_unit_price()
                    .checked_mul(receipt.fee_summary.total_execution_cost_units_consumed)
                    .unwrap()
            )
            .unwrap()
            .checked_sub(
                receipt
                    .effective_finalization_cost_unit_price()
                    .checked_mul(receipt.fee_summary.total_finalization_cost_units_consumed)
                    .unwrap()
            )
            .unwrap()
            .checked_sub(receipt.fee_summary.total_storage_cost_in_xrd)
            .unwrap()
            .checked_add(contingent_fee)
            .unwrap()
    );
    assert_eq!(
        account2_new_balance,
        account2_balance.checked_sub(contingent_fee).unwrap()
    );
}

#[test]
fn test_contingent_fee_accounting_failure() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key1, _, account1) = ledger.new_allocated_account();
    let (public_key2, _, account2) = ledger.new_allocated_account();
    let account1_balance = ledger
        .get_component_resources(account1)
        .get(&XRD)
        .cloned()
        .unwrap();
    let account2_balance = ledger
        .get_component_resources(account2)
        .get(&XRD)
        .cloned()
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, 500)
        .lock_contingent_fee(account2, dec!("0.001"))
        .assert_worktop_contains(XRD, 1)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![
            NonFungibleGlobalId::from_public_key(&public_key1),
            NonFungibleGlobalId::from_public_key(&public_key2),
        ],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::WorktopError(
                WorktopError::AssertionFailed(ResourceConstraintsError::ResourceConstraintFailed {
                    resource_address: XRD,
                    error: ResourceConstraintError::ExpectedAtLeastAmount {
                        expected_at_least_amount: Decimal::ONE,
                        actual_amount: Decimal::ZERO,
                    },
                })
            ))
        )
    });
    receipt.expect_commit(false);
    let account1_new_balance = ledger
        .get_component_resources(account1)
        .get(&XRD)
        .cloned()
        .unwrap();
    let account2_new_balance = ledger
        .get_component_resources(account2)
        .get(&XRD)
        .cloned()
        .unwrap();
    assert_eq!(
        account1_new_balance,
        account1_balance
            .checked_sub(receipt.fee_summary.total_cost())
            .unwrap()
    );
    assert_eq!(account2_new_balance, account2_balance);
}

#[test]
fn locked_fees_are_correct_in_execution_trace() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_account(false);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, dec!("104.676"))
        .build();
    let receipt = ledger.preview_manifest(
        manifest,
        vec![public_key.into()],
        0,
        PreviewFlags::default(),
    );

    // Assert
    let commit = receipt.expect_commit_success();
    assert_eq!(
        commit.execution_trace.as_ref().unwrap().fee_locks,
        FeeLocks {
            lock: dec!("104.676"),
            contingent_lock: Decimal::ZERO
        }
    )
}

#[test]
fn multiple_locked_fees_are_correct_in_execution_trace() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key1, _, account1) = ledger.new_account(false);
    let (public_key2, _, account2) = ledger.new_account(false);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, dec!("104.676"))
        .lock_fee(account2, dec!("102.180"))
        .build();
    let receipt = ledger.preview_manifest(
        manifest,
        vec![public_key1.into(), public_key2.into()],
        0,
        PreviewFlags::default(),
    );

    // Assert
    let commit = receipt.expect_commit_success();
    assert_eq!(
        commit.execution_trace.as_ref().unwrap().fee_locks,
        FeeLocks {
            lock: dec!("206.856"),
            contingent_lock: Decimal::ZERO
        }
    )
}

#[test]
fn regular_and_contingent_fee_locks_are_correct_in_execution_trace() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key1, _, account1) = ledger.new_account(false);
    let (public_key2, _, account2) = ledger.new_account(false);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, dec!("104.676"))
        .lock_contingent_fee(account2, dec!("102.180"))
        .build();
    let receipt = ledger.preview_manifest(
        manifest,
        vec![public_key1.into(), public_key2.into()],
        0,
        PreviewFlags::default(),
    );

    // Assert
    let commit = receipt.expect_commit_success();
    assert_eq!(
        commit.execution_trace.as_ref().unwrap().fee_locks,
        FeeLocks {
            lock: dec!("104.676"),
            contingent_lock: dec!("102.180")
        }
    )
}
