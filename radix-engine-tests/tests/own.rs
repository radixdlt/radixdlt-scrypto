use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto::prelude::{WORKTOP_BLUEPRINT, WORKTOP_DROP_IDENT};
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn mis_typed_own_passed_to_worktop_drop_function() {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Run manifest
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .take_from_worktop(RADIX_TOKEN, Decimal::ZERO, |builder, bucket| {
                builder.call_function(
                    RESOURCE_PACKAGE,
                    WORKTOP_BLUEPRINT,
                    WORKTOP_DROP_IDENT,
                    manifest_args!(bucket),
                )
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    let error_message = receipt
        .expect_commit_failure()
        .outcome
        .expect_failure()
        .to_string();
    assert!(error_message.contains("ValidationError"))
}
