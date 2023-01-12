use crate::api::{api::*, types::*};
use crate::math::*;
use crate::scrypto;
use crate::wasm::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

#[derive(Debug, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
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

impl Into<CallTableInvocation> for VaultPutInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::Put(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
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

impl Into<CallTableInvocation> for VaultTakeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::Take(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
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

impl Into<CallTableInvocation> for VaultTakeNonFungiblesInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::TakeNonFungibles(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct VaultGetAmountInvocation {
    pub receiver: VaultId,
}

impl Invocation for VaultGetAmountInvocation {
    type Output = Decimal;
}

impl SerializableInvocation for VaultGetAmountInvocation {
    type ScryptoOutput = Decimal;
}

impl Into<CallTableInvocation> for VaultGetAmountInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::GetAmount(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
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

impl Into<CallTableInvocation> for VaultRecallInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::Recall(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
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

impl Into<CallTableInvocation> for VaultRecallNonFungiblesInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::RecallNonFungibles(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct VaultGetResourceAddressInvocation {
    pub receiver: VaultId,
}

impl Invocation for VaultGetResourceAddressInvocation {
    type Output = ResourceAddress;
}

impl SerializableInvocation for VaultGetResourceAddressInvocation {
    type ScryptoOutput = ResourceAddress;
}

impl Into<CallTableInvocation> for VaultGetResourceAddressInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::GetResourceAddress(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct VaultGetNonFungibleIdsInvocation {
    pub receiver: VaultId,
}

impl Invocation for VaultGetNonFungibleIdsInvocation {
    type Output = BTreeSet<NonFungibleId>;
}

impl SerializableInvocation for VaultGetNonFungibleIdsInvocation {
    type ScryptoOutput = BTreeSet<NonFungibleId>;
}

impl Into<CallTableInvocation> for VaultGetNonFungibleIdsInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::GetNonFungibleIds(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct VaultCreateProofInvocation {
    pub receiver: VaultId,
}

impl Invocation for VaultCreateProofInvocation {
    type Output = Proof;
}

impl SerializableInvocation for VaultCreateProofInvocation {
    type ScryptoOutput = Proof;
}

impl Into<CallTableInvocation> for VaultCreateProofInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::CreateProof(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
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

impl Into<CallTableInvocation> for VaultCreateProofByAmountInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::CreateProofByAmount(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
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

impl Into<CallTableInvocation> for VaultCreateProofByIdsInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::CreateProofByIds(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
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

impl Into<CallTableInvocation> for VaultLockFeeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::LockFee(self)).into()
    }
}
