#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::DropFailure;
use radix_engine::engine::RuntimeError;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_worktop_resource_leak() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(RADIX_TOKEN, account)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::DropFailure(DropFailure::Worktop)));
}
