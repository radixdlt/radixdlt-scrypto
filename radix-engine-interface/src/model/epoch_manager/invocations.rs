use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

use crate::api::wasm::*;
use crate::api::*;
use crate::model::*;
use crate::*;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerCreateInvocation {
    pub validator_set: BTreeSet<EcdsaSecp256k1PublicKey>,
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
pub struct EpochManagerCreateValidatorMethodArgs {
    pub validator: EcdsaSecp256k1PublicKey,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerCreateValidatorInvocation {
    pub receiver: ComponentAddress,
    pub key: EcdsaSecp256k1PublicKey,
}

impl Invocation for EpochManagerCreateValidatorInvocation {
    type Output = ComponentAddress;
}

impl SerializableInvocation for EpochManagerCreateValidatorInvocation {
    type ScryptoOutput = ComponentAddress;
}

impl Into<CallTableInvocation> for EpochManagerCreateValidatorInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::CreateValidator(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerUpdateValidatorMethodArgs {
    pub validator_address: ComponentAddress,
    pub key: EcdsaSecp256k1PublicKey,
    pub register: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerUpdateValidatorInvocation {
    pub receiver: ComponentAddress,
    pub validator_address: ComponentAddress,
    pub key: EcdsaSecp256k1PublicKey,
    pub register: bool,
}

impl Invocation for EpochManagerUpdateValidatorInvocation {
    type Output = ();
}

// TODO: Should we have this or not?
impl SerializableInvocation for EpochManagerUpdateValidatorInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for EpochManagerUpdateValidatorInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::EpochManager(EpochManagerInvocation::UpdateValidator(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorRegisterMethodArgs {}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorRegisterInvocation {
    pub receiver: ComponentAddress,
}

impl Invocation for ValidatorRegisterInvocation {
    type Output = ();
}

impl SerializableInvocation for ValidatorRegisterInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for ValidatorRegisterInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Validator(ValidatorInvocation::Register(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorUnregisterValidatorMethodArgs {}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorUnregisterInvocation {
    pub receiver: ComponentAddress,
}

impl Invocation for ValidatorUnregisterInvocation {
    type Output = ();
}

impl SerializableInvocation for ValidatorUnregisterInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for ValidatorUnregisterInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Validator(ValidatorInvocation::Unregister(self)).into()
    }
}
