use radix_engine::execution::*;
use radix_engine::ledger::*;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::types::rust::collections::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::ledger::*;

pub fn invoke(
    package: Address,
    blueprint: &str,
    function: &str,
    args: Vec<Vec<u8>>,
    trace: bool,
) -> (Result<Vec<u8>, RuntimeError>, Vec<(Level, String)>) {
    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_data_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);

    if trace {
        println!("----");
        println!("Package: {}", package);
        println!("Blueprint: {}", blueprint);
        println!("Function: {}", function);
        println!("Arguments: {:02x?}", args);
        println!("----");
    }

    let (module, memory) = runtime.load_module(package).expect("Package not found");
    let mut process = Process::new(
        trace,
        &mut runtime,
        package,
        format!("{}_main", blueprint),
        function.to_owned(),
        args,
        0,
        &module,
        &memory,
        HashMap::new(),
        HashMap::new(),
    );
    let output = process.run();
    if output.is_ok() {
        runtime.flush();
    }

    (output, runtime.logs().to_owned())
}

pub fn get_abi(
    package: Address,
    blueprint: &str,
    trace: bool,
) -> Result<abi::Blueprint, RuntimeError> {
    let tx_hash = sha256("");
    let mut ledger = InMemoryLedger::new();
    ledger.put_package(
        package,
        FileBasedLedger::new(get_data_dir())
            .get_package(package)
            .expect("Package not found"),
    );
    let mut runtime = Runtime::new(tx_hash, &mut ledger);
    let (module, memory) = runtime.load_module(package).unwrap();

    let mut process = Process::new(
        trace,
        &mut runtime,
        package,
        format!("{}_abi", blueprint),
        String::new(),
        vec![],
        0,
        &module,
        &memory,
        HashMap::new(),
        HashMap::new(),
    );
    let output = process.run();
    let abi: abi::Blueprint = scrypto_decode(&output?).unwrap();
    Ok(abi)
}
