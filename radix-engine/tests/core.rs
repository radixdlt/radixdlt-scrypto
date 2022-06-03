#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use scrypto::call_data;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;
use transaction::signing::EcdsaPrivateKey;

#[test]
fn test_process_and_transaction() {
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.publish_package("core");

    let manifest1 = ManifestBuilder::new()
        .call_function(package_address, "CoreTest", call_data![query()])
        .build();
    let signers = vec![];
    let receipt1 = test_runner.execute_manifest(manifest1, signers);
    receipt1.result.expect("Should be okay.");
}

#[test]
fn test_call() {
    let mut test_runner = TestRunner::new(true);
    let (_, _, account) = test_runner.new_account();
    let package_address = test_runner.publish_package("core");

    let manifest = ManifestBuilder::new()
        .call_function(package_address, "MoveTest", call_data![move_bucket()])
        .call_function(package_address, "MoveTest", call_data![move_proof()])
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);
    receipt.result.expect("Should be okay.");
}
