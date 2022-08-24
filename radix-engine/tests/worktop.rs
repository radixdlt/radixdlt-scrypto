use radix_engine::engine::DropFailure;
use radix_engine::engine::KernelError;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::TypedInMemorySubstateStore;
use scrypto::core::NetworkDefinition;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_worktop_resource_leak() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .withdraw_from_account(RADIX_TOKEN, account)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_commit_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::DropFailure(DropFailure::Worktop))
        )
    });
}
