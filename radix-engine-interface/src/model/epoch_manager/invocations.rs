use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;

use crate::api::wasm::*;
use crate::api::*;
use crate::model::*;
use crate::*;

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorInit {
    pub validator_account_address: ComponentAddress,
    pub initial_stake: Bucket,
    pub stake_account_address: ComponentAddress,
}

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerCreateInvocation {
    pub olympia_validator_token_address: [u8; 26], // TODO: Clean this up
    pub component_address: [u8; 26],               // TODO: Clean this up
    pub validator_set: BTreeMap<EcdsaSecp256k1PublicKey, ValidatorInit>,
    pub initial_epoch: u64,
    pub rounds_per_epoch: u64,
    pub num_unstake_epochs: u64,
}

impl Clone for EpochManagerCreateInvocation {
    fn clone(&self) -> Self {
        let mut validator_set = BTreeMap::new();
        for (key, validator_init) in &self.validator_set {
            validator_set.insert(
                key.clone(),
                ValidatorInit {
                    validator_account_address: validator_init.validator_account_address,
                    stake_account_address: validator_init.stake_account_address,
                    initial_stake: Bucket(validator_init.initial_stake.0),
                },
            );
        }

        Self {
            olympia_validator_token_address: self.olympia_validator_token_address,
            component_address: self.component_address,
            validator_set,
            initial_epoch: self.initial_epoch,
            rounds_per_epoch: self.rounds_per_epoch,
            num_unstake_epochs: self.num_unstake_epochs,
        }
    }
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
    pub key: EcdsaSecp256k1PublicKey,
    pub owner_access_rule: AccessRule,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerCreateValidatorInvocation {
    pub receiver: ComponentAddress,
    pub key: EcdsaSecp256k1PublicKey,
    pub owner_access_rule: AccessRule,
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
pub enum UpdateValidator {
    Register(EcdsaSecp256k1PublicKey, Decimal),
    Unregister,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerUpdateValidatorMethodArgs {
    pub validator_address: ComponentAddress,
    pub update: UpdateValidator,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerUpdateValidatorInvocation {
    pub receiver: ComponentAddress,
    pub validator_address: ComponentAddress,
    pub update: UpdateValidator,
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

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorStakeMethodArgs {
    pub stake: Bucket,
}

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorStakeInvocation {
    pub receiver: ComponentAddress,
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
    type Output = Bucket;
}

impl SerializableInvocation for ValidatorStakeInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<CallTableInvocation> for ValidatorStakeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Validator(ValidatorInvocation::Stake(self)).into()
    }
}

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorUnstakeMethodArgs {
    pub lp_tokens: Bucket,
}

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorUnstakeInvocation {
    pub receiver: ComponentAddress,
    pub lp_tokens: Bucket,
}

impl Clone for ValidatorUnstakeInvocation {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver,
            lp_tokens: Bucket(self.lp_tokens.0),
        }
    }
}

impl Invocation for ValidatorUnstakeInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for ValidatorUnstakeInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<CallTableInvocation> for ValidatorUnstakeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Validator(ValidatorInvocation::Unstake(self)).into()
    }
}

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorClaimXrdMethodArgs {
    pub bucket: Bucket,
}

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorClaimXrdInvocation {
    pub receiver: ComponentAddress,
    pub unstake_nft: Bucket,
}

impl Clone for ValidatorClaimXrdInvocation {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver,
            unstake_nft: Bucket(self.unstake_nft.0),
        }
    }
}

impl Invocation for ValidatorClaimXrdInvocation {
    type Output = Bucket;
}

impl SerializableInvocation for ValidatorClaimXrdInvocation {
    type ScryptoOutput = Bucket;
}

impl Into<CallTableInvocation> for ValidatorClaimXrdInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Validator(ValidatorInvocation::ClaimXrd(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorUpdateKeyMethodArgs {
    pub key: EcdsaSecp256k1PublicKey,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorUpdateKeyInvocation {
    pub receiver: ComponentAddress,
    pub key: EcdsaSecp256k1PublicKey,
}

impl Invocation for ValidatorUpdateKeyInvocation {
    type Output = ();
}

impl SerializableInvocation for ValidatorUpdateKeyInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for ValidatorUpdateKeyInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Validator(ValidatorInvocation::UpdateKey(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorUpdateAcceptDelegatedStakeMethodArgs {
    pub accept_delegated_stake: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorUpdateAcceptDelegatedStakeInvocation {
    pub receiver: ComponentAddress,
    pub accept_delegated_stake: bool,
}

impl Invocation for ValidatorUpdateAcceptDelegatedStakeInvocation {
    type Output = ();
}

impl SerializableInvocation for ValidatorUpdateAcceptDelegatedStakeInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for ValidatorUpdateAcceptDelegatedStakeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Validator(ValidatorInvocation::UpdateAcceptDelegatedStake(self)).into()
    }
}
