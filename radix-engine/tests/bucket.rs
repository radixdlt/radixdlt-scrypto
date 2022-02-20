use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

pub fn compile(name: &str) -> Vec<u8> {
    compile_package!(format!("./tests/{}", name), name)
}

#[test]
fn test_bucket() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("bucket")).unwrap();

    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "BucketTest", "combine", vec![], Some(account))
        .call_function(package, "BucketTest", "split", vec![], Some(account))
        .call_function(package, "BucketTest", "borrow", vec![], Some(account))
        .call_function(package, "BucketTest", "query", vec![], Some(account))
        .call_function(
            package,
            "BucketTest",
            "test_restricted_transfer",
            vec![],
            Some(account),
        )
        .call_function(package, "BucketTest", "test_burn", vec![], Some(account))
        .call_function(
            package,
            "BucketTest",
            "test_burn_freely",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    assert!(receipt.result.is_ok());
}

#[test]
fn test_bucket_of_badges() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("bucket")).unwrap();

    let transaction = TransactionBuilder::new(&executor)
        .call_function(package, "BadgeTest", "combine", vec![], Some(account))
        .call_function(package, "BadgeTest", "split", vec![], Some(account))
        .call_function(package, "BadgeTest", "borrow", vec![], Some(account))
        .call_function(package, "BadgeTest", "query", vec![], Some(account))
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    assert!(receipt.result.is_ok());
}
