use radix_engine::engine::{KernelError, REActor, ResolvedFunction, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn should_not_be_able_to_node_create_with_invalid_blueprint() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/kernel");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "NodeCreate",
            "create_node_with_invalid_blueprint",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::InvalidCreateNodeVisibility {
                actor: REActor::Function(ResolvedFunction::Scrypto {
                    package_address: addr,
                    blueprint_name: blueprint,
                    ..
                }),
                ..
            }) if addr.eq(&package_address) && blueprint.eq("NodeCreate")
        )
    });
}

#[test]
fn should_not_be_able_to_node_create_with_invalid_package() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/kernel");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function(
            package_address,
            "NodeCreate",
            "create_node_with_invalid_package",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::InvalidCreateNodeVisibility {
                actor: REActor::Function(ResolvedFunction::Scrypto {
                    package_address: addr,
                    blueprint_name: blueprint,
                    ..
                }),
                ..
            }) if addr.eq(&package_address) && blueprint.eq("NodeCreate")
        )
    });
}
