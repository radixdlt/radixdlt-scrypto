use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

pub fn compile(name: &str) -> Vec<u8> {
    compile_package!(format!("./tests/{}", name), name.replace("-", "_"))
}

#[test]
fn test_bucket() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (_, account) = executor.new_public_key_with_account();
    let package = executor.publish_package(&compile("bucket")).unwrap();

    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "BucketTest", "combine", vec![])
        .call_function(package, "BucketTest", "split", vec![])
        .call_function(package, "BucketTest", "borrow", vec![])
        .call_function(package, "BucketTest", "query", vec![])
        .call_function(package, "BucketTest", "test_restricted_transfer", vec![])
        .call_function(package, "BucketTest", "test_burn", vec![])
        .call_function(package, "BucketTest", "test_burn_freely", vec![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    assert!(receipt.result.is_ok());
}

#[test]
fn test_bucket_of_badges() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let (_, account) = executor.new_public_key_with_account();
    let package = executor.publish_package(&compile("bucket")).unwrap();

    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "BadgeTest", "combine", vec![])
        .call_function(package, "BadgeTest", "split", vec![])
        .call_function(package, "BadgeTest", "borrow", vec![])
        .call_function(package, "BadgeTest", "query", vec![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    assert!(receipt.result.is_ok());
}
