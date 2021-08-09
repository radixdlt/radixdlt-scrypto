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

fn build(path: &str) {
    Command::new("cargo")
        .current_dir(path)
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn publish<T: Ledger>(ledger: &mut T, path: &str) -> Address {
    let code = fs::read(format!(
        "{}/target/wasm32-unknown-unknown/release/source.wasm",
        path
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
    let output = one_shot(&mut ledger, "./tests/source", "Greeting", "new", vec![]);
    assert!(output.is_ok())
}

#[test]
fn test_blueprint() {
    let mut ledger = InMemoryLedger::new();
    let output = one_shot(
        &mut ledger,
        "./tests/source",
        "BlueprintTest",
        "publish",
        vec![],
    );
    assert!(output.is_ok());
    let address: Address = scrypto_decode(&output.unwrap()).unwrap();

    let output2 = one_shot(
        &mut ledger,
        "./tests/source",
        "BlueprintTest",
        "invoke",
        vec![scrypto_encode(&address)],
    );
    assert!(output2.is_ok());
}
