use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::api::api::*;
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct EpochManagerCreateInvocation {
    pub validator_set: Vec<EcdsaSecp256k1PublicKey>,
    pub initial_epoch: u64,
    pub rounds_per_epoch: u64,
}

impl Invocation for EpochManagerCreateInvocation {
    type Output = SystemAddress;
}

impl SerializableInvocation for EpochManagerCreateInvocation {
    type ScryptoOutput = SystemAddress;
}

impl Into<SerializedInvocation> for EpochManagerCreateInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::EpochManager(
            EpochManagerFunctionInvocation::Create(self),
        ))
        .into()
    }
}

#[derive(Debug)]
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
        NativeFnInvocation::Method(NativeMethodInvocation::EpochManager(
            EpochManagerMethodInvocation::GetCurrentEpoch(self),
        ))
        .into()
    }
}

#[derive(Debug)]
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
        NativeFnInvocation::Method(NativeMethodInvocation::EpochManager(
            EpochManagerMethodInvocation::SetEpoch(self),
        ))
        .into()
    }
}

#[derive(Debug)]
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
        NativeFnInvocation::Method(NativeMethodInvocation::EpochManager(
            EpochManagerMethodInvocation::NextRound(self),
        ))
        .into()
    }
}
