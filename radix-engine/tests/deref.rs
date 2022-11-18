use radix_engine::engine::{CallFrameError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use radix_engine_interface::api::types::{RENodeId, ScryptoMethodIdent};
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::*;

#[test]
fn manifest_cannot_refer_to_persisted_component_by_id() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (victim_public_key, _, victim_account) = test_runner.new_allocated_account();
    let (_, _, attacker_account) = test_runner.new_allocated_account();

    let node_id = test_runner.deref_component_address(victim_account);
    let victim_account_component_id: ComponentId = node_id.into();

    // Act
    let mut builder = ManifestBuilder::new(&NetworkDefinition::simulator());
    let builder = builder.lock_fee(victim_account, dec!("10"));
    // NOTE - the following line is not flagged in the wallet - it just looks like we're paying a fee!
    let (builder, _, _) = builder.add_instruction(Instruction::CallMethod {
        method_ident: ScryptoMethodIdent {
            receiver: ScryptoReceiver::Component(victim_account_component_id),
            method_name: "withdraw".to_string(),
        },
        args: args!(RADIX_TOKEN),
    });

    let manifest = builder
        .call_method(
            attacker_account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&victim_public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        e.eq(&RuntimeError::CallFrameError(
            CallFrameError::RENodeNotVisible(RENodeId::Component(victim_account_component_id)),
        ))
    });
}

#[test]
fn no_new_visible_nodes_on_deref() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (_, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/deref");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package, "Deref", "verify_no_new_visible_nodes_on_deref", args!(account))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
