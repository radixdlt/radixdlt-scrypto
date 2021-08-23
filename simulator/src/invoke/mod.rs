use colored::*;
use radix_engine::execution::*;
use radix_engine::ledger::*;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::types::rust::collections::*;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::ledger::*;

pub struct TransactionReceipt {
    pub result: Result<Vec<u8>, RuntimeError>,
    pub logs: Vec<(Level, String)>,
}

pub fn invoke_blueprint(
    package: Address,
    blueprint: &str,
    function: &str,
    args: &Vec<Vec<u8>>,
    trace: bool,
) -> Result<TransactionReceipt, RuntimeError> {
    let tx_hash = sha256(Uuid::new_v4().to_string());
    let mut ledger = FileBasedLedger::new(get_data_dir());
    let mut runtime = Runtime::new(tx_hash, &mut ledger);

    let (module, memory) = runtime
        .load_module(package)
        .ok_or(RuntimeError::PackageNotFound(package))?;
    let mut process = Process::new(
        trace,
        &mut runtime,
        package,
        format!("{}_main", blueprint),
        function.to_owned(),
        args.clone(),
        0,
        &module,
        &memory,
        HashMap::new(),
        HashMap::new(),
    );
    let result = process.run();

    // flush changes if success
    if result.is_ok() {
        runtime.flush();
    }

    Ok(TransactionReceipt {
        result,
        logs: runtime.logs().to_owned(),
    })
}

pub fn invoke_component(
    component: Address,
    method: &str,
    args: &Vec<Vec<u8>>,
    trace: bool,
) -> Result<TransactionReceipt, RuntimeError> {
    let ledger = FileBasedLedger::new(get_data_dir());
    let com = ledger
        .get_component(component)
        .ok_or(RuntimeError::ComponentNotFound(component))?;
    let package = com.package();
    let blueprint = com.name();

    let mut self_and_args = args.clone();
    self_and_args.insert(0, scrypto_encode(&component)); // self

    invoke_blueprint(package, blueprint, method, &self_and_args, trace)
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
            .ok_or(RuntimeError::PackageNotFound(package))?,
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
    scrypto_decode::<abi::Blueprint>(&output?).map_err(|e| RuntimeError::InvalidSborValue(e))
}

pub fn print_logs(logs: &Vec<(Level, String)>) {
    for (level, msg) in logs {
        let (l, m) = match level {
            Level::Error => ("ERROR".red(), msg.red()),
            Level::Warn => ("WARN".yellow(), msg.yellow()),
            Level::Info => ("INFO".green(), msg.green()),
            Level::Debug => ("DEBUG".cyan(), msg.cyan()),
            Level::Trace => ("TRACE".normal(), msg.normal()),
        };
        println!("[{:5}] {}", l, m);
    }
}
