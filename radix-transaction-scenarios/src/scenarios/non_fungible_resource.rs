use crate::internal_prelude::*;
use radix_engine::updates::ProtocolVersion;
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::*;

#[allow(deprecated)]
pub struct NonFungibleResourceScenarioConfig {
    pub main_account: PreallocatedAccount,
    pub occasional_recipient_account: PreallocatedAccount,
}

#[derive(Default)]
pub struct NonFungibleResourceScenarioState {
    pub integer_non_fungible_resource: Option<ResourceAddress>,
    pub string_non_fungible_resource: Option<ResourceAddress>,
    pub bytes_non_fungible_resource: Option<ResourceAddress>,
    pub ruid_non_fungible_resource: Option<ResourceAddress>,
    pub integer_non_fungible_resource_with_empty_data: Option<ResourceAddress>,
    pub integer_non_fungible_resource_with_metadata_standard_data: Option<ResourceAddress>,
    pub integer_non_fungible_resource_with_complex_data: Option<ResourceAddress>,
    pub vault1: Option<InternalAddress>,
}

impl Default for NonFungibleResourceScenarioConfig {
    fn default() -> Self {
        Self {
            main_account: secp256k1_account_1(),
            occasional_recipient_account: secp256k1_account_2(),
        }
    }
}

pub struct NonFungibleResourceScenarioCreator;

impl ScenarioCreator for NonFungibleResourceScenarioCreator {
    type Config = NonFungibleResourceScenarioConfig;
    type State = NonFungibleResourceScenarioState;
    type Instance = Scenario<Self::Config, Self::State>;

    const METADATA: ScenarioMetadata = ScenarioMetadata {
        logical_name: "non_fungible_resource",
        protocol_min_requirement: ProtocolVersion::Babylon,
        protocol_max_requirement: ProtocolVersion::LATEST,
        testnet_run_at: Some(ProtocolVersion::Babylon),
        safe_to_run_on_used_ledger: false,
    };

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Self::Instance {
        #[allow(unused_variables, deprecated)]
        ScenarioBuilder::new(core, Self::METADATA, config, start_state)
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-create",
                        |builder| {
                            let mut entries = BTreeMap::new();
                            entries.insert(NonFungibleLocalId::integer(1), NestedFungibleData {
                                a: 859,
                                b: vec!["hi".repeat(50)],
                                c: AnotherObject {
                                    f1: btreemap!(
                                        "key".to_string() => (77u8, (888u16, vec![vec![56u8; 3]]))
                                    )
                                }
                            });
                            builder
                                .create_non_fungible_resource(
                                    OwnerRole::None,
                                    NonFungibleIdType::Integer,
                                    false,
                                    NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                                    metadata! {},
                                    Some(entries),
                                )
                                .try_deposit_entire_worktop_or_abort(config.main_account.address, None)
                        },
                        vec![],
                    )
                },
                |core, config, state, result| {
                    state.integer_non_fungible_resource = Some(result.new_resource_addresses()[0]);
                    state.vault1 = Some(result.new_vault_addresses()[0]);
                    Ok(())
                },
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-create-string",
                        |builder| {
                            let mut entries = BTreeMap::new();
                            entries.insert(
                                NonFungibleLocalId::string("my_nft").unwrap(),
                                NestedFungibleData {
                                    a: 859,
                                    b: vec!["hi".repeat(50)],
                                    c: AnotherObject { f1: btreemap!() },
                                },
                            );
                            builder
                                .create_non_fungible_resource(
                                    OwnerRole::None,
                                    NonFungibleIdType::String,
                                    false,
                                    NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                                    metadata! {},
                                    Some(entries),
                                )
                                .try_deposit_entire_worktop_or_abort(config.main_account.address, None)
                        },
                        vec![],
                    )
                },
                |core, config, state, result| {
                    state.string_non_fungible_resource = Some(result.new_resource_addresses()[0]);
                    Ok(())
                },
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-create-bytes",
                        |builder| {
                            let mut entries = BTreeMap::new();
                            entries.insert(
                                NonFungibleLocalId::bytes(vec![0u8; 16]).unwrap(),
                                NestedFungibleData {
                                    a: 859,
                                    b: vec!["hi".repeat(50)],
                                    c: AnotherObject { f1: btreemap!() },
                                },
                            );
                            builder
                                .create_non_fungible_resource(
                                    OwnerRole::None,
                                    NonFungibleIdType::Bytes,
                                    false,
                                    NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                                    metadata! {},
                                    Some(entries),
                                )
                                .try_deposit_entire_worktop_or_abort(config.main_account.address, None)
                        },
                        vec![],
                    )
                },
                |core, config, state, result| {
                    state.bytes_non_fungible_resource = Some(result.new_resource_addresses()[0]);
                    Ok(())
                },
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-create-ruid",
                        |builder| {
                            let mut entries = Vec::new();
                            entries.push(NestedFungibleData {
                                a: 859,
                                b: vec!["hi".repeat(50)],
                                c: AnotherObject { f1: btreemap!() },
                            });
                            builder
                                .create_ruid_non_fungible_resource(
                                    OwnerRole::None,
                                    false,
                                    metadata! {},
                                    NonFungibleResourceRoles::single_locked_rule(rule!(allow_all)),
                                    Some(entries),
                                )
                                .try_deposit_entire_worktop_or_abort(config.main_account.address, None)
                        },
                        vec![],
                    )
                },
                |core, config, state, result| {
                    state.ruid_non_fungible_resource = Some(result.new_resource_addresses()[0]);
                    Ok(())
                },
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-mint-32-nfts",
                        |builder| {
                            let mut entries = BTreeMap::new();
                            for i in 100..132 {
                                entries.insert(
                                    NonFungibleLocalId::integer(i),
                                    NestedFungibleData {
                                        a: 859,
                                        b: vec!["hi".repeat(50)],
                                        c: AnotherObject { f1: btreemap!() },
                                    },
                                );
                            }
                            builder
                                .mint_non_fungible(
                                    state.integer_non_fungible_resource.unwrap(),
                                    entries,
                                )
                                .try_deposit_entire_worktop_or_abort(config.main_account.address, None)
                        },
                        vec![],
                    )
                },
                |core, config, state, result| Ok(()),
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-burn",
                        |builder| {
                            builder
                                .withdraw_from_account(
                                    config.main_account.address,
                                    state.integer_non_fungible_resource.unwrap(),
                                    2,
                                )
                                .burn_all_from_worktop(state.integer_non_fungible_resource.unwrap())
                                .withdraw_non_fungibles_from_account(
                                    config.main_account.address,
                                    state.integer_non_fungible_resource.unwrap(),
                                    [
                                        NonFungibleLocalId::integer(110),
                                    ],
                                )
                                .take_non_fungibles_from_worktop(
                                    state.integer_non_fungible_resource.unwrap(),
                                    [
                                        NonFungibleLocalId::integer(110),
                                    ],
                                    "non_fungibles_to_burn",
                                )
                                .burn_resource("non_fungibles_to_burn")
                        },
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| Ok(()),
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-transfer",
                        |builder| {
                            builder
                                .withdraw_from_account(
                                    config.main_account.address,
                                    state.integer_non_fungible_resource.unwrap(),
                                    dec!(1),
                                )
                                .try_deposit_entire_worktop_or_abort(
                                    config.occasional_recipient_account.address,
                                    None
                                )
                        },
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| Ok(()),
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-freeze-deposit",
                        |builder| builder.freeze_deposit(state.vault1.unwrap()),
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| Ok(()),
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-freeze-deposit",
                        |builder| builder.freeze_burn(state.vault1.unwrap()),
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| Ok(()),
            )
            .successful_transaction(|core, config, state| {
                core.next_transaction_with_faucet_lock_fee(
                    "non-fungible-resource-recall-frozen-vault",
                    |builder| {
                        builder
                            .recall_non_fungibles(
                                state.vault1.unwrap(),
                                [
                                    NonFungibleLocalId::integer(120)
                                ],
                            )
                            .try_deposit_entire_worktop_or_abort(config.occasional_recipient_account.address, None)
                    },
                    vec![&config.main_account.key],
                )
            })
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-unfreeze-withdraw",
                        |builder| builder.unfreeze_withdraw(state.vault1.unwrap()),
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| Ok(()),
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-unfreeze-deposit",
                        |builder| builder.unfreeze_deposit(state.vault1.unwrap()),
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| Ok(()),
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-unfreeze-burn",
                        |builder| builder.unfreeze_burn(state.vault1.unwrap()),
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| Ok(()),
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-recall-unfrozen-vault",
                        |builder| {
                            builder
                                .recall_non_fungibles(
                                    state.vault1.unwrap(),
                                    [
                                        NonFungibleLocalId::integer(130)
                                    ],
                                )
                                .try_deposit_entire_worktop_or_abort(config.main_account.address, None)
                        },
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| Ok(()),
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-create-resource-with-supply-with-empty-data",
                        |builder| {
                            builder
                                .create_non_fungible_resource(
                                    OwnerRole::None,
                                    NonFungibleIdType::Integer,
                                    true,
                                    NonFungibleResourceRoles::default(),
                                    metadata!(),
                                    Some(btreemap!(
                                        NonFungibleLocalId::integer(1) => (),
                                        NonFungibleLocalId::integer(2) => (),
                                        NonFungibleLocalId::integer(3) => (),
                                    )),
                                )
                                .try_deposit_entire_worktop_or_abort(config.main_account.address, None)
                        },
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| {
                    state.integer_non_fungible_resource_with_empty_data = Some(result.new_resource_addresses()[0]);
                    Ok(())
                },
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-create-resource-with-supply-with-metadata-standard-data",
                        |builder| {
                            builder
                                .create_non_fungible_resource(
                                    OwnerRole::None,
                                    NonFungibleIdType::Integer,
                                    true,
                                    NonFungibleResourceRoles::default(),
                                    // Images sourced from https://developers.radixdlt.com/ecosystem
                                    metadata!(
                                        init {
                                            "name" => "Radix - Defi Use-cases", locked;
                                            "description" => "[EXAMPLE] An example NF using the metadata standard", locked;
                                            "tags" => ["collection", "example-tag"], locked;
                                            "icon_url" => UncheckedUrl::of("https://assets-global.website-files.com/618962e5f285fb3c879d82ca/61aded05f3208c78b028bf99_Scrypto-Icon-Round%20(1).png"), locked;
                                            "info_url" => UncheckedUrl::of("https://developers.radixdlt.com/ecosystem"), locked;
                                        }
                                    ),
                                    Some(btreemap!(
                                        NonFungibleLocalId::integer(1) => MetadataStandardNonFungibleData {
                                            name: "Decentralized Exchanges".into(),
                                            description: "Decentralized Exchanges, also known as DEXes, allow users to exchange two or more tokens in a single transaction, at a certain price, without the need for another party to facilitate the transaction.\n\nDEXes allow users to trade assets without the need for a trusted third party. They are the first ever examples of a censorship resistant, non-custodial, and permissionless way of exchanging assets electronically.".into(),
                                            key_image_url: UncheckedUrl::of("https://assets-global.website-files.com/618962e5f285fb3c879d82ca/61b8f414d213fd7349b654b9_icon-DEX.svg"),
                                            arbitrary_coolness_rating: 45,
                                        },
                                        NonFungibleLocalId::integer(2) => MetadataStandardNonFungibleData {
                                            name: "Stablecoins".into(),
                                            description: "A stablecoin is a class of cryptocurrencies that attempt to offer price stability and are backed by a reserve asset. Stablecoins have become popular as they provide the instant processing and security or privacy of cryptocurrencies' payments along with the volatility-free stable valuations of fiat currencies.\n\nStablecoins base their market value on an external reference. This could be a currency like the U.S. dollar or a commodity's price such as gold. Stablecoins achieve price stability via collateralization or algorithmic buying and selling of the reference asset or its derivatives.".into(),
                                            key_image_url: UncheckedUrl::of("https://assets-global.website-files.com/618962e5f285fb3c879d82ca/61b8f419f77f05b9086565e6_icon-stablecoin.svg"),
                                            arbitrary_coolness_rating: 5,
                                        },
                                        NonFungibleLocalId::integer(3) => MetadataStandardNonFungibleData {
                                            name: "Lending".into(),
                                            description: "In lending, borrowers pledge their crypto assets as collateral and avail loans in stablecoins or other crypto assets as a means of financing.\n\nHowever, unlike TradFi, DeFi lending uses algorithmic systems where lending and borrowing rates are determined automatically based on each asset's real-time supply and demand.\n\nThis automated approach means more flexibility and access for anyone looking for financing.".into(),
                                            key_image_url: UncheckedUrl::of("https://assets-global.website-files.com/618962e5f285fb3c879d82ca/61b8f414979a8b045995bf75_ICON-LENDING.svg"),
                                            arbitrary_coolness_rating: 30,
                                        },
                                        NonFungibleLocalId::integer(4) => MetadataStandardNonFungibleData {
                                            name: "Insurance".into(),
                                            description: "Insurance protocols provide cover against smart contract failure & exchange hacks using a decentralized protocol, so people can share risk without needing an insurance company.".into(),
                                            key_image_url: UncheckedUrl::of("https://assets-global.website-files.com/618962e5f285fb3c879d82ca/61b8f4146c4528458c93da96_ICON-NFTS-1.svg"),
                                            arbitrary_coolness_rating: 0,
                                        },
                                        NonFungibleLocalId::integer(5) => MetadataStandardNonFungibleData {
                                            name: "Futures, Options, & Derivatives".into(),
                                            description: "A futures contract is an arrangement between two users on an exchange to buy and sell an underlying crypto asset at an agreed-upon price on a certain date in the future.\n\nOptions give the holder the right to buy (call) or sell (put) an underlying crypto asset at a set date without being obligated.\n\nSynthetic assets, also known as synths, are an asset class formed by combining cryptocurrencies and traditional derivative assets, making them tokenized derivatives.".into(),
                                            key_image_url: UncheckedUrl::of("https://assets-global.website-files.com/618962e5f285fb3c879d82ca/61b8f414e8f31b971debfabe_ICON-FUTURES.svg"),
                                            arbitrary_coolness_rating: 10,
                                        },
                                        // Skipping 6 and 7 because they don't need to be complete
                                        NonFungibleLocalId::integer(8) => MetadataStandardNonFungibleData {
                                            name: "Gaming".into(),
                                            description: "Since we have so many gamers who spend endless hours and money on gaming platforms, DeFi gaming platforms will enable them to monetize their time and progress. The play-to-earn games offer the best of both worlds - they provide an entertaining experience and make playing games lucrative.".into(),
                                            key_image_url: UncheckedUrl::of("https://assets-global.website-files.com/618962e5f285fb3c879d82ca/61b8f416931770a50ed4a702_ICON-GAMING.svg"),
                                            arbitrary_coolness_rating: 71,
                                        },
                                    )),
                                )
                                .try_deposit_entire_worktop_or_abort(config.main_account.address, None)
                        },
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| {
                    state.integer_non_fungible_resource_with_metadata_standard_data = Some(result.new_resource_addresses()[0]);
                    Ok(())
                },
            )
            .successful_transaction(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-transfer-metadata-standard-nfs",
                        |builder| {
                            builder
                                .withdraw_non_fungibles_from_account(
                                    config.main_account.address,
                                    state.integer_non_fungible_resource_with_metadata_standard_data.unwrap(),
                                    [
                                        NonFungibleLocalId::integer(4),
                                        NonFungibleLocalId::integer(8),
                                    ]
                                )
                                .try_deposit_entire_worktop_or_abort(config.occasional_recipient_account.address, None)
                        },
                        vec![&config.main_account.key],
                    )
                },
            )
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-create-resource-with-supply-with-complex-data",
                        |builder| {
                            builder
                                .create_non_fungible_resource(
                                    // Make the Owner the same as the public key for the main account
                                    // In reality, this should probably be a concrete badge, not a virtual badge, but for tests
                                    // this is okay
                                    OwnerRole::Fixed(rule!(require(signature(&config.main_account.public_key)))),
                                    NonFungibleIdType::Integer,
                                    true,
                                    NonFungibleResourceRoles {
                                        non_fungible_data_update_roles: Some(NonFungibleDataUpdateRoles {
                                            // Using "None" here makes it the owner role
                                            non_fungible_data_updater: None,
                                            non_fungible_data_updater_updater: None,
                                        }),
                                        ..NonFungibleResourceRoles::default()
                                    },
                                    metadata!(),
                                    Some(btreemap!(
                                        NonFungibleLocalId::integer(1) => ComplexNonFungibleData {
                                            fixed_number: 100,
                                            fixed_non_fungible_global_id: NonFungibleGlobalId::new(
                                                state.integer_non_fungible_resource_with_metadata_standard_data.unwrap(),
                                                NonFungibleLocalId::integer(8)
                                            ),
                                            mutable_long_name_for_data_to_try_and_stretch_the_bounds_of_what_is_possible_in_user_interfaces: "Some string which could be made long for test cases".to_string(),
                                            inner_struct: InnerStruct { byte: 42u8, string: Some("Hello world!".into()) },
                                            mutable_inner_enum: InnerEnum::InnerEnum(Box::new(InnerEnum::None))
                                        },
                                    )),
                                )
                                .try_deposit_entire_worktop_or_abort(config.main_account.address, None)
                        },
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| {
                    state.integer_non_fungible_resource_with_complex_data = Some(result.new_resource_addresses()[0]);
                    Ok(())
                },
            )
            .successful_transaction(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-mutate-data",
                        |builder| {
                            let mut really_long_string = String::new();
                            for _ in 0..80 {
                                really_long_string.push_str("This is a long string with repeats of length 50!! ");
                            }
                            assert_eq!(really_long_string.len(), 4000);
                            builder
                                .update_non_fungible_data(
                                    state.integer_non_fungible_resource_with_complex_data.unwrap(),
                                    NonFungibleLocalId::integer(1),
                                    "mutable_long_name_for_data_to_try_and_stretch_the_bounds_of_what_is_possible_in_user_interfaces",
                                    really_long_string,
                                )
                                .update_non_fungible_data(
                                    state.integer_non_fungible_resource_with_complex_data.unwrap(),
                                    NonFungibleLocalId::integer(1),
                                    "mutable_inner_enum",
                                    InnerEnum::InnerEnum(Box::new(InnerEnum::InnerStruct(Box::new(
                                        InnerStruct {
                                            byte: 101u8,
                                            string: None,
                                        }
                                    )))),
                                )
                        },
                        // The owner of the resource is this key
                        vec![&config.main_account.key],
                    )
                },
            )
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("main_account", config.main_account.address)
                        .add(
                            "occasional_recipient_account",
                            config.occasional_recipient_account.address,
                        )
                        .add(
                            "integer_non_fungible_resource",
                            state.integer_non_fungible_resource.unwrap(),
                        )
                        .add(
                            "string_non_fungible_resource",
                            state.string_non_fungible_resource.unwrap(),
                        )
                        .add(
                            "bytes_non_fungible_resource",
                            state.bytes_non_fungible_resource.unwrap(),
                        )
                        .add(
                            "ruid_non_fungible_resource",
                            state.ruid_non_fungible_resource.unwrap(),
                        )
                        .add("non_fungible_vault", state.vault1.unwrap())
                        .add(
                            "integer_non_fungible_resource_with_empty_data",
                            state.integer_non_fungible_resource_with_empty_data.unwrap(),
                        )
                        .add(
                            "integer_non_fungible_resource_with_metadata_standard_data",
                            state.integer_non_fungible_resource_with_metadata_standard_data.unwrap(),
                        )
                        .add(
                            "integer_non_fungible_resource_with_complex_data",
                            state.integer_non_fungible_resource_with_complex_data.unwrap(),
                        ),
                })
            })
    }
}

#[derive(ScryptoSbor, ManifestSbor)]
struct NestedFungibleData {
    a: u32,
    b: Vec<String>,
    c: AnotherObject,
}

#[derive(ScryptoSbor, ManifestSbor)]
struct AnotherObject {
    f1: BTreeMap<String, (u8, (u16, Vec<Vec<u8>>))>,
}

impl NonFungibleData for NestedFungibleData {
    const MUTABLE_FIELDS: &'static [&'static str] = &["a", "c"];
}

#[derive(ScryptoSbor, ManifestSbor)]
pub struct MetadataStandardNonFungibleData {
    pub name: String,
    pub description: String,
    pub key_image_url: UncheckedUrl,
    // Additional fields
    pub arbitrary_coolness_rating: u64,
}

impl NonFungibleData for MetadataStandardNonFungibleData {
    const MUTABLE_FIELDS: &'static [&'static str] = &[];
}

#[derive(ScryptoSbor, ManifestSbor)]
pub struct ComplexNonFungibleData {
    pub fixed_number: u64,
    pub fixed_non_fungible_global_id: NonFungibleGlobalId,
    pub mutable_long_name_for_data_to_try_and_stretch_the_bounds_of_what_is_possible_in_user_interfaces:
        String,
    pub inner_struct: InnerStruct,
    pub mutable_inner_enum: InnerEnum,
}

impl NonFungibleData for ComplexNonFungibleData {
    const MUTABLE_FIELDS: &'static [&'static str] = &["mutable_long_name_for_data_to_try_and_stretch_the_bounds_of_what_is_possible_in_user_interfaces", "mutable_inner_enum"];
}

#[derive(ScryptoSbor, ManifestSbor)]
pub struct InnerStruct {
    pub byte: u8,
    pub string: Option<String>,
}

#[derive(ScryptoSbor, ManifestSbor)]
pub enum InnerEnum {
    None,
    InnerEnum(Box<InnerEnum>),
    InnerStruct(Box<InnerStruct>),
}
