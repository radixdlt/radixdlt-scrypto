use crate::internal_prelude::*;
use crate::utils::*;
use radix_engine::blueprints::account::DepositEvent;
use radix_engine::updates::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::*;

pub struct AccountLockerScenarioConfig {
    pub account_locker_admin_account_: PrivateKey,
    pub account_rejecting_fungible_resource_: PrivateKey,
    pub account_accepting_all_resources_: PrivateKey,
    pub account_rejecting_non_fungible_resource_: PrivateKey,
    pub account_rejecting_all_deposits_: PrivateKey,
}

#[derive(Default)]
pub struct AccountLockerScenarioState {
    pub(crate) account_locker: State<ComponentAddress>,
    pub(crate) account_locker_admin_badge: State<ResourceAddress>,

    pub(crate) fungible_resource: State<ResourceAddress>,
    pub(crate) non_fungible_resource: State<ResourceAddress>,

    pub(crate) account_locker_admin_account: State<ComponentAddress>,
    pub(crate) account_rejecting_fungible_resource: State<ComponentAddress>,
    pub(crate) account_accepting_all_resources: State<ComponentAddress>,
    pub(crate) account_rejecting_non_fungible_resource: State<ComponentAddress>,
    pub(crate) account_rejecting_all_deposits: State<ComponentAddress>,
}

impl Default for AccountLockerScenarioConfig {
    fn default() -> Self {
        Self {
            account_locker_admin_account_: new_ed25519_private_key(1).into(),
            account_rejecting_fungible_resource_: new_ed25519_private_key(2).into(),
            account_accepting_all_resources_: new_ed25519_private_key(3).into(),
            account_rejecting_non_fungible_resource_: new_ed25519_private_key(4).into(),
            account_rejecting_all_deposits_: new_ed25519_private_key(5).into(),
        }
    }
}

pub struct AccountLockerScenarioCreator;

impl ScenarioCreator for AccountLockerScenarioCreator {
    type Config = AccountLockerScenarioConfig;
    type State = AccountLockerScenarioState;
    type Instance = Scenario<Self::Config, Self::State>;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "account_locker",
        protocol_min_requirement: ProtocolVersion::Bottlenose,
        protocol_max_requirement: ProtocolVersion::LATEST,
        testnet_run_at: Some(ProtocolVersion::Bottlenose),
        safe_to_run_on_used_ledger: true,
    };

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Self::Instance {
        #[allow(unused_variables, deprecated)]
        ScenarioBuilder::new(core, Self::METADATA, config, start_state)
            .successful_transaction_with_result_handler(
                |core, config, _| {
                    core.next_transaction_with_faucet_lock_fee(
                        "account-locker-create-accounts",
                        |builder| {
                            [
                                &config.account_locker_admin_account_,
                                &config.account_rejecting_fungible_resource_,
                                &config.account_accepting_all_resources_,
                                &config.account_rejecting_non_fungible_resource_,
                                &config.account_rejecting_all_deposits_,
                            ]
                            .iter()
                            .fold(builder, |builder, key| {
                                builder.call_function(
                                    ACCOUNT_PACKAGE,
                                    ACCOUNT_BLUEPRINT,
                                    ACCOUNT_CREATE_ADVANCED_IDENT,
                                    AccountCreateAdvancedManifestInput {
                                        address_reservation: None,
                                        owner_role: OwnerRole::Fixed(rule!(require(
                                            NonFungibleGlobalId::from_public_key(&key.public_key())
                                        ))),
                                    },
                                )
                            })
                        },
                        vec![],
                    )
                },
                |_, _, state, result| {
                    state
                        .account_locker_admin_account
                        .set(result.new_component_addresses()[0]);
                    state
                        .account_rejecting_fungible_resource
                        .set(result.new_component_addresses()[1]);
                    state
                        .account_accepting_all_resources
                        .set(result.new_component_addresses()[2]);
                    state
                        .account_rejecting_non_fungible_resource
                        .set(result.new_component_addresses()[3]);
                    state
                        .account_rejecting_all_deposits
                        .set(result.new_component_addresses()[4]);
                    Ok(())
                },
            )
            .successful_transaction_with_result_handler(
                |core, _, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "account-locker-create-account-locker",
                        |builder| {
                            builder
                                .call_function(
                                    LOCKER_PACKAGE,
                                    ACCOUNT_LOCKER_BLUEPRINT,
                                    ACCOUNT_LOCKER_INSTANTIATE_SIMPLE_IDENT,
                                    AccountLockerInstantiateSimpleManifestInput {
                                        allow_recover: true,
                                    },
                                )
                                .try_deposit_entire_worktop_or_abort(
                                    state.account_locker_admin_account.unwrap(),
                                    None,
                                )
                        },
                        vec![],
                    )
                },
                |_, _, state, result| {
                    state
                        .account_locker
                        .set(result.new_component_addresses()[0]);
                    state
                        .account_locker_admin_badge
                        .set(result.new_resource_addresses()[0]);
                    Ok(())
                },
            )
            .successful_transaction_with_result_handler(
                |core, _, _| {
                    core.next_transaction_with_faucet_lock_fee(
                        "account-locker-create-resources",
                        |builder| {
                            builder
                                .create_fungible_resource(
                                    OwnerRole::None,
                                    true,
                                    0,
                                    FungibleResourceRoles {
                                        mint_roles: mint_roles! {
                                            minter => rule!(allow_all);
                                            minter_updater => rule!(deny_all);
                                        },
                                        burn_roles: burn_roles! {
                                            burner => rule!(allow_all);
                                            burner_updater => rule!(deny_all);
                                        },
                                        ..Default::default()
                                    },
                                    Default::default(),
                                    None,
                                )
                                .create_non_fungible_resource::<[(NonFungibleLocalId, ()); 0], _>(
                                    OwnerRole::None,
                                    NonFungibleIdType::Integer,
                                    true,
                                    NonFungibleResourceRoles {
                                        mint_roles: mint_roles! {
                                            minter => rule!(allow_all);
                                            minter_updater => rule!(deny_all);
                                        },
                                        burn_roles: burn_roles! {
                                            burner => rule!(allow_all);
                                            burner_updater => rule!(deny_all);
                                        },
                                        ..Default::default()
                                    },
                                    Default::default(),
                                    None,
                                )
                        },
                        vec![],
                    )
                },
                |_, _, state, result| {
                    state
                        .fungible_resource
                        .set(result.new_resource_addresses()[0]);
                    state
                        .non_fungible_resource
                        .set(result.new_resource_addresses()[1]);
                    Ok(())
                },
            )
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-setting-up-account-deposit-rules",
                    |builder| {
                        builder
                            .call_method(
                                state.account_rejecting_fungible_resource.unwrap(),
                                ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT,
                                AccountSetResourcePreferenceInput {
                                    resource_address: state.fungible_resource.unwrap().into(),
                                    resource_preference: ResourcePreference::Disallowed,
                                },
                            )
                            .call_method(
                                state.account_rejecting_non_fungible_resource.unwrap(),
                                ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT,
                                AccountSetResourcePreferenceInput {
                                    resource_address: state.non_fungible_resource.unwrap().into(),
                                    resource_preference: ResourcePreference::Disallowed,
                                },
                            )
                            .call_method(
                                state.account_rejecting_all_deposits.unwrap(),
                                ACCOUNT_SET_DEFAULT_DEPOSIT_RULE_IDENT,
                                AccountSetDefaultDepositRuleInput {
                                    default: DefaultDepositRule::Accept,
                                },
                            )
                            .call_method(
                                state.account_rejecting_all_deposits.unwrap(),
                                ACCOUNT_ADD_AUTHORIZED_DEPOSITOR_IDENT,
                                AccountAddAuthorizedDepositorInput {
                                    badge: global_caller(state.account_locker.unwrap()),
                                },
                            )
                    },
                    vec![
                        &config.account_rejecting_fungible_resource_,
                        &config.account_rejecting_non_fungible_resource_,
                        &config.account_rejecting_all_deposits_,
                    ],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-send-fungibles-and-try-direct-deposit-succeeds",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .mint_fungible(state.fungible_resource.unwrap(), dec!(100))
                            .take_all_from_worktop(state.fungible_resource.unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.unwrap(),
                                    ACCOUNT_LOCKER_STORE_IDENT,
                                    AccountLockerStoreManifestInput {
                                        bucket,
                                        claimant: state
                                            .account_accepting_all_resources
                                            .unwrap()
                                            .into(),
                                        try_direct_send: true,
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account_],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-send-fungibles-and-try-direct-deposit-refunds",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .mint_fungible(state.fungible_resource.unwrap(), dec!(100))
                            .take_all_from_worktop(state.fungible_resource.unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.unwrap(),
                                    ACCOUNT_LOCKER_STORE_IDENT,
                                    AccountLockerStoreManifestInput {
                                        bucket,
                                        claimant: state
                                            .account_rejecting_fungible_resource
                                            .unwrap()
                                            .into(),
                                        try_direct_send: true,
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account_],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-send-fungibles-and-dont-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .mint_fungible(state.fungible_resource.unwrap(), dec!(100))
                            .take_all_from_worktop(state.fungible_resource.unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.unwrap(),
                                    ACCOUNT_LOCKER_STORE_IDENT,
                                    AccountLockerStoreManifestInput {
                                        bucket,
                                        claimant: state
                                            .account_accepting_all_resources
                                            .unwrap()
                                            .into(),
                                        try_direct_send: false,
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account_],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-airdrop-fungibles-and-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .mint_fungible(state.fungible_resource.unwrap(), dec!(300))
                            .take_all_from_worktop(state.fungible_resource.unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.unwrap(),
                                    ACCOUNT_LOCKER_AIRDROP_IDENT,
                                    AccountLockerAirdropManifestInput {
                                        bucket,
                                        try_direct_send: true,
                                        claimants: [
                                            &state.account_rejecting_fungible_resource,
                                            &state.account_accepting_all_resources,
                                            &state.account_rejecting_non_fungible_resource,
                                        ]
                                        .into_iter()
                                        .map(|account| {
                                            (
                                                account.unwrap().into(),
                                                ResourceSpecifier::Fungible(dec!(100)),
                                            )
                                        })
                                        .collect(),
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account_],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-airdrop-fungibles-and-dont-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .mint_fungible(state.fungible_resource.unwrap(), dec!(300))
                            .take_all_from_worktop(state.fungible_resource.unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.unwrap(),
                                    ACCOUNT_LOCKER_AIRDROP_IDENT,
                                    AccountLockerAirdropManifestInput {
                                        bucket,
                                        try_direct_send: false,
                                        claimants: [
                                            &state.account_rejecting_fungible_resource,
                                            &state.account_accepting_all_resources,
                                            &state.account_rejecting_non_fungible_resource,
                                        ]
                                        .into_iter()
                                        .map(|account| {
                                            (
                                                account.unwrap().into(),
                                                ResourceSpecifier::Fungible(dec!(100)),
                                            )
                                        })
                                        .collect(),
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account_],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-send-non-fungibles-and-try-direct-deposit-succeeds",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .mint_non_fungible(
                                state.non_fungible_resource.unwrap(),
                                [(NonFungibleLocalId::integer(1), ())],
                            )
                            .take_all_from_worktop(state.non_fungible_resource.unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.unwrap(),
                                    ACCOUNT_LOCKER_STORE_IDENT,
                                    AccountLockerStoreManifestInput {
                                        bucket,
                                        claimant: state
                                            .account_accepting_all_resources
                                            .unwrap()
                                            .into(),
                                        try_direct_send: true,
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account_],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-send-non-fungibles-and-try-direct-deposit-refunds",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .mint_non_fungible(
                                state.non_fungible_resource.unwrap(),
                                [(NonFungibleLocalId::integer(2), ())],
                            )
                            .take_all_from_worktop(state.non_fungible_resource.unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.unwrap(),
                                    ACCOUNT_LOCKER_STORE_IDENT,
                                    AccountLockerStoreManifestInput {
                                        bucket,
                                        claimant: state
                                            .account_rejecting_fungible_resource
                                            .unwrap()
                                            .into(),
                                        try_direct_send: true,
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account_],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-send-non-fungibles-and-dont-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .mint_non_fungible(
                                state.non_fungible_resource.unwrap(),
                                [(NonFungibleLocalId::integer(3), ())],
                            )
                            .take_all_from_worktop(state.non_fungible_resource.unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.unwrap(),
                                    ACCOUNT_LOCKER_STORE_IDENT,
                                    AccountLockerStoreManifestInput {
                                        bucket,
                                        claimant: state
                                            .account_accepting_all_resources
                                            .unwrap()
                                            .into(),
                                        try_direct_send: false,
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account_],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-airdrop-non-fungibles-by-amount-and-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .mint_non_fungible(
                                state.non_fungible_resource.unwrap(),
                                [
                                    (NonFungibleLocalId::integer(4), ()),
                                    (NonFungibleLocalId::integer(5), ()),
                                    (NonFungibleLocalId::integer(6), ()),
                                ],
                            )
                            .take_all_from_worktop(state.non_fungible_resource.unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.unwrap(),
                                    ACCOUNT_LOCKER_AIRDROP_IDENT,
                                    AccountLockerAirdropManifestInput {
                                        bucket,
                                        try_direct_send: true,
                                        claimants: [
                                            &state.account_rejecting_fungible_resource,
                                            &state.account_accepting_all_resources,
                                            &state.account_rejecting_non_fungible_resource,
                                        ]
                                        .into_iter()
                                        .map(|account| {
                                            (
                                                account.unwrap().into(),
                                                ResourceSpecifier::Fungible(dec!(1)),
                                            )
                                        })
                                        .collect(),
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account_],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-airdrop-non-fungibles-by-amount-and-dont-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .mint_non_fungible(
                                state.non_fungible_resource.unwrap(),
                                [
                                    (NonFungibleLocalId::integer(7), ()),
                                    (NonFungibleLocalId::integer(8), ()),
                                    (NonFungibleLocalId::integer(9), ()),
                                ],
                            )
                            .take_all_from_worktop(state.non_fungible_resource.unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.unwrap(),
                                    ACCOUNT_LOCKER_AIRDROP_IDENT,
                                    AccountLockerAirdropManifestInput {
                                        bucket,
                                        try_direct_send: false,
                                        claimants: [
                                            &state.account_rejecting_fungible_resource,
                                            &state.account_accepting_all_resources,
                                            &state.account_rejecting_non_fungible_resource,
                                        ]
                                        .into_iter()
                                        .map(|account| {
                                            (
                                                account.unwrap().into(),
                                                ResourceSpecifier::Fungible(dec!(1)),
                                            )
                                        })
                                        .collect(),
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account_],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-airdrop-non-fungibles-by-ids-and-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .mint_non_fungible(
                                state.non_fungible_resource.unwrap(),
                                [
                                    (NonFungibleLocalId::integer(10), ()),
                                    (NonFungibleLocalId::integer(11), ()),
                                    (NonFungibleLocalId::integer(12), ()),
                                ],
                            )
                            .take_all_from_worktop(state.non_fungible_resource.unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.unwrap(),
                                    ACCOUNT_LOCKER_AIRDROP_IDENT,
                                    AccountLockerAirdropManifestInput {
                                        bucket,
                                        try_direct_send: true,
                                        claimants: [
                                            (&state.account_rejecting_fungible_resource, 10),
                                            (&state.account_accepting_all_resources, 11),
                                            (&state.account_rejecting_non_fungible_resource, 12),
                                        ]
                                        .into_iter()
                                        .map(|(account, id)| {
                                            (
                                                account.unwrap().into(),
                                                ResourceSpecifier::NonFungible(indexset![
                                                    NonFungibleLocalId::integer(id)
                                                ]),
                                            )
                                        })
                                        .collect(),
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account_],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-airdrop-non-fungibles-by-ids-and-dont-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .mint_non_fungible(
                                state.non_fungible_resource.unwrap(),
                                [
                                    (NonFungibleLocalId::integer(13), ()),
                                    (NonFungibleLocalId::integer(14), ()),
                                    (NonFungibleLocalId::integer(15), ()),
                                ],
                            )
                            .take_all_from_worktop(state.non_fungible_resource.unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.unwrap(),
                                    ACCOUNT_LOCKER_AIRDROP_IDENT,
                                    AccountLockerAirdropManifestInput {
                                        bucket,
                                        try_direct_send: false,
                                        claimants: [
                                            (&state.account_rejecting_fungible_resource, 13),
                                            (&state.account_accepting_all_resources, 14),
                                            (&state.account_rejecting_non_fungible_resource, 15),
                                        ]
                                        .into_iter()
                                        .map(|(account, id)| {
                                            (
                                                account.unwrap().into(),
                                                ResourceSpecifier::NonFungible(indexset![
                                                    NonFungibleLocalId::integer(id)
                                                ]),
                                            )
                                        })
                                        .collect(),
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account_],
                )
            })
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "account-locker-global-caller-badge-is-an-authorized-depositor",
                        |builder| {
                            builder
                                .create_proof_from_account_of_amount(
                                    state.account_locker_admin_account.unwrap(),
                                    state.account_locker_admin_badge.unwrap(),
                                    dec!(1),
                                )
                                .mint_fungible(state.fungible_resource.unwrap(), dec!(100))
                                .take_all_from_worktop(state.fungible_resource.unwrap(), "bucket")
                                .with_bucket("bucket", |builder, bucket| {
                                    builder.call_method(
                                        state.account_locker.unwrap(),
                                        ACCOUNT_LOCKER_STORE_IDENT,
                                        AccountLockerStoreManifestInput {
                                            bucket,
                                            claimant: state
                                                .account_rejecting_all_deposits
                                                .unwrap()
                                                .into(),
                                            try_direct_send: true,
                                        },
                                    )
                                })
                        },
                        vec![&config.account_locker_admin_account_],
                    )
                },
                |_, _, state, result| {
                    let event = result
                        .application_events
                        .iter()
                        .find(|item| {
                            item.0
                                == EventTypeIdentifier(
                                    Emitter::Method(
                                        state
                                            .account_rejecting_all_deposits
                                            .unwrap()
                                            .into_node_id(),
                                        ModuleId::Main,
                                    ),
                                    DepositEvent::EVENT_NAME.to_owned(),
                                )
                        })
                        .map(|(_, data)| scrypto_decode::<DepositEvent>(data).expect("Can't fail"))
                        .expect("The resources were not deposited into the account?");
                    assert_eq!(
                        event,
                        DepositEvent::Fungible(state.fungible_resource.unwrap(), dec!(100))
                    );
                    Ok(())
                },
            )
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-claim-fungibles-by-amount",
                    |builder| {
                        builder
                            .call_method(
                                state.account_locker.unwrap(),
                                ACCOUNT_LOCKER_CLAIM_IDENT,
                                AccountLockerClaimManifestInput {
                                    claimant: state
                                        .account_rejecting_fungible_resource
                                        .unwrap()
                                        .into(),
                                    amount: dec!(1),
                                    resource_address: state.fungible_resource.unwrap().into(),
                                },
                            )
                            .deposit_entire_worktop(
                                state.account_rejecting_fungible_resource.unwrap(),
                            )
                    },
                    vec![&config.account_rejecting_fungible_resource_],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-claim-non-fungibles-by-amount",
                    |builder| {
                        builder
                            .call_method(
                                state.account_locker.unwrap(),
                                ACCOUNT_LOCKER_CLAIM_IDENT,
                                AccountLockerClaimManifestInput {
                                    claimant: state
                                        .account_rejecting_fungible_resource
                                        .unwrap()
                                        .into(),
                                    amount: dec!(1),
                                    resource_address: state.non_fungible_resource.unwrap().into(),
                                },
                            )
                            .deposit_entire_worktop(
                                state.account_rejecting_fungible_resource.unwrap(),
                            )
                    },
                    vec![&config.account_rejecting_fungible_resource_],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-claim-non-fungibles-by-ids",
                    |builder| {
                        builder
                            .call_method(
                                state.account_locker.unwrap(),
                                ACCOUNT_LOCKER_CLAIM_NON_FUNGIBLES_IDENT,
                                AccountLockerClaimNonFungiblesManifestInput {
                                    claimant: state.account_accepting_all_resources.unwrap().into(),
                                    resource_address: state.non_fungible_resource.unwrap().into(),
                                    ids: indexset![NonFungibleLocalId::integer(3)],
                                },
                            )
                            .deposit_entire_worktop(state.account_accepting_all_resources.unwrap())
                    },
                    vec![&config.account_accepting_all_resources_],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-recover-fungibles-by-amount",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .call_method(
                                state.account_locker.unwrap(),
                                ACCOUNT_LOCKER_RECOVER_IDENT,
                                AccountLockerRecoverManifestInput {
                                    claimant: state
                                        .account_rejecting_fungible_resource
                                        .unwrap()
                                        .into(),
                                    amount: dec!(1),
                                    resource_address: state.fungible_resource.unwrap().into(),
                                },
                            )
                            .deposit_entire_worktop(
                                state.account_rejecting_fungible_resource.unwrap(),
                            )
                    },
                    vec![
                        &config.account_locker_admin_account_,
                        &config.account_rejecting_fungible_resource_,
                    ],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-recover-non-fungibles-by-amount",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .call_method(
                                state.account_locker.unwrap(),
                                ACCOUNT_LOCKER_RECOVER_IDENT,
                                AccountLockerRecoverManifestInput {
                                    claimant: state
                                        .account_rejecting_fungible_resource
                                        .unwrap()
                                        .into(),
                                    amount: dec!(1),
                                    resource_address: state.non_fungible_resource.unwrap().into(),
                                },
                            )
                            .deposit_entire_worktop(
                                state.account_rejecting_fungible_resource.unwrap(),
                            )
                    },
                    vec![
                        &config.account_locker_admin_account_,
                        &config.account_rejecting_fungible_resource_,
                    ],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-recover-non-fungibles-by-ids",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                state.account_locker_admin_account.unwrap(),
                                state.account_locker_admin_badge.unwrap(),
                                dec!(1),
                            )
                            .call_method(
                                state.account_locker.unwrap(),
                                ACCOUNT_LOCKER_RECOVER_NON_FUNGIBLES_IDENT,
                                AccountLockerRecoverNonFungiblesManifestInput {
                                    claimant: state
                                        .account_rejecting_non_fungible_resource
                                        .unwrap()
                                        .into(),
                                    resource_address: state.non_fungible_resource.unwrap().into(),
                                    ids: indexset![NonFungibleLocalId::integer(15)],
                                },
                            )
                            .deposit_entire_worktop(
                                state.account_rejecting_non_fungible_resource.unwrap(),
                            )
                    },
                    vec![
                        &config.account_locker_admin_account_,
                        &config.account_rejecting_non_fungible_resource_,
                    ],
                )
            })
            .finalize(|_, _, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add(
                            "badge_holder_account",
                            state.account_locker_admin_account.get()?,
                        )
                        .add(
                            "account_rejecting_fungible_resource",
                            state.account_rejecting_fungible_resource.get()?,
                        )
                        .add(
                            "account_accepting_all_resources",
                            state.account_accepting_all_resources.get()?,
                        )
                        .add(
                            "account_rejecting_non_fungible_resource",
                            state.account_rejecting_non_fungible_resource.get()?,
                        )
                        .add("account_locker", state.account_locker.get()?)
                        .add(
                            "account_locker_badge",
                            state.account_locker_admin_badge.get()?,
                        )
                        .add("fungible_resource", state.fungible_resource.get()?)
                        .add("non_fungible_resource", state.non_fungible_resource.get()?),
                })
            })
    }
}
