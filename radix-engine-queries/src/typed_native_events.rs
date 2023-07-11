//! This module contains the code and models that are required to convert native events into a typed
//! model based on the [`EventTypeIdentifier`] and the raw SBOR bytes of the event. This is used in
//! the toolkit and consumed by the gateway for some of its internal operations.

use crate::typed_substate_layout::*;
use radix_engine::blueprints::native_schema::*;
use radix_engine::blueprints::pool::multi_resource_pool::*;
use radix_engine::blueprints::pool::one_resource_pool::*;
use radix_engine::blueprints::pool::two_resource_pool::*;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::identity::*;

pub fn to_typed_native_event(
    event_type_identifier: &EventTypeIdentifier,
    event_data: &[u8],
) -> Result<TypedNativeEvent, TypedNativeEventError> {
    todo!()
}

define_structure! {
    /* Native Packages */
    AccessController => {
        AccessController => [
            InitiateRecoveryEvent,
            InitiateBadgeWithdrawAttemptEvent,
            RuleSetUpdateEvent,
            BadgeWithdrawEvent,
            CancelRecoveryProposalEvent,
            CancelBadgeWithdrawAttemptEvent,
            LockPrimaryRoleEvent,
            UnlockPrimaryRoleEvent,
            StopTimedRecoveryEvent,
        ],
    },
    Account => {
        Account => []
    },
    Identity => {
        Identity => []
    },
    Package => {
        Package => []
    },
    ConsensusManager => {
        ConsensusManager => [
            RoundChangeEvent,
            EpochChangeEvent
        ],
        Validator => [
            RegisterValidatorEvent,
            UnregisterValidatorEvent,
            StakeEvent,
            UnstakeEvent,
            ClaimXrdEvent,
            UpdateAcceptingStakeDelegationStateEvent,
            ProtocolUpdateReadinessSignalEvent,
            ValidatorEmissionAppliedEvent,
            ValidatorRewardAppliedEvent,
        ],
    },
    Pool => {
        OneResourcePool => [
            OneResourcePoolContributionEvent,
            OneResourcePoolRedemptionEvent,
            OneResourcePoolWithdrawEvent,
            OneResourcePoolDepositEvent,
        ],
        TwoResourcePool => [
            TwoResourcePoolContributionEvent,
            TwoResourcePoolRedemptionEvent,
            TwoResourcePoolWithdrawEvent,
            TwoResourcePoolDepositEvent,
        ],
        MultiResourcePool => [
            MultiResourcePoolContributionEvent,
            MultiResourcePoolRedemptionEvent,
            MultiResourcePoolWithdrawEvent,
            MultiResourcePoolDepositEvent,
        ],
    },
    Resource => {
        FungibleVault => [
            LockFeeEvent,
            WithdrawResourceEvent,
            DepositResourceEvent,
            RecallResourceEvent,
        ],
        NonFungibleVault => [
            LockFeeEvent,
            WithdrawResourceEvent,
            DepositResourceEvent,
            RecallResourceEvent,
        ],
        FungibleResourceManager => [
            VaultCreationEvent,
            MintFungibleResourceEvent,
            BurnFungibleResourceEvent,
        ],
        NonFungibleResourceManager => [
            VaultCreationEvent,
            MintNonFungibleResourceEvent,
            BurnNonFungibleResourceEvent,
        ]
    },
    TransactionProcessor => {
        TransactionProcessor => []
    },
    TransactionTracker => {
        TransactionTracker => []
    },

    /* Node Module Packages */
    AccessRules => {
        AccessRules => [
            SetRoleEvent,
            LockRoleEvent,
            SetAndLockRoleEvent,
            SetOwnerRoleEvent,
            LockOwnerRoleEvent,
            SetAndLockOwnerRoleEvent,
        ]
    },
    Metadata => {
        Metadata => [
            SetMetadataEvent,
            RemoveMetadataEvent,
        ]
    },
    Royalty => {
        Royalty => []
    },
}

// Type aliases for events with the same name in order not to cause use collision issues.
type OneResourcePoolContributionEvent = one_resource_pool::ContributionEvent;
type OneResourcePoolRedemptionEvent = one_resource_pool::RedemptionEvent;
type OneResourcePoolWithdrawEvent = one_resource_pool::WithdrawEvent;
type OneResourcePoolDepositEvent = one_resource_pool::DepositEvent;

type TwoResourcePoolContributionEvent = two_resource_pool::ContributionEvent;
type TwoResourcePoolRedemptionEvent = two_resource_pool::RedemptionEvent;
type TwoResourcePoolWithdrawEvent = two_resource_pool::WithdrawEvent;
type TwoResourcePoolDepositEvent = two_resource_pool::DepositEvent;

type MultiResourcePoolContributionEvent = multi_resource_pool::ContributionEvent;
type MultiResourcePoolRedemptionEvent = multi_resource_pool::RedemptionEvent;
type MultiResourcePoolWithdrawEvent = multi_resource_pool::WithdrawEvent;
type MultiResourcePoolDepositEvent = multi_resource_pool::DepositEvent;

//========
// Macros
//========

/// This enum uses some special syntax to define the structure of events. This makes the code for
/// model definitions very compact, allows for very easy addition of more packages, blueprints or
/// events in the future, keeps various models all in sync, and implements various functions and
/// methods on appropriate types.
///
/// The syntax allowed for by this macro looks like the following:
/// ```no_run
/// define_structure! {
///     package_name1 => {
///         blueprint_name1 => [
///             Event1,
///             Event2,
///             Event3,
///         ],
///         blueprint_name2 => [
///             Event1,
///         ]
///     },
///     package_name2 => {
///         blueprint_name1 => [
///             Event1,
///         ],
///         blueprint_name2 => [
///             Event1,
///             Event2,
///         ]
///     }
/// }
/// ```
macro_rules! define_structure {
    (
        $(
            $package_ident: ident => {
                $(
                    $blueprint_ident: ident => [
                        $($event_ty: ty $(as event_ident: ident)?),* $(,)?
                    ]
                ),* $(,)?
            }
        ),* $(,)?
    ) => {
        paste::paste! {
            // Defining the top-level type which will be of all of the packages and their blueprints.
            pub enum TypedNativeEvent {
                $(
                    $package_ident([< Typed $package_ident PackageEvent >]),
                )*
            }

            // Define a type for the package - this should be an enum of all of the blueprints that
            // the package has.
            $(
                pub enum [< Typed $package_ident PackageEvent >] {
                    $(
                        $blueprint_ident([< Typed $blueprint_ident BlueprintEvent >]),
                    )*
                }

                $(
                    #[derive(radix_engine_interface::prelude::ScryptoSbor)]
                    pub enum [< Typed $blueprint_ident BlueprintEvent >] {
                        $(
                            $event_ty ($event_ty),
                        )*
                    }
                )*
            )*

            // Defining the event key types which are the same as above but do not have any event
            // data inside of them.
            pub enum TypedNativeEventKey {
                $(
                    $package_ident([< Typed $package_ident PackageEventKey >]),
                )*
            }

            $(
                pub enum [< Typed $package_ident PackageEventKey >] {
                    $(
                        $blueprint_ident([< Typed $blueprint_ident BlueprintEventKey >]),
                    )*
                }

                $(
                    #[derive(radix_engine_interface::prelude::ScryptoSbor)]
                    pub enum [< Typed $blueprint_ident BlueprintEventKey >] {
                        $(
                            $event_ty,
                        )*
                    }

                    impl std::str::FromStr for [< Typed $blueprint_ident BlueprintEventKey >] {
                        type Err = TypedNativeEventError;

                        fn from_str(s: &str) -> Result<Self, Self::Err> {
                            match s {
                                $(
                                    _ if <$event_ty as radix_engine_interface::prelude::ScryptoEvent>::event_name() == s => Ok(Self::$event_ty),
                                )*
                                _ => Err(Self::Err::BlueprintEventKeyParseError {
                                    blueprint_event_key: stringify!([< Typed $blueprint_ident BlueprintEventKey >]).to_string(),
                                    event_name: s.to_string()
                                })
                            }
                        }
                    }
                )*
            )*

            // The implementation of a function that converts any `TypedNativeEventKey` + raw SBOR
            // bytes to the appropriate typed event type.
            #[allow(dead_code)] // TODO: Remove
            fn typed_event_with_event_key(
                event_key: &TypedNativeEventKey,
                data: &[u8]
            ) -> Result<TypedNativeEvent, TypedNativeEventError> {
                match event_key {
                    $(
                        $(
                            $(
                                TypedNativeEventKey::$package_ident(
                                    [< Typed $package_ident PackageEventKey >]::$blueprint_ident(
                                        [< Typed $blueprint_ident BlueprintEventKey >]::$event_ty
                                    )
                                ) => Ok(TypedNativeEvent::$package_ident(
                                    [< Typed $package_ident PackageEvent >]::$blueprint_ident(
                                        [< Typed $blueprint_ident BlueprintEvent >]::$event_ty(
                                            radix_engine_interface::prelude::scrypto_decode(data)?
                                        )
                                    )
                                )),
                            )*
                        )*
                    )*

                    // The following panic needs to be included to allow blueprints with no events
                    // to work with no issues. It's impossible for us to get to this point here!
                    _ => panic!("Illegal State! Matching over enum was not exhaustive.")
                }
            }
        }
    };
}
use define_structure;

pub enum TypedNativeEventError {
    EventNameIsInvalidForBlueprint {
        event_name: String,
        blueprint_name: String,
    },
    BlueprintEventKeyParseError {
        blueprint_event_key: String,
        event_name: String,
    },
    DecodeError(DecodeError),
}

impl From<DecodeError> for TypedNativeEventError {
    fn from(value: DecodeError) -> Self {
        Self::DecodeError(value)
    }
}
