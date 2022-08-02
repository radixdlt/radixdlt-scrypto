use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::prelude::AccessRule;

use crate::core::{DataAddress, Receiver};
use crate::engine::types::*;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Entrance to Radix Engine.
    pub fn radix_engine(input: *mut u8) -> *mut u8;
}

#[macro_export]
macro_rules! sfunctions {
    ($receiver:expr => { $($vis:vis $fn:ident $method_name:ident $s:tt -> $rtn:ty { $arg:expr })* } ) => {
        $(
            $vis $fn $method_name $s -> $rtn {
                let input = RadixEngineInput::InvokeFunction(
                    $receiver,
                    stringify!($method_name).to_string(),
                    scrypto::buffer::scrypto_encode(&$arg)
                );
                call_engine(input)
            }
        )+
    };
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum RadixEngineInput {
    InvokeFunction(Receiver, String, Vec<u8>),
    CreateComponent(String, Vec<u8>),
    CreateKeyValueStore(),
    ReadData(DataAddress),
    WriteData(DataAddress, Vec<u8>),
    GetActor(),
    EmitLog(Level, String),
    GenerateUuid(),
    CheckAccessRule(AccessRule, Vec<ProofId>),
}
