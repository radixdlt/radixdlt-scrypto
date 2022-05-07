#[rustfmt::skip]
pub mod test_runner;

//use crate::test_runner::TestRunner;
use radix_engine::ledger::*;
use radix_engine::transaction::*;
use scrypto::prelude::*;

macro_rules! setup {
    ($executor:expr) => {{
        // Set up environment.
        let executor = &mut $executor;
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

        ((pk, sk, account), package, component)
    }};
}

macro_rules! setup2 {
    ($executor:expr) => {{
        let ((pk, sk, account), package, hello_component) = setup!($executor);

        let executor = &mut $executor;

        // Test the `instantiate_hello` function.
        let transaction1 = TransactionBuilder::new()
            .call_function(package, "Caller", "instantiate_caller", args![])
            .build(executor.get_nonce([pk]))
            .sign([&sk]);
        let receipt2 = executor.validate_and_execute(&transaction1).unwrap();
        println!("{:?}\n", receipt2);
        assert!(receipt2.result.is_ok());
        let component = receipt2.new_component_addresses[0];

        ((pk, sk, account), hello_component, component)
    }};
}

#[test]
fn test_xfail_alice() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let ((pk, sk, account), _package, component) = setup!(executor);

    let transaction2 = TransactionBuilder::new()
        // fails in the default auth zone
        // .start_auth_zone()
        .call_method(component, "badge_for_alice", args![]) // returned Proof just goes to auth zone automatically (without using "push")
        .call_method(component, "free_token", args![]) // use the badge in the auth zone without knowing exactly what it is
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_err());
}

#[test]
fn test_alice() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let ((pk, sk, account), _package, component) = setup!(executor);

    let transaction2 = TransactionBuilder::new()
        .start_auth_zone() // enable caller auth zone by using the non-default auth zone (note this also segregates the tx signer badges)
        .call_method(component, "badge_for_alice", args![]) // returned Proof just goes to auth zone automatically (without using "push")
        .call_method(component, "free_token", args![]) // use the badge in the auth zone without knowing exactly what it is
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}

#[test]
fn test_bob() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let ((pk, sk, account), _package, component) = setup!(executor);

    let transaction2 = TransactionBuilder::new()
        .start_auth_zone() // enable caller auth zone by using the non-default auth zone (note this also segregates the tx signer badges)
        .call_method(component, "badge_for_bob", args![]) // returned Proof just goes to auth zone automatically (without using "push")
        .call_method(component, "free_token", args![]) // use the badge in the auth zone without knowing exactly what it is
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}

#[test]
fn test_xfail_both() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let ((pk, sk, account), _package, component) = setup!(executor);

    let transaction2 = TransactionBuilder::new()
        .start_auth_zone() // enable caller auth zone by using the non-default auth zone (note this also segregates the tx signer badges)
        .call_method(component, "badge_for_both", args![]) // returned Proof just goes to auth zone automatically (without using "push")
        .call_method(component, "free_token", args![]) // uses both badges, and the callee can decide if that's a problem or not (in this case free_token doesn't like it)
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_err()); // fails because free_token does not like getting proof with both
}

#[test]
fn test_xfail_start_auth_zone_is_empty() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let ((pk, sk, account), _package, component) = setup!(executor);

    let transaction2 = TransactionBuilder::new()
        .start_auth_zone() // enable caller auth zone by using the non-default auth zone (note this also segregates the tx signer badges)
        .call_method(component, "badge_for_alice", args![]) // returned Proof just goes to auth zone automatically (without using "push")
        .start_auth_zone() // this pushes the authzone and get's a new one
        .call_method(component, "free_token", args![]) // fails since no badges
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_err());
}

#[test]
fn test_alice_then_bob() {
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let ((pk, sk, account), _package, component) = setup!(executor);

    let transaction2 = TransactionBuilder::new()
        .start_auth_zone() // enable caller auth zone by using the non-default auth zone (note this also segregates the tx signer badges)
        .call_method(component, "badge_for_alice", args![]) // returned Proof just goes to auth zone automatically (without using "push")
        .start_auth_zone() // this pushes the authzone, getting a new one and leaving alice's badge in the pushed zone
        .call_method(component, "badge_for_bob", args![]) // returned Proof just goes to auth zone automatically (without using "push")
        .call_method(component, "free_token", args![]) // get tokens for bob (no conflict with alice's badge)
        .end_auth_zone() // end authzone which drops the bob token proof, alice's badge is now available
        .call_method(component, "free_token", args![]) // free tokens ok authorized with alice's badge
        // elided the end_auth-zone() calls as they are optional
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}

#[test]
fn test_component_as_alice() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let ((pk, sk, _account), hello_component, component) = setup2!(executor);

    let transaction2 = TransactionBuilder::new()
        .call_method(component, "run_as_alice", args![hello_component]) // returned Proof just goes to auth zone automatically (without using "push")
        //.call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}

#[test]
fn test_component_xfail_as_alice() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let ((pk, sk, _account), hello_component, component) = setup2!(executor);

    let transaction2 = TransactionBuilder::new()
        .call_method(component, "xfail_run_as_alice", args![hello_component]) // returned Proof just goes to auth zone automatically (without using "push")
        //.call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_err())
}

#[test]
fn test_component_as_both() {
    // Set up environment.
    let mut ledger = InMemorySubstateStore::with_bootstrap();
    let mut executor = TransactionExecutor::new(&mut ledger, false);
    let ((pk, sk, _account), hello_component, component) = setup2!(executor);

    let transaction2 = TransactionBuilder::new()
        .call_method(component, "run_as_both", args![hello_component]) // returned Proof just goes to auth zone automatically (without using "push")
        //.call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    assert!(receipt2.result.is_ok());
}
