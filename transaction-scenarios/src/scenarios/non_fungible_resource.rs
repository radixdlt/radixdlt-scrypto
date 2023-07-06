use crate::internal_prelude::*;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::*;

pub struct NonFungibleResourceScenarioConfig {
    pub main_account: VirtualAccount,
    pub occasional_recipient_account: VirtualAccount,
}

#[derive(Default)]
pub struct NonFungibleResourceScenarioState {
    pub integer_non_fungible_resource: Option<ResourceAddress>,
    pub string_non_fungible_resource: Option<ResourceAddress>,
    pub bytes_non_fungible_resource: Option<ResourceAddress>,
    pub ruid_non_fungible_resource: Option<ResourceAddress>,
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

    fn create_with_config_and_state(
        core: ScenarioCore,
        config: Self::Config,
        start_state: Self::State,
    ) -> Box<dyn ScenarioInstance> {
        let metadata = ScenarioMetadata {
            logical_name: "non_fungible_resource",
        };

        #[allow(unused_variables)]
        ScenarioBuilder::new(core, metadata, config, start_state)
            .successful_transaction_with_result_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-create",
                        |builder| {
                            let mut entries = BTreeMap::new();
                            entries.insert(NonFungibleLocalId::integer(1), ComplexFungibleData {
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
                                .try_deposit_batch_or_abort(config.main_account.address)
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
                                ComplexFungibleData {
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
                                .try_deposit_batch_or_abort(config.main_account.address)
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
                                ComplexFungibleData {
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
                                .try_deposit_batch_or_abort(config.main_account.address)
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
                            entries.push(ComplexFungibleData {
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
                                .try_deposit_batch_or_abort(config.main_account.address)
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
                                    ComplexFungibleData {
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
                                .try_deposit_batch_or_abort(config.main_account.address)
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
                                    &btreeset!(NonFungibleLocalId::integer(110)),
                                )
                                .take_non_fungibles_from_worktop(
                                    state.integer_non_fungible_resource.unwrap(),
                                    &btreeset!(NonFungibleLocalId::integer(110)),
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
                                    dec!("1"),
                                )
                                .try_deposit_batch_or_abort(
                                    config.occasional_recipient_account.address,
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
            .failed_transaction_with_error_handler(
                |core, config, state| {
                    core.next_transaction_with_faucet_lock_fee(
                        "non-fungible-resource-recall-frozen-vault",
                        |builder| {
                            builder
                                .recall_non_fungibles(
                                    state.vault1.unwrap(),
                                    &btreeset!(NonFungibleLocalId::integer(120)),
                                )
                                .try_deposit_batch_or_abort(config.main_account.address)
                        },
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| {
                    // FIXME: Recalling from frozen vaults should be allowed per product requirement
                    Ok(())
                },
            )
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
                                    &btreeset!(NonFungibleLocalId::integer(130)),
                                )
                                .try_deposit_batch_or_abort(config.main_account.address)
                        },
                        vec![&config.main_account.key],
                    )
                },
                |core, config, state, result| Ok(()),
            )
            .finalize(|core, config, state| {
                Ok(ScenarioOutput {
                    interesting_addresses: DescribedAddresses::new()
                        .add("main_account", config.main_account.address.clone())
                        .add(
                            "occasional_recipient_account",
                            config.occasional_recipient_account.address.clone(),
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
                        .add("non_fungible_vault", state.vault1.unwrap()),
                })
            })
    }
}

#[derive(ScryptoSbor, ManifestSbor)]
struct ComplexFungibleData {
    a: u32,
    b: Vec<String>,
    c: AnotherObject,
}

#[derive(ScryptoSbor, ManifestSbor)]
struct AnotherObject {
    f1: BTreeMap<String, (u8, (u16, Vec<Vec<u8>>))>,
}

impl NonFungibleData for ComplexFungibleData {
    const MUTABLE_FIELDS: &'static [&'static str] = &["a", "c"];
}
