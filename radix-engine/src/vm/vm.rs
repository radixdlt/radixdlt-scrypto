use crate::blueprints::package::PackageCodeTypeSubstate;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelInternalApi, KernelNodeApi, KernelSubstateApi};
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::types::*;
use crate::vm::vm::api::ClientApi;
use crate::vm::wasm::WasmEngine;
use crate::vm::{NativeVm, ScryptoVm};
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::blueprints::package::*;

pub struct Vm<'g, W: WasmEngine> {
    pub scrypto_vm: &'g ScryptoVm<W>,
}

impl<'g, W: WasmEngine + 'g> SystemCallbackObject for Vm<'g, W> {
    fn invoke<Y>(
        address: &PackageAddress,

        export_name: &str,
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
        let code_type = {
            let handle = api.kernel_lock_substate(
                address.as_node_id(),
                OBJECT_BASE_PARTITION,
                &PackageField::CodeType.into(),
                LockFlags::read_only(),
                SystemLockData::default(),
            )?;
            let code_type = api.kernel_read_substate(handle)?;
            let code_type: PackageCodeTypeSubstate = code_type.as_typed().unwrap();
            api.kernel_drop_lock(handle)?;
            code_type
        };

        let package_code = {
            let handle = api.kernel_lock_substate(
                address.as_node_id(),
                OBJECT_BASE_PARTITION,
                &PackageField::Code.into(),
                LockFlags::read_only(),
                SystemLockData::default(),
            )?;
            let code = api.kernel_read_substate(handle)?;
            let package_code: PackageCodeSubstate = code.as_typed().unwrap();
            api.kernel_drop_lock(handle)?;
            package_code
        };

        let output = match code_type {
            PackageCodeTypeSubstate::Native => {
                let mut vm_instance = { NativeVm::create_instance(address, &package_code.code)? };
                let output = { vm_instance.invoke(&export_name, input, api)? };

                output
            }
            PackageCodeTypeSubstate::Wasm => {
                let mut scrypto_vm_instance = {
                    api.kernel_get_system()
                        .callback_obj
                        .scrypto_vm
                        .create_instance(address, &package_code.code)
                };

                let output = { scrypto_vm_instance.invoke(&export_name, input, api)? };

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
