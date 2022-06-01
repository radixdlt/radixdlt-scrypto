use radix_engine::ledger::*;
use radix_engine::model::extract_package;
use radix_engine::transaction::*;
use radix_engine::wasm::default_wasm_engine;
use scrypto::prelude::*;
use scrypto::to_struct;

#[test]
fn test_hello() {
    // Set up environment.
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, false);
    let (pk, sk, account) = executor.new_account();
    let package = extract_package(compile_package!()).unwrap();
    let package_address = executor.publish_package(package).unwrap();

    // Test the `instantiate_hello` function.
    let transaction1 = TransactionBuilder::new()
        .call_function(package_address, "Hello", "instantiate_hello", to_struct!())
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt1 = executor.validate_and_execute(&transaction1).unwrap();
    println!("{:?}\n", receipt1);
    receipt1.result.expect("Should be okay.");

    // Test the `free_token` method.
    let component = receipt1.new_component_addresses[0];
    let transaction2 = TransactionBuilder::new()
        .call_method(component, "free_token", to_struct!())
        .call_method_with_all_resources(account, "deposit_batch")
        .build(executor.get_nonce([pk]))
        .sign([&sk]);
    let receipt2 = executor.validate_and_execute(&transaction2).unwrap();
    println!("{:?}\n", receipt2);
    receipt2.result.expect("Should be okay.");
}
