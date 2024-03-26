use super::state_updates::*;
use crate::{internal_prelude::*, track::StateUpdates};
use radix_substate_store_interface::interface::SubstateDatabase;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProtocolUpdate {
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
    AccountLocker,

    /// Moves various protocol parameters to state.
    ProtocolParamsToState,
}

pub struct ProtocolUpdates {
    updates: BTreeSet<ProtocolUpdate>,
}

impl ProtocolUpdates {
    pub fn none() -> Self {
        Self {
            updates: BTreeSet::new(),
        }
    }

    pub fn all() -> Self {
        Self::none().with_anemone().with_bottlenose()
    }

    /// Enables all the protocol updates included in the `anemone` release.
    pub fn with_anemone(mut self) -> Self {
        self.updates.extend(btreeset!(
            ProtocolUpdate::Bls12381AndKeccak256,
            ProtocolUpdate::SecondPrecisionTimestamp,
            ProtocolUpdate::PoolMathPrecisionFix,
            ProtocolUpdate::ValidatorCreationFeeFix,
        ));
        self
    }

    /// Enables all the protocol updates included in the `bottlenose` release.
    /// 
    /// Note that this does not include `anemone` protocol updates.
    pub fn with_bottlenose(mut self) -> Self {
        self.updates.extend(btreeset!(
            ProtocolUpdate::OwnerRoleGetter,
            ProtocolUpdate::SystemPatches,
            ProtocolUpdate::AccountLocker,
            ProtocolUpdate::ProtocolParamsToState,
        ));
        self
    }

    pub fn with_update(mut self, update: ProtocolUpdate) -> Self {
        self.updates.insert(update);
        self
    }

    pub fn without_update(mut self, update: ProtocolUpdate) -> Self {
        self.updates.remove(&update);
        self
    }

    pub fn generate_state_updates<S: SubstateDatabase>(&self, db: &S) -> Vec<StateUpdates> {
        let mut results = Vec::new();
        for update in &self.updates {
            results.push(match update {
                ProtocolUpdate::Bls12381AndKeccak256 => {
                    generate_bls128_and_keccak256_state_updates()
                }
                ProtocolUpdate::SecondPrecisionTimestamp => {
                    generate_seconds_precision_timestamp_state_updates(db)
                }
                ProtocolUpdate::PoolMathPrecisionFix => {
                    generate_pool_math_precision_fix_state_updates(db)
                }
                ProtocolUpdate::ValidatorCreationFeeFix => {
                    generate_validator_creation_fee_fix_state_updates(db)
                }
                // TODO implement the following
                ProtocolUpdate::OwnerRoleGetter => StateUpdates::default(),
                ProtocolUpdate::SystemPatches => StateUpdates::default(),
                ProtocolUpdate::AccountLocker => StateUpdates::default(),
                ProtocolUpdate::ProtocolParamsToState => StateUpdates::default(),
            });
        }
        results
    }
}
