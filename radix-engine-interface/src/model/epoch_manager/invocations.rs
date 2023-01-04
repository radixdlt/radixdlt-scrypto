use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use sbor::*;

use crate::api::api::*;
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;

#[derive(Debug, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerCreateInvocation {
    pub validator_set: BTreeMap<EcdsaSecp256k1PublicKey, Bucket>,
    pub initial_epoch: u64,
    pub rounds_per_epoch: u64,
}

impl Clone for EpochManagerCreateInvocation {
    fn clone(&self) -> Self {
        let mut validator_set = BTreeMap::new();
        for (key, bucket) in &self.validator_set {
            validator_set.insert(key.clone(), Bucket(bucket.0));
        }

        Self {
            validator_set,
            initial_epoch: self.initial_epoch,
            rounds_per_epoch: self.rounds_per_epoch,
        }
    }
}

impl Invocation for EpochManagerCreateInvocation {
    type Output = SystemAddress;
}

impl SerializableInvocation for EpochManagerCreateInvocation {
    type ScryptoOutput = SystemAddress;
}

impl Into<SerializedInvocation> for EpochManagerCreateInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::Create(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerGetCurrentEpochInvocation {
    pub receiver: SystemAddress,
}

impl Invocation for EpochManagerGetCurrentEpochInvocation {
    type Output = u64;
}

impl SerializableInvocation for EpochManagerGetCurrentEpochInvocation {
    type ScryptoOutput = u64;
}

impl Into<SerializedInvocation> for EpochManagerGetCurrentEpochInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::GetCurrentEpoch(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerSetEpochInvocation {
    pub receiver: SystemAddress,
    pub epoch: u64,
}

impl Invocation for EpochManagerSetEpochInvocation {
    type Output = ();
}

impl SerializableInvocation for EpochManagerSetEpochInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for EpochManagerSetEpochInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::SetEpoch(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerNextRoundInvocation {
    pub receiver: SystemAddress,
    pub round: u64,
}

impl Invocation for EpochManagerNextRoundInvocation {
    type Output = ();
}

impl SerializableInvocation for EpochManagerNextRoundInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for EpochManagerNextRoundInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::NextRound(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerCreateValidatorInvocation {
    pub receiver: SystemAddress,
    pub key: EcdsaSecp256k1PublicKey,
}

impl Invocation for EpochManagerCreateValidatorInvocation {
    type Output = SystemAddress;
}

impl SerializableInvocation for EpochManagerCreateValidatorInvocation {
    type ScryptoOutput = SystemAddress;
}

impl Into<SerializedInvocation> for EpochManagerCreateValidatorInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::CreateValidator(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum UpdateValidator {
    Register(EcdsaSecp256k1PublicKey, Decimal),
    Unregister,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerUpdateValidatorInvocation {
    pub receiver: SystemAddress,
    pub validator_address: SystemAddress,
    pub update: UpdateValidator,
}

impl Invocation for EpochManagerUpdateValidatorInvocation {
    type Output = ();
}

// TODO: Should we have this or not?
impl SerializableInvocation for EpochManagerUpdateValidatorInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for EpochManagerUpdateValidatorInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::UpdateValidator(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ValidatorRegisterInvocation {
    pub receiver: SystemAddress,
}

impl Invocation for ValidatorRegisterInvocation {
    type Output = ();
}

impl SerializableInvocation for ValidatorRegisterInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for ValidatorRegisterInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Validator(ValidatorInvocation::Register(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ValidatorUnregisterInvocation {
    pub receiver: SystemAddress,
}

impl Invocation for ValidatorUnregisterInvocation {
    type Output = ();
}

impl SerializableInvocation for ValidatorUnregisterInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for ValidatorUnregisterInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Validator(ValidatorInvocation::Unregister(self)).into()
    }
}

#[derive(Debug, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ValidatorStakeInvocation {
    pub receiver: SystemAddress,
    pub stake: Bucket,
}

impl Clone for ValidatorStakeInvocation {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver,
            stake: Bucket(self.stake.0),
        }
    }
}

impl Invocation for ValidatorStakeInvocation {
    type Output = ();
}

impl SerializableInvocation for ValidatorStakeInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for ValidatorStakeInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Validator(ValidatorInvocation::Stake(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ValidatorUnstakeInvocation {
    pub receiver: SystemAddress,
    pub amount: Decimal,
}

impl Invocation for ValidatorUnstakeInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for ValidatorUnstakeInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<SerializedInvocation> for ValidatorUnstakeInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Validator(ValidatorInvocation::Unstake(self)).into()
    }
}

#[derive(Debug, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ValidatorClaimXrdInvocation {
    pub receiver: SystemAddress,
    pub bucket: Bucket,
}

impl Clone for ValidatorClaimXrdInvocation {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver,
            bucket: Bucket(self.bucket.0),
        }
    }
}

impl Invocation for ValidatorClaimXrdInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for ValidatorClaimXrdInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<SerializedInvocation> for ValidatorClaimXrdInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Validator(ValidatorInvocation::ClaimXrd(self)).into()
    }
}
