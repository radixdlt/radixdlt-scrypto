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
pub use sbor::{Decode, DecodeError, Encode, Type, TypeId, Value};
pub use scrypto::abi::{BlueprintAbi, Fn, ScryptoType};
pub use radix_engine_lib::address::{AddressError, Bech32Decoder, Bech32Encoder};
pub use scrypto::component::{
    ComponentAddAccessCheckInvocation, PackagePublishInvocation,
};
pub use radix_engine_lib::component::ComponentAddress;
pub use radix_engine_lib::component::PackageAddress;
pub use radix_engine_lib::component::SystemAddress;
pub use radix_engine_lib::resource::ResourceAddress;
pub use radix_engine_lib::resource::NonFungibleId;
pub use radix_engine_lib::resource::NonFungibleAddress;
pub use scrypto::constants::*;
pub use scrypto::core::{
    EpochManagerCreateInvocation, EpochManagerGetCurrentEpochInvocation,
    EpochManagerSetEpochInvocation, Expression, ScryptoActor,
};
pub use scrypto::crypto::{
    EcdsaSecp256k1PublicKey, EcdsaSecp256k1Signature, EddsaEd25519PublicKey, EddsaEd25519Signature,
    Hash, PublicKey, Signature,
};
pub use scrypto::engine::{scrypto_env::RadixEngineInput, types::*};
pub use scrypto::math::{Decimal, RoundingMode, I256};
pub use scrypto::resource::{
    AccessRule, AccessRuleNode, AccessRules, AuthZoneClearInvocation,
    AuthZoneCreateProofByAmountInvocation, AuthZoneCreateProofByIdsInvocation,
    AuthZoneCreateProofInvocation, AuthZonePopInvocation, AuthZonePushInvocation,
    BucketCreateProofInvocation, BucketGetAmountInvocation, BucketGetNonFungibleIdsInvocation,
    BucketGetResourceAddressInvocation, BucketPutInvocation, BucketTakeInvocation,
    BucketTakeNonFungiblesInvocation, MintParams, Mutability,
    ProofCloneInvocation, ProofGetAmountInvocation, ProofGetNonFungibleIdsInvocation,
    ProofGetResourceAddressInvocation, ProofRule, ResourceManagerBurnInvocation,
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
    VaultPutInvocation, VaultTakeInvocation, VaultTakeNonFungiblesInvocation,
    WorktopAssertContainsAmountInvocation, WorktopAssertContainsInvocation,
    WorktopAssertContainsNonFungiblesInvocation, WorktopDrainInvocation, WorktopPutInvocation,
    WorktopTakeAllInvocation, WorktopTakeAmountInvocation, WorktopTakeNonFungiblesInvocation,
    LOCKED, MUTABLE,
};
pub use scrypto::values::{ScryptoValue, ScryptoValueReplaceError};
use std::fmt::Debug;
pub use utils::crypto::Blob;

// methods and macros
use crate::engine::Invocation;
pub use sbor::decode_any;
pub use scrypto::buffer::{scrypto_decode, scrypto_encode};
pub use scrypto::crypto::hash;
pub use scrypto::resource::{
    require, require_all_of, require_amount, require_any_of, require_n_of,
};
pub use scrypto::{access_and_or, access_rule_node, args, dec, pdec, rule};

/// Scrypto function/method invocation.
#[derive(Debug)]
pub enum ScryptoInvocation {
    Function(ScryptoFunctionIdent, ScryptoValue),
    Method(ScryptoMethodIdent, ScryptoValue),
}

impl Invocation for ScryptoInvocation {
    type Output = ScryptoValue;
}

impl ScryptoInvocation {
    pub fn args(&self) -> &ScryptoValue {
        match self {
            ScryptoInvocation::Function(_, args) => &args,
            ScryptoInvocation::Method(_, args) => &args,
        }
    }
}

#[derive(Debug)]
pub struct NativeMethodInvocation(pub NativeMethod, pub RENodeId, pub ScryptoValue);

impl Invocation for NativeMethodInvocation {
    type Output = ScryptoValue;
}

impl NativeMethodInvocation {
    pub fn args(&self) -> &ScryptoValue {
        &self.2
    }
}
