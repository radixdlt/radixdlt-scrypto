mod configuration;
mod executor;
mod scenarios;
mod wasm_engines;

use crate::configuration::*;
use crate::executor::*;
use crate::scenarios::*;
use crate::wasm_engines::cache::*;
use crate::wasm_engines::wasmer_v2::engine::WasmerV2Engine;
use crate::wasm_engines::wasmer_v2::module::WasmerV2Module;
use crate::wasm_engines::wasmi::engine::WasmiEngine;
use crate::wasm_engines::wasmi::module::WasmiModule;
use clap::Parser;
use radix_engine::vm::wasm::*;
use radix_engine::vm::*;
use wasmer_compiler_cranelift::Cranelift;
use wasmer_compiler_singlepass::Singlepass;

macro_rules! define_configuration_and_scenario_functions {
    (
        configurations: [
            $($config: expr),* $(,)?
        ],
        scenario_functions: [
            $($func: expr),* $(,)?
        ] $(,)?
    ) => {
        // Define the configurations function.
        fn all_configurations() -> Vec<(
            ScryptoVm<Box<dyn WasmEngine<WasmInstance = Box<dyn WasmInstance>>>>,
            CompilationFeatures,
            ConfigurationDescriptor,
        )> {
            vec![
                $(
                    {
                        let config = $config;
                        let descriptor = config.configuration_descriptor();
                        let features = config.compilation_features;
                        let scrypto_vm = config.scrypto_vm();

                        (scrypto_vm, features, descriptor)
                    }
                ),*
            ]
        }

        // Define a function that returns all of the test functions.
        fn all_scenario_functions() -> Vec<(
            &'static str,
            ScenarioFunction
        )> {
            vec![
                $(
                    (
                        stringify!($func),
                        $func
                    )
                ),*
            ]
        }
    };
}

define_configuration_and_scenario_functions! {
    configurations: [
        // WASMI with different caches
        Configuration {
            wasm_engine: WasmiEngine::<NoCache<WasmiModule>>::default(),
            compilation_features: CompilationFeatures {
                decimal_in_engine: false
            }
        },
        // Configuration {
        //     wasm_engine: WasmiEngine::<MokaModuleCache<WasmiModule>>::default(),
        //     compilation_features: CompilationFeatures {
        //         decimal_in_engine: false
        //     }
        // },
        // Configuration {
        //     wasm_engine: WasmiEngine::<LruModuleCache<WasmiModule>>::default(),
        //     compilation_features: CompilationFeatures {
        //         decimal_in_engine: false
        //     }
        // },
        // Wasmer with different caches and compilers
        // Configuration {
        //     wasm_engine: WasmerV2Engine::<NoCache<WasmerV2Module>, Singlepass>::default(),
        //     compilation_features: CompilationFeatures {
        //         decimal_in_engine: false
        //     }
        // },
        // Configuration {
        //     wasm_engine: WasmerV2Engine::<MokaModuleCache<WasmerV2Module>, Singlepass>::default(),
        //     compilation_features: CompilationFeatures {
        //         decimal_in_engine: false
        //     }
        // },
        // Configuration {
        //     wasm_engine: WasmerV2Engine::<LruModuleCache<WasmerV2Module>, Singlepass>::default(),
        //     compilation_features: CompilationFeatures {
        //         decimal_in_engine: false
        //     }
        // },
        // Configuration {
        //     wasm_engine: WasmerV2Engine::<NoCache<WasmerV2Module>, Cranelift>::default(),
        //     compilation_features: CompilationFeatures {
        //         decimal_in_engine: false
        //     }
        // },
        // Configuration {
        //     wasm_engine: WasmerV2Engine::<MokaModuleCache<WasmerV2Module>, Cranelift>::default(),
        //     compilation_features: CompilationFeatures {
        //         decimal_in_engine: false
        //     }
        // },
        // Configuration {
        //     wasm_engine: WasmerV2Engine::<LruModuleCache<WasmerV2Module>, Cranelift>::default(),
        //     compilation_features: CompilationFeatures {
        //         decimal_in_engine: false
        //     }
        // },
    ],
    scenario_functions: [
        /* Faucet */
        faucet_lock_fee,
        faucet_lock_fee_and_free_xrd,
        /* Radiswap */
        radiswap_publish_package,
        radiswap_create_pool,
        radiswap_add_liquidity,
        radiswap_remove_liquidity,
        radiswap_single_swap,
        radiswap_two_swaps,
        /* Ignition */
        ignition_caviarnine_v1_open_position,
        ignition_ociswap_v1_open_position,
        ignition_ociswap_v2_open_position,
        ignition_defiplaza_v2_open_position,
        ignition_caviarnine_v1_close_position,
        ignition_ociswap_v1_close_position,
        ignition_ociswap_v2_close_position,
        ignition_defiplaza_v2_close_position,
    ]
}

/// Executes the scenarios with the various defined configurations and prints out a final analysis
/// report in the end.
#[derive(Parser, Debug, Clone)]
pub struct Cli {
    #[clap(short, long, default_value_t = 1)]
    pub sample_size: usize,
}

fn main() {
    let Cli { sample_size } = Cli::parse();

    let configurations = all_configurations();
    let all_functions = all_scenario_functions();

    let results = execute_scenarios_with_configurations(configurations, all_functions, sample_size);

    println!("{results:#?}")
}
