use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

pub fn compile(name: &str) -> Vec<u8> {
    compile_package!(format!("./tests/{}", name), name.replace("-", "_"))
}

#[test]
fn test_non_fungible() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, true);
    let key = executor.new_public_key();
    let account = executor.new_account(key);
    let package = executor.publish_package(&compile("non_fungible")).unwrap();

    let transaction = TransactionBuilder::new(&executor)
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_mutable",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_fixed",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "NonFungibleTest",
            "update_and_get_non_fungible",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "NonFungibleTest",
            "non_fungible_exists",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "NonFungibleTest",
            "take_and_put_bucket",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "NonFungibleTest",
            "take_and_put_vault",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "NonFungibleTest",
            "get_non_fungible_ids_bucket",
            vec![],
            Some(account),
        )
        .call_function(
            package,
            "NonFungibleTest",
            "get_non_fungible_ids_vault",
            vec![],
            Some(account),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build(vec![key])
        .unwrap();
    let receipt = executor.run(transaction).unwrap();
    println!("{:?}", receipt);
    assert!(receipt.result.is_ok());
}
