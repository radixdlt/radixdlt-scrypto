use super::env::*;
use super::host_functions::*;
use radix_common::prelude::*;
use radix_engine::errors::InvokeError;
use radix_engine::vm::wasm::*;
use radix_wasmi::core::Value;
use radix_wasmi::*;

pub struct WasmiInstance {
    pub(super) store: Store<HostState>,
    pub(super) instance: Instance,
    pub(super) memory: Memory,
}

impl WasmiInstance {
    fn get_export_func(&mut self, name: &str) -> Result<Func, InvokeError<WasmRuntimeError>> {
        self.instance
            .get_export(self.store.as_context_mut(), name)
            .and_then(Extern::into_func)
            .ok_or_else(|| {
                InvokeError::SelfError(WasmRuntimeError::UnknownExport(name.to_string()))
            })
    }
}

impl WasmInstance for WasmiInstance {
    fn invoke_export<'r>(
        &mut self,
        func_name: &str,
        args: Vec<Buffer>,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
        {
            // set up runtime pointer
            // Using triple casting is to workaround this error message:
            // error[E0521]: borrowed data escapes outside of associated function
            //  `runtime` escapes the associated function body here argument requires that `'r` must outlive `'static`
            self.store
                .data_mut()
                .runtime_ptr
                .write(runtime as *mut _ as usize as *mut _);
        }

        let func = self.get_export_func(func_name).unwrap();
        let input: Vec<Value> = args
            .into_iter()
            .map(|buffer| Value::I64(buffer.as_i64()))
            .collect();
        let mut ret = [Value::I64(0)];

        let call_result = func
            .call(self.store.as_context_mut(), &input, &mut ret)
            .map_err(|e| {
                let err: InvokeError<WasmRuntimeError> = e.into();
                err
            });

        let result = match call_result {
            Ok(_) => match i64::try_from(ret[0]) {
                Ok(ret) => super::host_functions::read_slice(
                    self.store.as_context_mut(),
                    self.memory,
                    Slice::transmute_i64(ret),
                ),
                _ => Err(InvokeError::SelfError(WasmRuntimeError::InvalidWasmPointer)),
            },
            Err(err) => Err(err),
        };

        result
    }
}
