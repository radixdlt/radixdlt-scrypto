use crate::blueprints::package::PackageCodeTypeSubstate;
use crate::errors::{RuntimeError, VmError};
use crate::kernel::kernel_api::{KernelInternalApi, KernelNodeApi, KernelSubstateApi};
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::types::*;
use crate::vm::vm::api::ClientApi;
use crate::vm::wasm::WasmEngine;
use crate::vm::{NativeVm, ScryptoVm};
use radix_engine_interface::api::substate_lock_api::LockFlags;
use radix_engine_interface::blueprints::package::*;

pub struct Vm<'g, W: WasmEngine> {
    pub scrypto_vm: &'g ScryptoVm<W>,
}

impl<'g, W: WasmEngine + 'g> SystemCallbackObject for Vm<'g, W> {
    fn invoke<Y>(
        address: &PackageAddress,
        receiver: Option<&NodeId>,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>
            + KernelInternalApi<SystemConfig<Self>>
            + KernelNodeApi
            + KernelSubstateApi,
        W: WasmEngine,
    {
        let code_type = {
            let handle = api.kernel_lock_substate(
                address.as_node_id(),
                SysModuleId::Object.into(),
                &PackageOffset::CodeType.into(),
                LockFlags::read_only(),
            )?;
            let code_type = api.kernel_read_substate(handle)?;
            let code_type: PackageCodeTypeSubstate = code_type.as_typed().unwrap();
            api.kernel_drop_lock(handle)?;
            code_type
        };

        let package_code = {
            let handle = api.kernel_lock_substate(
                address.as_node_id(),
                SysModuleId::Object.into(),
                &PackageOffset::Code.into(),
                LockFlags::read_only(),
            )?;
            let code = api.kernel_read_substate(handle)?;
            let package_code: PackageCodeSubstate = code.as_typed().unwrap();
            api.kernel_drop_lock(handle)?;
            package_code
        };

        let output = match code_type {
            PackageCodeTypeSubstate::Native => {
                let mut vm_instance = { NativeVm::create_instance(address, &package_code.code)? };
                let output = { vm_instance.invoke(receiver, &export_name, input, api)? };

                output
            }
            PackageCodeTypeSubstate::Wasm => {
                let mut scrypto_vm_instance = {
                    api.kernel_get_callback()
                        .callback_obj
                        .scrypto_vm
                        .create_instance(address, &package_code.code)
                        // It is expected that we can always create an instance of a package.
                        // All code is checked upon upload that it can be instantiated - so if it can't be instantiated
                        // now, something is fundamentally wrong. So for now, we panic here.
                        // In future - we could return an error instead, and have this flagged as a broken invariant
                        // in a TransactionResult::SystemInvariantFailed
                        // Which would let the node decide how to handle it (eg by logging/panicking).
                        // See thread here: https://github.com/radixdlt/radixdlt-scrypto/pull/984#discussion_r1181973193
                        .expect("Package WASM could not be instantiated despite the fact this was checked at upload time")
                };

                let output = { scrypto_vm_instance.invoke(receiver, &export_name, input, api)? };

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
        receiver: Option<&NodeId>,
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi;
}
