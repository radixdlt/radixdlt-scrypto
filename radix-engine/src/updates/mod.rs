pub mod state_updates;

use crate::{internal_prelude::*, track::StateUpdates};
use radix_substate_store_interface::interface::SubstateDatabase;
use state_updates::*;

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
}

pub struct ProtocolUpdates {
    updates: BTreeSet<ProtocolUpdate>,
}

impl ProtocolUpdates {
    pub fn new() -> Self {
        Self {
            updates: BTreeSet::new(),
        }
    }

    pub fn anemone(mut self) -> Self {
        self.updates.extend(btreeset!(
            ProtocolUpdate::Bls12381AndKeccak256,
            ProtocolUpdate::SecondPrecisionTimestamp,
            ProtocolUpdate::PoolMathPrecisionFix,
            ProtocolUpdate::ValidatorCreationFeeFix,
        ));
        self
    }

    pub fn bottlenose(mut self) -> Self {
        self.updates.extend(btreeset!(
            ProtocolUpdate::OwnerRoleGetter,
            ProtocolUpdate::SystemPatches,
            ProtocolUpdate::AccountLocker
        ));
        self
    }

    pub fn to_state_updates<S: SubstateDatabase>(&self, db: &S) -> Vec<StateUpdates> {
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
                ProtocolUpdate::OwnerRoleGetter => {
                    unimplemented!()
                }
                ProtocolUpdate::SystemPatches => {
                    unimplemented!()
                }
                ProtocolUpdate::AccountLocker => {
                    unimplemented!()
                }
            });
        }
        results
    }
}
