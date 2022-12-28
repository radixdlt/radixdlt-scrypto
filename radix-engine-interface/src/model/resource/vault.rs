use sbor::rust::borrow::ToOwned;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

use crate::abi::*;
use crate::api::{api::*, types::*};
use crate::data::ScryptoCustomTypeId;
use crate::math::*;
use crate::scrypto;
use crate::scrypto_type;
use crate::wasm::*;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct VaultPutInvocation {
    pub receiver: VaultId,
    pub bucket: Bucket,
}

impl Clone for VaultPutInvocation {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver,
            bucket: Bucket(self.bucket.0),
        }
    }
}

impl Invocation for VaultPutInvocation {
    type Output = ();
}

impl SerializableInvocation for VaultPutInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for VaultPutInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Vault(VaultInvocation::Put(self)).into()
    }
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct VaultTakeInvocation {
    pub receiver: VaultId,
    pub amount: Decimal,
}

impl Invocation for VaultTakeInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for VaultTakeInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<SerializedInvocation> for VaultTakeInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Vault(VaultInvocation::Take(self)).into()
    }
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct VaultTakeNonFungiblesInvocation {
    pub receiver: VaultId,
    pub non_fungible_ids: BTreeSet<NonFungibleId>,
}

impl Invocation for VaultTakeNonFungiblesInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for VaultTakeNonFungiblesInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<SerializedInvocation> for VaultTakeNonFungiblesInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Vault(VaultInvocation::TakeNonFungibles(self)).into()
    }
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct VaultGetAmountInvocation {
    pub receiver: VaultId,
}

impl Invocation for VaultGetAmountInvocation {
    type Output = Decimal;
}

impl SerializableInvocation for VaultGetAmountInvocation {
    type ScryptoOutput = Decimal;
}

impl Into<SerializedInvocation> for VaultGetAmountInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Vault(VaultInvocation::GetAmount(self)).into()
    }
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct VaultRecallInvocation {
    pub receiver: VaultId,
    pub amount: Decimal,
}

impl Invocation for VaultRecallInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for VaultRecallInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<SerializedInvocation> for VaultRecallInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Vault(VaultInvocation::Recall(self)).into()
    }
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct VaultRecallNonFungiblesInvocation {
    pub receiver: VaultId,
    pub non_fungible_ids: BTreeSet<NonFungibleId>,
}

impl Invocation for VaultRecallNonFungiblesInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for VaultRecallNonFungiblesInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<SerializedInvocation> for VaultRecallNonFungiblesInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Vault(VaultInvocation::RecallNonFungibles(self)).into()
    }
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct VaultGetResourceAddressInvocation {
    pub receiver: VaultId,
}

impl Invocation for VaultGetResourceAddressInvocation {
    type Output = ResourceAddress;
}

impl SerializableInvocation for VaultGetResourceAddressInvocation {
    type ScryptoOutput = ResourceAddress;
}

impl Into<SerializedInvocation> for VaultGetResourceAddressInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Vault(VaultInvocation::GetResourceAddress(self)).into()
    }
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct VaultGetNonFungibleIdsInvocation {
    pub receiver: VaultId,
}

impl Invocation for VaultGetNonFungibleIdsInvocation {
    type Output = BTreeSet<NonFungibleId>;
}

impl SerializableInvocation for VaultGetNonFungibleIdsInvocation {
    type ScryptoOutput = BTreeSet<NonFungibleId>;
}

impl Into<SerializedInvocation> for VaultGetNonFungibleIdsInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Vault(VaultInvocation::GetNonFungibleIds(self)).into()
    }
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct VaultCreateProofInvocation {
    pub receiver: VaultId,
}

impl Invocation for VaultCreateProofInvocation {
    type Output = Proof;
}

impl SerializableInvocation for VaultCreateProofInvocation {
    type ScryptoOutput = Proof;
}

impl Into<SerializedInvocation> for VaultCreateProofInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Vault(VaultInvocation::CreateProof(self)).into()
    }
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct VaultCreateProofByAmountInvocation {
    pub receiver: VaultId,
    pub amount: Decimal,
}

impl Invocation for VaultCreateProofByAmountInvocation {
    type Output = Proof;
}

impl SerializableInvocation for VaultCreateProofByAmountInvocation {
    type ScryptoOutput = Proof;
}

impl Into<SerializedInvocation> for VaultCreateProofByAmountInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Vault(VaultInvocation::CreateProofByAmount(self)).into()
    }
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct VaultCreateProofByIdsInvocation {
    pub receiver: VaultId,
    pub ids: BTreeSet<NonFungibleId>,
}

impl Invocation for VaultCreateProofByIdsInvocation {
    type Output = Proof;
}

impl SerializableInvocation for VaultCreateProofByIdsInvocation {
    type ScryptoOutput = Proof;
}

impl Into<SerializedInvocation> for VaultCreateProofByIdsInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Vault(VaultInvocation::CreateProofByIds(self)).into()
    }
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct VaultLockFeeInvocation {
    pub receiver: VaultId,
    pub amount: Decimal,
    pub contingent: bool,
}

impl Invocation for VaultLockFeeInvocation {
    type Output = ();
}

impl SerializableInvocation for VaultLockFeeInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for VaultLockFeeInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Vault(VaultInvocation::LockFee(self)).into()
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct Vault(pub VaultId);

//========
// error
//========

/// Represents an error when decoding vault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseVaultError {
    InvalidHex(String),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseVaultError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseVaultError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Vault {
    type Error = ParseVaultError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            36 => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParseVaultError::InvalidLength(slice.len())),
        }
    }
}

impl Vault {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(Vault, ScryptoCustomTypeId::Vault, Type::Vault, 36);

//======
// text
//======

impl FromStr for Vault {
    type Err = ParseVaultError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ParseVaultError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Vault {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Vault {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
