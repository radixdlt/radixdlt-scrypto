use crate::vm::ScryptoVmVersion;
use radix_rust::prelude::String;
use wasmparser::WasmFeatures;

macro_rules! cflag {
    ($flag:expr, $name:expr) => {
        if $flag {
            concat!(" -m", $name)
        } else {
            concat!(" -mno-", $name)
        }
    };
}

#[derive(Debug, Clone)]
pub struct WasmFeaturesConfig {
    pub features: WasmFeatures,
}

impl Default for WasmFeaturesConfig {
    fn default() -> Self {
        Self::latest()
    }
}

// The Radix Engine supports the WASM MVP and below proposals:
// - since Babylon:
//   a) mutable-globals
//   b) sign-extension-ops
// - since Dugong
//   a) mutable-globals
//   b) sign-extension-ops
//   c) reference-types
//      Enabling this is safe since Rust does not support `externref`.
//      Regarding indirect function calls—which are common in Rust code—there is no issue with the 5-byte LEB128 encoding.
//      The proposal has been around for a while and WebAssembly engines, including `wasmi`, have had sufficient time to stabilize it.
//   d) multi-value
//      This is also safe to enable because the Radix Engine has never used the Nightly `extern "wasm"` feature.
//
//   New features have been enabled by default in LLVM version 19, which Rust has used starting from version 1.82.
//   To ensure compatibility with Rust versions 1.82 and later, both features must be enabled.
impl WasmFeaturesConfig {
    pub const fn mvp() -> Self {
        Self {
            features: WasmFeatures {
                mutable_global: false,
                sign_extension: false,
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

    pub const fn set_mutable_global(mut self, enabled: bool) -> Self {
        self.features.mutable_global = enabled;
        self
    }

    pub const fn set_sign_extension(mut self, enabled: bool) -> Self {
        self.features.sign_extension = enabled;
        self
    }

    pub const fn set_reference_types(mut self, enabled: bool) -> Self {
        self.features.reference_types = enabled;
        self
    }

    pub const fn set_multi_value(mut self, enabled: bool) -> Self {
        self.features.multi_value = enabled;
        self
    }

    pub const fn babylon_genesis() -> Self {
        Self::mvp()
            .set_mutable_global(true)
            .set_sign_extension(true)
    }

    pub const fn anemone() -> Self {
        Self::babylon_genesis()
    }

    pub const fn bottlenose() -> Self {
        Self::anemone()
    }

    pub const fn cuttlefish() -> Self {
        Self::bottlenose()
    }

    pub const fn dugong() -> Self {
        Self::cuttlefish()
            .set_reference_types(true)
            .set_multi_value(true)
    }

    pub const fn latest() -> Self {
        Self::dugong()
    }

    pub const fn from_scrypto_vm_version(version: ScryptoVmVersion) -> Self {
        match version {
            ScryptoVmVersion::V1_0 => Self::babylon_genesis(),
            ScryptoVmVersion::V1_1 => Self::anemone(),
            ScryptoVmVersion::V1_2 => Self::cuttlefish(),
            ScryptoVmVersion::V1_3 => Self::dugong(),
        }
    }

    // More on CFLAGS for WASM: https://clang.llvm.org/docs/ClangCommandLineReference.html#webassembly
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
