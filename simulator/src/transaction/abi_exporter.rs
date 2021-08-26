use radix_engine::execution::*;
use radix_engine::ledger::*;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::types::*;
use scrypto::utils::*;

use crate::ledger::*;
use crate::transaction::*;

/// Export the ABI of a blueprint.
pub fn export_abi(
    package: Address,
    blueprint: &str,
    trace: bool,
) -> Result<abi::Blueprint, RuntimeError> {
    let tx_hash = sha256(""); // fixed tx hash for determinism
    let mut ledger = InMemoryLedger::new();
    let mut runtime = Runtime::new(tx_hash, &mut ledger);

    // Load package code from file system
    runtime.put_package(
        package,
        FileBasedLedger::new(get_data_dir())
            .get_package(package)
            .ok_or(RuntimeError::PackageNotFound(package))?,
    );

    // Start a process and run abi generator
    let mut process = Process::new(0, trace, &mut runtime);
    let result = process.run(package, format!("{}_abi", blueprint), String::new(), vec![]);

    // Parse ABI
    scrypto_decode::<abi::Blueprint>(&result?).map_err(|e| RuntimeError::InvalidData(e))
}

/// Export the ABI of the blueprint of which the given component is instantiated.
pub fn export_abi_by_component(
    component: Address,
    trace: bool,
) -> Result<abi::Blueprint, RuntimeError> {
    let com = FileBasedLedger::new(get_data_dir())
        .get_component(component)
        .ok_or(RuntimeError::ComponentNotFound(component))?;

    export_abi(com.package(), com.blueprint(), trace)
}
