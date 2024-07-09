//! A module with the logic for executing the various scenarios with the various configurations.

use crate::configuration::*;
use radix_engine::vm::wasm::*;
use radix_engine::vm::*;
use scrypto_compiler::*;
use scrypto_test::prelude::*;
use std::fs::File;
use std::path::*;
use tempfile::tempfile;

pub type ScenarioFunction =
    fn(&mut DefaultLedgerSimulator, &mut PackageLoader) -> (TransactionManifestV1, Vec<PublicKey>);

pub fn execute_scenarios_with_configurations(
    configurations: Vec<(
        ScryptoVm<Box<dyn WasmEngine<WasmInstance = Box<dyn WasmInstance>>>>,
        CompilationFeatures,
        ConfigurationDescriptor,
    )>,
    scenarios: Vec<(&'static str, ScenarioFunction)>,
    sample_size: usize,
) -> IndexMap<ConfigurationDescriptor, IndexMap<&'static str, ExecutionReport>> {
    // The results from the tests
    let mut results =
        IndexMap::<ConfigurationDescriptor, IndexMap<&'static str, ExecutionReport>>::new();

    // There's a single ledger that is shared across all of the tests.
    let mut ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();

    // There is a single PackageLoader we're using to have cached packages.
    let mut package_loader = PackageLoader::new();

    // Iterate over each of the configurations to prepare for running the scenarios with them.
    for (scrypto_vm, compilation_features, configuration_descriptor) in configurations.into_iter() {
        println!("Current config: {configuration_descriptor:?}");

        // Update the package loader with the compilation features.
        package_loader.set_compilation_features(compilation_features);

        // Iterating over the scenario functions passed.
        for (scenario_name, scenario_function) in scenarios.iter() {
            println!("\tCurrent scenario: {}", scenario_name);

            // Calling the scenario function with the ledger and the package loader.
            let (manifest, signers) = scenario_function(&mut ledger, &mut package_loader);

            // Previewing the manifest _n_ times, timing the overall execution, and determining the
            // average execution time of this manifest.
            let (mut receipts, execution_time) = execution_time(|| {
                (0..sample_size)
                    .map(|_| {
                        preview_manifest_with_debug_mode_enabled(
                            &ledger,
                            manifest.clone(),
                            signers.clone(),
                            &scrypto_vm,
                        )
                    })
                    .collect::<Vec<_>>()
            });
            let receipt = receipts.pop().unwrap();
            receipt.expect_commit_success();

            // Divide the total execution time by the number of times the preview was done. This
            // gives us the average execution time.
            let average_execution_time_in_nano_seconds =
                Decimal::from(execution_time.as_nanos()) / Decimal::from(sample_size);

            results.entry(configuration_descriptor).or_default().insert(
                scenario_name,
                ExecutionReport {
                    manifest,
                    // detailed_execution_cost_breakdown: receipt
                    //     .debug_information
                    //     .unwrap()
                    //     .detailed_execution_cost_breakdown,
                    fee_summary: receipt.fee_summary,
                    average_execution_time_in_nano_seconds,
                },
            );
        }
    }

    results
}

fn preview_manifest_with_debug_mode_enabled(
    ledger: &DefaultLedgerSimulator,
    manifest: TransactionManifestV1,
    signers: Vec<PublicKey>,
    scrypto_vm: &ScryptoVm<Box<dyn WasmEngine<WasmInstance = Box<dyn WasmInstance>>>>,
) -> TransactionReceipt {
    let vm_init = VmInit {
        scrypto_vm: &scrypto_vm,
        native_vm_extension: NoExtension,
    };

    let execution_config = ExecutionConfig {
        enable_debug_information: true,
        ..ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator())
    };

    let nonce = u32::MAX;
    let transaction = TestTransaction::new_from_nonce(manifest, nonce);
    let prepared = transaction.prepare().expect("Must succeed");
    let executable = prepared.get_executable(
        signers
            .into_iter()
            .map(|pk| NonFungibleGlobalId::from_public_key(&pk))
            .collect(),
    );

    execute_transaction(
        ledger.substate_db(),
        vm_init,
        &execution_config,
        &executable,
    )
}

#[derive(Debug)]
pub struct ExecutionReport {
    pub manifest: TransactionManifestV1,
    // pub detailed_execution_cost_breakdown: Vec<DetailedExecutionCostBreakdownEntry>,
    pub fee_summary: TransactionFeeSummary,
    pub average_execution_time_in_nano_seconds: Decimal,
}

pub struct PackageLoader {
    /// The compilation features enabled.
    compilation_features: CompilationFeatures,
    /// A cache of the compiled packages, this is kept so that if a package has already been
    /// compiled with the same compilation features then it will not be compiled again. The cache is
    /// keyed by the manifest path and the compilation features.
    cache: HashMap<(PathBuf, CompilationFeatures), (Vec<u8>, PackageDefinition)>,
    /// A temporary file that all of the stdout and stderr from the compilation will be written to
    /// in order to minimize the noise when this is running.
    sink: File,
}

impl PackageLoader {
    pub fn new() -> Self {
        Self {
            compilation_features: CompilationFeatures {
                decimal_in_engine: false,
            },
            cache: Default::default(),
            sink: tempfile().unwrap(),
        }
    }

    fn set_compilation_features(&mut self, compilation_features: CompilationFeatures) {
        self.compilation_features = compilation_features
    }

    pub fn get(&mut self, manifest_path: &Path) -> (Vec<u8>, PackageDefinition) {
        // Check the cache first to determine if there is an entry there or not, if there is not
        // then proceed to compile.
        match self
            .cache
            .get(&(manifest_path.to_owned(), self.compilation_features))
        {
            Some((wasm, definition)) => (wasm.clone(), definition.clone()),
            None => {
                // Compile
                let mut compiler = self.construct_compiler(manifest_path);
                let BuildArtifacts {
                    wasm: BuildArtifact { content: wasm, .. },
                    package_definition:
                        BuildArtifact {
                            content: package_definition,
                            ..
                        },
                } = compiler
                    .compile_with_stdio(
                        None,
                        Some(self.sink.try_clone().unwrap()),
                        Some(self.sink.try_clone().unwrap()),
                    )
                    .unwrap()
                    .pop()
                    .unwrap();

                // Insert into the cache.
                self.cache.insert(
                    (manifest_path.to_owned(), self.compilation_features),
                    (wasm.clone(), package_definition.clone()),
                );

                // Return
                (wasm, package_definition)
            }
        }
    }

    fn construct_compiler(&self, manifest_path: &Path) -> ScryptoCompiler {
        let mut compiler = ScryptoCompiler::builder();
        if self.compilation_features.decimal_in_engine {
            compiler.feature("scrypto/outsource-decimal-arithmetic-to-engine");
        }
        compiler.manifest_path(manifest_path).build().unwrap()
    }
}

impl Default for PackageLoader {
    fn default() -> Self {
        Self::new()
    }
}

fn execution_time<F, O>(callback: F) -> (O, std::time::Duration)
where
    F: FnOnce() -> O,
{
    let now = std::time::Instant::now();
    let rtn = callback();
    let elapsed = now.elapsed();
    (rtn, elapsed)
}
