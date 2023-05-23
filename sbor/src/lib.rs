#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// SBOR basic, no custom types
pub mod basic;
/// SBOR Categorize trait
pub mod categorize;
/// SBOR codec for core Rust types.
pub mod codec;
/// SBOR constants
pub mod constants;
/// SBOR decode trait.
pub mod decode;
/// SBOR decoding.
pub mod decoder;
/// SBOR encode trait.
pub mod encode;
/// SBOR payload wrappers.
/// These are new types around an encoded payload or sub-payload, with helper methods / traits implemented.
/// They can be used as a more efficient wrapper a ScryptoValue if the content of that value is not needed.
pub mod encoded_wrappers;
/// SBOR encoding.
pub mod encoder;
mod enum_variant;
/// SBOR paths.
pub mod path;
/// SBOR payload validation.
pub mod payload_validation;
/// SBOR textual representations
pub mod representations;
/// A facade of Rust types.
pub mod rust;
/// SBOR Schema
pub mod schema;
/// SBOR structured payload traversal.
pub mod traversal;
/// SBOR value model and any decoding/encoding.
pub mod value;
/// SBOR value kinds - ie the types of value that are supported.
pub mod value_kind;

pub use basic::*;
pub(crate) use categorize::{categorize_generic, categorize_simple};
pub use categorize::{Categorize, SborEnum, SborTuple};
pub use constants::*;
pub use decode::Decode;
pub use decoder::{BorrowingDecoder, DecodeError, Decoder, VecDecoder};
pub use encode::Encode;
pub use encoder::{EncodeError, Encoder, VecEncoder};
pub use path::{SborPath, SborPathBuf};

pub use encoded_wrappers::*;
pub use enum_variant::*;
pub use payload_validation::*;
pub use schema::*;
pub use value::*;
pub use value_kind::*;

// Re-export derives
extern crate sbor_derive;
pub use sbor_derive::{
    BasicCategorize, BasicDecode, BasicDescribe, BasicEncode, BasicSbor, Categorize, Decode,
    Describe, Encode, Sbor,
};

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
extern crate self as sbor;

/// Each module should have its own prelude, which:
/// * Adds preludes of upstream crates
/// * Exports types with specific-enough names which mean they can safely be used downstream.
///
/// The idea is that we can just include the current crate's prelude and avoid messing around with tons of includes.
/// This makes refactors easier, and makes integration into the node less painful.
///
/// Feel free to add more types to the prelude
pub mod prelude {
    // Upstream preludes
    pub use utils::prelude::*;

    // Exports from current crate
    pub use crate::encoded_wrappers::{RawPayload as SborRawPayload, RawValue as SborRawValue};
    pub use crate::enum_variant::FixedEnumVariant as SborFixedEnumVariant;
    pub use crate::path::{SborPath, SborPathBuf};
    pub use crate::representations;
    pub use crate::value::{CustomValue as SborCustomValue, Value as SborValue};
    pub use crate::value_kind::*;
    pub use crate::{
        basic_decode, basic_encode, BasicCategorize, BasicDecode, BasicDescribe, BasicEncode,
        BasicSbor,
    };
    pub use crate::{Categorize, Decode, Encode, Sbor, SborEnum, SborTuple};
    pub use crate::{DecodeError, EncodeError};
}
