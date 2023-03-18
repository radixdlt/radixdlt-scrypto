use radix_engine::errors::{ModuleError, RuntimeError};
use radix_engine::system::kernel_modules::auth::AuthError;
use radix_engine::types::*;
use radix_engine_interface::blueprints::account::ACCOUNT_DEPOSIT_BATCH_IDENT;
use radix_engine_interface::blueprints::identity::{IDENTITY_SECURIFY_TO_SINGLE_BADGE_IDENT, IdentitySecurifyToSingleBadgeInput};
use radix_engine_interface::blueprints::resource::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::ecdsa_secp256k1::EcdsaSecp256k1PrivateKey;
use transaction::model::Instruction;

#[test]
fn cannot_securify_in_advanced_mode() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (pk, _, account) = test_runner.new_account(false);
    let component_address = test_runner.new_identity(pk.clone(), false);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .add_instruction(Instruction::CallMethod {
            component_address,
            method_name: IDENTITY_SECURIFY_TO_SINGLE_BADGE_IDENT.to_string(),
            args: to_manifest_value(&IdentitySecurifyToSingleBadgeInput {}).unwrap(),
        }).0
        .call_method(
            account,
            ACCOUNT_DEPOSIT_BATCH_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. }))
        )
    });
}

#[test]
fn can_securify_from_virtual_identity() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (pk, _, account) = test_runner.new_account(false);
    let component_address = test_runner.new_identity(pk.clone(), true);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .add_instruction(Instruction::CallMethod {
            component_address,
            method_name: IDENTITY_SECURIFY_TO_SINGLE_BADGE_IDENT.to_string(),
            args: to_manifest_value(&IdentitySecurifyToSingleBadgeInput {}).unwrap(),
        }).0
        .call_method(
            account,
            ACCOUNT_DEPOSIT_BATCH_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    receipt.expect_commit_success();
}