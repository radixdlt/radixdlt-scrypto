use radix_engine::engine::{InterpreterError, RuntimeError, ScryptoActorError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto::core::{FnIdent, MethodIdent, ReceiverMethodIdent};
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
        .call_scrypto_function(
            package_address,
            "Invoke",
            "call_invalid_scrypto_call_on_vault",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::InterpreterError(InterpreterError::InvalidScryptoActor(
                FnIdent::Method(ReceiverMethodIdent {
                    receiver: Receiver::Ref(RENodeId::Vault(..)),
                    method_ident: MethodIdent::Scrypto(..),
                }),
                ScryptoActorError::InvalidReceiver
            ))
        )
    });
}
