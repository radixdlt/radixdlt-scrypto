use radix_engine::engine::{ApplicationError, KernelError, TrackError};
use radix_engine::engine::{RejectionError, RuntimeError};
use radix_engine::model::WorktopError;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::*;

fn run_manifest<F>(f: F) -> TransactionReceipt
where
    F: FnOnce(ComponentAddress) -> TransactionManifest,
{
    let (mut test_runner, component_address) = setup_test_runner();

    // Run the provided manifest
    let manifest = f(component_address);
    test_runner.execute_manifest(manifest, vec![])
}

fn setup_test_runner() -> (TestRunner, ComponentAddress) {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package and instantiate component
    let package_address = test_runner.compile_and_publish("./tests/blueprints/fee");
    let receipt1 = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 10u32.into())
            .withdraw_from_account_by_amount(account, 10u32.into(), RADIX_TOKEN)
            .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.call_function(package_address, "Fee", "new", args!(bucket_id));
                builder
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let component_address = receipt1
        .expect_commit()
        .entity_changes
        .new_component_addresses[0];

    (test_runner, component_address)
}

#[test]
fn should_be_aborted_when_loan_repaid() {
    let (mut test_runner, component_address) = setup_test_runner();

    let manifest = ManifestBuilder::new()
        // First, lock the fee so that the loan will be repaid
        .call_method(component_address, "lock_fee", args!(Decimal::from(10)))
        // Now spin-loop to wait for the fee loan to burn through
        .call_method(component_address, "spin_loop", args!())
        .build();

    let start = std::time::Instant::now();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let duration = start.elapsed();
    println!("Time elapsed is: {:?}", duration);
    println!("{:?}", receipt);
    receipt.expect_commit_failure();
}

#[test]
fn should_succeed_when_fee_is_paid() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new()
            .call_method(component_address, "lock_fee", args!(Decimal::from(10)))
            .build()
    });

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
                args!(Decimal::from(10)),
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
                args!(Decimal::from(10)),
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
                args!(Decimal::from_str("0.001").unwrap()), // = 1000 cost units
            )
            .build()
    });

    receipt.expect_rejection();
}

#[test]
fn should_be_rejected_when_lock_fee_with_temp_vault() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new()
            .call_method(
                component_address,
                "lock_fee_with_temp_vault",
                args!(Decimal::from(10)),
            )
            .build()
    });
    receipt.expect_specific_rejection(|e| {
        matches!(
            e,
            RejectionError::ErrorBeforeFeeLoanRepaid(RuntimeError::KernelError(
                KernelError::TrackError(TrackError::LockUnmodifiedBaseOnNewSubstate(..))
            ))
        )
    });
}

#[test]
fn should_be_success_when_query_vault_and_lock_fee() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new()
            .call_method(
                component_address,
                "query_vault_and_lock_fee",
                args!(Decimal::from(10)),
            )
            .build()
    });

    receipt.expect_commit_success();
}

#[test]
fn should_be_rejected_when_mutate_vault_and_lock_fee() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new()
            .call_method(
                component_address,
                "update_vault_and_lock_fee",
                args!(Decimal::from(10)),
            )
            .build()
    });

    receipt.expect_specific_rejection(|e| {
        matches!(
            e,
            RejectionError::ErrorBeforeFeeLoanRepaid(RuntimeError::KernelError(
                KernelError::TrackError(TrackError::LockUnmodifiedBaseOnOnUpdatedSubstate(..))
            ))
        )
    });
}

#[test]
fn should_succeed_when_lock_fee_and_query_vault() {
    let receipt = run_manifest(|component_address| {
        ManifestBuilder::new()
            .call_method(
                component_address,
                "lock_fee_and_query_vault",
                args!(Decimal::from(10)),
            )
            .build()
    });

    receipt.expect_commit_success();
}

#[test]
fn test_fee_accounting_success() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account1) = test_runner.new_allocated_account();
    let (_, _, account2) = test_runner.new_allocated_account();
    let account1_balance = test_runner
        .get_component_resources(account1)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();
    let account2_balance = test_runner
        .get_component_resources(account2)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, 10.into())
        .withdraw_from_account_by_amount(account1, 66.into(), RADIX_TOKEN)
        .call_method(
            account2,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
    let account1_new_balance = test_runner
        .get_component_resources(account1)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();
    let account2_new_balance = test_runner
        .get_component_resources(account2)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();
    let summary = &receipt.execution.fee_summary;
    assert_eq!(
        account1_new_balance,
        account1_balance
            - 66
            - (summary.cost_unit_price + summary.cost_unit_price * summary.tip_percentage / 100)
                * summary.cost_unit_consumed
    );
    assert_eq!(account2_new_balance, account2_balance + 66);
}

#[test]
fn test_fee_accounting_failure() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account1) = test_runner.new_allocated_account();
    let (_, _, account2) = test_runner.new_allocated_account();
    let account1_balance = test_runner
        .get_component_resources(account1)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();
    let account2_balance = test_runner
        .get_component_resources(account2)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, 10.into())
        .withdraw_from_account_by_amount(account1, 66.into(), RADIX_TOKEN)
        .call_method(
            account2,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
        .assert_worktop_contains_by_amount(1.into(), RADIX_TOKEN)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::WorktopError(
                WorktopError::AssertionFailed
            ))
        )
    });
    let account1_new_balance = test_runner
        .get_component_resources(account1)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();
    let account2_new_balance = test_runner
        .get_component_resources(account2)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();
    let summary = &receipt.execution.fee_summary;
    assert_eq!(
        account1_new_balance,
        account1_balance
            - (summary.cost_unit_price + summary.cost_unit_price * summary.tip_percentage / 100)
                * summary.cost_unit_consumed
    );
    assert_eq!(account2_new_balance, account2_balance);
}

#[test]
fn test_fee_accounting_rejection() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account1) = test_runner.new_allocated_account();
    let account1_balance = test_runner
        .get_component_resources(account1)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, Decimal::from_str("0.000000000000000001").unwrap())
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_rejection();
    let account1_new_balance = test_runner
        .get_component_resources(account1)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();
    assert_eq!(account1_new_balance, account1_balance);
}

#[test]
fn test_contingent_fee_accounting_success() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key1, _, account1) = test_runner.new_allocated_account();
    let (public_key2, _, account2) = test_runner.new_allocated_account();
    let account1_balance = test_runner
        .get_component_resources(account1)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();
    let account2_balance = test_runner
        .get_component_resources(account2)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, dec!("10"))
        .lock_contingent_fee(account2, dec!("0.001"))
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![
            NonFungibleGlobalId::from_public_key(&public_key1),
            NonFungibleGlobalId::from_public_key(&public_key2),
        ],
    );

    // Assert
    receipt.expect_commit_success();
    let account1_new_balance = test_runner
        .get_component_resources(account1)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();
    let account2_new_balance = test_runner
        .get_component_resources(account2)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();
    let summary = &receipt.execution.fee_summary;
    let effective_price =
        summary.cost_unit_price + summary.cost_unit_price * summary.tip_percentage / 100;
    let contingent_fee = dec!("0.001");
    assert_eq!(
        account1_new_balance,
        account1_balance - effective_price * summary.cost_unit_consumed + contingent_fee
    );
    assert_eq!(account2_new_balance, account2_balance - contingent_fee);
}

#[test]
fn test_contingent_fee_accounting_failure() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key1, _, account1) = test_runner.new_allocated_account();
    let (public_key2, _, account2) = test_runner.new_allocated_account();
    let account1_balance = test_runner
        .get_component_resources(account1)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();
    let account2_balance = test_runner
        .get_component_resources(account2)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, dec!("10"))
        .lock_contingent_fee(account2, dec!("0.001"))
        .assert_worktop_contains_by_amount(1.into(), RADIX_TOKEN)
        .build();
    let receipt = test_runner.execute_manifest(
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
                WorktopError::AssertionFailed
            ))
        )
    });
    let account1_new_balance = test_runner
        .get_component_resources(account1)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();
    let account2_new_balance = test_runner
        .get_component_resources(account2)
        .get(&RADIX_TOKEN)
        .cloned()
        .unwrap();
    let summary = &receipt.execution.fee_summary;
    let effective_price =
        summary.cost_unit_price + summary.cost_unit_price * summary.tip_percentage / 100;
    assert_eq!(
        account1_new_balance,
        account1_balance - effective_price * summary.cost_unit_consumed
    );
    assert_eq!(account2_new_balance, account2_balance);
}
