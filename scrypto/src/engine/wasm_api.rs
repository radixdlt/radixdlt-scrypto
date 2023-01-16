use radix_engine_interface::wasm::*;

#[cfg(target_arch = "wasm32")]
extern "C" {
    pub fn radix_engine(api: u8, input: *mut u8) -> *mut u8;
}

/// Utility function for making a radix engine call.
#[cfg(target_arch = "wasm32")]
pub fn call_engine_wasm_api<W: EngineWasmApi>(input: W::Input) -> W::Output {
    use crate::buffer::*;

    let input_ptr = scrypto_encode_to_buffer(&input).unwrap();
    let output_ptr = unsafe { radix_engine(W::ID, input_ptr) };
    scrypto_decode_from_buffer(output_ptr).unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn call_engine_wasm_api<W: EngineWasmApi>(_input: W::Input) -> W::Output {
    todo!()
}
