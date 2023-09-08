mod package_loader;

use package_loader::PackageLoader;
use radix_engine::errors::SystemError;
use radix_engine::system::system_type_checker::TypeCheckError;
use radix_engine::{errors::RuntimeError, types::*};
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_add_direct_access_ref_to_stored_substate_external_vault() {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource = test_runner.create_recallable_token(account);
    let vault_id = test_runner
        .get_component_vaults(account, resource)
        .pop()
        .unwrap();
    println!("Recallable vault id: {:?}", vault_id);

    // Publish package
    let (code, mut package_def) = PackageLoader::get("reference");
    package_def
        .blueprints
        .values_mut()
        .next()
        .unwrap()
        .is_transient = true;
    let package_address = test_runner.publish_package_simple((code, package_def));

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .call_function(package_address, "ReferenceTest", "new", manifest_args!())
            .build();

        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    // Call method
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_method(
                component_address,
                "add_direct_access_ref_to_stored_substate",
                manifest_args!(InternalAddress::try_from(vault_id).unwrap()),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}", receipt);

    // Assert
    receipt.expect_specific_failure(|e| {
        e.to_string().contains("RefNotFound")
            && e.to_string().contains(&hex::encode(vault_id.as_bytes()))
    });
}

#[test]
fn test_add_direct_access_ref_to_heap_substate_external_vault() {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource = test_runner.create_recallable_token(account);
    let vault_id = test_runner
        .get_component_vaults(account, resource)
        .pop()
        .unwrap();
    println!("Recallable vault id: {:?}", vault_id);

    // Publish package
    let (code, mut package_def) = PackageLoader::get("reference");
    package_def
        .blueprints
        .values_mut()
        .next()
        .unwrap()
        .is_transient = true;
    let package_address = test_runner.publish_package_simple((code, package_def));

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .call_function(package_address, "ReferenceTest", "new", manifest_args!())
            .build();

        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    // Call method
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_method(
                component_address,
                "add_direct_access_ref_to_heap_substate",
                manifest_args!(InternalAddress::try_from(vault_id).unwrap()),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}", receipt);

    // Assert
    receipt.expect_specific_failure(|e| {
        e.to_string().contains("RefNotFound")
            && e.to_string().contains(&hex::encode(vault_id.as_bytes()))
    });
}

#[test]
fn test_add_direct_access_ref_to_stored_substate_internal_vault() {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource = test_runner.create_recallable_token(account);

    // Publish package
    let (code, mut package_def) = PackageLoader::get("reference");
    package_def
        .blueprints
        .values_mut()
        .next()
        .unwrap()
        .is_transient = true;
    let package_address = test_runner.publish_package_simple((code, package_def));

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(account, resource, dec!(1))
            .take_all_from_worktop(resource, "bucket")
            .call_function_with_name_lookup(
                package_address,
                "ReferenceTest",
                "new_with_bucket",
                |lookup| manifest_args!(lookup.bucket("bucket")),
            )
            .build();

        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    let vault_id = test_runner
        .get_component_vaults(component_address, resource)
        .pop()
        .unwrap();
    println!("Recallable vault id: {:?}", vault_id);

    // Call function
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_method(
                component_address,
                "add_direct_access_ref_to_stored_substate",
                manifest_args!(InternalAddress::try_from(vault_id).unwrap()),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}", receipt);

    // Assert
    receipt.expect_specific_failure(|e| {
        e.to_string().contains("NonGlobalRefNotAllowed")
            && e.to_string().contains(&hex::encode(vault_id.as_bytes()))
    });
}

#[test]
fn test_add_direct_access_ref_to_heap_substate_internal_vault() {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource = test_runner.create_recallable_token(account);

    // Publish package
    let (code, mut package_def) = PackageLoader::get("reference");
    package_def
        .blueprints
        .values_mut()
        .next()
        .unwrap()
        .is_transient = true;
    let package_address = test_runner.publish_package_simple((code, package_def));

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(account, resource, dec!(1))
            .take_all_from_worktop(resource, "bucket")
            .call_function_with_name_lookup(
                package_address,
                "ReferenceTest",
                "new_with_bucket",
                |lookup| manifest_args!(lookup.bucket("bucket")),
            )
            .build();

        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    let vault_id = test_runner
        .get_component_vaults(component_address, resource)
        .pop()
        .unwrap();
    println!("Recallable vault id: {:?}", vault_id);

    // Call function
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_method(
                component_address,
                "add_direct_access_ref_to_heap_substate",
                manifest_args!(InternalAddress::try_from(vault_id).unwrap()),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}", receipt);

    // Assert
    receipt.expect_specific_failure(|e| {
        e.to_string().contains("RefNotFound")
            && e.to_string().contains(&hex::encode(vault_id.as_bytes()))
    });
}

#[test]
fn test_add_direct_access_ref_to_kv_store_substate_internal_vault() {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource = test_runner.create_recallable_token(account);

    // Publish package
    let (code, mut package_def) = PackageLoader::get("reference");
    package_def
        .blueprints
        .values_mut()
        .next()
        .unwrap()
        .is_transient = true;
    let package_address = test_runner.publish_package_simple((code, package_def));

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(account, resource, dec!(1))
            .take_all_from_worktop(resource, "bucket")
            .call_function_with_name_lookup(
                package_address,
                "ReferenceTest",
                "new_with_bucket",
                |lookup| manifest_args!(lookup.bucket("bucket")),
            )
            .build();

        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    let vault_id = test_runner
        .get_component_vaults(component_address, resource)
        .pop()
        .unwrap();
    println!("Recallable vault id: {:?}", vault_id);

    // Call function
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_method(
                component_address,
                "add_direct_access_ref_to_kv_store_substate",
                manifest_args!(InternalAddress::try_from(vault_id).unwrap()),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}", receipt);

    // Assert
    receipt.expect_specific_failure(|e| {
        e.to_string()
            .contains("Non Global Reference is not allowed")
    });
}

#[test]
fn test_create_global_node_with_local_ref() {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.publish_package_simple(PackageLoader::get("reference"));

    // Call function
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_function(
                package_address,
                "ReferenceTest",
                "create_global_node_with_local_ref",
                manifest_args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemError(SystemError::TypeCheckError(
            TypeCheckError::BlueprintPayloadValidationError(.., error),
        )) => error.contains("Non Global Reference"),
        _ => false,
    });
}

#[test]
fn test_add_local_ref_to_stored_substate() {
    // Basic setup
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.publish_package_simple(PackageLoader::get("reference"));

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .call_function(package_address, "ReferenceTest", "new", manifest_args!())
            .build();

        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    // Call method
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_method(
                component_address,
                "add_local_ref_to_stored_substate",
                manifest_args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::SystemError(SystemError::TypeCheckError(
            TypeCheckError::BlueprintPayloadValidationError(.., error),
        )) => error.contains("Non Global Reference"),
        _ => false,
    });
}
