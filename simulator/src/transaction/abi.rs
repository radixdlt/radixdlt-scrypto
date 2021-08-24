use radix_engine::execution::*;
use radix_engine::ledger::*;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::types::*;
use scrypto::utils::*;

use crate::ledger::*;
use crate::transaction::*;

pub fn get_abi(
    package: Address,
    blueprint: &str,
    trace: bool,
) -> Result<abi::Blueprint, TransactionError> {
    let tx_hash = sha256(""); // fixed tx hash for determinism
    let mut ledger = InMemoryLedger::new();
    let mut runtime = Runtime::new(tx_hash, &mut ledger);

    // Load package code from file system
    runtime.put_package(
        package,
        FileBasedLedger::new(get_data_dir())
            .get_package(package)
            .ok_or(TransactionError::PackageNotFound(package))?,
    );

    // Start a process and run abi generator
    let mut process = Process::new(0, trace, &mut runtime);
    let result = process.run(package, format!("{}_abi", blueprint), String::new(), vec![]);

    // Parse ABI
    let output = result.map_err(|e| TransactionError::FailedToExportAbi(e))?;
    scrypto_decode::<abi::Blueprint>(&output).map_err(|e| TransactionError::FailedToParseAbi(e))
}
