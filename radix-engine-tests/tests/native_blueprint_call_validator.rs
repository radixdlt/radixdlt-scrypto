use radix_engine::utils::{
    validate_call_arguments_to_native_components, InstructionSchemaValidationError,
    LocatedInstructionSchemaValidationError,
};
use radix_engine_common::prelude::NetworkDefinition;
use scrypto::prelude::*;
use transaction::{
    manifest::{compile, MockBlobProvider},
    prelude::ManifestBuilder,
    signing::secp256k1::Secp256k1PrivateKey,
};
use walkdir::WalkDir;

use transaction::manifest::e2e::apply_address_replacements;

#[test]
fn validator_sees_valid_transfer_manifest_as_valid() {
    // Arrange
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account1(), RADIX_TOKEN, dec!("10"))
        .try_deposit_batch_or_abort(account2())
        .build();

    // Act
    let validation_result = validate_call_arguments_to_native_components(&manifest.instructions);

    // Assert
    validation_result
        .clone()
        .expect(format!("Validation failed: {:?}", validation_result).as_str())
}

#[test]
fn validator_sees_invalid_transfer_manifest_as_invalid() {
    // Arrange
    let manifest = ManifestBuilder::new()
        .call_method(account1(), "withdraw", manifest_args!())
        .try_deposit_batch_or_abort(account2())
        .build();

    // Act
    let validation_result = validate_call_arguments_to_native_components(&manifest.instructions);

    // Assert
    assert!(is_schema_validation_error(validation_result))
}

#[test]
fn validator_invalidates_calls_to_unknown_methods_on_a_known_blueprint() {
    // Arrange
    let manifest = ManifestBuilder::new()
        .call_method(account1(), "my_made_up_method", manifest_args!())
        .try_deposit_batch_or_abort(account2())
        .build();

    // Act
    let validation_result = validate_call_arguments_to_native_components(&manifest.instructions);

    // Assert
    assert!(is_method_not_found(validation_result))
}

#[test]
fn common_manifests_are_all_valid() {
    // Arrange
    // let path = "../transaction";
    let path = "../transaction/examples/access_rule";
    for entry in WalkDir::new(path) {
        let path = entry.unwrap().path().canonicalize().unwrap();

        if path.extension().and_then(|str| str.to_str()) != Some("rtm") {
            continue;
        }

        let manifest_string = std::fs::read_to_string(&path)
            .map(|str| apply_address_replacements(str))
            .unwrap();
        let manifest = compile(
            &manifest_string,
            &NetworkDefinition::simulator(),
            MockBlobProvider::new(),
        )
        .unwrap();

        // Act
        let validation_result =
            validate_call_arguments_to_native_components(&manifest.instructions);

        // Assert
        validation_result.clone().expect(
            format!(
                "Validation failed for manifest \"{:?}\" with error: \"{:?}\"",
                path, validation_result
            )
            .as_str(),
        )
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
    ComponentAddress::virtual_account_from_public_key(&private_key1().public_key())
}

fn account2() -> ComponentAddress {
    ComponentAddress::virtual_account_from_public_key(&private_key2().public_key())
}
