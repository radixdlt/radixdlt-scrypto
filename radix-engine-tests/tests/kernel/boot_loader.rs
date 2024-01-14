use radix_engine::errors::{KernelError, RejectionReason, RuntimeError};
use radix_engine::transaction::{CostingParameters, ExecutionConfig};
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn boot_loader_state_should_not_be_visible() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .build();
    let nonce = test_runner.next_transaction_nonce();
    let prepared = TestTransaction::new_from_nonce(manifest, nonce)
        .prepare()
        .expect("expected transaction to be preparable");
    let mut executable = prepared.get_executable(btreeset!());
    executable.references.insert(Reference(BOOT_LOADER_STATE));
    let receipt = test_runner.execute_transaction(
        executable,
        CostingParameters::default(),
        ExecutionConfig::for_test_transaction(),
    );

    // Assert
    let reason = receipt.expect_rejection();
    assert!(matches!(reason, RejectionReason::ErrorBeforeLoanAndDeferredCostsRepaid(RuntimeError::KernelError(KernelError::InvalidReference(..)))));
}