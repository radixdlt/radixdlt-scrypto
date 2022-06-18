use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::prelude::{AccessRule, AccessRules};

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
                let output: sbor::rust::vec::Vec<u8> = call_engine(input);
                scrypto_decode(&output).unwrap()
            }
        )+
    };
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum RadixEngineInput {
    InvokeSNode(SNodeRef, String, Vec<u8>),
    CreateComponent(String, Vec<u8>, Vec<AccessRules>),
    GetComponentInfo(ComponentAddress),
    CreateKeyValueStore(),
    ReadData(DataAddress),
    WriteData(DataAddress, Vec<u8>),
    EmitLog(Level, String),
    GenerateUuid(),
    GetActor(),
    CheckAccessRule(AccessRule, Vec<ProofId>),
}
