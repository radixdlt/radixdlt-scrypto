use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::prelude::{AccessRule, AccessRules};

use crate::core::SNodeRef;
use crate::engine::types::*;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Entrance to Radix Engine.
    pub fn radix_engine(input: *mut u8) -> *mut u8;
}

#[macro_export]
macro_rules! sfunctions {
    ($snode_ref:expr => { $($vis:vis $fn:ident $method_name:ident $s:tt -> $rtn:ty { $method_enum:expr })* } ) => {
        $(
            $vis $fn $method_name $s -> $rtn {
                let input = RadixEngineInput::InvokeSNode(
                    $snode_ref,
                    scrypto::buffer::scrypto_encode(&$method_enum)
                );
                let output: sbor::rust::vec::Vec<u8> = call_engine(input);
                scrypto_decode(&output).unwrap()
            }
        )+
    };
}

#[macro_export]
macro_rules! sfunctions2 {
    ($snode_ref:expr => { $($vis:vis $fn:ident $method_name:ident $s:tt -> $rtn:ty { $arg:expr })* } ) => {
        $(
            $vis $fn $method_name $s -> $rtn {
                let input = RadixEngineInput::InvokeSNode2(
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
    InvokeSNode(SNodeRef, Vec<u8>),
    InvokeSNode2(SNodeRef, String, Vec<u8>),
    CreateComponent(String, Vec<u8>, Vec<AccessRules>),
    GetComponentInfo(ComponentAddress),
    GetComponentState(ComponentAddress),
    PutComponentState(ComponentAddress, Vec<u8>),
    CreateLazyMap(),
    GetLazyMapEntry(KeyValueStoreId, Vec<u8>),
    PutLazyMapEntry(KeyValueStoreId, Vec<u8>, Vec<u8>),
    EmitLog(Level, String),
    GenerateUuid(),
    GetActor(),
    CheckAccessRule(AccessRule, Vec<ProofId>),
}
