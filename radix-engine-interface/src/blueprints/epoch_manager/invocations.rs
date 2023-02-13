use crate::api::component::ComponentAddress;
use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::*;
use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use radix_engine_interface::data::types::ManifestBucket;
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use scrypto_abi::BlueprintAbi;

pub struct EpochManagerAbi;

impl EpochManagerAbi {
    pub fn blueprint_abis() -> BTreeMap<String, BlueprintAbi> {
        BTreeMap::new()
    }
}

pub const EPOCH_MANAGER_BLUEPRINT: &str = "EpochManager";

// TODO: Remove this and replace with a macro/function making it easy
// TODO: to use manifest buckets for any input struct
#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ManifestValidatorInit {
    pub validator_account_address: ComponentAddress,
    pub initial_stake: ManifestBucket,
    pub stake_account_address: ComponentAddress,
}

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorInit {
    pub validator_account_address: ComponentAddress,
    pub initial_stake: Bucket,
    pub stake_account_address: ComponentAddress,
}

pub const EPOCH_MANAGER_CREATE_IDENT: &str = "create";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerCreateInput {
    pub olympia_validator_token_address: [u8; 26], // TODO: Clean this up
    pub component_address: [u8; 26],               // TODO: Clean this up
    pub validator_set: BTreeMap<EcdsaSecp256k1PublicKey, ValidatorInit>,
    pub initial_epoch: u64,
    pub rounds_per_epoch: u64,
    pub num_unstake_epochs: u64,
}

impl Clone for EpochManagerCreateInput {
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

pub const EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT: &str = "get_current_epoch";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerGetCurrentEpochInput;

pub const EPOCH_MANAGER_SET_EPOCH_IDENT: &str = "set_epoch";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerSetEpochInput {
    pub epoch: u64,
}

pub const EPOCH_MANAGER_NEXT_ROUND_IDENT: &str = "next_round";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerNextRoundInput {
    pub round: u64,
}

pub const EPOCH_MANAGER_CREATE_VALIDATOR_IDENT: &str = "create_validator";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerCreateValidatorInput {
    pub key: EcdsaSecp256k1PublicKey,
    pub owner_access_rule: AccessRule,
}

pub const EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT: &str = "update_validator";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum UpdateValidator {
    Register(EcdsaSecp256k1PublicKey, Decimal),
    Unregister,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct EpochManagerUpdateValidatorInput {
    pub validator_address: ComponentAddress,
    pub update: UpdateValidator,
}


#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorRegisterMethodArgs {}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ValidatorRegisterInvocation {
    pub receiver: ComponentAddress,
}

impl Invocation for ValidatorRegisterInvocation {
    type Output = ();

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Validator(ValidatorFn::Register))
    }
}

impl SerializableInvocation for ValidatorRegisterInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Validator(ValidatorFn::Register)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Validator(ValidatorFn::Unregister))
    }
}

impl SerializableInvocation for ValidatorUnregisterInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Validator(ValidatorFn::Unregister)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Validator(ValidatorFn::Stake))
    }
}

impl SerializableInvocation for ValidatorStakeInvocation {
    type ScryptoOutput = Bucket;

    fn native_fn() -> NativeFn {
        NativeFn::Validator(ValidatorFn::Stake)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Validator(ValidatorFn::Unstake))
    }
}

impl SerializableInvocation for ValidatorUnstakeInvocation {
    type ScryptoOutput = Bucket;

    fn native_fn() -> NativeFn {
        NativeFn::Validator(ValidatorFn::Unstake)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Validator(ValidatorFn::ClaimXrd))
    }
}

impl SerializableInvocation for ValidatorClaimXrdInvocation {
    type ScryptoOutput = Bucket;

    fn native_fn() -> NativeFn {
        NativeFn::Validator(ValidatorFn::ClaimXrd)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Validator(ValidatorFn::UpdateKey))
    }
}

impl SerializableInvocation for ValidatorUpdateKeyInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Validator(ValidatorFn::UpdateKey)
    }
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

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Validator(ValidatorFn::UpdateAcceptDelegatedStake))
    }
}

impl SerializableInvocation for ValidatorUpdateAcceptDelegatedStakeInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::Validator(ValidatorFn::UpdateAcceptDelegatedStake)
    }
}

impl Into<CallTableInvocation> for ValidatorUpdateAcceptDelegatedStakeInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Validator(ValidatorInvocation::UpdateAcceptDelegatedStake(self)).into()
    }
}
