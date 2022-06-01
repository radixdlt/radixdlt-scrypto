#[rustfmt::skip]
pub mod test_runner;

use sbor::Type;
use scrypto::abi::BlueprintAbi;
use scrypto::prelude::{HashMap, Package};
use radix_engine::{
    ledger::InMemorySubstateStore,
    transaction::{NonceProvider, TransactionBuilder, TransactionExecutor},
    wasm::{default_wasm_engine, InvokeError},
};
use scrypto::to_struct;
use test_runner::wat2wasm;

fn mocked_abi(blueprint_name: String) -> HashMap<String, BlueprintAbi> {
    let mut blueprint_abis = HashMap::new();
    blueprint_abis.insert(blueprint_name, BlueprintAbi {
        value: Type::Unit,
        methods: Vec::new(),
        functions: Vec::new(),
    });
    blueprint_abis
}

#[test]
fn test_loop() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000"));
    let package = Package {
        code,
        blueprints: mocked_abi("Test".to_string()),
    };
    let package_address = executor
        .publish_package(package)
        .expect("Failed to publish package");
    let transaction = TransactionBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor
        .validate_and_execute(&transaction)
        .expect("Failed to execute transaction");

    // Assert
    receipt.result.expect("It should work")
}

#[test]
fn test_loop_out_of_tbd() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);

    // Act
    let code = wat2wasm(&include_str!("wasm/loop.wat").replace("${n}", "2000000"));
    let package = Package {
        code,
        blueprints: mocked_abi("Test".to_string()),
    };
    let package_address = executor
        .publish_package(package)
        .expect("Failed to publish package");
    let transaction = TransactionBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor
        .validate_and_execute(&transaction)
        .expect("Failed to execute transaction");

    // Assert
    assert_invoke_error!(receipt.result, InvokeError::OutOfTbd { .. })
}

#[test]
fn test_recursion() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);

    // Act
    // In this test case, each call frame costs 4 stack units
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "128"));
    let package = Package {
        code,
        blueprints: mocked_abi("Test".to_string()),
    };
    let package_address = executor
        .publish_package(package)
        .expect("Failed to publish package");
    let transaction = TransactionBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor
        .validate_and_execute(&transaction)
        .expect("Failed to execute transaction");

    // Assert
    receipt.result.expect("It should work")
}

#[test]
fn test_recursion_stack_overflow() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);

    // Act
    let code = wat2wasm(&include_str!("wasm/recursion.wat").replace("${n}", "129"));
    let package = Package {
        code,
        blueprints: mocked_abi("Test".to_string()),
    };
    let package_address = executor
        .publish_package(package)
        .expect("Failed to publish package");
    let transaction = TransactionBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor
        .validate_and_execute(&transaction)
        .expect("Failed to execute transaction");

    // Assert
    assert_invoke_error!(receipt.result, InvokeError::WasmError { .. })
}

#[test]
fn test_grow_memory() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "99999"));
    let package = Package {
        code,
        blueprints: mocked_abi("Test".to_string()),
    };
    let package_address = executor
        .publish_package(package)
        .expect("Failed to publish package");
    let transaction = TransactionBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor
        .validate_and_execute(&transaction)
        .expect("Failed to execute transaction");

    // Assert
    receipt.result.expect("It should work")
}

#[test]
fn test_grow_memory_out_of_tbd() {
    // Arrange
    let mut substate_store = InMemorySubstateStore::new();
    let mut wasm_engine = default_wasm_engine();
    let mut executor = TransactionExecutor::new(&mut substate_store, &mut wasm_engine, true);

    // Act
    let code = wat2wasm(&include_str!("wasm/memory.wat").replace("${n}", "100000"));
    let package = Package {
        code,
        blueprints: mocked_abi("Test".to_string()),
    };
    let package_address = executor
        .publish_package(package)
        .expect("Failed to publish package");
    let transaction = TransactionBuilder::new()
        .call_function(package_address, "Test", "f", to_struct!())
        .build(executor.get_nonce([]))
        .sign([]);
    let receipt = executor
        .validate_and_execute(&transaction)
        .expect("Failed to execute transaction");

    // Assert
    assert_invoke_error!(receipt.result, InvokeError::OutOfTbd { .. })
}
