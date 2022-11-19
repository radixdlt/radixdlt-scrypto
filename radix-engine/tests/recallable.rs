use radix_engine::engine::{CallFrameError, KernelError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::Instruction;

/*
#[test]
fn non_existent_vault_in_component_creation_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (_, _, account) = test_runner.new_allocated_account();

    let resource_address = test_runner.create_fungible_resource(10u32.into(), 0u8, account);
    test_runner.deref_component()

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10u32.into())
        .add_instruction(Instruction::CallMethod {

        })
        .call_function(
            package_address,
            "NonExistentVault",
            "create_component_with_non_existent_vault",
            args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    let (builder, _, _) = builder.add_instruction(Instruction::CallMethod {
        method_ident: ScryptoMethodIdent {
            receiver: ScryptoReceiver::Component(victim_account_component_id),
            method_name: "withdraw".to_string(),
        },
        args: args!(RADIX_TOKEN),
    });


    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::CallFrameError(CallFrameError::RENodeNotOwned(RENodeId::Vault(_)))
        )
    });
}
 */
