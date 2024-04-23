use super::internal_prelude::*;
use crate::blueprints::access_controller::v1::{
    AccessControllerStateV1, AccessControllerV1Substate,
};
use crate::internal_prelude::*;
use crate::*;
use radix_blueprint_schema_init::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::metadata::*;
use sbor::rust::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
#[sbor(type_name = "AccessControllerSubstate")]
pub struct AccessControllerV2Substate {
    /// A vault where the asset controlled by the access controller lives.
    pub controlled_asset: Vault,

    /// A vault that stores some XRD that can be used by any of the three roles for locking fees.
    pub xrd_fee_vault: Option<Vault>,

    /// The amount of time (in minutes) that it takes for timed recovery to be done. Maximum is
    /// 4,294,967,295 minutes which is 8171.5511700913 years. When this is [`None`], then timed
    /// recovery can not be performed through this access controller.
    pub timed_recovery_delay_in_minutes: Option<u32>,

    /// The resource address of the recovery badge that will be used by the wallet and optionally
    /// by other clients as well.
    pub recovery_badge: ResourceAddress,

    /// The states of the Access Controller.
    pub state: (
        // Controls whether the primary role is locked or unlocked
        PrimaryRoleLockingState,
        // Primary role recovery and withdraw states
        PrimaryRoleRecoveryAttemptState,
        PrimaryRoleBadgeWithdrawAttemptState,
        // Recovery role recovery and withdraw states
        RecoveryRoleRecoveryAttemptState,
        RecoveryRoleBadgeWithdrawAttemptState,
    ),
}

impl AccessControllerV2Substate {
    pub fn new(
        controlled_asset: Vault,
        xrd_fee_vault: Option<Vault>,
        timed_recovery_delay_in_minutes: Option<u32>,
        recovery_badge: ResourceAddress,
    ) -> Self {
        Self {
            controlled_asset,
            xrd_fee_vault,
            timed_recovery_delay_in_minutes,
            recovery_badge,
            state: Default::default(),
        }
    }
}

impl From<AccessControllerV1Substate> for AccessControllerV2Substate {
    fn from(
        AccessControllerV1Substate {
            controlled_asset,
            timed_recovery_delay_in_minutes,
            recovery_badge,
            state,
        }: AccessControllerV1Substate,
    ) -> Self {
        Self {
            controlled_asset,
            xrd_fee_vault: None,
            timed_recovery_delay_in_minutes,
            recovery_badge,
            state,
        }
    }
}

declare_native_blueprint_state! {
    blueprint_ident: AccessControllerV2,
    blueprint_snake_case: access_controller,
    features: {
    },
    fields: {
        state:  {
            ident: State,
            field_type: {
                kind: StaticMultiVersioned,
                previous_versions: [
                    1 => { updates_to: 2 }
                ],
                latest_version: 2,
            },
            condition: Condition::Always,
        }
    },
    collections: {}
}

pub type AccessControllerV2PartitionOffset = AccessControllerPartitionOffset;
pub type AccessControllerV2StateV1 = AccessControllerStateV1;
pub type AccessControllerV2StateV2 = AccessControllerV2Substate;
