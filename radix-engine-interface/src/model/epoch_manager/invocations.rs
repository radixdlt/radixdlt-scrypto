use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use sbor::rust::collections::HashSet;
use sbor::rust::fmt::Debug;

use crate::api::api::*;
use crate::model::*;
use crate::wasm::*;
use crate::*;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for EpochManagerCreateInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::Create(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerGetCurrentEpochMethodArgs {}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerGetCurrentEpochInvocation {
    pub receiver: ComponentAddress,
}

impl Invocation for EpochManagerGetCurrentEpochInvocation {
    type Output = u64;
}

impl SerializableInvocation for EpochManagerGetCurrentEpochInvocation {
    type ScryptoOutput = u64;
}

impl Into<CallTableInvocation> for EpochManagerGetCurrentEpochInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::GetCurrentEpoch(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerSetEpochMethodArgs {
    pub epoch: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for EpochManagerSetEpochInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::SetEpoch(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerNextRoundMethodArgs {
    pub round: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for EpochManagerNextRoundInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::NextRound(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerRegisterValidatorMethodArgs {
    pub validator: EcdsaSecp256k1PublicKey,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for EpochManagerRegisterValidatorInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::RegisterValidator(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerUnregisterValidatorMethodArgs {
    pub validator: EcdsaSecp256k1PublicKey,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for EpochManagerUnregisterValidatorInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::UnregisterValidator(self)).into()
    }
}
