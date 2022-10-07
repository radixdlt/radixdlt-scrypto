use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::{Decode, Encode, TypeId};

use crate::core::{FnIdent, Level, ScryptoRENode};
use crate::engine::types::*;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Entrance to Radix Engine.
    pub fn radix_engine(input: *mut u8) -> *mut u8;
}

#[macro_export]
macro_rules! native_functions {
    ($receiver:expr, $type_ident:expr => { $($vis:vis $fn:ident $method_name:ident $s:tt -> $rtn:ty { $fn_ident:expr, $arg:expr })* } ) => {
        $(
            $vis $fn $method_name $s -> $rtn {
                let input = RadixEngineInput::Invoke(
                    scrypto::core::FnIdent::Method(scrypto::core::ReceiverMethodIdent {
                        receiver: $receiver,
                        method_ident: scrypto::core::MethodIdent::Native($type_ident($fn_ident)),
                    }),
                    scrypto::buffer::scrypto_encode(&$arg)
                );
                call_engine(input)
            }
        )+
    };
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum RadixEngineInput {
    Invoke(FnIdent, Vec<u8>),

    RENodeCreate(ScryptoRENode),
    RENodeGlobalize(RENodeId),
    GetOwnedRENodeIds(),

    SubstateRead(SubstateId),
    SubstateWrite(SubstateId, Vec<u8>),

    CreateRef(SubstateId, bool),
    DropRef(SubstateId),

    GetActor(),
    EmitLog(Level, String),
    GenerateUuid(),
}
