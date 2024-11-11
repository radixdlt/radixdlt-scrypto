use radix_engine::errors::RuntimeError;
use radix_engine::errors::SystemError;
use radix_engine::system::system_type_checker::TypeCheckError;
use radix_engine_interface::prelude::*;
use radix_engine_interface::types::FromPublicKey;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn test_add_direct_access_ref_to_stored_substate_external_vault() {
    // Basic setup
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource = ledger.create_recallable_token(account);
    let vault_id = ledger
        .get_component_vaults(account, resource)
        .pop()
        .unwrap();
    println!("Recallable vault id: {:?}", vault_id);

    // Publish package
    let (code, package_def) = PackageLoader::get("reference");
    let package_address = ledger.publish_package_simple((code, package_def));

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package_address, "ReferenceTest", "new", manifest_args!())
            .build();

        let receipt = ledger.execute_manifest(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    // Call method
    let receipt = ledger.execute_manifest(
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
    receipt.expect_commit_failure_containing_error("Non Global Reference is not allowed");
}

#[test]
fn test_add_direct_access_ref_to_heap_substate_external_vault() {
    // Basic setup
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource = ledger.create_recallable_token(account);
    let vault_id = ledger
        .get_component_vaults(account, resource)
        .pop()
        .unwrap();
    println!("Recallable vault id: {:?}", vault_id);

    // Publish package
    let (code, package_def) = PackageLoader::get("reference");
    let package_address = ledger.publish_package_simple((code, package_def));

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package_address, "ReferenceTest", "new", manifest_args!())
            .build();

        let receipt = ledger.execute_manifest(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    // Call method
    let receipt = ledger.execute_manifest(
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
    receipt.expect_commit_failure_containing_error("Non Global Reference is not allowed");
}

#[test]
fn test_add_direct_access_ref_to_kv_store_substate_external_vault() {
    // Basic setup
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource = ledger.create_recallable_token(account);
    let vault_id = ledger
        .get_component_vaults(account, resource)
        .pop()
        .unwrap();
    println!("Recallable vault id: {:?}", vault_id);

    // Publish package
    let (code, package_def) = PackageLoader::get("reference");
    let package_address = ledger.publish_package_simple((code, package_def));

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package_address, "ReferenceTest", "new", manifest_args!())
            .build();

        let receipt = ledger.execute_manifest(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    // Call method
    let receipt = ledger.execute_manifest(
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
    receipt.expect_commit_failure_containing_error("Non Global Reference is not allowed");
}

#[test]
fn test_add_direct_access_ref_to_stored_substate_internal_vault() {
    // Basic setup
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource = ledger.create_recallable_token(account);

    // Publish package
    let (code, package_def) = PackageLoader::get("reference");
    let package_address = ledger.publish_package_simple((code, package_def));

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, resource, dec!(1))
            .take_all_from_worktop(resource, "bucket")
            .call_function_with_name_lookup(
                package_address,
                "ReferenceTest",
                "new_with_bucket",
                |lookup| manifest_args!(lookup.bucket("bucket")),
            )
            .build();

        let receipt = ledger.execute_manifest(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    let vault_id = ledger
        .get_component_vaults(component_address, resource)
        .pop()
        .unwrap();
    println!("Recallable vault id: {:?}", vault_id);

    // Call function
    let receipt = ledger.execute_manifest(
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
    receipt.expect_commit_failure_containing_error("Non Global Reference is not allowed");
}

#[test]
fn test_add_direct_access_ref_to_heap_substate_internal_vault() {
    // Basic setup
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource = ledger.create_recallable_token(account);

    // Publish package
    let (code, package_def) = PackageLoader::get("reference");
    let package_address = ledger.publish_package_simple((code, package_def));

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, resource, dec!(1))
            .take_all_from_worktop(resource, "bucket")
            .call_function_with_name_lookup(
                package_address,
                "ReferenceTest",
                "new_with_bucket",
                |lookup| manifest_args!(lookup.bucket("bucket")),
            )
            .build();

        let receipt = ledger.execute_manifest(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    let vault_id = ledger
        .get_component_vaults(component_address, resource)
        .pop()
        .unwrap();
    println!("Recallable vault id: {:?}", vault_id);

    // Call function
    let receipt = ledger.execute_manifest(
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
    receipt.expect_commit_failure_containing_error("Non Global Reference is not allowed");
}

#[test]
fn test_add_direct_access_ref_to_kv_store_substate_internal_vault() {
    // Basic setup
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource = ledger.create_recallable_token(account);

    // Publish package
    let (code, package_def) = PackageLoader::get("reference");
    let package_address = ledger.publish_package_simple((code, package_def));

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account, resource, dec!(1))
            .take_all_from_worktop(resource, "bucket")
            .call_function_with_name_lookup(
                package_address,
                "ReferenceTest",
                "new_with_bucket",
                |lookup| manifest_args!(lookup.bucket("bucket")),
            )
            .build();

        let receipt = ledger.execute_manifest(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    let vault_id = ledger
        .get_component_vaults(component_address, resource)
        .pop()
        .unwrap();
    println!("Recallable vault id: {:?}", vault_id);

    // Call function
    let receipt = ledger.execute_manifest(
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
    receipt.expect_commit_failure_containing_error("Non Global Reference is not allowed");
}

#[test]
fn test_create_global_node_with_local_ref() {
    // Basic setup
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Publish package
    let package_address = ledger.publish_package_simple(PackageLoader::get("reference"));

    // Call function
    let receipt = ledger.execute_manifest(
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Publish package
    let package_address = ledger.publish_package_simple(PackageLoader::get("reference"));

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package_address, "ReferenceTest", "new", manifest_args!())
            .build();

        let receipt = ledger.execute_manifest(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    // Call method
    let receipt = ledger.execute_manifest(
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

#[test]
fn test_internal_typed_reference() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource = ledger.create_recallable_token(account);
    let vault_id = ledger
        .get_component_vaults(account, resource)
        .pop()
        .unwrap();
    let package_address = ledger.publish_package_simple(PackageLoader::get("reference"));

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .call_function(
                package_address,
                "ReferenceTest",
                "recall",
                manifest_args!(InternalAddress::new_or_panic(vault_id.into())),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_send_and_receive_reference_from_child_call_frame() {
    // This test checks what happens if I create a reference to an owned node, send it to a child, and receive it back.
    // At present, the "send to child" check requires that the reference is a direct reference, which can only
    // be created by the root call frame - therefore this errors with a "DirectRefNotFound".

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("reference"));

    // Act
    let receipt = ledger.call_function(
        package_address,
        "ReferenceTest",
        "send_and_receive_reference",
        manifest_args!(),
    );

    // Assert
    receipt.expect_specific_failure(|e| format!("{e:?}").contains("DirectRefNotFound"));
}

#[test]
fn test_send_and_receive_reference_wrapped_in_non_transient_wrapper() {
    // This test just checks the limits of engine/scrypto behaviour with a "fake proof" style model.
    // * Creating a normal `ChildReferenceHolder` means it's not transient, so it's a validation
    //   error when we try to create an object with a non-global reference.

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("reference"));

    // Act
    let receipt = ledger.call_function(
        package_address,
        "ReferenceTest",
        "send_and_receive_reference_wrapped_in_owned",
        manifest_args!(),
    );

    // Assert
    receipt.expect_commit_failure_containing_error("BlueprintPayloadValidationError");
    receipt.expect_commit_failure_containing_error("Non Global Reference is not allowed");
}

#[test]
fn test_send_and_receive_reference_wrapped_in_transient_wrapper() {
    // This test just checks the limits of engine/scrypto behaviour with a "fake proof" style model.
    // * Having the `ChildReferenceHolder` be transient would allow for it to be created with an internal
    // reference in its substates.
    //
    // BUT:
    // * Currently scrypto components can't be set to be "transient"
    // * And also, relatedly, drop isn't supported in scrypto, so this test would error during clean up with an undropped node error
    //
    // As such, at present, the only place this pattern is used is in Proofs - notably proofs
    // against buckets and vaults.

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let upload_package_manifest = manifest_to_publish_package_with_transient_blueprints(
        PackageLoader::get("reference"),
        Default::default(),
        OwnerRole::None,
        indexset!("ChildReferenceHolder".to_string()),
    );

    let upload_package_receipt = ledger.execute_manifest(upload_package_manifest, vec![]);

    upload_package_receipt
        .expect_commit_failure_containing_error("Transient blueprints not supported");
}

fn manifest_to_publish_package_with_transient_blueprints<P: Into<PackagePublishingSource>>(
    source: P,
    metadata: BTreeMap<String, MetadataValue>,
    owner_role: OwnerRole,
    transient_blueprints: IndexSet<String>,
) -> TransactionManifestV1 {
    let (code, mut definition) = source.into().code_and_definition();
    for blueprint in transient_blueprints.into_iter() {
        let bp_definition_init = definition
            .blueprints
            .get_mut(&blueprint)
            .expect("Blueprint was not found");
        bp_definition_init.is_transient = true;
    }
    ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(None, code, definition, metadata, owner_role)
        .build()
}
