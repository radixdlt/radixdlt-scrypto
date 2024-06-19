use std::path::PathBuf;
use radix_engine_interface::blueprints::package::PackageDefinition;
use scrypto_compiler::*;
use proc_lock::proc_lock;

#[proc_lock(name = "build_for_fuzzing.lock")]
pub fn build_for_fuzzing(path: PathBuf) -> (Vec<u8>, PackageDefinition) {
    let mut compiler_builder = ScryptoCompiler::builder();
    compiler_builder
        .manifest_path(path)
        .optimize_with_wasm_opt(None);

    let flags = vec![
        "-Cllvm-args=-sanitizer-coverage-inline-8bit-counters",
        "-Cpasses=sancov-module",
        "-Cllvm-args=-sanitizer-coverage-level=3",
    ];

    compiler_builder.env("CARGO_ENCODED_RUSTFLAGS", EnvironmentVariableAction::Set(flags.join("\x1f")));

    let build_results = compiler_builder
        .compile()
        .unwrap()
        .pop()
        .unwrap();
    (build_results.wasm.content, build_results.package_definition.content)    
}