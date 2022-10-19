use sbor::Decode;

use super::api::RadixEngineInput;

/// Utility function for making a radix engine call.
#[cfg(target_arch = "wasm32")]
pub fn call_engine<V: Decode>(input: RadixEngineInput) -> V {
    use crate::buffer::{scrypto_decode_from_buffer, *};
    use crate::engine::api::radix_engine;

    unsafe {
        let input_ptr = scrypto_encode_to_buffer(&input);
        let output_ptr = radix_engine(input_ptr);
        scrypto_decode_from_buffer::<V>(output_ptr).unwrap()
    }
}

/// Utility function for making a radix engine call.
#[cfg(not(target_arch = "wasm32"))]
pub fn call_engine<V: Decode>(_input: RadixEngineInput) -> V {
    todo!()
}

#[macro_export]
macro_rules! native_functions {
    ($receiver:expr, $type_ident:expr => { $($vis:vis $fn:ident $method_name:ident $s:tt -> $rtn:ty { $fn_ident:expr, $arg:expr })* } ) => {
        $(
            $vis $fn $method_name $s -> $rtn {
                let input = RadixEngineInput::InvokeNativeMethod(
                    scrypto::engine::types::NativeMethodIdent {
                        receiver: $receiver,
                        method_name: $fn_ident.to_string(),
                    },
                    scrypto::buffer::scrypto_encode(&$arg)
                );
                call_engine(input)
            }
        )+
    };
}
