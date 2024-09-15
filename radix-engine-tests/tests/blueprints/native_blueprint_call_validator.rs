use radix_common::prelude::NetworkDefinition;
use radix_engine::utils::{
    validate_call_arguments_to_native_components, InstructionSchemaValidationError,
    LocatedInstructionSchemaValidationError,
};
use radix_engine_tests::common::*;
use radix_transactions::manifest::e2e::apply_address_replacements;
use radix_transactions::manifest::*;
use radix_transactions::prelude::*;
use scrypto::prelude::*;
use walkdir::WalkDir;

#[test]
fn validator_sees_valid_transfer_manifest_as_valid() {
    // Arrange
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account1(), XRD, dec!("10"))
        .try_deposit_entire_worktop_or_abort(account2(), None)
        .build();

    // Act
    let validation_result = validate_call_arguments_to_native_components(&manifest);

    // Assert
    validation_result
        .clone()
        .unwrap_or_else(|_| panic!("Validation failed: {:?}", validation_result))
}

#[test]
fn validator_sees_invalid_transfer_manifest_as_invalid() {
    // Arrange
    let manifest = ManifestBuilder::new()
        .call_method(account1(), "withdraw", manifest_args!())
        .try_deposit_entire_worktop_or_abort(account2(), None)
        .build();

    // Act
    let validation_result = validate_call_arguments_to_native_components(&manifest);

    // Assert
    assert!(is_schema_validation_error(validation_result))
}

#[test]
fn validator_invalidates_calls_to_unknown_methods_on_a_known_blueprint() {
    // Arrange
    let manifest = ManifestBuilder::new()
        .call_method(account1(), "my_made_up_method", manifest_args!())
        .try_deposit_entire_worktop_or_abort(account2(), None)
        .build();

    // Act
    let validation_result = validate_call_arguments_to_native_components(&manifest);

    // Assert
    assert!(is_method_not_found(validation_result))
}

#[test]
fn common_manifests_are_all_valid() {
    // Arrange
    for entry in WalkDir::new(path_workspace_transaction_examples!("access_rule")) {
        let path = entry.unwrap().path().canonicalize().unwrap();

        if path.extension().and_then(|str| str.to_str()) != Some("rtm") {
            continue;
        }

        let manifest_string = std::fs::read_to_string(&path)
            .map(|str| apply_address_replacements(str))
            .unwrap();
        let manifest = compile_manifest_v1(
            &manifest_string,
            &NetworkDefinition::simulator(),
            MockBlobProvider::new(),
        )
        .unwrap();

        // Act
        let validation_result = validate_call_arguments_to_native_components(&manifest);

        // Assert
        validation_result.clone().unwrap_or_else(|_| {
            panic!(
                "Validation failed for manifest \"{:?}\" with error: \"{:?}\"",
                path, validation_result
            )
        })
    }
}

fn is_schema_validation_error<T>(
    result: Result<T, LocatedInstructionSchemaValidationError>,
) -> bool {
    if let Err(error) = result {
        matches!(
            error.cause,
            InstructionSchemaValidationError::SchemaValidationError(..)
        )
    } else {
        false
    }
}

fn is_method_not_found<T>(result: Result<T, LocatedInstructionSchemaValidationError>) -> bool {
    if let Err(error) = result {
        matches!(
            error.cause,
            InstructionSchemaValidationError::MethodNotFound(..)
        )
    } else {
        false
    }
}

fn private_key1() -> Secp256k1PrivateKey {
    Secp256k1PrivateKey::from_u64(1).unwrap()
}

fn private_key2() -> Secp256k1PrivateKey {
    Secp256k1PrivateKey::from_u64(2).unwrap()
}

fn account1() -> ComponentAddress {
    ComponentAddress::preallocated_account_from_public_key(&private_key1().public_key())
}

fn account2() -> ComponentAddress {
    ComponentAddress::preallocated_account_from_public_key(&private_key2().public_key())
}
