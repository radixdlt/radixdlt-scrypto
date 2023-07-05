use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto::prelude::{WORKTOP_BLUEPRINT, WORKTOP_DROP_IDENT};
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn mis_typed_own_passed_to_worktop_drop_function() {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Run manifest
    let receipt = test_runner.execute_manifest(
        ManifestBuilderV2::new()
            .lock_fee(account, 500)
            .take_from_worktop(XRD, Decimal::ZERO, "bucket")
            .with_namer(|builder, namer| {
                builder.call_function(
                    RESOURCE_PACKAGE,
                    WORKTOP_BLUEPRINT,
                    WORKTOP_DROP_IDENT,
                    manifest_args!(namer.bucket("bucket")),
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
