use radix_common::prelude::*;
use radix_engine::errors::{CallFrameError, KernelError, RuntimeError};
use radix_engine::kernel::call_frame::DropNodeError;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn test_component() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("component"));

    // Create component
    let manifest1 = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "ComponentTest",
            "create_component",
            manifest_args!(),
        )
        .build();
    let receipt1 = ledger.execute_manifest(manifest1, vec![]);
    receipt1.expect_commit_success();
}

#[test]
fn invalid_blueprint_name_should_cause_error() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_addr = ledger.publish_package_simple(PackageLoader::get("component"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_addr,
            "NonExistentBlueprint",
            "create_component",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| matches!(e, RuntimeError::SystemError(..)));
}

#[test]
fn blueprint_name_can_be_obtained_from_a_function() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("component"));

    // Act
    let blueprint_name = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    package_address,
                    "ComponentTest",
                    "blueprint_name_function",
                    manifest_args!(),
                )
                .build(),
            vec![],
        )
        .expect_commit_success()
        .output::<String>(1);

    // Assert
    assert_eq!(blueprint_name, "ComponentTest")
}

#[test]
fn blueprint_name_can_be_obtained_from_a_method() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("component"));

    let component_address = *ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    package_address,
                    "ComponentTest",
                    "create_component",
                    manifest_args!(),
                )
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .unwrap();

    // Act
    let blueprint_name = ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(component_address, "blueprint_name_method", manifest_args!())
                .build(),
            vec![],
        )
        .expect_commit_success()
        .output::<String>(1);

    // Assert
    assert_eq!(blueprint_name, "ComponentTest")
}

#[test]
fn pass_bucket_and_proof_to_other_component() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("component"));

    // create 1st component
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                "ComponentTest2",
                "create_component",
                manifest_args!(),
            )
            .build(),
        vec![],
    );
    let result = receipt.expect_commit_success();

    let component_address_1 = result.new_component_addresses().first().cloned().unwrap();
    let resource_address_1 = result.new_resource_addresses().first().cloned().unwrap();

    // create 2nd component passing resource address from 1st component
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                "ComponentTest3",
                "create_component",
                manifest_args!(resource_address_1),
            )
            .build(),
        vec![],
    );
    let result = receipt.expect_commit_success();
    let component_address_2 = result.new_component_addresses().first().cloned().unwrap();

    // take bucket and proof from the 1st component and pass
    // to the 2nd component for proof check and bucket burn
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(component_address_1, "generate_nft_proof", manifest_args!())
            .take_all_from_worktop(resource_address_1, "bucket_1")
            .pop_from_auth_zone("proof_1")
            .call_method_with_name_lookup(
                component_address_2,
                "check_proof_and_burn_bucket",
                |lookup| (lookup.bucket("bucket_1"), lookup.proof("proof_1")),
            )
            .build(),
        vec![],
    );

    // verify if manifest executed with success
    receipt.expect_commit_success();
}

#[test]
fn pass_bucket_and_proof_to_other_component_fail() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("component"));

    // create 1st component
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                "ComponentTest2",
                "create_component",
                manifest_args!(),
            )
            .build(),
        vec![],
    );
    let result = receipt.expect_commit_success();

    let component_address_1 = result.new_component_addresses().first().cloned().unwrap();
    let resource_address_1 = result.new_resource_addresses().first().cloned().unwrap();

    // create 2nd component passing resource address from 1st component
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                "ComponentTest3",
                "create_component",
                manifest_args!(resource_address_1),
            )
            .build(),
        vec![],
    );
    let result = receipt.expect_commit_success();
    let component_address_2 = result.new_component_addresses().first().cloned().unwrap();

    // take bucket and proof from the 1st component and pass
    // to the 2nd component for proof check and bucket burn
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(component_address_1, "generate_nft_proof", manifest_args!())
            .take_all_from_worktop(resource_address_1, "bucket_1")
            .pop_from_auth_zone("proof_1")
            .call_method_with_name_lookup(
                component_address_2,
                "burn_bucket_and_check_proof",
                |lookup| (lookup.bucket("bucket_1"), lookup.proof("proof_1")),
            )
            .build(),
        vec![],
    );

    // verify if manifest executed with an error
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
fn pass_vault_and_proof_to_other_component() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("component"));

    // create 1st component
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                "ComponentTest2",
                "create_component",
                manifest_args!(),
            )
            .build(),
        vec![],
    );
    let result = receipt.expect_commit_success();

    let component_address_1 = result.new_component_addresses().first().cloned().unwrap();

    // 1st component creates proof and vault and passes it to newly created 2nd component
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                component_address_1,
                "pass_vault_to_new_component",
                manifest_args!(),
            )
            .build(),
        vec![],
    );

    // verify if manifest executed with success
    receipt.expect_commit_success();
}
