use scrypto::prelude::*;

#[blueprint]
mod wasm_non_mvp {
    struct WasmNonMvp {}

    impl WasmNonMvp {
        // If 'sign-ext' feature is enabled for wasm32 target then it is expected that below
        //  function shall be compiled into such WASM code:
        //  (
        //      func $generate_sign_ext_opcode (type 1) (param i32 i32) (result i32)
        //          local.get 1
        //          local.get 0
        //          i32.add
        //          i32.extend8_s   ;; non-MVP WASM sign-extension operator that is expected to occur
        //  )
        pub fn generate_sign_ext_opcode(&mut self, a: u8, b: u8) -> i32 {
            (a + b) as i32
        }
    }
}
