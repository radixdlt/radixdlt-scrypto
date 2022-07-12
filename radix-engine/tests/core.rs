#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn test_process_and_transaction() {
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.extract_and_publish_package("core");

    let manifest1 = ManifestBuilder::new()
        .call_function(package_address, "CoreTest", "query", to_struct![])
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![], false);
    receipt1.expect_success();
}

#[test]
fn test_call() {
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("core");

    let manifest = ManifestBuilder::new()
        .call_function(package_address, "MoveTest", "move_bucket", to_struct![])
        .call_function(package_address, "MoveTest", "move_proof", to_struct![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key], false);
    receipt.expect_success();
}
