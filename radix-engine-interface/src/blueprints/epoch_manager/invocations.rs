use crate::api::sorted_store_api::SortedKey;
use crate::blueprints::resource::*;
use crate::*;
use radix_engine_common::types::*;
use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use radix_engine_interface::math::Decimal;
use sbor::rust::fmt::Debug;

pub const EPOCH_MANAGER_BLUEPRINT: &str = "EpochManager";
pub const VALIDATOR_BLUEPRINT: &str = "Validator";

pub const EPOCH_MANAGER_CREATE_IDENT: &str = "create";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct EpochManagerCreateInput {
    pub validator_owner_token: [u8; NodeId::LENGTH], // TODO: Clean this up
    pub component_address: [u8; NodeId::LENGTH],     // TODO: Clean this up
    pub initial_epoch: u64,
    pub max_validators: u32,
    pub rounds_per_epoch: u64,
    pub num_unstake_epochs: u64,
}

pub type EpochManagerCreateOutput = ();

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

pub const EPOCH_MANAGER_START_IDENT: &str = "start";

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub struct EpochManagerStartInput {}

pub type EpochManagerStartOutput = ();

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
pub enum UpdateSecondaryIndex {
    Create {
        index_key: SortedKey,
        primary: ComponentAddress,
        key: EcdsaSecp256k1PublicKey,
        stake: Decimal,
    },
    UpdateStake {
        index_key: SortedKey,
        new_index_key: SortedKey,
        new_stake_amount: Decimal,
    },
    UpdatePublicKey {
        index_key: SortedKey,
        key: EcdsaSecp256k1PublicKey,
    },
    Remove {
        index_key: SortedKey,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct EpochManagerUpdateValidatorInput {
    pub update: UpdateSecondaryIndex,
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
