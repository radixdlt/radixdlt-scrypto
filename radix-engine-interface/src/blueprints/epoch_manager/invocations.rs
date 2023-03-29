use crate::blueprints::resource::*;
use crate::data::manifest::model::*;
use crate::*;
use radix_engine_common::types::*;
use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;

pub const EPOCH_MANAGER_BLUEPRINT: &str = "EpochManager";
pub const VALIDATOR_BLUEPRINT: &str = "Validator";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorInit {
    pub validator_account_address: ComponentAddress,
    pub initial_stake: Bucket,
    pub stake_account_address: ComponentAddress,
}

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct ManifestValidatorInit {
    pub validator_account_address: ComponentAddress,
    pub initial_stake: ManifestBucket,
    pub stake_account_address: ComponentAddress,
}

pub const EPOCH_MANAGER_CREATE_IDENT: &str = "create";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct EpochManagerCreateInput {
    pub validator_owner_token: [u8; 27], // TODO: Clean this up
    pub component_address: [u8; 27],     // TODO: Clean this up
    pub validator_set: BTreeMap<EcdsaSecp256k1PublicKey, ValidatorInit>,
    pub initial_epoch: u64,
    pub rounds_per_epoch: u64,
    pub num_unstake_epochs: u64,
}

pub type EpochManagerCreateOutput = ComponentAddress;

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
            validator_owner_token: self.validator_owner_token,
            component_address: self.component_address,
            validator_set,
            initial_epoch: self.initial_epoch,
            rounds_per_epoch: self.rounds_per_epoch,
            num_unstake_epochs: self.num_unstake_epochs,
        }
    }
}

pub const EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT: &str = "get_current_epoch";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct EpochManagerGetCurrentEpochInput;

pub type EpochManagerGetCurrentEpochOutput = u64;

pub const EPOCH_MANAGER_SET_EPOCH_IDENT: &str = "set_epoch";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct EpochManagerSetEpochInput {
    pub epoch: u64,
}

pub type EpochManagerSetEpochOutput = ();

pub const EPOCH_MANAGER_NEXT_ROUND_IDENT: &str = "next_round";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct EpochManagerNextRoundInput {
    pub round: u64,
}

pub type EpochManagerNextRoundOutput = ();

pub const EPOCH_MANAGER_CREATE_VALIDATOR_IDENT: &str = "create_validator";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct EpochManagerCreateValidatorInput {
    pub key: EcdsaSecp256k1PublicKey,
}

pub type EpochManagerCreateValidatorOutput = (ComponentAddress, Bucket);

pub const EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT: &str = "update_validator";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub enum UpdateValidator {
    Register(EcdsaSecp256k1PublicKey, Decimal),
    Unregister,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct EpochManagerUpdateValidatorInput {
    pub validator_address: ComponentAddress,
    pub update: UpdateValidator,
}

pub type EpochManagerUpdateValidatorOutput = ();

pub const VALIDATOR_REGISTER_IDENT: &str = "register";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorRegisterInput {}

pub type ValidatorRegisterOutput = ();

pub const VALIDATOR_UNREGISTER_IDENT: &str = "unregister";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorUnregisterInput {}

pub type ValidatorUnregisterOutput = ();

pub const VALIDATOR_STAKE_IDENT: &str = "stake";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorStakeInput {
    pub stake: Bucket,
}

pub type ValidatorStakeOutput = Bucket;

pub const VALIDATOR_UNSTAKE_IDENT: &str = "unstake";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorUnstakeInput {
    pub lp_tokens: Bucket,
}

pub type ValidatorUnstakeOutput = Bucket;

pub const VALIDATOR_CLAIM_XRD_IDENT: &str = "claim_xrd";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ValidatorClaimXrdInput {
    pub bucket: Bucket,
}

pub type ValidatorClaimXrdOutput = Bucket;

pub const VALIDATOR_UPDATE_KEY_IDENT: &str = "update_key";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorUpdateKeyInput {
    pub key: EcdsaSecp256k1PublicKey,
}

pub type ValidatorUpdateKeyOutput = ();

pub const VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT: &str = "update_accept_delegated_stake";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct ValidatorUpdateAcceptDelegatedStakeInput {
    pub accept_delegated_stake: bool,
}

pub type ValidatorUpdateAcceptDelegatedStakeOutput = ();
