use std::fs;
use std::process::Command;

use sbor::collections::*;
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

    let address = runtime.new_blueprint_address(&code);
    load_module(&code).unwrap();
    runtime.put_blueprint(address, Blueprint::new(code));
    runtime.flush();

    address
}

fn call<T: Ledger>(
    ledger: &mut T,
    blueprint: Address,
    component: &str,
    method: &str,
    args: Vec<Vec<u8>>,
) -> Result<Vec<u8>, RuntimeError> {
    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut runtime = Runtime::new(tx_hash, ledger);
    let (module, memory) = runtime.load_module(blueprint).expect("Blueprint not found");

    let mut process = Process::new(
        &mut runtime,
        blueprint,
        component.to_string(),
        method.to_string(),
        args,
        0,
        &module,
        &memory,
        HashMap::new(),
        HashMap::new(),
    );
    let result = process.run();
    if result.is_ok() {
        runtime.flush();
    }
    result
}

fn one_shot<T: Ledger>(
    ledger: &mut T,
    path: &str,
    component: &str,
    method: &str,
    args: Vec<Vec<u8>>,
) -> Result<Vec<u8>, RuntimeError> {
    build(path);
    let blueprint = publish(ledger, path);
    call(ledger, blueprint, component, method, args)
}

#[test]
fn test_greeting() {
    let mut ledger = InMemoryLedger::new();
    let output = one_shot(&mut ledger, "everything", "Greeting", "new", vec![]);
    assert!(output.is_ok())
}

#[test]
fn test_blueprint() {
    let mut ledger = InMemoryLedger::new();
    let output = one_shot(
        &mut ledger,
        "everything",
        "BlueprintTest",
        "publish",
        vec![],
    );
    assert!(output.is_ok());
    let address: Address = scrypto_decode(&output.unwrap()).unwrap();

    let output2 = one_shot(
        &mut ledger,
        "everything",
        "BlueprintTest",
        "invoke",
        vec![scrypto_encode(&address)],
    );
    assert!(output2.is_ok());
}

#[test]
fn test_component() {
    let mut ledger = InMemoryLedger::new();
    let output = one_shot(
        &mut ledger,
        "everything",
        "ComponentTest",
        "create_component",
        vec![],
    );
    assert!(output.is_ok());
    let address: Address = scrypto_decode(&output.unwrap()).unwrap();

    let output2 = one_shot(
        &mut ledger,
        "everything",
        "ComponentTest",
        "get_component_info",
        vec![scrypto_encode(&address)],
    );
    assert!(output2.is_ok());

    let output3 = one_shot(
        &mut ledger,
        "everything",
        "ComponentTest",
        "get_component_state",
        vec![scrypto_encode(&address)],
    );
    assert!(output3.is_ok());

    let output4 = one_shot(
        &mut ledger,
        "everything",
        "ComponentTest",
        "put_component_state",
        vec![scrypto_encode(&address)],
    );
    assert!(output4.is_ok());
}

#[test]
fn test_resource() {
    let mut ledger = InMemoryLedger::new();
    let output = one_shot(&mut ledger, "everything", "ResourceTest", "create", vec![]);
    assert!(output.is_ok());

    let output2 = one_shot(&mut ledger, "everything", "ResourceTest", "query", vec![]);
    assert!(output2.is_ok());
}

#[test]
fn test_bucket() {
    let mut ledger = InMemoryLedger::new();
    let output = one_shot(&mut ledger, "everything", "BucketTest", "combine", vec![]);
    assert!(output.is_ok());

    let output2 = one_shot(&mut ledger, "everything", "BucketTest", "split", vec![]);
    assert!(output2.is_ok());

    let output3 = one_shot(&mut ledger, "everything", "BucketTest", "borrow", vec![]);
    assert!(output3.is_ok());

    let output4 = one_shot(&mut ledger, "everything", "BucketTest", "query", vec![]);
    assert!(output4.is_ok());
}

#[test]
fn test_account() {
    let mut ledger = InMemoryLedger::new();
    let output = one_shot(
        &mut ledger,
        "everything",
        "AccountTest",
        "deposit_and_withdraw",
        vec![],
    );
    assert!(output.is_ok());
}
