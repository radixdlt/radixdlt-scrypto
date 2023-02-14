use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::math::*;
use crate::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

pub const VAULT_BLUEPRINT: &str = "Vault";

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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Vault(VaultFn::Put))
    }
}

impl SerializableInvocation for VaultPutInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Vault(VaultFn::Put)
    }
}

impl Into<CallTableInvocation> for VaultPutInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::Put(self)).into()
    }
}

pub const VAULT_TAKE_IDENT: &str = "take";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct VaultTakeInput {
    pub amount: Decimal,
}

pub const VAULT_TAKE_NON_FUNGIBLES_IDENT: &str = "take_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct VaultTakeNonFungiblesInput {
    pub non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
}

pub const VAULT_LOCK_FEE_IDENT: &str = "lock_fee";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct VaultLockFeeInput {
    pub amount: Decimal,
    pub contingent: bool,
}

pub const VAULT_RECALL_IDENT: &str = "recall";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct VaultRecallInput {
    pub amount: Decimal,
}

pub const VAULT_RECALL_NON_FUNGIBLES_IDENT: &str = "recall_non_fungibles";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct VaultRecallNonFungiblesInput {
    pub non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct VaultGetAmountInvocation {
    pub receiver: VaultId,
}

impl Invocation for VaultGetAmountInvocation {
    type Output = Decimal;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Vault(VaultFn::GetAmount))
    }
}

impl SerializableInvocation for VaultGetAmountInvocation {
    type ScryptoOutput = Decimal;

    fn native_fn() -> NativeFn {
        NativeFn::Vault(VaultFn::GetAmount)
    }
}

impl Into<CallTableInvocation> for VaultGetAmountInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::GetAmount(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct VaultGetResourceAddressInvocation {
    pub receiver: VaultId,
}

impl Invocation for VaultGetResourceAddressInvocation {
    type Output = ResourceAddress;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Vault(VaultFn::GetResourceAddress))
    }
}

impl SerializableInvocation for VaultGetResourceAddressInvocation {
    type ScryptoOutput = ResourceAddress;

    fn native_fn() -> NativeFn {
        NativeFn::Vault(VaultFn::GetResourceAddress)
    }
}

impl Into<CallTableInvocation> for VaultGetResourceAddressInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::GetResourceAddress(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct VaultGetNonFungibleLocalIdsInvocation {
    pub receiver: VaultId,
}

impl Invocation for VaultGetNonFungibleLocalIdsInvocation {
    type Output = BTreeSet<NonFungibleLocalId>;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Vault(VaultFn::GetNonFungibleLocalIds))
    }
}

impl SerializableInvocation for VaultGetNonFungibleLocalIdsInvocation {
    type ScryptoOutput = BTreeSet<NonFungibleLocalId>;

    fn native_fn() -> NativeFn {
        NativeFn::Vault(VaultFn::GetNonFungibleLocalIds)
    }
}

impl Into<CallTableInvocation> for VaultGetNonFungibleLocalIdsInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::GetNonFungibleLocalIds(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct VaultCreateProofInvocation {
    pub receiver: VaultId,
}

impl Invocation for VaultCreateProofInvocation {
    type Output = Proof;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Vault(VaultFn::CreateProof))
    }
}

impl SerializableInvocation for VaultCreateProofInvocation {
    type ScryptoOutput = Proof;

    fn native_fn() -> NativeFn {
        NativeFn::Vault(VaultFn::CreateProof)
    }
}

impl Into<CallTableInvocation> for VaultCreateProofInvocation {
    fn into(self) -> CallTableInvocation {
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Vault(VaultFn::CreateProofByAmount))
    }
}

impl SerializableInvocation for VaultCreateProofByAmountInvocation {
    type ScryptoOutput = Proof;

    fn native_fn() -> NativeFn {
        NativeFn::Vault(VaultFn::CreateProofByAmount)
    }
}

impl Into<CallTableInvocation> for VaultCreateProofByAmountInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::CreateProofByAmount(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct VaultCreateProofByIdsInvocation {
    pub receiver: VaultId,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

impl Invocation for VaultCreateProofByIdsInvocation {
    type Output = Proof;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Vault(VaultFn::CreateProofByIds))
    }
}

impl SerializableInvocation for VaultCreateProofByIdsInvocation {
    type ScryptoOutput = Proof;

    fn native_fn() -> NativeFn {
        NativeFn::Vault(VaultFn::CreateProofByIds)
    }
}

impl Into<CallTableInvocation> for VaultCreateProofByIdsInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Vault(VaultInvocation::CreateProofByIds(self)).into()
    }
}
