use super::state_updates::*;
use crate::{internal_prelude::*, track::StateUpdates};
use radix_substate_store_interface::interface::SubstateDatabase;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProtocolUpdateEntry {
    /// Introduces BLS12-381 and Keccak-256 features.
    Bls12381AndKeccak256,

    /// Exposes second-precision timestamp.
    SecondPrecisionTimestamp,

    /// Increases the math precision with native pool implementations.
    PoolMathPrecisionFix,

    /// Changes the cost associated with validator creation.
    ValidatorCreationFeeFix,

    /// Exposes a getter method for reading owner role rule.
    OwnerRoleGetter,

    /// Various system patches.
    SystemPatches,

    /// Introduces the account locker blueprint.
    LockerPackage,

    /// Moves various protocol parameters to state.
    ProtocolParamsToState,

    /// Makes some behavioral changes to the try_deposit_or_refund (and batch variants too) method
    /// on the account blueprint.
    AccountTryDepositOrRefundBehaviorChanges,
}

impl ProtocolUpdateEntry {
    pub fn generate_state_updates<S: SubstateDatabase>(
        &self,
        db: &S,
        _network: &NetworkDefinition,
    ) -> StateUpdates {
        match self {
            ProtocolUpdateEntry::Bls12381AndKeccak256 => {
                generate_bls128_and_keccak256_state_updates()
            }
            ProtocolUpdateEntry::SecondPrecisionTimestamp => {
                generate_seconds_precision_timestamp_state_updates(db)
            }
            ProtocolUpdateEntry::PoolMathPrecisionFix => {
                generate_pool_math_precision_fix_state_updates(db)
            }
            ProtocolUpdateEntry::ValidatorCreationFeeFix => {
                generate_validator_creation_fee_fix_state_updates(db)
            }
            ProtocolUpdateEntry::OwnerRoleGetter => generate_owner_role_getter_state_updates(db),
            ProtocolUpdateEntry::LockerPackage => generate_locker_package_state_updates(),
            ProtocolUpdateEntry::AccountTryDepositOrRefundBehaviorChanges => {
                generate_account_bottlenose_extension_state_updates(db)
            }
            // TODO implement the following
            ProtocolUpdateEntry::SystemPatches => StateUpdates::default(),
            ProtocolUpdateEntry::ProtocolParamsToState => StateUpdates::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolUpdate {
    Anemone,

    Bottlenose,
}

impl ProtocolUpdate {
    pub fn generate_state_updates<S: SubstateDatabase>(
        &self,
        db: &S,
        network: &NetworkDefinition,
    ) -> Vec<StateUpdates> {
        match self {
            ProtocolUpdate::Anemone => btreeset!(
                ProtocolUpdateEntry::Bls12381AndKeccak256,
                ProtocolUpdateEntry::SecondPrecisionTimestamp,
                ProtocolUpdateEntry::PoolMathPrecisionFix,
                ProtocolUpdateEntry::ValidatorCreationFeeFix,
            ),

            ProtocolUpdate::Bottlenose => btreeset!(
                ProtocolUpdateEntry::OwnerRoleGetter,
                ProtocolUpdateEntry::SystemPatches,
                ProtocolUpdateEntry::LockerPackage,
                ProtocolUpdateEntry::ProtocolParamsToState,
            ),
        }
        .iter()
        .map(|update| update.generate_state_updates(db, network))
        .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ProtocolUpdates {
    protocol_updates: Vec<ProtocolUpdate>,
    additional_updates: Vec<ProtocolUpdateEntry>,
}

impl ProtocolUpdates {
    pub fn none() -> Self {
        Self {
            protocol_updates: vec![],
            additional_updates: vec![],
        }
    }

    pub fn up_to_anemone() -> Self {
        Self {
            protocol_updates: vec![ProtocolUpdate::Anemone],
            additional_updates: vec![],
        }
    }

    pub fn up_to_bottlenose() -> Self {
        Self {
            protocol_updates: vec![ProtocolUpdate::Anemone, ProtocolUpdate::Bottlenose],
            additional_updates: vec![],
        }
    }

    pub fn all() -> Self {
        Self::up_to_bottlenose()
    }

    pub fn and(mut self, protocol_update: ProtocolUpdateEntry) -> Self {
        self.additional_updates.push(protocol_update);
        self
    }

    pub fn generate_state_updates<S: SubstateDatabase>(
        &self,
        db: &S,
        network: &NetworkDefinition,
    ) -> Vec<StateUpdates> {
        let mut results = Vec::new();
        for protocol_update in &self.protocol_updates {
            results.extend(protocol_update.generate_state_updates(db, network));
        }
        for protocol_update in &self.additional_updates {
            results.push(protocol_update.generate_state_updates(db, network));
        }
        results
    }
}
