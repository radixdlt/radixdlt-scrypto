use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::prelude::AccessRule;

use crate::core::{DataAddress, SNodeRef};
use crate::engine::types::*;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Entrance to Radix Engine.
    pub fn radix_engine(input: *mut u8) -> *mut u8;
}

#[macro_export]
macro_rules! sfunctions {
    ($snode_ref:expr => { $($vis:vis $fn:ident $method_name:ident $s:tt -> $rtn:ty { $arg:expr })* } ) => {
        $(
            $vis $fn $method_name $s -> $rtn {
                let input = RadixEngineInput::InvokeSNode(
                    $snode_ref,
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
    InvokeSNode(SNodeRef, String, Vec<u8>),
    Globalize(ComponentAddress),
    CreateComponent(String, Vec<u8>),
    CreateKeyValueStore(),
    GetActor(),
    ReadData(DataAddress),
    WriteData(DataAddress, Vec<u8>),
    EmitLog(Level, String),
    GenerateUuid(),
    CheckAccessRule(AccessRule, Vec<ProofId>),
}
