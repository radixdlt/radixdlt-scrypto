use scrypto::prelude::*;

#[blueprint]
mod wasm_limits {
    struct WasmLimits {}

    impl WasmLimits {
        pub fn create_buffers(n: usize) {
            for _ in 0..n {
                let _ = unsafe { wasm_api::actor::actor_get_blueprint_name() };
            }
        }
    }
}
