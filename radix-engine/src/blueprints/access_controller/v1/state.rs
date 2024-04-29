use super::internal_prelude::*;
use crate::internal_prelude::*;
use crate::*;
use radix_blueprint_schema_init::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::metadata::*;
use sbor::rust::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
#[sbor(type_name = "AccessControllerSubstate")]
pub struct AccessControllerV1Substate {
    /// A vault where the asset controlled by the access controller lives.
    pub controlled_asset: Vault,

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

impl AccessControllerV1Substate {
    pub fn new(
        controlled_asset: Vault,
        timed_recovery_delay_in_minutes: Option<u32>,
        recovery_badge: ResourceAddress,
    ) -> Self {
        Self {
            controlled_asset,
            timed_recovery_delay_in_minutes,
            recovery_badge,
            state: Default::default(),
        }
    }
}

declare_native_blueprint_state! {
    blueprint_ident: AccessController,
    blueprint_snake_case: access_controller,
    features: {
    },
    fields: {
        state:  {
            ident: State,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        }
    },
    collections: {}
}

pub type AccessControllerStateV1 = AccessControllerV1Substate;
