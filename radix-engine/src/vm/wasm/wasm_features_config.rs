use wasmparser::WasmFeatures;

#[derive(Debug, Clone)]
pub struct WasmFeaturesConfig {
    pub features: WasmFeatures,
}

impl Default for WasmFeaturesConfig {
    fn default() -> Self {
        Self {
            // Radix Engine supports MVP + proposals:
            // - mutable globals and sign-extension-ops
            features: WasmFeatures {
                mutable_global: true,
                sign_extension: true,
                reference_types: false,
                multi_value: false,
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

    pub fn set_reference_types(&mut self, value: bool) -> &mut Self {
        self.features.reference_types = value;
        self
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
