use crate::abi::*;
use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::data::types::Own;
use crate::data::ScryptoCustomValueKind;
use crate::math::*;
use crate::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

pub const BUCKET_BLUEPRINT: &str = "Bucket";

pub const BUCKET_TAKE_IDENT: &str = "Bucket_take";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct BucketTakeInput {
    pub amount: Decimal,
}

pub const BUCKET_PUT_IDENT: &str = "Bucket_put";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct BucketPutInput {
    pub bucket: Bucket,
}

impl Clone for BucketPutInput {
    fn clone(&self) -> Self {
        Self {
            bucket: Bucket(self.bucket.0),
        }
    }
}

pub const BUCKET_TAKE_NON_FUNGIBLES_IDENT: &str = "Bucket_take_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct BucketTakeNonFungiblesInput {
    pub ids: BTreeSet<NonFungibleLocalId>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct BucketGetNonFungibleLocalIdsInvocation {
    pub receiver: BucketId,
}

impl Invocation for BucketGetNonFungibleLocalIdsInvocation {
    type Output = BTreeSet<NonFungibleLocalId>;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Bucket(BucketFn::GetNonFungibleLocalIds))
    }
}

impl SerializableInvocation for BucketGetNonFungibleLocalIdsInvocation {
    type ScryptoOutput = BTreeSet<NonFungibleLocalId>;

    fn native_fn() -> NativeFn {
        NativeFn::Bucket(BucketFn::GetNonFungibleLocalIds)
    }
}

impl Into<CallTableInvocation> for BucketGetNonFungibleLocalIdsInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Bucket(BucketInvocation::GetNonFungibleLocalIds(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct BucketGetAmountInvocation {
    pub receiver: BucketId,
}

impl Invocation for BucketGetAmountInvocation {
    type Output = Decimal;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Bucket(BucketFn::GetAmount))
    }
}

impl SerializableInvocation for BucketGetAmountInvocation {
    type ScryptoOutput = Decimal;

    fn native_fn() -> NativeFn {
        NativeFn::Bucket(BucketFn::GetAmount)
    }
}

impl Into<CallTableInvocation> for BucketGetAmountInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Bucket(BucketInvocation::GetAmount(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct BucketGetResourceAddressInvocation {
    pub receiver: BucketId,
}

impl Invocation for BucketGetResourceAddressInvocation {
    type Output = ResourceAddress;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Bucket(BucketFn::GetResourceAddress))
    }
}

impl SerializableInvocation for BucketGetResourceAddressInvocation {
    type ScryptoOutput = ResourceAddress;

    fn native_fn() -> NativeFn {
        NativeFn::Bucket(BucketFn::GetResourceAddress)
    }
}

impl Into<CallTableInvocation> for BucketGetResourceAddressInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Bucket(BucketInvocation::GetResourceAddress(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct BucketCreateProofInvocation {
    pub receiver: BucketId,
}

impl Invocation for BucketCreateProofInvocation {
    type Output = Proof;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Bucket(BucketFn::CreateProof))
    }
}

impl SerializableInvocation for BucketCreateProofInvocation {
    type ScryptoOutput = Proof;

    fn native_fn() -> NativeFn {
        NativeFn::Bucket(BucketFn::CreateProof)
    }
}

impl Into<CallTableInvocation> for BucketCreateProofInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Bucket(BucketInvocation::CreateProof(self)).into()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Bucket(pub BucketId); // scrypto stub

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Bucket {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Bucket {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Own::Bucket(self.0).encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Bucket {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let o = Own::decode_body_with_value_kind(decoder, value_kind)?;
        match o {
            Own::Bucket(bucket_id) => Ok(Self(bucket_id)),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl scrypto_abi::LegacyDescribe for Bucket {
    fn describe() -> scrypto_abi::Type {
        Type::Bucket
    }
}
