extern crate radix_wasm_instrument as wasm_instrument;

use crate::internal_prelude::*;
use crate::vm::wasm::{constants::*, errors::*, PrepareError};
use num_traits::CheckedAdd;
use radix_engine_interface::blueprints::package::BlueprintDefinitionInit;
use syn::Ident;
use wasm_instrument::{
    gas_metering::{self, Rules},
    inject_stack_limiter,
    utils::module_info::ModuleInfo,
};
use wasmparser::{ExternalKind, FuncType, Operator, Type, TypeRef, ValType, WasmFeatures};

use super::WasmiModule;
use crate::vm::ScryptoVmVersion;

#[derive(Debug)]
pub struct WasmModule {
    module: ModuleInfo,
}

impl WasmModule {
    pub fn init(code: &[u8]) -> Result<Self, PrepareError> {
        // deserialize
        let module = ModuleInfo::new(code).map_err(|_| PrepareError::DeserializationError)?;

        // Radix Engine supports MVP + proposals: mutable globals and sign-extension-ops
        let features = WasmFeatures {
            mutable_global: true,
            saturating_float_to_int: false,
            sign_extension: true,
            reference_types: false,
            multi_value: false,
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
        };

        module
            .validate(features)
            .map_err(|err| PrepareError::ValidationError(err.to_string()))?;

        Ok(Self { module })
    }

    pub fn enforce_no_start_function(self) -> Result<Self, PrepareError> {
        if self.module.start_function.is_some() {
            Err(PrepareError::StartFunctionNotAllowed)
        } else {
            Ok(self)
        }
    }

    pub fn enforce_import_constraints(
        self,
        version: ScryptoVmVersion,
    ) -> Result<Self, PrepareError> {
        // Only allow `env::radix_engine` import
        for entry in self
            .module
            .import_section()
            .map_err(|err| PrepareError::ModuleInfoError(err.to_string()))?
            .unwrap_or(vec![])
        {
            if entry.module == MODULE_ENV_NAME {
                match entry.name {
                    BUFFER_CONSUME_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32],
                                vec![],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    OBJECT_CALL_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                ],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    OBJECT_CALL_MODULE_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                ],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    OBJECT_CALL_DIRECT_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                ],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    BLUEPRINT_CALL_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                ],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    KEY_VALUE_STORE_OPEN_ENTRY_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                ],
                                vec![ValType::I32],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    KEY_VALUE_ENTRY_READ_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    KEY_VALUE_ENTRY_WRITE_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32, ValType::I32],
                                vec![],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    KEY_VALUE_ENTRY_REMOVE_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    KEY_VALUE_ENTRY_CLOSE_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32],
                                vec![],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    KEY_VALUE_STORE_REMOVE_ENTRY_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    ACTOR_OPEN_FIELD_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32, ValType::I32],
                                vec![ValType::I32],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    FIELD_ENTRY_READ_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    FIELD_ENTRY_WRITE_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32, ValType::I32],
                                vec![],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    FIELD_ENTRY_CLOSE_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32],
                                vec![],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    ACTOR_GET_OBJECT_ID_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    ACTOR_GET_PACKAGE_ADDRESS_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    ACTOR_GET_BLUEPRINT_NAME_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }

                    OBJECT_NEW_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }

                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }

                    COSTING_GET_EXECUTION_COST_UNIT_LIMIT_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![],
                                vec![ValType::I32],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    COSTING_GET_EXECUTION_COST_UNIT_PRICE_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    COSTING_GET_FINALIZATION_COST_UNIT_LIMIT_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![],
                                vec![ValType::I32],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    COSTING_GET_FINALIZATION_COST_UNIT_PRICE_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    COSTING_GET_USD_PRICE_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    COSTING_GET_TIP_PERCENTAGE_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![],
                                vec![ValType::I32],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    COSTING_GET_FEE_BALANCE_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }

                    ADDRESS_ALLOCATE_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    ADDRESS_GET_RESERVATION_ADDRESS_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    OBJECT_GLOBALIZE_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                ],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    KEY_VALUE_STORE_NEW_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    OBJECT_INSTANCE_OF_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                ],
                                vec![ValType::I32],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    OBJECT_GET_BLUEPRINT_ID_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    OBJECT_GET_OUTER_OBJECT_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    ACTOR_EMIT_EVENT_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                ],
                                vec![],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    SYS_LOG_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                                vec![],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    SYS_BECH32_ENCODE_ADDRESS_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    SYS_PANIC_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32],
                                vec![],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    SYS_GET_TRANSACTION_HASH_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    SYS_GENERATE_RUID_FUNCTION_NAME => {
                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    // Crypto Utils v1 begin
                    CRYPTO_UTILS_BLS12381_V1_VERIFY_FUNCTION_NAME => {
                        if version < ScryptoVmVersion::crypto_utils_v1() {
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::ProtocolVersionMismatch {
                                    name: entry.name.to_string(),
                                    current_version: version.into(),
                                    expected_version: ScryptoVmVersion::crypto_utils_v1().into(),
                                },
                            ));
                        }

                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                ],
                                vec![ValType::I32],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    CRYPTO_UTILS_BLS12381_V1_AGGREGATE_VERIFY_FUNCTION_NAME => {
                        if version < ScryptoVmVersion::crypto_utils_v1() {
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::ProtocolVersionMismatch {
                                    name: entry.name.to_string(),
                                    current_version: version.into(),
                                    expected_version: ScryptoVmVersion::crypto_utils_v1().into(),
                                },
                            ));
                        }

                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                                vec![ValType::I32],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    CRYPTO_UTILS_BLS12381_V1_FAST_AGGREGATE_VERIFY_FUNCTION_NAME => {
                        if version < ScryptoVmVersion::crypto_utils_v1() {
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::ProtocolVersionMismatch {
                                    name: entry.name.to_string(),
                                    current_version: version.into(),
                                    expected_version: ScryptoVmVersion::crypto_utils_v1().into(),
                                },
                            ));
                        }

                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                ],
                                vec![ValType::I32],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    CRYPTO_UTILS_BLS12381_G2_SIGNATURE_AGGREGATE_FUNCTION_NAME => {
                        if version < ScryptoVmVersion::crypto_utils_v1() {
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::ProtocolVersionMismatch {
                                    name: entry.name.to_string(),
                                    current_version: version.into(),
                                    expected_version: ScryptoVmVersion::crypto_utils_v1().into(),
                                },
                            ));
                        }

                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    CRYPTO_UTILS_KECCAK256_HASH_FUNCTION_NAME => {
                        if version < ScryptoVmVersion::crypto_utils_v1() {
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::ProtocolVersionMismatch {
                                    name: entry.name.to_string(),
                                    current_version: version.into(),
                                    expected_version: ScryptoVmVersion::crypto_utils_v1().into(),
                                },
                            ));
                        }

                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    // Crypto Utils v1 end
                    // Crypto Utils v2 begin
                    CRYPTO_UTILS_BLAKE2B_256_HASH_FUNCTION_NAME => {
                        if version < ScryptoVmVersion::crypto_utils_v2() {
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::ProtocolVersionMismatch {
                                    name: entry.name.to_string(),
                                    current_version: version.into(),
                                    expected_version: ScryptoVmVersion::crypto_utils_v2().into(),
                                },
                            ));
                        }

                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    CRYPTO_UTILS_ED25519_VERIFY_FUNCTION_NAME
                    | CRYPTO_UTILS_SECP256K1_ECDSA_VERIFY_FUNCTION_NAME => {
                        if version < ScryptoVmVersion::crypto_utils_v2() {
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::ProtocolVersionMismatch {
                                    name: entry.name.to_string(),
                                    current_version: version.into(),
                                    expected_version: ScryptoVmVersion::crypto_utils_v2().into(),
                                },
                            ));
                        }

                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                    ValType::I32,
                                ],
                                vec![ValType::I32],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    CRYPTO_UTILS_SECP256K1_ECDSA_VERIFY_AND_KEY_RECOVER_FUNCTION_NAME => {
                        if version < ScryptoVmVersion::crypto_utils_v2() {
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::ProtocolVersionMismatch {
                                    name: entry.name.to_string(),
                                    current_version: version.into(),
                                    expected_version: ScryptoVmVersion::crypto_utils_v2().into(),
                                },
                            ));
                        }

                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    CRYPTO_UTILS_SECP256K1_ECDSA_VERIFY_AND_KEY_RECOVER_UNCOMPRESSED_FUNCTION_NAME =>
                    {
                        if version < ScryptoVmVersion::crypto_utils_v2() {
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::ProtocolVersionMismatch {
                                    name: entry.name.to_string(),
                                    current_version: version.into(),
                                    expected_version: ScryptoVmVersion::crypto_utils_v2().into(),
                                },
                            ));
                        }

                        if let TypeRef::Func(type_index) = entry.ty {
                            if Self::function_type_matches(
                                &self.module,
                                type_index,
                                vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                                vec![ValType::I64],
                            ) {
                                continue;
                            }
                            return Err(PrepareError::InvalidImport(
                                InvalidImport::InvalidFunctionType(entry.name.to_string()),
                            ));
                        }
                    }
                    // Crypto Utils v2 end
                    _ => {}
                };
            }

            return Err(PrepareError::InvalidImport(
                InvalidImport::ImportNotAllowed(entry.name.to_string()),
            ));
        }

        Ok(self)
    }

    pub fn enforce_memory_limit_and_inject_max(
        mut self,
        max_memory_size_in_pages: u32,
    ) -> Result<Self, PrepareError> {
        // Check if memory section exists
        let memory_section = self
            .module
            .memory_section()
            .map_err(|err| PrepareError::ModuleInfoError(err.to_string()))?
            .ok_or(PrepareError::InvalidMemory(
                InvalidMemory::MissingMemorySection,
            ))?;

        // Check if there is only one memory definition
        let mut memory = match memory_section.len() {
            0 => Err(PrepareError::InvalidMemory(
                InvalidMemory::NoMemoryDefinition,
            )),
            1 => Ok(memory_section[0]),
            _ => Err(PrepareError::InvalidMemory(
                InvalidMemory::TooManyMemoryDefinition,
            )),
        }?;

        // Check the memory limits
        if memory.initial > max_memory_size_in_pages.into() {
            return Err(PrepareError::InvalidMemory(
                InvalidMemory::MemorySizeLimitExceeded,
            ));
        }
        if let Some(max) = memory.maximum {
            if max > max_memory_size_in_pages.into() {
                return Err(PrepareError::InvalidMemory(
                    InvalidMemory::MemorySizeLimitExceeded,
                ));
            }
        } else {
            memory.maximum = Some(max_memory_size_in_pages.into());
            self.module
                .modify_memory_type(0, memory)
                .map_err(|err| PrepareError::ModuleInfoError(err.to_string()))?;
        }

        // Check if the memory is exported
        if !self
            .module
            .export_section()
            .map_err(|err| PrepareError::ModuleInfoError(err.to_string()))?
            .unwrap_or(vec![])
            .iter()
            .any(|e| e.kind == ExternalKind::Memory && e.name == EXPORT_MEMORY)
        {
            return Err(PrepareError::InvalidMemory(
                InvalidMemory::MemoryNotExported,
            ));
        }

        Ok(self)
    }

    pub fn enforce_table_limit(self, max_initial_table_size: u32) -> Result<Self, PrepareError> {
        let section = self
            .module
            .table_section()
            .map_err(|err| PrepareError::ModuleInfoError(err.to_string()))?;

        if let Some(section) = section {
            if section.len() > 1 {
                // Sanity check MVP rule
                return Err(PrepareError::InvalidTable(InvalidTable::MoreThanOneTable));
            }

            if let Some(table) = section.get(0) {
                if table.ty.initial > max_initial_table_size {
                    return Err(PrepareError::InvalidTable(
                        InvalidTable::InitialTableSizeLimitExceeded,
                    ));
                }
            }
        }

        Ok(self)
    }

    pub fn enforce_br_table_limit(
        self,
        max_number_of_br_table_targets: u32,
    ) -> Result<Self, PrepareError> {
        for fb in self
            .module
            .code_section()
            .map_err(|err| PrepareError::ModuleInfoError(err.to_string()))?
            .unwrap_or(vec![])
        {
            let reader = fb
                .get_operators_reader()
                .map_err(|err| PrepareError::WasmParserError(err.to_string()))?;

            for op in reader {
                let inst = op.map_err(|err| PrepareError::WasmParserError(err.to_string()))?;

                if let Operator::BrTable {
                    targets: table_data,
                } = inst
                {
                    if table_data.len() > max_number_of_br_table_targets {
                        return Err(PrepareError::TooManyTargetsInBrTable);
                    }
                }
            }
        }
        Ok(self)
    }

    pub fn enforce_function_limit(
        self,
        max_number_of_functions: u32,
        max_number_of_function_params: u32,
        max_number_of_function_locals: u32,
    ) -> Result<Self, PrepareError> {
        if self.module.num_local_functions() > max_number_of_functions {
            return Err(PrepareError::TooManyFunctions);
        }

        for func_idx in 0..self.module.num_local_functions() {
            if let wasmparser::Type::Func(ty) = self
                .module
                .get_type_by_func_idx(func_idx)
                .map_err(|err| PrepareError::ModuleInfoError(err.to_string()))?
            {
                if ty.params().len() > max_number_of_function_params as usize {
                    return Err(PrepareError::TooManyFunctionParams);
                }
            }
        }

        for func_body in self
            .module
            .code_section()
            .map_err(|err| PrepareError::ModuleInfoError(err.to_string()))?
            .unwrap_or(vec![])
        {
            let local_reader = func_body
                .get_locals_reader()
                .map_err(|err| PrepareError::WasmParserError(err.to_string()))?;
            let mut locals_count = 0;

            // According to the documentation local_reader.get_count() would do the job here
            // see: https://docs.rs/wasmparser/latest/wasmparser/struct.LocalsReader.html#method.get_count
            // But the description is misleading, get_count() returns the number of different types of
            // locals (or number of LocalReader iterator items).
            // To get the number of locals we need to iterate over LocalReader, which
            // returns following tuple for each item:
            //  ( u32, ValType) - where u32 is the number of locals of ValType
            for local in local_reader.into_iter() {
                // Number of locals of some type
                let (count, _ty) =
                    local.map_err(|err| PrepareError::WasmParserError(err.to_string()))?;
                locals_count = locals_count
                    .checked_add(&count)
                    .ok_or(PrepareError::Overflow)?;
            }

            if locals_count > max_number_of_function_locals {
                return Err(PrepareError::TooManyFunctionLocals {
                    max: max_number_of_function_locals,
                    actual: locals_count,
                });
            }
        }

        Ok(self)
    }

    pub fn enforce_export_names(self) -> Result<Self, PrepareError> {
        // Any exported name should follow Rust Identifier specification
        for name in &self.module.export_names {
            syn::parse_str::<Ident>(name)
                .map_err(|_| PrepareError::InvalidExportName(name.to_string()))?;
        }

        Ok(self)
    }

    pub fn enforce_global_limit(self, max_number_of_globals: u32) -> Result<Self, PrepareError> {
        if self.module.num_local_globals() > max_number_of_globals {
            return Err(PrepareError::TooManyGlobals {
                max: max_number_of_globals,
                current: self.module.num_local_globals(),
            });
        }

        Ok(self)
    }

    pub fn enforce_export_constraints<'a, I: Iterator<Item = &'a BlueprintDefinitionInit>>(
        self,
        blueprints: I,
    ) -> Result<Self, PrepareError> {
        let exports = self
            .module
            .export_section()
            .map_err(|err| PrepareError::ModuleInfoError(err.to_string()))?;

        if let Some(exports) = exports {
            for blueprint_def_init in blueprints {
                for export_name in blueprint_def_init.schema.exports() {
                    if !exports.iter().any(|x| {
                        x.name.eq(&export_name) && {
                            if let ExternalKind::Func = x.kind {
                                Self::function_matches(
                                    &self.module,
                                    x.index as usize,
                                    vec![ValType::I64],
                                    vec![ValType::I64],
                                )
                            } else {
                                false
                            }
                        }
                    }) {
                        return Err(PrepareError::MissingExport {
                            export_name: export_name.to_string(),
                        });
                    }
                }
            }

            Ok(self)
        } else {
            Err(PrepareError::NoExportSection)
        }
    }

    pub fn inject_instruction_metering<R: Rules>(
        mut self,
        rules: &R,
    ) -> Result<Self, PrepareError> {
        #[cfg(not(feature = "coverage"))]
        {
            let backend = gas_metering::host_function::Injector::new(
                MODULE_ENV_NAME,
                COSTING_CONSUME_WASM_EXECUTION_UNITS_FUNCTION_NAME,
            );
            gas_metering::inject(&mut self.module, backend, rules).map_err(|err| {
                PrepareError::RejectedByInstructionMetering {
                    reason: err.to_string(),
                }
            })?;
        }

        Ok(self)
    }

    pub fn inject_stack_metering(mut self, wasm_max_stack_size: u32) -> Result<Self, PrepareError> {
        inject_stack_limiter(&mut self.module, wasm_max_stack_size).map_err(|err| {
            PrepareError::RejectedByStackMetering {
                reason: err.to_string(),
            }
        })?;
        Ok(self)
    }

    pub fn ensure_instantiatable(self) -> Result<Self, PrepareError> {
        // During instantiation time, the following procedures are applied:

        // 1. Resolve imports with external values
        // This should always succeed as we only allow `env::radix_engine` function import

        // 2. Allocate externals, functions, tables, memory and globals
        // This should always succeed as we enforce an upper bound for each type

        // 3. Update table with elements
        // It may fail if the offset is out of bound

        // 4. Update memory with data segments
        // It may fail if the offset is out of bound

        // Because the offset can be an `InitExpr` that requires evaluation against an WASM instance,
        // we're using the `wasmi` logic as a shortcut.
        let code = self.module.bytes();
        WasmiModule::new(&code[..])
            .map_err(|_| PrepareError::NotCompilable)?
            .instantiate()
            .map_err(|e| PrepareError::NotInstantiatable {
                reason: format!("{:?}", e),
            })?;

        Ok(self)
    }

    pub fn ensure_compilable(self) -> Result<Self, PrepareError> {
        // TODO: Understand WASM JIT compilability
        //
        // Can we make the assumption that all "prepared" modules are compilable,
        // if machine resource is "sufficient"?
        //
        // Another option is to attempt to compile, although it may make RE protocol
        // coupled with a specific implementation.

        Ok(self)
    }

    pub fn to_bytes(self) -> Result<(Vec<u8>, Vec<String>), PrepareError> {
        let mut function_exports = vec![];

        for export in self
            .module
            .export_section()
            .map_err(|err| PrepareError::ModuleInfoError(err.to_string()))?
            .unwrap_or(vec![])
        {
            if let wasmparser::ExternalKind::Func = export.kind {
                function_exports.push(export.name.to_string());
            }
        }
        let code = self.module.bytes();

        Ok((code, function_exports))
    }

    fn function_matches(
        module: &ModuleInfo,
        func_index: usize,
        params: Vec<ValType>,
        results: Vec<ValType>,
    ) -> bool {
        match module.function_map.get(func_index) {
            Some(type_index) => Self::function_type_matches(module, *type_index, params, results),
            None => false,
        }
    }

    fn function_type_matches(
        module: &ModuleInfo,
        type_index: u32,
        params: Vec<ValType>,
        results: Vec<ValType>,
    ) -> bool {
        let ty = module.get_type_by_idx(type_index);
        match ty {
            Ok(ty) => match ty {
                Type::Func(ty) => ty == &FuncType::new(params, results),
                _ => false,
            },
            Err(_) => false,
        }
    }

    #[cfg(feature = "radix_engine_tests")]
    pub fn contains_sign_ext_ops(self) -> bool {
        for func_body in self
            .module
            .code_section()
            .map_err(|err| PrepareError::ModuleInfoError(err.to_string()))
            .unwrap()
            .expect("no code section")
        {
            let reader = func_body
                .get_operators_reader()
                .map_err(|err| PrepareError::WasmParserError(err.to_string()))
                .unwrap();
            for op in reader {
                let inst = op
                    .map_err(|err| PrepareError::WasmParserError(err.to_string()))
                    .unwrap();

                match inst {
                    Operator::I32Extend8S
                    | Operator::I32Extend16S
                    | Operator::I64Extend8S
                    | Operator::I64Extend16S
                    | Operator::I64Extend32S => return true,
                    _ => (),
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use radix_blueprint_schema_init::{
        BlueprintFunctionsSchemaInit, BlueprintHooksInit, BlueprintSchemaInit,
        BlueprintStateSchemaInit, BlueprintTypeSchemaInit, FieldSchema, FunctionSchemaInit,
    };
    use radix_engine_interface::blueprints::package::BlueprintType;
    use sbor::basic_well_known_types::{ANY_TYPE, UNIT_TYPE};
    use wabt::{wat2wasm_with_features, Features};

    macro_rules! wat2wasm {
        ($wat: expr) => {{
            let mut features = Features::new();
            features.enable_sign_extension();
            features.enable_mutable_globals();
            let code = wat2wasm_with_features($wat, features).unwrap();
            code
        }};
    }

    macro_rules! assert_invalid_wasm {
        ($wat: expr, $err: expr) => {
            let code = wat2wasm!($wat);
            assert_eq!($err, WasmModule::init(&code).unwrap_err());
        };

        ($wat: expr, $err: expr, $func: expr) => {
            let code = wat2wasm!($wat);
            assert_eq!($err, WasmModule::init(&code).and_then($func).unwrap_err());
        };
    }

    #[test]
    fn test_floating_point() {
        // return
        assert_invalid_wasm!(
            r#"
            (module
                (func (result f64)
                    f64.const 123
                )
            )
            "#,
            PrepareError::ValidationError(
                "WasmParserError(BinaryReaderError { floating-point support is disabled (at offset 0xb) })".to_string()
            )
        );
        // input
        assert_invalid_wasm!(
            r#"
            (module
                (func (param f64)
                )
            )
            "#,
            PrepareError::ValidationError(
                "WasmParserError(BinaryReaderError { floating-point support is disabled (at offset 0xb) })".to_string()
            )
        );
        // instruction
        assert_invalid_wasm!(
            r#"
            (module
                (func
                    f64.const 1
                    f64.const 2
                    f64.add
                    drop
                )
            )
            "#,
            PrepareError::ValidationError(
                "WasmParserError(BinaryReaderError { floating-point instruction disallowed (at offset 0x17) })".to_string()
            )
        );
        // global
        assert_invalid_wasm!(
            r#"
            (module
                (global $fp f32 (f32.const 10))
            )
            "#,
            PrepareError::ValidationError(
                "WasmParserError(BinaryReaderError { floating-point support is disabled (at offset 0xb) })".to_string()
            )
        );
    }

    #[test]
    fn test_start_function() {
        assert_invalid_wasm!(
            r#"
            (module
                (func $main)
                (start $main)
            )
            "#,
            PrepareError::StartFunctionNotAllowed,
            WasmModule::enforce_no_start_function
        );
    }

    #[test]
    fn test_enforce_import_limit() {
        let wat = r#"
            (module
                (import "env" "name_to_replace" (func $some_func (param i32 i32 i32 i32 i32 i32 i32 i32)))
            )
            "#;
        assert_invalid_wasm!(
            wat,
            PrepareError::InvalidImport(InvalidImport::ImportNotAllowed(
                "name_to_replace".to_string()
            )),
            |s| WasmModule::enforce_import_constraints(s, ScryptoVmVersion::V1_0)
        );

        for name in [
            BUFFER_CONSUME_FUNCTION_NAME,
            OBJECT_CALL_FUNCTION_NAME,
            OBJECT_CALL_MODULE_FUNCTION_NAME,
            OBJECT_CALL_DIRECT_FUNCTION_NAME,
            BLUEPRINT_CALL_FUNCTION_NAME,
            KEY_VALUE_STORE_OPEN_ENTRY_FUNCTION_NAME,
            KEY_VALUE_ENTRY_READ_FUNCTION_NAME,
            KEY_VALUE_ENTRY_WRITE_FUNCTION_NAME,
            KEY_VALUE_ENTRY_REMOVE_FUNCTION_NAME,
            KEY_VALUE_ENTRY_CLOSE_FUNCTION_NAME,
            KEY_VALUE_STORE_REMOVE_ENTRY_FUNCTION_NAME,
            ACTOR_OPEN_FIELD_FUNCTION_NAME,
            FIELD_ENTRY_READ_FUNCTION_NAME,
            FIELD_ENTRY_WRITE_FUNCTION_NAME,
            FIELD_ENTRY_CLOSE_FUNCTION_NAME,
            ACTOR_GET_OBJECT_ID_FUNCTION_NAME,
            ACTOR_GET_PACKAGE_ADDRESS_FUNCTION_NAME,
            ACTOR_GET_BLUEPRINT_NAME_FUNCTION_NAME,
            OBJECT_NEW_FUNCTION_NAME,
            COSTING_GET_EXECUTION_COST_UNIT_LIMIT_FUNCTION_NAME,
            COSTING_GET_EXECUTION_COST_UNIT_PRICE_FUNCTION_NAME,
            COSTING_GET_FINALIZATION_COST_UNIT_LIMIT_FUNCTION_NAME,
            COSTING_GET_FINALIZATION_COST_UNIT_PRICE_FUNCTION_NAME,
            COSTING_GET_USD_PRICE_FUNCTION_NAME,
            COSTING_GET_TIP_PERCENTAGE_FUNCTION_NAME,
            COSTING_GET_FEE_BALANCE_FUNCTION_NAME,
            ADDRESS_ALLOCATE_FUNCTION_NAME,
            ADDRESS_GET_RESERVATION_ADDRESS_FUNCTION_NAME,
            OBJECT_GLOBALIZE_FUNCTION_NAME,
            KEY_VALUE_STORE_NEW_FUNCTION_NAME,
            OBJECT_INSTANCE_OF_FUNCTION_NAME,
            OBJECT_GET_BLUEPRINT_ID_FUNCTION_NAME,
            OBJECT_GET_OUTER_OBJECT_FUNCTION_NAME,
            ACTOR_EMIT_EVENT_FUNCTION_NAME,
            SYS_LOG_FUNCTION_NAME,
            SYS_BECH32_ENCODE_ADDRESS_FUNCTION_NAME,
            SYS_PANIC_FUNCTION_NAME,
            SYS_GET_TRANSACTION_HASH_FUNCTION_NAME,
            SYS_GENERATE_RUID_FUNCTION_NAME,
        ] {
            assert_invalid_wasm!(
                wat.replace("name_to_replace", name),
                PrepareError::InvalidImport(InvalidImport::InvalidFunctionType(name.to_string())),
                |w| WasmModule::enforce_import_constraints(w, ScryptoVmVersion::V1_0)
            );
        }
    }

    #[test]
    fn test_invalid_import_protocol_mismatch() {
        let wat = r#"
            (module
                (import "env" "name_to_replace" (func $some_func (param i32) (result i32)))
            )
            "#;

        for (current_version, expected_version, names) in [
            (
                ScryptoVmVersion::V1_0,
                ScryptoVmVersion::crypto_utils_v1(),
                vec![
                    CRYPTO_UTILS_BLS12381_V1_VERIFY_FUNCTION_NAME,
                    CRYPTO_UTILS_BLS12381_V1_AGGREGATE_VERIFY_FUNCTION_NAME,
                    CRYPTO_UTILS_BLS12381_V1_FAST_AGGREGATE_VERIFY_FUNCTION_NAME,
                    CRYPTO_UTILS_BLS12381_G2_SIGNATURE_AGGREGATE_FUNCTION_NAME,
                    CRYPTO_UTILS_KECCAK256_HASH_FUNCTION_NAME,
                ],
            ),
            (
                ScryptoVmVersion::V1_1,
                ScryptoVmVersion::crypto_utils_v2(),
                vec![
                    CRYPTO_UTILS_BLAKE2B_256_HASH_FUNCTION_NAME,
                    CRYPTO_UTILS_ED25519_VERIFY_FUNCTION_NAME,
                    CRYPTO_UTILS_SECP256K1_ECDSA_VERIFY_FUNCTION_NAME,
                    CRYPTO_UTILS_SECP256K1_ECDSA_VERIFY_AND_KEY_RECOVER_FUNCTION_NAME,
                    CRYPTO_UTILS_SECP256K1_ECDSA_VERIFY_AND_KEY_RECOVER_UNCOMPRESSED_FUNCTION_NAME,
                ],
            ),
        ] {
            for name in names {
                assert_invalid_wasm!(
                    wat.replace("name_to_replace", name),
                    PrepareError::InvalidImport(InvalidImport::ProtocolVersionMismatch {
                        name: name.to_string(),
                        current_version: current_version.into(),
                        expected_version: expected_version.into(),
                    }),
                    |w| WasmModule::enforce_import_constraints(w, current_version)
                );
            }
        }
    }

    #[test]
    fn test_enforce_global_limit() {
        assert_invalid_wasm!(
            r#"
            (module
                (global $g1 i32 (i32.const 0))
                (global $g2 i32 (i32.const 0))
                (global $g3 i32 (i32.const 0))
                (global $g4 i32 (i32.const 0))
            )
            "#,
            PrepareError::TooManyGlobals { max: 3, current: 4 },
            |x| WasmModule::enforce_global_limit(x, 3)
        );
    }

    #[test]
    fn test_memory() {
        assert_invalid_wasm!(
            r#"
            (module
            )
            "#,
            PrepareError::InvalidMemory(InvalidMemory::MissingMemorySection),
            |x| WasmModule::enforce_memory_limit_and_inject_max(x, 5)
        );
        // NOTE: Disabled as MVP only allow 1 memory definition
        // assert_invalid_wasm!(
        //     r#"
        //     (module
        //         (memory 2)
        //         (memory 2)
        //     )
        //     "#,
        //     PrepareError::InvalidMemory(InvalidMemory::TooManyMemories),
        //     |x| WasmModule::enforce_memory_limit(x, 5)
        // );
        assert_invalid_wasm!(
            r#"
            (module
                (memory 6)
            )
            "#,
            PrepareError::InvalidMemory(InvalidMemory::MemorySizeLimitExceeded),
            |x| WasmModule::enforce_memory_limit_and_inject_max(x, 5)
        );
        assert_invalid_wasm!(
            r#"
            (module
                (memory 2)
            )
            "#,
            PrepareError::InvalidMemory(InvalidMemory::MemoryNotExported),
            |x| WasmModule::enforce_memory_limit_and_inject_max(x, 5)
        );
    }

    #[test]
    fn test_table() {
        assert_invalid_wasm!(
            r#"
            (module
                (table 6 funcref)
            )
            "#,
            PrepareError::InvalidTable(InvalidTable::InitialTableSizeLimitExceeded),
            |x| WasmModule::enforce_table_limit(x, 5)
        );
    }

    #[test]
    fn test_br_table() {
        assert_invalid_wasm!(
            r#"
            (module
                (func (param i32) (result i32)
                    (block
                        (block
                            (br_table 1 0 1 0 1 0 1 (local.get 0))
                            (return (i32.const 21))
                        )
                        (return (i32.const 20))
                    )
                    (i32.const 22)
                )
            )
            "#,
            PrepareError::TooManyTargetsInBrTable,
            |x| WasmModule::enforce_br_table_limit(x, 5)
        );
    }

    #[test]
    fn test_function_limits() {
        assert_invalid_wasm!(
            r#"
            (module
                (func (result i32)
                    (i32.const 11)
                )
                (func (result i32)
                    (i32.const 22)
                )
                (func (result i32)
                    (i32.const 33)
                )
            )
            "#,
            PrepareError::TooManyFunctions,
            |x| WasmModule::enforce_function_limit(x, 2, 3, 3)
        );

        assert_invalid_wasm!(
            r#"
            (module
                (func (param i32 i32 i32 i32) (result i32)
                    (i32.const 22)
                )
            )
            "#,
            PrepareError::TooManyFunctionParams,
            |x| WasmModule::enforce_function_limit(x, 2, 3, 3)
        );

        assert_invalid_wasm!(
            r#"
            (module
                (func (result i32)
                    (local $v1 i32)

                    (local.set $v1 (i32.const 1))

                    (i32.const 22)
                )
                (func (result i32)
                    (local $v1 i32)
                    (local $v2 i64)
                    (local $v3 i64)
                    (local $v4 i32)

                    (local.set $v1 (i32.const 1))
                    (local.set $v2 (i64.const 2))
                    (local.set $v3 (i64.const 3))
                    (local.set $v4 (i32.const 4))

                    (i32.const 22)
                )
            )
            "#,
            PrepareError::TooManyFunctionLocals { max: 3, actual: 4 },
            |x| WasmModule::enforce_function_limit(x, 2, 3, 3)
        );
    }

    #[test]
    fn test_blueprint_constraints() {
        let mut blueprints = index_map_new();
        blueprints.insert(
            "Test".to_string(),
            BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                is_transient: false,
                feature_set: indexset!(),
                dependencies: indexset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema: SchemaV1 {
                        type_kinds: vec![],
                        type_metadata: vec![],
                        type_validations: vec![],
                    }.into_versioned(),
                    state: BlueprintStateSchemaInit {
                        fields: vec![FieldSchema::static_field(LocalTypeId::WellKnown(UNIT_TYPE))],
                        collections: vec![],
                    },
                    events: Default::default(),
                    types: BlueprintTypeSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        functions: indexmap!(
                            "f".to_string() => FunctionSchemaInit {
                                receiver: Option::None,
                                input: radix_blueprint_schema_init::TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
                                output: radix_blueprint_schema_init::TypeRef::Static(LocalTypeId::WellKnown(UNIT_TYPE)),
                                export: "Test_f".to_string(),
                            }
                        ),
                    },
                    hooks: BlueprintHooksInit::default(),
                },

                royalty_config: Default::default(),
                auth_config: Default::default(),
            },
        );

        assert_invalid_wasm!(
            r#"
            (module
            )
            "#,
            PrepareError::NoExportSection,
            |x| WasmModule::enforce_export_constraints(x, blueprints.values())
        );
        // symbol not found
        assert_invalid_wasm!(
            r#"
            (module
                (func (export "foo") (result i32)
                    (i32.const 0)
                )
            )
            "#,
            PrepareError::MissingExport {
                export_name: "Test_f".to_string()
            },
            |x| WasmModule::enforce_export_constraints(x, blueprints.values())
        );
        // signature does not match
        assert_invalid_wasm!(
            r#"
            (module
                (func (export "Test_f") (result i32)
                    (i32.const 0)
                )
            )
            "#,
            PrepareError::MissingExport {
                export_name: "Test_f".to_string()
            },
            |x| WasmModule::enforce_export_constraints(x, blueprints.values())
        );

        // export kind does not match
        assert_invalid_wasm!(
            r#"
            (module
                (global (export "Test_f") i32 (i32.const 0))
            )
            "#,
            PrepareError::MissingExport {
                export_name: "Test_f".to_string()
            },
            |x| WasmModule::enforce_export_constraints(x, blueprints.values())
        );
    }

    #[cfg(feature = "radix_engine_tests")]
    #[test]
    fn test_contains_sign_ext_ops() {
        let code = wat2wasm!(
            r#"
            (module
                (func $f
                    (i64.const 1)
                    (i64.extend8_s) ;; sign extension op
                    drop
                )
            )
            "#
        );

        assert!(WasmModule::init(&code).unwrap().contains_sign_ext_ops());

        let code = wat2wasm!(
            r#"
            (module
                (func $f
                    (i64.const 1)
                    drop
                )
            )
            "#
        );

        assert!(!WasmModule::init(&code).unwrap().contains_sign_ext_ops());
    }
}
