use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

#[test]
fn test_tx() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let (pk, sk, account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "caller_auth_zone")))
        .unwrap();

    // Test the `instantiate_hello` function.
    let transaction1 = TransactionBuilder::new()
        .call_function(package, "Hello", "instantiate_hello", args![])
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());

    let component = receipt1.new_component_addresses[0];
    let transaction2 = TransactionBuilder::new()
        .call_method(component, "badge_for_alice", args![]) // returned Proof just goes to auth zone automatically (without using "push")
        .call_method(component, "free_token", args![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());

    let component = receipt1.new_component_addresses[0];
    let transaction2 = TransactionBuilder::new()
        .call_method(component, "badge_for_bob", args![]) // returned Proof just goes to auth zone automatically (without using "push")
        .call_method(component, "free_token", args![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());

    let component = receipt1.new_component_addresses[0];
    let transaction2 = TransactionBuilder::new()
        .call_method(component, "badge_for_both", args![]) // returned Proof just goes to auth zone automatically (without using "push")
        .call_method(component, "free_token", args![])
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(!receipt2.result.is_ok()); // fails because free_token does not like getting proof with both

    // test AuthZone::start
    let component = receipt1.new_component_addresses[0];
    let transaction2 = TransactionBuilder::new()
        .call_method(component, "badge_for_alice", args![]) // returned Proof just goes to auth zone automatically (without using "push")
        .start_auth_zone() // this pushes the authzone
        .call_method(component, "free_token", args![]) // fails since no badges
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(!receipt2.result.is_ok());

    // test AuthZone::start and AuthZone::end
    let component = receipt1.new_component_addresses[0];
    let transaction2 = TransactionBuilder::new()
        .call_method(component, "badge_for_alice", args![]) // returned Proof just goes to auth zone automatically (without using "push")
        .start_auth_zone() // this pushes the authzone
        .call_method(component, "badge_for_bob", args![]) // returned Proof just goes to auth zone automatically (without using "push")
        .call_method(component, "free_token", args![]) // get tokens for bob
        .end_auth_zone() // end authzone which drops the bob token proof
        .call_method(component, "free_token", args![]) // gets token with alice
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}

#[test]
fn test_component() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let (pk, sk, _account) = executor.new_account();
    let package = executor
        .publish_package(&compile_package!(format!("./tests/{}", "caller_auth_zone")))
        .unwrap();

    // Test the `instantiate_hello` function.
    let transaction1 = TransactionBuilder::new()
        .call_function(package, "Hello", "instantiate_hello", args![])
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt1.result.is_ok());
    let hello_component = receipt1.new_component_addresses[0];

    // Test the `instantiate_hello` function.
    let transaction1 = TransactionBuilder::new()
        .call_function(package, "Caller", "instantiate_caller", args![])
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction1).unwrap();
    println!("{:?}\n", receipt1);
    assert!(receipt2.result.is_ok());
    let component = receipt2.new_component_addresses[0];

    let transaction2 = TransactionBuilder::new()
        .call_method(component, "run_as_alice", args![hello_component]) // returned Proof just goes to auth zone automatically (without using "push")
        //.call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());

    let transaction2 = TransactionBuilder::new()
        .call_method(component, "run_as_both", args![hello_component]) // returned Proof just goes to auth zone automatically (without using "push")
        //.call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}
