use radix_engine::blueprints::resource::{
    FungibleResourceManagerError, NonFungibleResourceManagerError, ProofError, VaultError,
};
use radix_engine::errors::SystemError;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::{
    blueprints::resource::BucketError,
    errors::{ApplicationError, CallFrameError, KernelError, RuntimeError},
    kernel::call_frame::DropNodeError,
};
use radix_engine_interface::prelude::*;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

fn test_bucket_internal(method_name: &str, args: ManifestValue, expect_success: bool) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function_raw(package_address, "BucketTest", method_name, args)
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    if expect_success {
        receipt.expect_commit_success();
    } else {
        receipt.expect_commit_failure();
    }
}

fn test_bucket_internal2<F: FnOnce(TransactionReceipt)>(
    method_name: &str,
    args: ManifestValue,
    on_receipt: F,
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function_raw(package_address, "BucketTest", method_name, args)
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    on_receipt(receipt);
}

#[test]
fn test_drop_bucket() {
    test_bucket_internal("drop_bucket", manifest_args!().into(), false);
}

#[test]
fn test_fungible_bucket_drop_empty() {
    test_bucket_internal2("drop_fungible_empty", manifest_args!(0u32).into(), |r| {
        r.expect_commit_success();
    });
}

#[test]
fn test_fungible_bucket_drop_not_empty() {
    test_bucket_internal2("drop_fungible_empty", manifest_args!(1u32).into(), |r| {
        r.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::FungibleResourceManagerError(
                    FungibleResourceManagerError::DropNonEmptyBucket
                ))
            )
        });
    });
}

#[test]
fn test_non_fungible_bucket_drop_empty() {
    test_bucket_internal2(
        "drop_non_fungible_empty",
        manifest_args!(true).into(),
        |r| {
            r.expect_commit_success();
        },
    );
}

#[test]
fn test_non_fungible_bucket_drop_not_empty() {
    test_bucket_internal2(
        "drop_non_fungible_empty",
        manifest_args!(false).into(),
        |r| {
            r.expect_specific_failure(|e| {
                matches!(
                    e,
                    RuntimeError::ApplicationError(
                        ApplicationError::NonFungibleResourceManagerError(
                            NonFungibleResourceManagerError::DropNonEmptyBucket
                        )
                    )
                )
            });
        },
    );
}

#[test]
fn test_bucket_combine() {
    test_bucket_internal("combine", manifest_args!().into(), true);
}

#[test]
fn test_bucket_split() {
    test_bucket_internal("split", manifest_args!().into(), true);
}

#[test]
fn test_bucket_borrow() {
    test_bucket_internal("borrow", manifest_args!().into(), true);
}

#[test]
fn test_bucket_query() {
    test_bucket_internal("query", manifest_args!().into(), true);
}

#[test]
fn test_bucket_restricted_transfer() {
    test_bucket_internal("test_restricted_transfer", manifest_args!().into(), true);
}

#[test]
fn test_bucket_burn() {
    test_bucket_internal("test_burn", manifest_args!().into(), true);
}

#[test]
fn test_bucket_burn_freely() {
    test_bucket_internal("test_burn_freely", manifest_args!().into(), true);
}

#[test]
fn test_bucket_empty_fungible() {
    test_bucket_internal(
        "create_empty_bucket_fungible",
        manifest_args!().into(),
        true,
    );
}

#[test]
fn test_bucket_empty_non_fungible() {
    test_bucket_internal(
        "create_empty_bucket_non_fungible",
        manifest_args!().into(),
        true,
    );
}

#[test]
fn test_bucket_of_badges() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function(package_address, "BadgeTest", "combine", manifest_args!())
        .call_function(package_address, "BadgeTest", "split", manifest_args!())
        .call_function(package_address, "BadgeTest", "borrow", manifest_args!())
        .call_function(package_address, "BadgeTest", "query", manifest_args!())
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn test_take_with_invalid_granularity() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address = ledger.create_fungible_resource(100.into(), 2, account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .withdraw_from_account(account, resource_address, 100)
        .take_all_from_worktop(resource_address, "bucket")
        .with_name_lookup(|builder, lookup| {
            builder.call_function(
                package_address,
                "BucketTest",
                "take_from_bucket",
                manifest_args!(lookup.bucket("bucket"), dec!("1.123")),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::InvalidAmount(..),
            ))
        )
    });
}

#[test]
fn test_take_with_negative_amount() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address = ledger.create_fungible_resource(100.into(), 2, account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .withdraw_from_account(account, resource_address, 100)
        .take_all_from_worktop(resource_address, "bucket")
        .with_name_lookup(|builder, lookup| {
            let bucket = lookup.bucket("bucket");
            builder.call_function(
                package_address,
                "BucketTest",
                "take_from_bucket",
                manifest_args!(bucket, dec!("-2")),
            )
        })
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::InvalidAmount(..),
            ))
        )
    });
}

#[test]
fn create_empty_bucket() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let non_fungible_resource = ledger.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .take_all_from_worktop(XRD, "bucket1")
        .return_to_worktop("bucket1")
        .take_from_worktop(XRD, Decimal::zero(), "bucket2")
        .return_to_worktop("bucket2")
        .take_non_fungibles_from_worktop(non_fungible_resource, [], "bucket3")
        .return_to_worktop("bucket3")
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!(
        "{}",
        receipt.display(&AddressBech32Encoder::for_simulator())
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_drop_locked_fungible_bucket() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function(
            package_address,
            "BucketTest",
            "drop_locked_fungible_bucket",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}", receipt);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(CallFrameError::DropNodeError(
                DropNodeError::NodeBorrowed(..)
            )))
        )
    });
}

#[test]
fn create_proof_of_invalid_amount_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function(
            package_address,
            "BucketTest",
            "create_proof_of_amount",
            manifest_args!(dec!("1.01")),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::BucketError(
                BucketError::InvalidAmount(..)
            ))
        )
    });
}

#[test]
fn create_proof_of_zero_amount_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function(
            package_address,
            "BucketTest",
            "create_proof_of_amount",
            manifest_args!(Decimal::ZERO),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::BucketError(BucketError::ProofError(
                ProofError::EmptyProofNotAllowed
            )))
        )
    });
}

#[test]
fn create_vault_proof_of_invalid_amount_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function(
            package_address,
            "BucketTest",
            "create_vault_proof_of_amount",
            manifest_args!(dec!("1.01")),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::VaultError(
                VaultError::InvalidAmount(..)
            ))
        )
    });
}

#[test]
fn create_vault_proof_of_zero_amount_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function(
            package_address,
            "BucketTest",
            "create_vault_proof_of_amount",
            manifest_args!(Decimal::ZERO),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::ProofError(
                ProofError::EmptyProofNotAllowed
            )))
        )
    });
}

#[test]
fn test_drop_locked_non_fungible_bucket() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function(
            package_address,
            "BucketTest",
            "drop_locked_non_fungible_bucket",
            manifest_args!(),
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}", receipt);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(CallFrameError::DropNodeError(
                DropNodeError::NodeBorrowed(..)
            )))
        )
    });
}

#[test]
fn test_bucket_combine_fungible_invalid() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function(
            package_address,
            "InvalidCombine",
            "combine_fungible_invalid",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::InvalidDropAccess(..))
        )
    });
}

#[test]
fn test_bucket_combine_non_fungible_invalid() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function(
            package_address,
            "InvalidCombine",
            "combine_non_fungible_invalid",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::InvalidDropAccess(..))
        )
    });
}

#[test]
fn test_vault_combine_fungible_invalid() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function(
            package_address,
            "InvalidCombine",
            "combine_fungible_vault_invalid",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::InvalidDropAccess(..))
        )
    });
}

#[test]
fn test_vault_combine_non_fungible_invalid() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function(
            package_address,
            "InvalidCombine",
            "combine_non_fungible_vault_invalid",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::InvalidDropAccess(..))
        )
    });
}

#[test]
fn burn_invalid_fungible_bucket_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function(
            package_address,
            "InvalidCombine",
            "burn_fungible_invalid",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::InvalidDropAccess(..))
        )
    });
}

#[test]
fn burn_invalid_non_fungible_bucket_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function(
            package_address,
            "InvalidCombine",
            "burn_non_fungible_invalid",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::InvalidDropAccess(..))
        )
    });
}

fn should_not_be_able_to_lock_fee_with_non_xrd<F: FnOnce(TransactionReceipt) -> ()>(
    contingent: bool,
    amount: Decimal,
    on_receipt: F,
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("bucket"));
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_function(package_address, "InvalidCombine", "new", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let result = receipt.expect_commit_success();
    let component = result.new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .call_method(
            component,
            if contingent {
                "lock_contingent_fee"
            } else {
                "lock_fee"
            },
            manifest_args!(amount),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    on_receipt(receipt);
}

#[test]
fn should_not_be_able_to_lock_non_contingent_fee_with_non_xrd() {
    should_not_be_able_to_lock_fee_with_non_xrd(false, 1.into(), |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::LockFeeNotRadixToken
                ))
            )
        });
    });
}

#[test]
fn should_not_be_able_to_lock_contingent_fee_with_non_xrd() {
    should_not_be_able_to_lock_fee_with_non_xrd(true, 1.into(), |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::ApplicationError(ApplicationError::VaultError(
                    VaultError::LockFeeNotRadixToken
                ))
            )
        });
    });
}
