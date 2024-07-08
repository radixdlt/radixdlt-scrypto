use radix_common::prelude::*;
use radix_engine::vm::wasm::*;
use sbor::rust::mem::MaybeUninit;

pub type FakeHostState = FakeWasmiInstanceEnv;
pub type HostState = WasmiInstanceEnv;

/// This is to construct a stub `Store<FakeWasmiInstanceEnv>`, which is a part of
/// `WasmiModule` struct and serves as a placeholder for the real `Store<WasmiInstanceEnv>`.
/// The real store is set (prior being transumted) when the `WasmiModule` is being instantiated.
/// In fact the only difference between a stub and real Store is the `Send + Sync` manually
/// implemented for the former one, which is required by `WasmiModule` cache (for `std`
/// configuration) but shall not be implemented for the latter one to prevent sharing it between
/// the threads since pointer might point to volatile data.
#[derive(Clone)]
pub struct FakeWasmiInstanceEnv {
    #[allow(dead_code)]
    pub(super) runtime_ptr: MaybeUninit<*mut Box<dyn WasmRuntime>>,
}

impl FakeWasmiInstanceEnv {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            runtime_ptr: MaybeUninit::uninit(),
        }
    }
}

unsafe impl Send for FakeWasmiInstanceEnv {}
unsafe impl Sync for FakeWasmiInstanceEnv {}

/// This is to construct a real `Store<WasmiInstanceEnv>
pub struct WasmiInstanceEnv {
    pub(super) runtime_ptr: MaybeUninit<*mut Box<dyn WasmRuntime>>,
}

impl WasmiInstanceEnv {
    pub fn new() -> Self {
        Self {
            runtime_ptr: MaybeUninit::uninit(),
        }
    }
}
