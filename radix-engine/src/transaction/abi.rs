use scrypto::abi;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::engine::*;
use crate::ledger::*;
use crate::utils::*;

/// Export the ABI of a blueprint.
pub fn export_abi<T: Ledger>(
    ledger: &mut T,
    blueprint: (Address, String),
    trace: bool,
) -> Result<abi::Blueprint, RuntimeError> {
    let mut engine = InMemoryRadixEngine::new();
    let mut track = engine.start_transaction();

    // Load package code from file system
    track.put_package(
        blueprint.0,
        ledger
            .get_package(blueprint.0)
            .ok_or(RuntimeError::PackageNotFound(blueprint.0))?,
    );

    // Start a process and run abi generator
    let mut proc = track.start_process(trace);
    let output: (Vec<abi::Function>, Vec<abi::Method>) =
        proc.call_abi(blueprint.clone()).and_then(decode_return)?;

    Ok(abi::Blueprint {
        package: blueprint.0.to_string(),
        name: blueprint.1,
        functions: output.0,
        methods: output.1,
    })
}

/// Export the ABI of the blueprint of a component.
pub fn export_abi_by_component<T: Ledger>(
    ledger: &mut T,
    component: Address,
    trace: bool,
) -> Result<abi::Blueprint, RuntimeError> {
    let com = ledger
        .get_component(component)
        .ok_or(RuntimeError::ComponentNotFound(component))?;

    export_abi(ledger, com.blueprint().clone(), trace)
}
