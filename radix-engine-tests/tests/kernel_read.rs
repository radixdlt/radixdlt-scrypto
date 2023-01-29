use radix_engine::engine::{KernelError, ResolvedActor, RuntimeError};
use radix_engine::types::*;
use radix_engine_interface::api::types::RENodeId;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn should_not_be_able_to_read_global_substate() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/kernel");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(
            package_address,
            "Read",
            "read_global_substate",
            args!(account),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::InvalidSubstateVisibility {
                actor: ResolvedActor {
                    identifier: FnIdentifier::Scrypto(..),
                    ..
                },
                node_id: RENodeId::Global(GlobalAddress::Component(..)),
                offset: SubstateOffset::Global(GlobalOffset::Global),
                ..
            })
        )
    });
}
