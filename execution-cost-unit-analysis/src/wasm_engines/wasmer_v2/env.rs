use radix_common::prelude::*;
use std::sync::*;
use wasmer::*;

/// The WasmerV2InstanceEnv implements WasmerEnv - and this needs to be `Send + Sync` for
/// Wasmer to work (see `Function::new_native_with_env`).
///
/// This is likely because Wasmer wants to be forward-compatible with multi-threaded WASM,
/// or that it uses multiple threads internally.
///
/// Currently, the SystemAPI is not Sync (and so should not be accessed by multiple threads)
/// we believe our use of Wasmer does not allow it to call us from multiple threads -
/// but we need to double-check this.
///
/// In any case, we temporarily work around this incompatibility by masking the pointer as a usize.
///
/// There are still a number of changes we should consider to improve things:
/// * `WasmerV2InstanceEnv` shouldn't contain an Instance - just a memory reference - see
///    the docs on the `WasmerEnv` trait
/// * If we instantiate the module just before we call into it, we could potentially pass an actual
///   `Arc<Mutex<T>>` for `a', T: WasmRuntime<'a>` (wrapping a `&'a mut SystemAPI`) into the WasmerV2InstanceEnv
///   on *module instantiation*. In this case, it doesn't need to be on a WasmerV2Instance at all
/// * Else at the very least, change this to be a pointer type, and manually implement Sync/Send
#[derive(Clone)]
pub struct WasmerV2InstanceEnv {
    pub(super) instance: LazyInit<Instance>,
    /// See notes on `WasmerV2Instance.runtime_ptr`
    pub(super) runtime_ptr: Arc<Mutex<usize>>,
}

impl WasmerEnv for WasmerV2InstanceEnv {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.instance.initialize(instance.clone());
        Ok(())
    }
}
