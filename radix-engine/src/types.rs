pub use radix_engine_lib::address::{AddressError, Bech32Decoder, Bech32Encoder};
pub use radix_engine_lib::constants::*;
pub use radix_engine_lib::core::Expression;
pub use radix_engine_lib::crypto::*;
use radix_engine_lib::data::IndexedScryptoValue;
pub use radix_engine_lib::data::{scrypto_decode, scrypto_encode};
pub use radix_engine_lib::dec;
use radix_engine_lib::engine::types::{
    NativeMethod, RENodeId, ScryptoFunctionIdent, ScryptoMethodIdent,
};
pub use radix_engine_lib::engine::{types::*, wasm_input::RadixEngineInput};
pub use radix_engine_lib::math::{Decimal, RoundingMode, I256};
pub use radix_engine_lib::model::*;
pub use radix_engine_lib::scrypto;

pub use sbor::decode_any;
pub use sbor::rust::borrow::ToOwned;
pub use sbor::rust::boxed::Box;
pub use sbor::rust::cell::{Ref, RefCell, RefMut};
pub use sbor::rust::collections::*;
pub use sbor::rust::fmt;
pub use sbor::rust::format;
pub use sbor::rust::marker::PhantomData;
pub use sbor::rust::ops::AddAssign;
pub use sbor::rust::ptr;
pub use sbor::rust::rc::Rc;
pub use sbor::rust::str::FromStr;
pub use sbor::rust::string::String;
pub use sbor::rust::string::ToString;
pub use sbor::rust::vec;
pub use sbor::rust::vec::Vec;
pub use sbor::{Decode, DecodeError, Encode, SborPath, SborPathBuf, SborTypeId, SborValue, TypeId};

pub use scrypto::abi::{BlueprintAbi, Fields, Fn, Type, Variant};

use std::fmt::Debug;

// methods and macros
use crate::engine::Invocation;

/// Scrypto function/method invocation.
#[derive(Debug)]
pub enum ScryptoInvocation {
    Function(ScryptoFunctionIdent, IndexedScryptoValue),
    Method(ScryptoMethodIdent, IndexedScryptoValue),
}

impl Invocation for ScryptoInvocation {
    type Output = IndexedScryptoValue;
}

impl ScryptoInvocation {
    pub fn args(&self) -> &IndexedScryptoValue {
        match self {
            ScryptoInvocation::Function(_, args) => &args,
            ScryptoInvocation::Method(_, args) => &args,
        }
    }
}

#[derive(Debug)]
pub struct NativeMethodInvocation(pub NativeMethod, pub RENodeId, pub IndexedScryptoValue);

impl Invocation for NativeMethodInvocation {
    type Output = IndexedScryptoValue;
}

impl NativeMethodInvocation {
    pub fn args(&self) -> &IndexedScryptoValue {
        &self.2
    }
}
