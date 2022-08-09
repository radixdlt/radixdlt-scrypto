use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::{Decode, Encode, TypeId};
use scrypto::prelude::AccessRule;

use crate::core::{FnIdentifier, Receiver, ScryptoRENode, TypeName};
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
                let input = RadixEngineInput::InvokeMethod(
                    $receiver,
                    scrypto::core::FnIdentifier::Native(stringify!($method_name).to_string()),
                    scrypto::buffer::scrypto_encode(&$arg)
                );
                call_engine(input)
            }
        )+
    };
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum RadixEngineInput {
    InvokeFunction(TypeName, String, Vec<u8>),
    InvokeMethod(Receiver, FnIdentifier, Vec<u8>),
    RENodeCreate(ScryptoRENode),
    RENodeGlobalize(RENodeId),
    SubstateRead(SubstateId),
    SubstateWrite(SubstateId, Vec<u8>),
    GetActor(),
    EmitLog(Level, String),
    GenerateUuid(),
    CheckAccessRule(AccessRule, Vec<ProofId>),
}
