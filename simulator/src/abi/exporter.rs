use radix_engine::engine::*;
use radix_engine::execution::*;
use radix_engine::ledger::*;
use scrypto::abi;
use scrypto::types::*;

/// Export the ABI of a blueprint.
pub fn export_abi<T: Ledger>(
    ledger: &mut T,
    package: Address,
    blueprint: &str,
    trace: bool,
) -> Result<abi::Blueprint, RuntimeError> {
    let mut engine = InMemoryRadixEngine::new();
    let mut runtime = engine.start_transaction();

    // Load package code from file system
    runtime.put_package(
        package,
        ledger
            .get_package(package)
            .ok_or(RuntimeError::PackageNotFound(package))?,
    );

    // Start a process and run abi generator
    let mut proc = runtime.start_process(trace);
    let output: (Vec<abi::Function>, Vec<abi::Method>) =
        proc.call_abi(package, blueprint).and_then(decode_return)?;

    Ok(abi::Blueprint {
        package: package.to_string(),
        blueprint: blueprint.to_owned(),
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

    export_abi(ledger, com.package(), com.blueprint(), trace)
}
