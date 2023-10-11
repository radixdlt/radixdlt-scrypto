mod package_loader;

use package_loader::PackageLoader;
use radix_engine::errors::RuntimeError;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_component() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let package = test_runner.publish_package_simple(PackageLoader::get("component"));

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
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    receipt1.expect_commit_success();
}

#[test]
fn invalid_blueprint_name_should_cause_error() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_addr = test_runner.publish_package_simple(PackageLoader::get("component"));

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
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| matches!(e, RuntimeError::SystemError(..)));
}

#[test]
fn blueprint_name_can_be_obtained_from_a_function() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("component"));

    // Act
    let blueprint_name = test_runner
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
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("component"));

    let component_address = *test_runner
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
    let blueprint_name = test_runner
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
fn pass_bucket_to_component_and_check_amount() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_account(false);
    let package_address = test_runner.publish_package_simple(PackageLoader::get("component"));

    // create two components
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                "ComponentTest",
                "create_component",
                manifest_args!(),
            )
            .call_function(
                package_address,
                "ComponentTest",
                "create_component",
                manifest_args!(),
            )
            .build(),
        vec![],
    );
    let result = receipt.expect_commit_success();

    let component_address_1 = result.new_component_addresses()[0];
    let component_address_2 = result.new_component_addresses()[1];
    let resource_address_1 = result.new_resource_addresses()[0];
    let resource_address_2 = result.new_resource_addresses()[1];

    // take bucket with resources from component 1 and pass that bucket to component 2
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(component_address_1, "put_component_state", manifest_args!())
            .take_all_from_worktop(resource_address_1, "bucket_name")
            .call_method_with_name_lookup(
                component_address_2,
                "take_resource_amount_of_bucket",
                |lookup| (lookup.bucket("bucket_name"),),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![],
    );

    // verify if manifest executed with success and deposited account balances
    receipt.expect_commit_success();

    let balance_1 = test_runner
        .get_component_resources(account)
        .get(&resource_address_1)
        .cloned()
        .unwrap();
    let balance_2 = test_runner
        .get_component_resources(account)
        .get(&resource_address_2)
        .cloned()
        .unwrap();

    assert_eq!(balance_1, dec!(1));
    assert_eq!(balance_2, dec!(2));
}


#[test]
fn pass_proof_to_component_and_check() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.publish_package_simple(PackageLoader::get("component"));

    // create two components
    let receipt = test_runner.execute_manifest(
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

    let receipt = test_runner.execute_manifest(
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

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(component_address_1, "generate_nft", manifest_args!())
            .take_all_from_worktop(resource_address_1, "bucket_1")
            .create_proof_from_bucket_of_all("bucket_1", "proof_1")
            .call_method_with_name_lookup(component_address_2, "check", 
                |lookup| (lookup.proof("proof_1"),))
            .return_to_worktop("bucket_1")
            .burn_all_from_worktop(resource_address_1)
            .build(),
        vec![],
    );

    receipt.expect_commit_success();
}

