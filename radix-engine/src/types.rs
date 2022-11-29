pub use radix_engine_interface::abi::{BlueprintAbi, Fields, Fn, Type, Variant};
pub use radix_engine_interface::address::{AddressError, Bech32Decoder, Bech32Encoder};
pub use radix_engine_interface::api::types::*;
pub use radix_engine_interface::constants::*;
pub use radix_engine_interface::core::Expression;
pub use radix_engine_interface::crypto::*;
pub use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode, IndexedScryptoValue, ScryptoDecode, ScryptoEncode,
    ScryptoTypeId,
};
pub use radix_engine_interface::dec;
pub use radix_engine_interface::math::{Decimal, RoundingMode, I256};
pub use radix_engine_interface::model::*;
pub use radix_engine_interface::scrypto;
pub use sbor::rust::borrow::ToOwned;
pub use sbor::rust::boxed::Box;
pub use sbor::rust::cell::{Ref, RefCell, RefMut};
pub use sbor::rust::collections::*;
pub use sbor::rust::fmt;
pub use sbor::rust::format;
pub use sbor::rust::marker::PhantomData;
pub use sbor::rust::num::NonZeroUsize;
pub use sbor::rust::ops::AddAssign;
pub use sbor::rust::ptr;
pub use sbor::rust::rc::Rc;
pub use sbor::rust::str::FromStr;
pub use sbor::rust::string::String;
pub use sbor::rust::string::ToString;
pub use sbor::rust::vec;
pub use sbor::rust::vec::Vec;
pub use sbor::{Decode, DecodeError, Encode, SborPath, SborPathBuf, SborTypeId, SborValue, TypeId};

use radix_engine_interface::api::api::Invocation;

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
