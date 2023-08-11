//! This module contains the code and models that are required to convert native events into a typed
//! model based on the [`EventTypeIdentifier`] and the raw SBOR bytes of the event. This is used in
//! the toolkit and consumed by the gateway for some of its internal operations.

use crate::typed_substate_layout::*;
use radix_engine::blueprints::native_schema::*;
use radix_engine::types::*;
use radix_engine_interface::api::*;

/// Given an [`EventTypeIdentifier`] and the raw event data, this function attempts to convert the
/// event data into a structured model provided that the event is registered to a native blueprint.
///
/// # Panics
///
/// This function panics if the even't [`BlueprintPayloadDef`] is of variant [`BlueprintPayloadDef::Generic`] as
/// generics are not supported in events.
pub fn to_typed_native_event(
    event_type_identifier: &EventTypeIdentifier,
    event_data: &[u8],
) -> Result<TypedNativeEvent, TypedNativeEventError> {
    let typed_native_event_key =
        resolve_typed_event_key_from_event_type_identifier(event_type_identifier)?;
    to_typed_event_with_event_key(&typed_native_event_key, event_data)
}

fn resolve_typed_event_key_from_event_type_identifier(
    event_type_identifier: &EventTypeIdentifier,
) -> Result<TypedNativeEventKey, TypedNativeEventError> {
    let event_name = &event_type_identifier.1;

    match &event_type_identifier.0 {
        /* Method or Function emitter on a known node module */
        Emitter::Method(_, ObjectModuleId::RoleAssignment) => {
            TypedRoleAssignmentBlueprintEventKey::new(
                &event_name,
            )
            .map(TypedNativeEventKey::from)
        }
        Emitter::Method(_, ObjectModuleId::Metadata) => TypedMetadataBlueprintEventKey::new(
            &event_name,
        )
        .map(TypedNativeEventKey::from),
        Emitter::Method(_, ObjectModuleId::Royalty) => TypedComponentRoyaltyBlueprintEventKey::new(
            &event_name,
        )
        .map(TypedNativeEventKey::from),

        /* Functions on well-known packages */
        Emitter::Function(blueprint_id) => match blueprint_id.package_address {
            PACKAGE_PACKAGE => TypedPackagePackageEventKey::new(
                &PACKAGE_PACKAGE_DEFINITION,
                &blueprint_id.blueprint_name,
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            RESOURCE_PACKAGE => TypedResourcePackageEventKey::new(
                &RESOURCE_PACKAGE_DEFINITION,
                &blueprint_id.blueprint_name,
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            ACCOUNT_PACKAGE => TypedAccountPackageEventKey::new(
                &ACCOUNT_PACKAGE_DEFINITION,
                &blueprint_id.blueprint_name,
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            IDENTITY_PACKAGE => TypedIdentityPackageEventKey::new(
                &IDENTITY_PACKAGE_DEFINITION,
                &blueprint_id.blueprint_name,
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            CONSENSUS_MANAGER_PACKAGE => TypedConsensusManagerPackageEventKey::new(
                &CONSENSUS_MANAGER_PACKAGE_DEFINITION,
                &blueprint_id.blueprint_name,
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            ACCESS_CONTROLLER_PACKAGE => TypedAccessControllerPackageEventKey::new(
                &ACCESS_CONTROLLER_PACKAGE_DEFINITION,
                &blueprint_id.blueprint_name,
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            POOL_PACKAGE => TypedPoolPackageEventKey::new(
                &POOL_PACKAGE_DEFINITION,
                &blueprint_id.blueprint_name,
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            TRANSACTION_PROCESSOR_PACKAGE => TypedTransactionProcessorPackageEventKey::new(
                &TRANSACTION_PROCESSOR_PACKAGE_DEFINITION,
                &blueprint_id.blueprint_name,
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            METADATA_MODULE_PACKAGE => TypedMetadataPackageEventKey::new(
                &METADATA_PACKAGE_DEFINITION,
                &blueprint_id.blueprint_name,
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            ROYALTY_MODULE_PACKAGE => TypedRoyaltyPackageEventKey::new(
                &ROYALTY_PACKAGE_DEFINITION,
                &blueprint_id.blueprint_name,
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            ROLE_ASSIGNMENT_MODULE_PACKAGE => TypedRoleAssignmentPackageEventKey::new(
                &ROLE_ASSIGNMENT_PACKAGE_DEFINITION,
                &blueprint_id.blueprint_name,
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            TRANSACTION_TRACKER_PACKAGE => TypedTransactionTrackerPackageEventKey::new(
                &TRANSACTION_TRACKER_PACKAGE_DEFINITION,
                &blueprint_id.blueprint_name,
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            _ => Err(TypedNativeEventError::NotANativeBlueprint(
                event_type_identifier.clone(),
            )),
        },

        /* Methods on non-generic components */
        Emitter::Method(node_id, ObjectModuleId::Main) => match node_id.entity_type().unwrap() {
            EntityType::GlobalPackage => TypedPackageBlueprintEventKey::new(
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            EntityType::GlobalConsensusManager => TypedConsensusManagerBlueprintEventKey::new(
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            EntityType::GlobalValidator => TypedValidatorBlueprintEventKey::new(
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            EntityType::GlobalTransactionTracker => TypedTransactionTrackerBlueprintEventKey::new(
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            EntityType::GlobalAccount
            | EntityType::InternalAccount
            | EntityType::GlobalVirtualSecp256k1Account
            | EntityType::GlobalVirtualEd25519Account => TypedAccountBlueprintEventKey::new(
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            EntityType::GlobalIdentity
            | EntityType::GlobalVirtualSecp256k1Identity
            | EntityType::GlobalVirtualEd25519Identity => TypedIdentityBlueprintEventKey::new(
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            EntityType::GlobalAccessController => TypedAccessControllerBlueprintEventKey::new(
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            EntityType::GlobalOneResourcePool => TypedOneResourcePoolBlueprintEventKey::new(
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            EntityType::GlobalTwoResourcePool => TypedTwoResourcePoolBlueprintEventKey::new(
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            EntityType::GlobalMultiResourcePool => TypedMultiResourcePoolBlueprintEventKey::new(
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            EntityType::GlobalFungibleResourceManager => {
                TypedFungibleResourceManagerBlueprintEventKey::new(
                    &event_name,
                )
                .map(TypedNativeEventKey::from)
            }
            EntityType::GlobalNonFungibleResourceManager => {
                TypedNonFungibleResourceManagerBlueprintEventKey::new(
                    &event_name,
                )
                .map(TypedNativeEventKey::from)
            }
            EntityType::InternalFungibleVault => TypedFungibleVaultBlueprintEventKey::new(
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            EntityType::InternalNonFungibleVault => TypedNonFungibleVaultBlueprintEventKey::new(
                &event_name,
            )
            .map(TypedNativeEventKey::from),
            EntityType::GlobalGenericComponent
            | EntityType::InternalGenericComponent
            | EntityType::InternalKeyValueStore => Err(TypedNativeEventError::NotANativeBlueprint(
                event_type_identifier.clone(),
            )),
        },
    }
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
    RoleAssignment => {
        RoleAssignment => [
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
        ComponentRoyalty => []
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
                        $($event_ty: ty),* $(,)?
                    ]
                ),* $(,)?
            }
        ),* $(,)?
    ) => {
        paste::paste! {
            // Defining the top-level type which will be of all of the packages and their blueprints.
            #[derive(Debug)]
            pub enum TypedNativeEvent {
                $(
                    $package_ident([< Typed $package_ident PackageEvent >]),
                )*
            }

            // Define a type for the package - this should be an enum of all of the blueprints that
            // the package has.
            $(
                #[derive(Debug)]
                pub enum [< Typed $package_ident PackageEvent >] {
                    $(
                        $blueprint_ident([< Typed $blueprint_ident BlueprintEvent >]),
                    )*
                }

                $(
                    #[derive(Debug)]
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

                    impl sbor::prelude::FromStr for [< Typed $blueprint_ident BlueprintEventKey >] {
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

            $(
                $(
                    impl From<[< Typed $blueprint_ident BlueprintEventKey >]> for TypedNativeEventKey {
                        fn from(value: [< Typed $blueprint_ident BlueprintEventKey >]) -> Self {
                            Self::$package_ident(
                                [< Typed $package_ident PackageEventKey >]::$blueprint_ident(value)
                            )
                        }
                    }

                    impl [< Typed $blueprint_ident BlueprintEventKey >] {
                        pub fn new(
                            name: &String,
                        ) -> Result<Self, TypedNativeEventError> {
                            Self::from_str(name)
                        }

                        #[allow(unused_mut)]
                        pub fn registered_events() -> sbor::prelude::HashSet<String> {
                            let mut set = sbor::prelude::HashSet::default();
                            $(
                                set.insert(<$event_ty as radix_engine_interface::prelude::ScryptoEvent>::event_name().to_owned());
                            )*
                            set
                        }
                    }
                )*
            )*

            $(
                impl From<[< Typed $package_ident PackageEventKey >]> for TypedNativeEventKey {
                    fn from(value: [< Typed $package_ident PackageEventKey >]) -> Self {
                        Self::$package_ident(value)
                    }
                }

                impl [< Typed $package_ident PackageEventKey >] {
                    pub fn new(
                        package_definition: &PackageDefinition,
                        blueprint_ident: &str,
                        event_name: &String,
                    ) -> Result<Self, TypedNativeEventError> {
                        match blueprint_ident {
                            $(
                                stringify!($blueprint_ident) => Ok(Self::$blueprint_ident([< Typed $blueprint_ident BlueprintEventKey >]::new(
                                    event_name)?
                                )),
                            )*
                            _ => Err(TypedNativeEventError::BlueprintNotFound {
                                package_definition: package_definition.clone(),
                                blueprint_name: blueprint_ident.to_owned(),
                            })
                        }
                    }

                    pub fn registered_events() -> sbor::prelude::HashMap<String, sbor::prelude::HashSet<String>> {
                        let mut map = sbor::prelude::HashMap::<String, sbor::prelude::HashSet<String>>::default();
                        $(
                            map.insert(
                                stringify!($blueprint_ident).to_owned(),
                                [< Typed $blueprint_ident BlueprintEventKey >]::registered_events()
                            );
                        )*
                        map
                    }
                }
            )*

            impl TypedNativeEvent {
                pub fn registered_events() -> sbor::prelude::HashMap<String, sbor::prelude::HashMap<String, sbor::prelude::HashSet<String>>> {
                    let mut map = sbor::prelude::HashMap::<String, sbor::prelude::HashMap<String, sbor::prelude::HashSet<String>>>::default();
                    $(
                        map.insert(stringify!($package_ident).to_owned(), [< Typed $package_ident PackageEventKey >]::registered_events());
                    )*
                    map
                }
            }

            // The implementation of a function that converts any `TypedNativeEventKey` + raw SBOR
            // bytes to the appropriate typed event type.
            fn to_typed_event_with_event_key(
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

#[derive(Debug)]
pub enum TypedNativeEventError {
    BlueprintEventKeyParseError {
        blueprint_event_key: String,
        event_name: String,
    },
    BlueprintNotFound {
        package_definition: PackageDefinition,
        blueprint_name: String,
    },
    TypeHasNoName {
        package_definition: PackageDefinition,
        blueprint_name: String,
        local_type_index: LocalTypeIndex,
    },
    NotANativeBlueprint(EventTypeIdentifier),
    DecodeError(DecodeError),
}

impl From<DecodeError> for TypedNativeEventError {
    fn from(value: DecodeError) -> Self {
        Self::DecodeError(value)
    }
}
