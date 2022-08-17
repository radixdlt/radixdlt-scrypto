use radix_engine::ledger::TypedInMemorySubstateStore;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_process_and_transaction() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("core");

    let manifest1 = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "CoreTest", "query", args![])
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    receipt1.expect_success();
}

#[test]
fn test_call() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("core");

    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "MoveTest", "move_bucket", args![])
        .call_function(package_address, "MoveTest", "move_proof", args![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    receipt.expect_success();
}
