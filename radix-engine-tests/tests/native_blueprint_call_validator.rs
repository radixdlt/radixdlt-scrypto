use radix_engine::{
    errors::RuntimeError,
    utils::{validate_call_arguments_to_native_components, ValidationError},
};
use scrypto::prelude::*;
use transaction::{prelude::ManifestBuilder, validation::EcdsaSecp256k1PrivateKey};

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
    assert!(validation_result.is_ok())
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

fn is_schema_validation_error<T>(result: Result<T, ValidationError>) -> bool {
    if let Err(error) = result {
        matches!(error, ValidationError::SchemaValidationError(..))
    } else {
        false
    }
}

fn is_method_not_found<T>(result: Result<T, ValidationError>) -> bool {
    if let Err(error) = result {
        matches!(error, ValidationError::MethodNotFound(..))
    } else {
        false
    }
}

fn private_key1() -> EcdsaSecp256k1PrivateKey {
    EcdsaSecp256k1PrivateKey::from_u64(1).unwrap()
}

fn private_key2() -> EcdsaSecp256k1PrivateKey {
    EcdsaSecp256k1PrivateKey::from_u64(2).unwrap()
}

fn account1() -> ComponentAddress {
    ComponentAddress::virtual_account_from_public_key(&private_key1().public_key())
}

fn account2() -> ComponentAddress {
    ComponentAddress::virtual_account_from_public_key(&private_key2().public_key())
}
