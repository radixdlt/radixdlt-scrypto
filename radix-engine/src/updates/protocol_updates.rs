use super::state_updates::*;
use crate::{internal_prelude::*, track::StateUpdates};
use radix_substate_store_interface::interface::SubstateDatabase;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Introduces the account locker blueprint.
    LockerPackage,

    /// Moves various protocol parameters to state.
    ProtocolParamsToState,

    /// Makes some behavioral changes to the try_deposit_or_refund (and batch variants too) method
    /// on the account blueprint.
    AccountTryDepositOrRefundBehaviorChanges,

    /// Add blob limits to transaction processor.
    TransactionProcessorBlobLimits,
}

impl ProtocolUpdateEntry {
    pub fn generate_state_updates<S: SubstateDatabase>(
        &self,
        db: &S,
        network: &NetworkDefinition,
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
            ProtocolUpdateEntry::ProtocolParamsToState => {
                generate_protocol_params_to_state_state_updates(network.clone())
            }
            ProtocolUpdateEntry::TransactionProcessorBlobLimits => {
                generate_transaction_processor_blob_limits_state_updates(db)
            }
        }
    }
}

macro_rules! count {
    (
        $ident: ident, $($other_idents: ident),* $(,)?
    ) => {
        1 + count!( $($other_idents),* )
    };
    (
        $ident: ident $(,)?
    ) => {
        1
    }
}

macro_rules! enum_const_array {
    (
        $(#[$meta:meta])*
        $vis: vis enum $ident: ident {
            $(
                $variant_ident: ident
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $ident {
            $(
                $variant_ident
            ),*
        }

        impl $ident {
            pub const VARIANTS: [Self; count!( $($variant_ident),* )] = [
                $(
                    Self::$variant_ident
                ),*
            ];
        }
    };
}

enum_const_array! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum ProtocolUpdate {
        Anemone,
        Bottlenose,
    }
}

impl ProtocolUpdate {
    pub fn generate_state_updates<S: SubstateDatabase>(
        &self,
        db: &S,
        network: &NetworkDefinition,
    ) -> Vec<StateUpdates> {
        match self {
            ProtocolUpdate::Anemone => vec![
                ProtocolUpdateEntry::Bls12381AndKeccak256,
                ProtocolUpdateEntry::SecondPrecisionTimestamp,
                ProtocolUpdateEntry::PoolMathPrecisionFix,
                ProtocolUpdateEntry::ValidatorCreationFeeFix,
            ],
            ProtocolUpdate::Bottlenose => vec![
                ProtocolUpdateEntry::OwnerRoleGetter,
                ProtocolUpdateEntry::LockerPackage,
                ProtocolUpdateEntry::AccountTryDepositOrRefundBehaviorChanges,
                ProtocolUpdateEntry::ProtocolParamsToState,
                ProtocolUpdateEntry::TransactionProcessorBlobLimits,
            ],
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ProtocolVersion {
    Genesis,
    ProtocolUpdate(ProtocolUpdate),
}

impl ProtocolVersion {
    pub fn all_iterator() -> impl Iterator<Item = Self> {
        core::iter::once(Self::Genesis).chain(ProtocolUpdate::VARIANTS.map(Self::ProtocolUpdate))
    }
}
