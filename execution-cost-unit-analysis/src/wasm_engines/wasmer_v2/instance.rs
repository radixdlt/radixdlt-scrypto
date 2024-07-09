use super::host_functions::*;
use radix_common::prelude::*;
use radix_engine::errors::InvokeError;
use radix_engine::vm::wasm::*;
use sbor::rust::sync::*;
use wasmer::*;

/// WARNING - this type should not actually be Send + Sync - it should really store a raw pointer,
/// not a raw pointer masked as a usize.
///
/// For information on why the pointer is masked, see the docs for `WasmerV2InstanceEnv`
pub struct WasmerV2Instance {
    pub(super) instance: Instance,

    /// This field stores a (masked) runtime pointer to a `Box<dyn WasmRuntime>` which is shared
    /// by the instance and each WasmerV2InstanceEnv (every function that requires `env`).
    ///
    /// On every call into the WASM (ie every call to `invoke_export`), a `&'a mut System API` is
    /// wrapped in a temporary `RadixEngineWasmRuntime<'a>` and boxed, and a pointer to the freshly
    /// created `Box<dyn WasmRuntime>` is written behind the Mutex into this field.
    ///
    /// This same Mutex (via Arc cloning) is shared into each `WasmerV2InstanceEnv`, and so
    /// when the WASM makes calls back into env, it can read the pointer to the current
    /// WasmRuntime, and use that to call into the `&mut System API`.
    ///
    /// For information on why the pointer is masked, see the docs for `WasmerV2InstanceEnv`
    pub(super) runtime_ptr: Arc<Mutex<usize>>,
}

impl WasmInstance for WasmerV2Instance {
    fn invoke_export<'r>(
        &mut self,
        func_name: &str,
        args: Vec<Buffer>,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
        {
            // set up runtime pointer
            let mut guard = self.runtime_ptr.lock().expect("Runtime ptr unavailable");
            *guard = runtime as *mut _ as usize;
        }

        let input: Vec<Val> = args
            .into_iter()
            .map(|buffer| Val::I64(buffer.as_i64()))
            .collect();
        let return_data = self
            .instance
            .exports
            .get_function(func_name)
            .map_err(|_| {
                InvokeError::SelfError(WasmRuntimeError::UnknownExport(func_name.to_string()))
            })?
            .call(&input)
            .map_err(|error| {
                let e_str = format!("{:?}", error);
                match error.downcast::<InvokeError<WasmRuntimeError>>() {
                    Ok(e) => e,
                    _ => InvokeError::SelfError(WasmRuntimeError::ExecutionError(e_str)),
                }
            });

        let result = match return_data {
            Ok(data) => {
                if let Some(v) = data.as_ref().first().and_then(|x| x.i64()) {
                    read_slice(&self.instance, Slice::transmute_i64(v))
                        .map_err(InvokeError::SelfError)
                } else {
                    Err(InvokeError::SelfError(WasmRuntimeError::InvalidWasmPointer))
                }
            }
            Err(err) => Err(err),
        };

        result
    }
}
