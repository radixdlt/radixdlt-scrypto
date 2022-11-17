use sbor::rust::collections::BTreeSet;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::misc::copy_u8_array;

use crate::abi::*;
use crate::data::ScryptoCustomTypeId;
use crate::engine::scrypto_env::*;
use crate::engine::{api::*, types::*};
use crate::math::*;
use crate::model::*;
use crate::scrypto;
use crate::scrypto_type;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct BucketTakeInvocation {
    pub receiver: BucketId,
    pub amount: Decimal,
}

impl SysInvocation for BucketTakeInvocation {
    type Output = Bucket;
}

impl ScryptoNativeInvocation for BucketTakeInvocation {}

impl Into<NativeFnInvocation> for BucketTakeInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Bucket(
            BucketMethodInvocation::Take(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct BucketPutInvocation {
    pub receiver: BucketId,
    pub bucket: Bucket,
}

impl SysInvocation for BucketPutInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for BucketPutInvocation {}

impl Into<NativeFnInvocation> for BucketPutInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Bucket(BucketMethodInvocation::Put(
            self,
        )))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct BucketTakeNonFungiblesInvocation {
    pub receiver: BucketId,
    pub ids: BTreeSet<NonFungibleId>,
}

impl SysInvocation for BucketTakeNonFungiblesInvocation {
    type Output = Bucket;
}

impl ScryptoNativeInvocation for BucketTakeNonFungiblesInvocation {}

impl Into<NativeFnInvocation> for BucketTakeNonFungiblesInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Bucket(
            BucketMethodInvocation::TakeNonFungibles(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct BucketGetNonFungibleIdsInvocation {
    pub receiver: BucketId,
}

impl SysInvocation for BucketGetNonFungibleIdsInvocation {
    type Output = BTreeSet<NonFungibleId>;
}

impl ScryptoNativeInvocation for BucketGetNonFungibleIdsInvocation {}

impl Into<NativeFnInvocation> for BucketGetNonFungibleIdsInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Bucket(
            BucketMethodInvocation::GetNonFungibleIds(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct BucketGetAmountInvocation {
    pub receiver: BucketId,
}

impl SysInvocation for BucketGetAmountInvocation {
    type Output = Decimal;
}

impl ScryptoNativeInvocation for BucketGetAmountInvocation {}

impl Into<NativeFnInvocation> for BucketGetAmountInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Bucket(
            BucketMethodInvocation::GetAmount(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct BucketGetResourceAddressInvocation {
    pub receiver: BucketId,
}

impl SysInvocation for BucketGetResourceAddressInvocation {
    type Output = ResourceAddress;
}

impl ScryptoNativeInvocation for BucketGetResourceAddressInvocation {}

impl Into<NativeFnInvocation> for BucketGetResourceAddressInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Bucket(
            BucketMethodInvocation::GetResourceAddress(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct BucketCreateProofInvocation {
    pub receiver: BucketId,
}

impl SysInvocation for BucketCreateProofInvocation {
    type Output = Proof;
}

impl ScryptoNativeInvocation for BucketCreateProofInvocation {}

impl Into<NativeFnInvocation> for BucketCreateProofInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Bucket(
            BucketMethodInvocation::CreateProof(self),
        ))
    }
}

/// Represents a transient resource container.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Bucket(pub BucketId);

//========
// error
//========

/// Represents an error when decoding bucket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseBucketError {
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseBucketError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseBucketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Bucket {
    type Error = ParseBucketError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            4 => Ok(Self(u32::from_le_bytes(copy_u8_array(slice)))),
            _ => Err(ParseBucketError::InvalidLength(slice.len())),
        }
    }
}

impl Bucket {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

scrypto_type!(Bucket, ScryptoCustomTypeId::Bucket, Type::Bucket, 4);
