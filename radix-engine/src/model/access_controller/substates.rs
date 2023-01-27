use crate::types::*;
use radix_engine_interface::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerSubstate {
    /// A vault where the asset controlled by the access controller lives.
    pub controlled_asset: VaultId,

    /// The amount of time (in minutes) that it takes for timed recovery to be done. Maximum is
    /// 4,294,967,295 minutes which is 8171.5511700913 years. When this is [`None`], then timed
    /// recovery can not be performed through this access controller.
    pub timed_recovery_delay_in_minutes: Option<u32>,

    /// The next nonce to use for recovery proposals created on this access controller
    pub next_nonce: u32,

    /// The states of the Access Controller.
    pub state: (
        PrimaryRoleState,
        PrimaryOperationState,
        RecoveryOperationState,
    ),
}

impl AccessControllerSubstate {
    pub fn new(controlled_asset: VaultId, timed_recovery_delay_in_minutes: Option<u32>) -> Self {
        Self {
            controlled_asset,
            timed_recovery_delay_in_minutes,
            state: Default::default(),
            next_nonce: 0,
        }
    }

    pub fn next_nonce(&mut self) -> u32 {
        let nonce = self.next_nonce;
        self.next_nonce += 1;
        nonce
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, Default)]
pub enum PrimaryRoleState {
    #[default]
    Unlocked,
    Locked,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, Default)]
pub enum PrimaryOperationState {
    #[default]
    Normal,
    Recovery(RecoveryProposal, u32),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, Default)]
pub enum RecoveryOperationState {
    #[default]
    Normal,
    Recovery(RecoveryRecoveryState, u32),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum RecoveryRecoveryState {
    Untimed(RecoveryProposal),
    Timed {
        proposal: RecoveryProposal,
        timed_recovery_allowed_after: Instant,
    },
}
