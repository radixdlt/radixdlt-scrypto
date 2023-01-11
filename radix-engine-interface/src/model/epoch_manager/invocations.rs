use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use sbor::rust::collections::HashSet;
use sbor::rust::fmt::Debug;
use sbor::*;

use crate::api::api::*;
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct EpochManagerCreateInvocation {
    pub validator_set: HashSet<EcdsaSecp256k1PublicKey>,
    pub initial_epoch: u64,
    pub rounds_per_epoch: u64,
}

impl Invocation for EpochManagerCreateInvocation {
    type Output = ComponentAddress;
}

impl SerializableInvocation for EpochManagerCreateInvocation {
    type ScryptoOutput = ComponentAddress;
}

impl Into<SerializedInvocation> for EpochManagerCreateInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::Create(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct EpochManagerGetCurrentEpochMethodArgs {
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct EpochManagerGetCurrentEpochInvocation {
    pub receiver: ComponentAddress,
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
#[scrypto(Categorize, Encode, Decode)]
pub struct EpochManagerSetEpochMethodArgs {
    pub epoch: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct EpochManagerSetEpochInvocation {
    pub receiver: ComponentAddress,
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
#[scrypto(Categorize, Encode, Decode)]
pub struct EpochManagerNextRoundMethodArgs {
    pub round: u64,
}


#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct EpochManagerNextRoundInvocation {
    pub receiver: ComponentAddress,
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
#[scrypto(Categorize, Encode, Decode)]
pub struct EpochManagerRegisterValidatorMethodArgs {
    pub validator: EcdsaSecp256k1PublicKey,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct EpochManagerRegisterValidatorInvocation {
    pub receiver: ComponentAddress,
    pub validator: EcdsaSecp256k1PublicKey,
}

impl Invocation for EpochManagerRegisterValidatorInvocation {
    type Output = ();
}

impl SerializableInvocation for EpochManagerRegisterValidatorInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for EpochManagerRegisterValidatorInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::RegisterValidator(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct EpochManagerUnregisterValidatorMethodArgs {
    pub validator: EcdsaSecp256k1PublicKey,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct EpochManagerUnregisterValidatorInvocation {
    pub receiver: ComponentAddress,
    pub validator: EcdsaSecp256k1PublicKey,
}

impl Invocation for EpochManagerUnregisterValidatorInvocation {
    type Output = ();
}

impl SerializableInvocation for EpochManagerUnregisterValidatorInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for EpochManagerUnregisterValidatorInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::UnregisterValidator(self)).into()
    }
}
