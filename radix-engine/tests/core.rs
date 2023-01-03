use radix_engine::types::*;
use radix_engine_interface::data::*;
use radix_engine_interface::model::FromPublicKey;
use radix_engine_interface::node::NetworkDefinition;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_process_and_transaction() {
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/core");

    let manifest1 = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "CoreTest", "query", args![])
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    receipt1.expect_commit_success();
}

#[test]
fn test_call() {
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/core");

    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "MoveTest", "move_bucket", args![])
        .call_function(package_address, "MoveTest", "move_proof", args![])
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}
