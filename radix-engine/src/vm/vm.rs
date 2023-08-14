use crate::blueprints::package::{
    PackageCodeInstrumentedCodeEntrySubstate, PackageCodeOriginalCodeEntrySubstate,
    PackageCodeVmTypeEntrySubstate, PackageError, PackagePartition, VmType,
};
use crate::errors::{ApplicationError, RuntimeError};
use crate::kernel::kernel_api::{KernelInternalApi, KernelNodeApi, KernelSubstateApi};
use crate::system::system::KeyValueEntrySubstate;
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::types::*;
use crate::vm::wasm::{WasmEngine, WasmValidator};
use crate::vm::{NativeVm, NativeVmExtension, ScryptoVm};
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::package::*;

pub struct Vm<'g, W: WasmEngine, E: NativeVmExtension> {
    pub scrypto_vm: &'g ScryptoVm<W>,
    pub native_vm: NativeVm<E>,
}

impl<'g, W: WasmEngine, E: NativeVmExtension> Vm<'g, W, E> {
    pub fn new(scrypto_vm: &'g ScryptoVm<W>, native_vm: NativeVm<E>) -> Self {
        Self {
            scrypto_vm,
            native_vm,
        }
    }
}

impl<'g, W: WasmEngine, E: NativeVmExtension> Clone for Vm<'g, W, E> {
    fn clone(&self) -> Self {
        Self {
            scrypto_vm: self.scrypto_vm,
            native_vm: self.native_vm.clone(),
        }
    }
}

impl<'g, W: WasmEngine + 'g, E: NativeVmExtension> SystemCallbackObject for Vm<'g, W, E> {
    fn invoke<Y>(
        address: &PackageAddress,
        export: PackageExport,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>
            + KernelInternalApi<SystemConfig<Self>>
            + KernelNodeApi
            + KernelSubstateApi<SystemLockData>,
        W: WasmEngine,
    {
        let vm_type = {
            let handle = api.kernel_open_substate_with_default(
                address.as_node_id(),
                PackagePartition::CodeVmTypeKeyValue.as_main_partition(),
                &SubstateKey::Map(scrypto_encode(&export.code_hash).unwrap()),
                LockFlags::read_only(),
                Some(|| {
                    let kv_entry = KeyValueEntrySubstate::<()>::default();
                    IndexedScryptoValue::from_typed(&kv_entry)
                }),
                SystemLockData::default(),
            )?;
            let vm_type = api.kernel_read_substate(handle)?;
            let vm_type: PackageCodeVmTypeEntrySubstate = vm_type.as_typed().unwrap();
            api.kernel_close_substate(handle)?;
            vm_type
                .value
                .expect(&format!("Vm type not found: {:?}", export))
        };

        let output = match vm_type.0.into_latest().vm_type {
            VmType::Native => {
                let original_code = {
                    let handle = api.kernel_open_substate_with_default(
                        address.as_node_id(),
                        PackagePartition::CodeOriginalCodeKeyValue.as_main_partition(),
                        &SubstateKey::Map(scrypto_encode(&export.code_hash).unwrap()),
                        LockFlags::read_only(),
                        Some(|| {
                            let kv_entry = KeyValueEntrySubstate::<()>::default();
                            IndexedScryptoValue::from_typed(&kv_entry)
                        }),
                        SystemLockData::default(),
                    )?;
                    let original_code = api.kernel_read_substate(handle)?;
                    let original_code: PackageCodeOriginalCodeEntrySubstate =
                        original_code.as_typed().unwrap();
                    api.kernel_close_substate(handle)?;
                    original_code
                        .value
                        .expect(&format!("Original code not found: {:?}", export))
                };

                let mut vm_instance = api
                    .kernel_get_system()
                    .callback_obj
                    .native_vm
                    .create_instance(address, &original_code.0.into_latest().code)?;
                let output = { vm_instance.invoke(export.export_name.as_str(), input, api)? };

                output
            }
            VmType::ScryptoV1 => {
                let instrumented_code = {
                    let handle = api.kernel_open_substate_with_default(
                        address.as_node_id(),
                        PackagePartition::CodeInstrumentedCodeKeyValue.as_main_partition(),
                        &SubstateKey::Map(scrypto_encode(&export.code_hash).unwrap()),
                        LockFlags::read_only(),
                        Some(|| {
                            let kv_entry = KeyValueEntrySubstate::<()>::default();
                            IndexedScryptoValue::from_typed(&kv_entry)
                        }),
                        SystemLockData::default(),
                    )?;
                    let instrumented_code = api.kernel_read_substate(handle)?;
                    let instrumented_code: PackageCodeInstrumentedCodeEntrySubstate =
                        instrumented_code.as_typed().unwrap();
                    api.kernel_close_substate(handle)?;
                    instrumented_code
                        .value
                        .expect(&format!("Instrumented code not found: {:?}", export))
                        .0
                        .into_latest()
                };

                let mut scrypto_vm_instance = {
                    api.kernel_get_system()
                        .callback_obj
                        .scrypto_vm
                        .create_instance(
                            address,
                            export.code_hash,
                            &instrumented_code.instrumented_code,
                        )
                };

                api.consume_cost_units(ClientCostingEntry::PrepareWasmCode {
                    size: instrumented_code.instrumented_code.len(),
                })?;

                let output =
                    { scrypto_vm_instance.invoke(export.export_name.as_str(), input, api)? };

                output
            }
        };

        Ok(output)
    }
}

pub trait VmInvoke {
    // TODO: Remove KernelNodeAPI + KernelSubstateAPI from api
    fn invoke<Y>(
        &mut self,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>;
}

pub struct VmPackageValidation;

impl VmPackageValidation {
    pub fn validate(
        definition: &PackageDefinition,
        vm_type: VmType,
        code: &[u8],
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        match vm_type {
            VmType::Native => Ok(None),
            VmType::ScryptoV1 => {
                // Validate WASM
                let instrumented_code = WasmValidator::default()
                    .validate(&code, definition.blueprints.values())
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::PackageError(
                            PackageError::InvalidWasm(e),
                        ))
                    })?
                    .0;

                for BlueprintDefinitionInit {
                    blueprint_type,
                    feature_set,
                    schema:
                        BlueprintSchemaInit {
                            generics,
                            state: BlueprintStateSchemaInit { collections, .. },
                            functions,
                            hooks,
                            ..
                        },
                    ..
                } in definition.blueprints.values()
                {
                    match blueprint_type {
                        BlueprintType::Outer => {}
                        BlueprintType::Inner { .. } => {
                            return Err(RuntimeError::ApplicationError(
                                ApplicationError::PackageError(PackageError::WasmUnsupported(
                                    "Inner blueprints not supported".to_string(),
                                )),
                            ));
                        }
                    }

                    if !feature_set.is_empty() {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::PackageError(PackageError::WasmUnsupported(
                                "Feature set not supported".to_string(),
                            )),
                        ));
                    }

                    if !collections.is_empty() {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::PackageError(PackageError::WasmUnsupported(
                                "Static collections not supported".to_string(),
                            )),
                        ));
                    }

                    if !hooks.hooks.is_empty() {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::PackageError(PackageError::WasmUnsupported(
                                "Hooks not supported".to_string(),
                            )),
                        ));
                    }

                    for (_name, schema) in &functions.functions {
                        if let Some(info) = &schema.receiver {
                            if info.ref_types != RefTypes::NORMAL {
                                return Err(RuntimeError::ApplicationError(
                                    ApplicationError::PackageError(PackageError::WasmUnsupported(
                                        "Irregular ref types not supported".to_string(),
                                    )),
                                ));
                            }
                        }
                    }

                    if !generics.is_empty() {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::PackageError(PackageError::WasmUnsupported(
                                "Generics not supported".to_string(),
                            )),
                        ));
                    }
                }
                Ok(Some(instrumented_code))
            }
        }
    }
}
