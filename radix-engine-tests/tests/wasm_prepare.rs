use radix_engine::{
    errors::{ApplicationError, RuntimeError},
    types::*,
    vm::wasm::PrepareError,
};
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_queries::typed_substate_layout::PackageError;
use scrypto::prelude::FromPublicKey;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_too_many_locals() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    let code = include_bytes!("./assets/too_many_locals.wasm").to_vec();
    let definition = PackageDefinition::default();

    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_standard_test_fee(account)
            .publish_package(code, definition)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    assert!(matches!(
        receipt.expect_commit_failure().outcome.expect_failure(),
        RuntimeError::ApplicationError(ApplicationError::PackageError(PackageError::InvalidWasm(
            PrepareError::ValidationError { .. }
        )))
    ));
}
