use crate::api::{api::*, types::*};
use crate::math::*;
use crate::wasm::*;
use crate::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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
