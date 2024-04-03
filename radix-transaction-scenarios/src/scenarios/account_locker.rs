use crate::internal_prelude::*;
use radix_engine_interface::{
    blueprints::account::{
        AccountSetResourcePreferenceInput, ResourcePreference,
        ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT,
    },
    *,
};

pub struct AccountLockerScenarioConfig {
    pub account_locker_admin_account: VirtualAccount,
    pub user_account1: VirtualAccount,
    pub user_account2: VirtualAccount,
    pub user_account3: VirtualAccount,
}

#[derive(Default)]
pub struct AccountLockerScenarioState {
    pub(crate) account_locker: State<ComponentAddress>,
    pub(crate) account_locker_admin_badge: State<ResourceAddress>,

    pub(crate) fungible_resource: State<ResourceAddress>,
    pub(crate) non_fungible_resource: State<ResourceAddress>,
}

impl Default for AccountLockerScenarioConfig {
    fn default() -> Self {
        Self {
            account_locker_admin_account: ed25519_account_for_private_key(1),
            user_account1: ed25519_account_for_private_key(2),
            user_account2: ed25519_account_for_private_key(3),
            user_account3: ed25519_account_for_private_key(4),
        }
    }
}

pub struct AccountLockerScenarioCreator;

impl ScenarioCreator for AccountLockerScenarioCreator {
    type Config = AccountLockerScenarioConfig;

    type State = AccountLockerScenarioState;

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Box<dyn ScenarioInstance> {
        let metadata = ScenarioMetadata {
            logical_name: "account_locker",
        };

        ScenarioBuilder::new(core, metadata, config, start_state)
            .successful_transaction_with_result_handler(
                |core, config, _| {
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
                                    config.account_locker_admin_account.address,
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
                                config.user_account1.address,
                                ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT,
                                AccountSetResourcePreferenceInput {
                                    resource_address: state.fungible_resource.get().unwrap(),
                                    resource_preference: ResourcePreference::Disallowed,
                                },
                            )
                            .call_method(
                                config.user_account3.address,
                                ACCOUNT_SET_RESOURCE_PREFERENCE_IDENT,
                                AccountSetResourcePreferenceInput {
                                    resource_address: state.non_fungible_resource.get().unwrap(),
                                    resource_preference: ResourcePreference::Disallowed,
                                },
                            )
                    },
                    vec![&config.user_account1.key, &config.user_account3.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-send-fungibles-and-try-direct-deposit-succeeds",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .mint_fungible(state.fungible_resource.get().unwrap(), dec!(100))
                            .take_all_from_worktop(state.fungible_resource.get().unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.get().unwrap(),
                                    ACCOUNT_LOCKER_STORE_IDENT,
                                    AccountLockerStoreManifestInput {
                                        bucket,
                                        claimant: config.user_account2.address,
                                        try_direct_send: true,
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-send-fungibles-and-try-direct-deposit-refunds",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .mint_fungible(state.fungible_resource.get().unwrap(), dec!(100))
                            .take_all_from_worktop(state.fungible_resource.get().unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.get().unwrap(),
                                    ACCOUNT_LOCKER_STORE_IDENT,
                                    AccountLockerStoreManifestInput {
                                        bucket,
                                        claimant: config.user_account1.address,
                                        try_direct_send: true,
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-send-fungibles-and-dont-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .mint_fungible(state.fungible_resource.get().unwrap(), dec!(100))
                            .take_all_from_worktop(state.fungible_resource.get().unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.get().unwrap(),
                                    ACCOUNT_LOCKER_STORE_IDENT,
                                    AccountLockerStoreManifestInput {
                                        bucket,
                                        claimant: config.user_account2.address,
                                        try_direct_send: false,
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-airdrop-fungibles-and-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .mint_fungible(state.fungible_resource.get().unwrap(), dec!(300))
                            .take_all_from_worktop(state.fungible_resource.get().unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.get().unwrap(),
                                    ACCOUNT_LOCKER_AIRDROP_IDENT,
                                    AccountLockerAirdropManifestInput {
                                        bucket,
                                        try_direct_send: true,
                                        claimants: [
                                            &config.user_account1,
                                            &config.user_account2,
                                            &config.user_account3,
                                        ]
                                        .into_iter()
                                        .map(|account| {
                                            (
                                                account.address,
                                                ResourceSpecifier::Fungible(dec!(100)),
                                            )
                                        })
                                        .collect(),
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-airdrop-fungibles-and-dont-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .mint_fungible(state.fungible_resource.get().unwrap(), dec!(300))
                            .take_all_from_worktop(state.fungible_resource.get().unwrap(), "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.get().unwrap(),
                                    ACCOUNT_LOCKER_AIRDROP_IDENT,
                                    AccountLockerAirdropManifestInput {
                                        bucket,
                                        try_direct_send: false,
                                        claimants: [
                                            &config.user_account1,
                                            &config.user_account2,
                                            &config.user_account3,
                                        ]
                                        .into_iter()
                                        .map(|account| {
                                            (
                                                account.address,
                                                ResourceSpecifier::Fungible(dec!(100)),
                                            )
                                        })
                                        .collect(),
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-send-non-fungibles-and-try-direct-deposit-succeeds",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .mint_non_fungible(
                                state.non_fungible_resource.get().unwrap(),
                                [(NonFungibleLocalId::integer(1), ())],
                            )
                            .take_all_from_worktop(
                                state.non_fungible_resource.get().unwrap(),
                                "bucket",
                            )
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.get().unwrap(),
                                    ACCOUNT_LOCKER_STORE_IDENT,
                                    AccountLockerStoreManifestInput {
                                        bucket,
                                        claimant: config.user_account2.address,
                                        try_direct_send: true,
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-send-non-fungibles-and-try-direct-deposit-refunds",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .mint_non_fungible(
                                state.non_fungible_resource.get().unwrap(),
                                [(NonFungibleLocalId::integer(2), ())],
                            )
                            .take_all_from_worktop(
                                state.non_fungible_resource.get().unwrap(),
                                "bucket",
                            )
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.get().unwrap(),
                                    ACCOUNT_LOCKER_STORE_IDENT,
                                    AccountLockerStoreManifestInput {
                                        bucket,
                                        claimant: config.user_account1.address,
                                        try_direct_send: true,
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-send-non-fungibles-and-dont-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .mint_non_fungible(
                                state.non_fungible_resource.get().unwrap(),
                                [(NonFungibleLocalId::integer(3), ())],
                            )
                            .take_all_from_worktop(
                                state.non_fungible_resource.get().unwrap(),
                                "bucket",
                            )
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.get().unwrap(),
                                    ACCOUNT_LOCKER_STORE_IDENT,
                                    AccountLockerStoreManifestInput {
                                        bucket,
                                        claimant: config.user_account2.address,
                                        try_direct_send: false,
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-airdrop-non-fungibles-by-amount-and-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .mint_non_fungible(
                                state.non_fungible_resource.get().unwrap(),
                                [
                                    (NonFungibleLocalId::integer(4), ()),
                                    (NonFungibleLocalId::integer(5), ()),
                                    (NonFungibleLocalId::integer(6), ()),
                                ],
                            )
                            .take_all_from_worktop(
                                state.non_fungible_resource.get().unwrap(),
                                "bucket",
                            )
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.get().unwrap(),
                                    ACCOUNT_LOCKER_AIRDROP_IDENT,
                                    AccountLockerAirdropManifestInput {
                                        bucket,
                                        try_direct_send: true,
                                        claimants: [
                                            &config.user_account1,
                                            &config.user_account2,
                                            &config.user_account3,
                                        ]
                                        .into_iter()
                                        .map(|account| {
                                            (account.address, ResourceSpecifier::Fungible(dec!(1)))
                                        })
                                        .collect(),
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-airdrop-non-fungibles-by-amount-and-dont-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .mint_non_fungible(
                                state.non_fungible_resource.get().unwrap(),
                                [
                                    (NonFungibleLocalId::integer(7), ()),
                                    (NonFungibleLocalId::integer(8), ()),
                                    (NonFungibleLocalId::integer(9), ()),
                                ],
                            )
                            .take_all_from_worktop(
                                state.non_fungible_resource.get().unwrap(),
                                "bucket",
                            )
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.get().unwrap(),
                                    ACCOUNT_LOCKER_AIRDROP_IDENT,
                                    AccountLockerAirdropManifestInput {
                                        bucket,
                                        try_direct_send: false,
                                        claimants: [
                                            &config.user_account1,
                                            &config.user_account2,
                                            &config.user_account3,
                                        ]
                                        .into_iter()
                                        .map(|account| {
                                            (account.address, ResourceSpecifier::Fungible(dec!(1)))
                                        })
                                        .collect(),
                                    },
                                )
                            })
                    },
                    vec![&config.account_locker_admin_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-airdrop-non-fungibles-by-ids-and-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .mint_non_fungible(
                                state.non_fungible_resource.get().unwrap(),
                                [
                                    (NonFungibleLocalId::integer(10), ()),
                                    (NonFungibleLocalId::integer(11), ()),
                                    (NonFungibleLocalId::integer(12), ()),
                                ],
                            )
                            .take_all_from_worktop(
                                state.non_fungible_resource.get().unwrap(),
                                "bucket",
                            )
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.get().unwrap(),
                                    ACCOUNT_LOCKER_AIRDROP_IDENT,
                                    AccountLockerAirdropManifestInput {
                                        bucket,
                                        try_direct_send: true,
                                        claimants: [
                                            (&config.user_account1, 10),
                                            (&config.user_account2, 11),
                                            (&config.user_account3, 12),
                                        ]
                                        .into_iter()
                                        .map(|(account, id)| {
                                            (
                                                account.address,
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
                    vec![&config.account_locker_admin_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-airdrop-non-fungibles-by-ids-and-dont-try-direct-deposit",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .mint_non_fungible(
                                state.non_fungible_resource.get().unwrap(),
                                [
                                    (NonFungibleLocalId::integer(13), ()),
                                    (NonFungibleLocalId::integer(14), ()),
                                    (NonFungibleLocalId::integer(15), ()),
                                ],
                            )
                            .take_all_from_worktop(
                                state.non_fungible_resource.get().unwrap(),
                                "bucket",
                            )
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    state.account_locker.get().unwrap(),
                                    ACCOUNT_LOCKER_AIRDROP_IDENT,
                                    AccountLockerAirdropManifestInput {
                                        bucket,
                                        try_direct_send: false,
                                        claimants: [
                                            (&config.user_account1, 13),
                                            (&config.user_account2, 14),
                                            (&config.user_account3, 15),
                                        ]
                                        .into_iter()
                                        .map(|(account, id)| {
                                            (
                                                account.address,
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
                    vec![&config.account_locker_admin_account.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-claim-fungibles-by-amount",
                    |builder| {
                        builder
                            .call_method(
                                state.account_locker.get().unwrap(),
                                ACCOUNT_LOCKER_CLAIM_IDENT,
                                AccountLockerClaimManifestInput {
                                    claimant: config.user_account1.address,
                                    amount: dec!(1),
                                    resource_address: state.fungible_resource.get().unwrap(),
                                },
                            )
                            .deposit_batch(config.user_account1.address)
                    },
                    vec![&config.user_account1.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-claim-non-fungibles-by-amount",
                    |builder| {
                        builder
                            .call_method(
                                state.account_locker.get().unwrap(),
                                ACCOUNT_LOCKER_CLAIM_IDENT,
                                AccountLockerClaimManifestInput {
                                    claimant: config.user_account1.address,
                                    amount: dec!(1),
                                    resource_address: state.non_fungible_resource.get().unwrap(),
                                },
                            )
                            .deposit_batch(config.user_account1.address)
                    },
                    vec![&config.user_account1.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-claim-non-fungibles-by-ids",
                    |builder| {
                        builder
                            .call_method(
                                state.account_locker.get().unwrap(),
                                ACCOUNT_LOCKER_CLAIM_NON_FUNGIBLES_IDENT,
                                AccountLockerClaimNonFungiblesManifestInput {
                                    claimant: config.user_account2.address,
                                    resource_address: state.non_fungible_resource.get().unwrap(),
                                    ids: indexset![NonFungibleLocalId::integer(3)],
                                },
                            )
                            .deposit_batch(config.user_account2.address)
                    },
                    vec![&config.user_account2.key],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-recover-fungibles-by-amount",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .call_method(
                                state.account_locker.get().unwrap(),
                                ACCOUNT_LOCKER_RECOVER_IDENT,
                                AccountLockerRecoverManifestInput {
                                    claimant: config.user_account1.address,
                                    amount: dec!(1),
                                    resource_address: state.fungible_resource.get().unwrap(),
                                },
                            )
                            .deposit_batch(config.user_account1.address)
                    },
                    vec![
                        &config.account_locker_admin_account.key,
                        &config.user_account1.key,
                    ],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-recover-non-fungibles-by-amount",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .call_method(
                                state.account_locker.get().unwrap(),
                                ACCOUNT_LOCKER_RECOVER_IDENT,
                                AccountLockerRecoverManifestInput {
                                    claimant: config.user_account1.address,
                                    amount: dec!(1),
                                    resource_address: state.non_fungible_resource.get().unwrap(),
                                },
                            )
                            .deposit_batch(config.user_account1.address)
                    },
                    vec![
                        &config.account_locker_admin_account.key,
                        &config.user_account1.key,
                    ],
                )
            })
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "account-locker-recover-non-fungibles-by-ids",
                    |builder| {
                        builder
                            .create_proof_from_account_of_amount(
                                config.account_locker_admin_account.address,
                                state.account_locker_admin_badge.get().unwrap(),
                                dec!(1),
                            )
                            .call_method(
                                state.account_locker.get().unwrap(),
                                ACCOUNT_LOCKER_RECOVER_NON_FUNGIBLES_IDENT,
                                AccountLockerRecoverNonFungiblesManifestInput {
                                    claimant: config.user_account3.address,
                                    resource_address: state.non_fungible_resource.get().unwrap(),
                                    ids: indexset![NonFungibleLocalId::integer(15)],
                                },
                            )
                            .deposit_batch(config.user_account3.address)
                    },
                    vec![
                        &config.account_locker_admin_account.key,
                        &config.user_account3.key,
                    ],
                )
            })
            .finalize(|_, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add(
                            "badge_holder_account",
                            config.account_locker_admin_account.address,
                        )
                        .add("user_account1", config.user_account1.address)
                        .add("user_account2", config.user_account2.address)
                        .add("user_account3", config.user_account3.address)
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
