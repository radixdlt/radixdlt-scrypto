use std::fs;
use std::process::Command;

use scrypto::buffer::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

extern crate radix_engine;
use radix_engine::execution::*;
use radix_engine::ledger::*;
use radix_engine::model::*;

fn build(name: &str) {
    Command::new("cargo")
        .current_dir(format!("./tests/{}", name))
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn publish<T: Ledger>(ledger: &mut T, name: &str) -> Address {
    let code = fs::read(format!(
        "./tests/{}/target/wasm32-unknown-unknown/release/{}.wasm",
        name, name
    ))
    .unwrap();

    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut runtime = Runtime::new(tx_hash, ledger);

    let address = runtime.new_package_address();
    load_module(&code).unwrap();
    runtime.put_package(address, Package::new(code));
    runtime.flush();

    address
}

fn build_and_publish<T: Ledger>(ledger: &mut T, name: &str) -> Address {
    build(name);
    publish(ledger, name)
}

fn call<T: Ledger>(
    ledger: &mut T,
    package: Address,
    blueprint: &str,
    function: &str,
    args: Vec<Vec<u8>>,
) -> Result<Vec<u8>, RuntimeError> {
    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut runtime = Runtime::new(tx_hash, ledger);

    let mut process = Process::new(0, true, &mut runtime);
    let result = process.run(
        package,
        format!("{}_main", blueprint),
        function.to_owned(),
        args,
    );
    process.finalize()?;

    if result.is_ok() {
        runtime.flush();
    }
    result
}

#[test]
fn test_greeting() {
    let mut ledger = InMemoryLedger::new();
    let package = build_and_publish(&mut ledger, "everything");

    let output = call(&mut ledger, package, "Greeting", "new", vec![]);
    assert!(output.is_ok())
}

#[test]
fn test_package() {
    let mut ledger = InMemoryLedger::new();
    let package = build_and_publish(&mut ledger, "everything");

    let output = call(&mut ledger, package, "PackageTest", "publish", vec![]);
    assert!(output.is_ok());
}

#[test]
fn test_blueprint() {
    let mut ledger = InMemoryLedger::new();
    let package = build_and_publish(&mut ledger, "everything");

    let output = call(
        &mut ledger,
        package,
        "BlueprintTest",
        "invoke_blueprint",
        vec![],
    );
    assert!(output.is_ok());
}

#[test]
fn test_component() {
    let mut ledger = InMemoryLedger::new();
    let package = build_and_publish(&mut ledger, "everything");

    let output = call(
        &mut ledger,
        package,
        "ComponentTest",
        "create_component",
        vec![],
    );
    assert!(output.is_ok());
    let address: Address = scrypto_decode(&output.unwrap()).unwrap();

    let output2 = call(
        &mut ledger,
        package,
        "ComponentTest",
        "get_component_info",
        vec![scrypto_encode(&address)],
    );
    assert!(output2.is_ok());

    let output3 = call(
        &mut ledger,
        package,
        "ComponentTest",
        "get_component_state",
        vec![scrypto_encode(&address)],
    );
    assert!(output3.is_ok());

    let output4 = call(
        &mut ledger,
        package,
        "ComponentTest",
        "put_component_state",
        vec![scrypto_encode(&address)],
    );
    assert!(output4.is_ok());
}

#[test]
fn test_resource() {
    let mut ledger = InMemoryLedger::new();
    let package = build_and_publish(&mut ledger, "everything");

    let output = call(
        &mut ledger,
        package,
        "ResourceTest",
        "create_mutable",
        vec![],
    );
    assert!(output.is_ok());

    let output2 = call(&mut ledger, package, "ResourceTest", "create_fixed", vec![]);
    assert!(output2.is_ok());

    let output3 = call(&mut ledger, package, "ResourceTest", "query", vec![]);
    assert!(output3.is_ok());
}

#[test]
fn test_bucket() {
    let mut ledger = InMemoryLedger::new();
    let package = build_and_publish(&mut ledger, "everything");

    let output = call(&mut ledger, package, "BucketTest", "combine", vec![]);
    assert!(output.is_ok());

    let output2 = call(&mut ledger, package, "BucketTest", "split", vec![]);
    assert!(output2.is_ok());

    let output3 = call(&mut ledger, package, "BucketTest", "borrow", vec![]);
    assert!(output3.is_ok());

    let output4 = call(&mut ledger, package, "BucketTest", "query", vec![]);
    assert!(output4.is_ok());
}

#[test]
fn test_account() {
    let mut ledger = InMemoryLedger::new();
    let package = build_and_publish(&mut ledger, "everything");

    let output = call(
        &mut ledger,
        package,
        "AccountTest",
        "deposit_and_withdraw",
        vec![],
    );
    assert!(output.is_ok());
}

#[test]
fn test_move_bucket() {
    let mut ledger = InMemoryLedger::new();
    let package = build_and_publish(&mut ledger, "everything");

    let output = call(&mut ledger, package, "MoveTest", "move_bucket", vec![]);
    assert!(output.is_ok());
}

#[test]
fn test_move_reference() {
    let mut ledger = InMemoryLedger::new();
    let package = build_and_publish(&mut ledger, "everything");

    let output = call(&mut ledger, package, "MoveTest", "move_reference", vec![]);
    assert!(output.is_ok());
}
