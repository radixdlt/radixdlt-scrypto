use wasmparser::WasmFeatures;

#[derive(Debug, Clone)]
pub struct WasmFeaturesConfig {
    pub features: WasmFeatures,
}

impl Default for WasmFeaturesConfig {
    fn default() -> Self {
        Self {
            // The Radix Engine supports the MVP and additional proposals, specifically:
            // - mutable-globals and sign-extension-ops
            // - reference-types and multi-value returns
            //   These features have been enabled by default in LLVM version 19, which Rust has used starting from version 1.82.
            //   To ensure compatibility with Rust versions 1.82 and later, both features must be enabled.
            //
            //   - Reference types
            //     Enabling this is safe since Rust does not support `externref`.
            //     Regarding indirect function calls—which are common in Rust code—there is no issue with the 5-byte LEB128 encoding.
            //     The proposal has been around for a while, and WebAssembly engines, including `wasmi`, have had sufficient time to stabilize it.
            //   - Multi-value returns
            //     This is also safe to enable because the Radix Engine has never used the Nightly `extern "wasm"` feature.
            //   You can find more details here: https://blog.rust-lang.org/2024/09/24/webassembly-targets-change-in-default-target-features.html
            features: WasmFeatures {
                mutable_global: true,
                sign_extension: true,
                reference_types: true,
                multi_value: true,
                saturating_float_to_int: false,
                bulk_memory: false,
                simd: false,
                relaxed_simd: false,
                threads: false,
                tail_call: false,
                floats: false,
                multi_memory: false,
                exceptions: false,
                memory64: false,
                extended_const: false,
                component_model: false,
                function_references: false,
                memory_control: false,
                gc: false,
            },
        }
    }
}

macro_rules! cflag {
    ($flag:expr, $name:expr) => {
        if $flag {
            concat!(" -m", $name)
        } else {
            concat!(" -mno-", $name)
        }
    };
}

impl WasmFeaturesConfig {
    pub fn new() -> Self {
        Self::default()
    }

    // More on CFLAGS for WASM:  https://clang.llvm.org/docs/ClangCommandLineReference.html#webassembly
    pub fn into_target_cflags(&self) -> String {
        let mut cflags = String::from("-mcpu=mvp");
        // Assuming that remaining options have sensible defaults
        cflags += cflag!(self.features.mutable_global, "mutable-globals");
        cflags += cflag!(self.features.sign_extension, "sign-ext");
        cflags += cflag!(self.features.reference_types, "reference-types");
        cflags += cflag!(self.features.multi_value, "multivalue");

        cflags
    }
}
