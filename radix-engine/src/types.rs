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
pub use scrypto::address::{AddressError, Bech32Decoder, Bech32Encoder};
pub use scrypto::component::{
    ComponentAddAccessCheckInvocation, ComponentAddress, PackageAddress, PackagePublishInvocation,
};
pub use scrypto::constants::*;
pub use scrypto::core::{
    Blob, EpochManagerCreateInvocation, EpochManagerGetCurrentEpochInvocation,
    EpochManagerSetEpochInvocation, Expression, NetworkDefinition, ScryptoActor, SystemAddress,
};
pub use scrypto::crypto::{
    EcdsaSecp256k1PublicKey, EcdsaSecp256k1Signature, EddsaEd25519PublicKey, EddsaEd25519Signature,
    Hash, PublicKey, Signature,
};
pub use scrypto::data::*;
pub use scrypto::engine::{api::RadixEngineInput, types::*};
pub use scrypto::math::{Decimal, RoundingMode, I256};
pub use scrypto::resource::{
    AccessRule, AccessRuleNode, AccessRules, AuthZoneClearInvocation,
    AuthZoneCreateProofByAmountInvocation, AuthZoneCreateProofByIdsInvocation,
    AuthZoneCreateProofInvocation, AuthZonePopInvocation, AuthZonePushInvocation,
    BucketCreateProofInvocation, BucketGetAmountInvocation, BucketGetNonFungibleIdsInvocation,
    BucketGetResourceAddressInvocation, BucketPutInvocation, BucketTakeInvocation,
    BucketTakeNonFungiblesInvocation, MintParams, Mutability, NonFungibleAddress, NonFungibleId,
    ProofCloneInvocation, ProofGetAmountInvocation, ProofGetNonFungibleIdsInvocation,
    ProofGetResourceAddressInvocation, ProofRule, ResourceAddress, ResourceManagerBurnInvocation,
    ResourceManagerCreateBucketInvocation, ResourceManagerCreateInvocation,
    ResourceManagerCreateVaultInvocation, ResourceManagerGetMetadataInvocation,
    ResourceManagerGetNonFungibleInvocation, ResourceManagerGetResourceTypeInvocation,
    ResourceManagerGetTotalSupplyInvocation, ResourceManagerLockAuthInvocation,
    ResourceManagerMintInvocation, ResourceManagerNonFungibleExistsInvocation,
    ResourceManagerSetResourceAddressInvocation, ResourceManagerUpdateAuthInvocation,
    ResourceManagerUpdateMetadataInvocation, ResourceManagerUpdateNonFungibleDataInvocation,
    ResourceMethodAuthKey, ResourceType, SoftCount, SoftDecimal, SoftResource,
    SoftResourceOrNonFungible, SoftResourceOrNonFungibleList, VaultCreateProofByAmountInvocation,
    VaultCreateProofByIdsInvocation, VaultCreateProofInvocation, VaultGetAmountInvocation,
    VaultGetNonFungibleIdsInvocation, VaultGetResourceAddressInvocation, VaultLockFeeInvocation,
    VaultPutInvocation, VaultTakeInvocation, VaultTakeNonFungiblesInvocation, LOCKED, MUTABLE,
};

// methods and macros
use crate::engine::Invocation;
pub use sbor::decode_any;
pub use scrypto::buffer::{scrypto_decode, scrypto_encode};
pub use scrypto::crypto::hash;
pub use scrypto::resource::{
    require, require_all_of, require_amount, require_any_of, require_n_of,
};
pub use scrypto::scrypto;
pub use scrypto::{access_and_or, access_rule_node, args, dec, pdec, rule};

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
