use crate::blueprints::package::*;
use crate::errors::{ApplicationError, RuntimeError};
use crate::internal_prelude::*;
use crate::kernel::kernel_api::{KernelInternalApi, KernelNodeApi, KernelSubstateApi};
use crate::system::system_callback::{System, SystemLockData};
use crate::system::system_callback_api::{SystemCallbackObject, SystemInvokeResult};
use crate::system::system_substates::KeyValueEntrySubstate;
use crate::track::BootStore;
use crate::vm::wasm::{ScryptoV1WasmValidator, WasmEngine};
use crate::vm::{NativeVm, NativeVmExtension, ScryptoVm};
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::SystemApi;

use crate::vm::ScryptoVmVersion;

pub const BOOT_LOADER_VM_BOOT_FIELD_KEY: FieldKey = 2u8;

pub type VmBootSubstate = VmBoot;

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum VmBoot {
    V1 { scrypto_version: u64 },
}

impl VmBoot {
    pub fn latest() -> Self {
        Self::V1 {
            scrypto_version: ScryptoVmVersion::latest().into(),
        }
    }

    pub fn babylon() -> Self {
        Self::V1 {
            scrypto_version: ScryptoVmVersion::V1_0.into(),
        }
    }
}

pub trait VmApi {
    fn get_scrypto_version(&self) -> ScryptoVmVersion;
}

impl VmApi for VmBoot {
    fn get_scrypto_version(&self) -> ScryptoVmVersion {
        match self {
            VmBoot::V1 { scrypto_version } => ScryptoVmVersion::try_from(*scrypto_version)
                .expect(&format!("Unexpected scrypto version: {}", scrypto_version)),
        }
    }
}

pub struct VmInit<'g, W: WasmEngine, E: NativeVmExtension> {
    pub scrypto_vm: &'g ScryptoVm<W>,
    pub native_vm_extension: E,
}

impl<'g, W: WasmEngine, E: NativeVmExtension> VmInit<'g, W, E> {
    pub fn new(scrypto_vm: &'g ScryptoVm<W>, native_vm_extension: E) -> Self {
        Self {
            scrypto_vm,
            native_vm_extension,
        }
    }
}

impl<'g, W: WasmEngine, E: NativeVmExtension> Clone for VmInit<'g, W, E> {
    fn clone(&self) -> Self {
        Self {
            scrypto_vm: self.scrypto_vm,
            native_vm_extension: self.native_vm_extension.clone(),
        }
    }
}

pub struct Vm<'g, W: WasmEngine, E: NativeVmExtension> {
    pub scrypto_vm: &'g ScryptoVm<W>,
    pub native_vm: NativeVm<E>,
    pub vm_boot: VmBoot,
}

impl<'g, W: WasmEngine + 'g, E: NativeVmExtension> SystemCallbackObject for Vm<'g, W, E> {
    type Init = VmInit<'g, W, E>;

    fn init<S: BootStore>(store: &S, vm_init: VmInit<'g, W, E>) -> Result<Self, BootloadingError> {
        let vm_boot = store
            .read_boot_substate(
                TRANSACTION_TRACKER.as_node_id(),
                BOOT_LOADER_PARTITION,
                &SubstateKey::Field(BOOT_LOADER_VM_BOOT_FIELD_KEY),
            )
            .map(|v| scrypto_decode(v.as_slice()).unwrap())
            .unwrap_or(VmBoot::babylon());

        Ok(Self {
            scrypto_vm: vm_init.scrypto_vm,
            native_vm: NativeVm::new_with_extension(vm_init.native_vm_extension),
            vm_boot,
        })
    }

    fn invoke<
        Y: SystemApi<RuntimeError>
            + KernelInternalApi<System<Self>>
            + KernelNodeApi
            + KernelSubstateApi<SystemLockData>,
    >(
        address: &PackageAddress,
        export: PackageExport,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<SystemInvokeResult, RuntimeError> {
        let vm_type = {
            let handle = api.kernel_open_substate_with_default(
                address.as_node_id(),
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_VM_TYPE_PARTITION_OFFSET)
                    .unwrap(),
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
                .into_value()
                .unwrap_or_else(|| panic!("Vm type not found: {:?}", export))
        };

        let vm_api = api
            .kernel_get_system_state()
            .system
            .callback
            .vm_boot
            .clone();

        let output = match vm_type.fully_update_and_into_latest_version().vm_type {
            VmType::Native => {
                let original_code = {
                    let handle = api.kernel_open_substate_with_default(
                        address.as_node_id(),
                        MAIN_BASE_PARTITION
                            .at_offset(PACKAGE_ORIGINAL_CODE_PARTITION_OFFSET)
                            .unwrap(),
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
                        .into_value()
                        .unwrap_or_else(|| panic!("Original code not found: {:?}", export))
                };

                let mut vm_instance = api.kernel_get_system().callback.native_vm.create_instance(
                    address,
                    &original_code.fully_update_and_into_latest_version().code,
                )?;
                let output =
                    { vm_instance.invoke(export.export_name.as_str(), input, api, &vm_api)? };

                SystemInvokeResult::Done(output)
            }
            VmType::ScryptoV1 => {
                let instrumented_code = {
                    let handle = api.kernel_open_substate_with_default(
                        address.as_node_id(),
                        MAIN_BASE_PARTITION
                            .at_offset(PACKAGE_INSTRUMENTED_CODE_PARTITION_OFFSET)
                            .unwrap(),
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
                        .into_value()
                        .unwrap_or_else(|| panic!("Instrumented code not found: {:?}", export))
                        .fully_update_and_into_latest_version()
                };

                let mut scrypto_vm_instance = {
                    api.kernel_get_system().callback.scrypto_vm.create_instance(
                        address,
                        export.code_hash,
                        &instrumented_code.instrumented_code,
                    )
                };

                api.consume_cost_units(ClientCostingEntry::PrepareWasmCode {
                    size: instrumented_code.instrumented_code.len(),
                })?;

                let output = {
                    scrypto_vm_instance.invoke(export.export_name.as_str(), input, api, &vm_api)?
                };

                SystemInvokeResult::Done(output)
            }
        };

        Ok(output)
    }
}

pub trait VmInvoke {
    // TODO: Remove KernelNodeAPI + KernelSubstateAPI from api, unify with VmApi
    fn invoke<
        Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
        V: VmApi,
    >(
        &mut self,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
        vm_api: &V,
    ) -> Result<IndexedScryptoValue, RuntimeError>;
}

pub struct VmPackageValidation;

impl VmPackageValidation {
    pub fn validate<V: VmApi>(
        definition: &PackageDefinition,
        vm_type: VmType,
        code: &[u8],
        vm_api: &V,
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        match vm_type {
            VmType::Native => Ok(None),
            VmType::ScryptoV1 => {
                let version = vm_api.get_scrypto_version();

                // Validate WASM
                let instrumented_code = ScryptoV1WasmValidator::new(version)
                    .validate(&code, definition.blueprints.values())
                    .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::PackageError(
                            PackageError::InvalidWasm(e),
                        ))
                    })?
                    .0;

                for BlueprintDefinitionInit {
                    is_transient,
                    blueprint_type,
                    feature_set,
                    schema:
                        BlueprintSchemaInit {
                            generics,
                            state:
                                BlueprintStateSchemaInit {
                                    collections,
                                    fields,
                                },
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

                    if fields.len() > 1 {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::PackageError(PackageError::WasmUnsupported(
                                "More than 1 substate field not supported".to_string(),
                            )),
                        ));
                    }

                    for field in fields {
                        match &field.condition {
                            Condition::Always => {}
                            _ => {
                                return Err(RuntimeError::ApplicationError(
                                    ApplicationError::PackageError(PackageError::WasmUnsupported(
                                        "Conditional fields are not supported".to_string(),
                                    )),
                                ));
                            }
                        }

                        match field.transience {
                            FieldTransience::NotTransient => {}
                            _ => {
                                return Err(RuntimeError::ApplicationError(
                                    ApplicationError::PackageError(PackageError::WasmUnsupported(
                                        "Transient fields are not supported".to_string(),
                                    )),
                                ));
                            }
                        }
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

                    if *is_transient {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::PackageError(PackageError::WasmUnsupported(
                                "Transient blueprints not supported".to_string(),
                            )),
                        ));
                    }
                }
                Ok(Some(instrumented_code))
            }
        }
    }
}
