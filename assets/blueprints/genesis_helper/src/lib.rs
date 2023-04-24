use scrypto::api::node_modules::metadata::MetadataValue;
use scrypto::blueprints::account::AccountDepositInput;
use scrypto::blueprints::epoch_manager::*;
use scrypto::prelude::*;

// Important: the types defined here must match those in bootstrap.rs
type AccountIdx = usize;
type ResourceIdx = usize;
type ValidatorIdx = usize;

// This data represents data from Olympia Network state
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisData {
    validators: Vec<GenesisValidator>,
    resources: Vec<GenesisResource>,
    accounts: Vec<ComponentAddress>,
    resource_balances: BTreeMap<ResourceIdx, Vec<(AccountIdx, Decimal)>>,
    xrd_balances: BTreeMap<AccountIdx, Decimal>,
    stakes: BTreeMap<ValidatorIdx, Vec<(AccountIdx, Decimal)>>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisValidator {
    pub key: EcdsaSecp256k1PublicKey,
    pub component_address: ComponentAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisResource {
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon_url: String,
    pub address_bytes: [u8; NodeId::LENGTH],
    pub owner_with_mint_and_burn_rights: Option<AccountIdx>,
}

#[blueprint]
mod genesis_helper {
    struct GenesisHelper;

    impl GenesisHelper {
        pub fn init(
            mut genesis_data: GenesisData,
            mut whole_lotta_xrd: Bucket,
            validator_owner_token: [u8; NodeId::LENGTH], // TODO: Clean this up
            epoch_manager_component_address: [u8; NodeId::LENGTH], // TODO: Clean this up
            initial_epoch: u64,
            max_validators: u32,
            rounds_per_epoch: u64,
            num_unstake_epochs: u64,
        ) -> Bucket {
            // Create the resources
            for (resource_idx, resource) in genesis_data.resources.into_iter().enumerate() {
                let mut initial_supply = Decimal::ZERO;
                let mut initial_allocation = BTreeMap::new();
                for (account_idx, amount) in genesis_data
                    .resource_balances
                    .remove(&resource_idx)
                    .unwrap_or(vec![])
                {
                    // TODO: check for/handle overflows
                    initial_supply += amount;
                    let account_component_address = genesis_data.accounts[account_idx].clone();
                    initial_allocation.insert(account_component_address, amount);
                }
                let owner = resource
                    .owner_with_mint_and_burn_rights
                    .map(|idx| genesis_data.accounts[idx].clone());
                Self::create_resource(resource, initial_supply, initial_allocation, owner);
            }

            // Create the epoch manager with initial validator set...
            let mut validators_with_initial_stake = vec![];
            for (validator_idx, validator) in genesis_data.validators.into_iter().enumerate() {
                let initial_stake_amount = genesis_data
                    .stakes
                    .get(&validator_idx)
                    .map(|stakes| {
                        stakes
                            .iter()
                            .map(|(_, xrd_amount)| xrd_amount)
                            .cloned()
                            .sum()
                    })
                    .unwrap_or(Decimal::ZERO);
                let initial_stake_bucket = whole_lotta_xrd.take(initial_stake_amount);
                validators_with_initial_stake.push((
                    validator.key,
                    validator.component_address,
                    initial_stake_bucket,
                ));
            }

            let lp_buckets: Vec<Bucket> = Runtime::call_function(
                EPOCH_MANAGER_PACKAGE,
                EPOCH_MANAGER_BLUEPRINT,
                EPOCH_MANAGER_CREATE_IDENT,
                scrypto_encode(&EpochManagerCreateInput {
                    validator_owner_token,
                    component_address: epoch_manager_component_address,
                    validator_set: validators_with_initial_stake,
                    initial_epoch,
                    max_validators,
                    rounds_per_epoch,
                    num_unstake_epochs,
                })
                .unwrap(),
            );

            // ...and distribute the LP tokens to stakers
            for (validator_idx, mut lp_bucket) in lp_buckets.into_iter().enumerate() {
                let stakes = genesis_data.stakes.remove(&validator_idx).unwrap_or(vec![]);
                for (account_idx, stake_xrd_amount) in stakes {
                    // TODO: currently xrd amount matches stake tokens amount, but can this change later on?
                    let stake_bucket = lp_bucket.take(stake_xrd_amount);
                    let account_address = genesis_data.accounts[account_idx];
                    let _: () = Runtime::call_method(
                        account_address,
                        "deposit",
                        scrypto_encode(&AccountDepositInput {
                            bucket: stake_bucket,
                        })
                        .unwrap(),
                    );
                }
                lp_bucket.drop_empty();
            }

            // Allocate XRD
            for (account_idx, xrd_amount) in genesis_data.xrd_balances.into_iter() {
                let account_address = genesis_data.accounts[account_idx];
                let bucket = whole_lotta_xrd.take(xrd_amount);
                let _: () = Runtime::call_method(
                    account_address,
                    "deposit",
                    scrypto_encode(&AccountDepositInput { bucket }).unwrap(),
                );
            }

            // return the remainder
            whole_lotta_xrd
        }

        fn create_resource(
            resource: GenesisResource,
            initial_supply: Decimal,
            initial_allocation: BTreeMap<ComponentAddress, Decimal>,
            owner_with_mint_and_burn_rights: Option<ComponentAddress>,
        ) -> () {
            // Just a sanity check that XRD wasn't acccidentally included in genesis resources
            if resource.symbol.eq_ignore_ascii_case("XRD") {
                panic!("XRD shouldn't be included in genesis resources");
            }

            let mut metadata = BTreeMap::new();
            metadata.insert("symbol".to_owned(), resource.symbol.clone());
            metadata.insert("name".to_owned(), resource.name);
            metadata.insert("description".to_owned(), resource.description);

            // TODO: Use url metadata type
            metadata.insert("url".to_owned(), resource.url);
            metadata.insert("icon_url".to_owned(), resource.icon_url);

            let mut access_rules = BTreeMap::new();
            access_rules.insert(Deposit, (rule!(allow_all), rule!(deny_all)));
            access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));

            if let Some(owner) = owner_with_mint_and_burn_rights {
                // TODO: Should we use securify style non fungible resource for the owner badge?:167
                // Note that we also set "tags" metadata later on
                let owner_badge = ResourceBuilder::new_fungible()
                    .divisibility(DIVISIBILITY_NONE)
                    .metadata(
                        "name",
                        format!("Resource Owner Badge ({})", resource.symbol),
                    )
                    .mint_initial_supply(1);

                access_rules.insert(
                    Mint,
                    (
                        rule!(require(owner_badge.resource_address())),
                        rule!(deny_all),
                    ),
                );
                access_rules.insert(
                    Burn,
                    (
                        rule!(require(owner_badge.resource_address())),
                        rule!(deny_all),
                    ),
                );
                access_rules.insert(
                    UpdateMetadata,
                    (
                        rule!(require(owner_badge.resource_address())),
                        rule!(deny_all),
                    ),
                );

                let _: () = Runtime::call_method(
                    owner,
                    "deposit",
                    scrypto_encode(&AccountDepositInput {
                        bucket: owner_badge,
                    })
                    .unwrap(),
                );
            }

            let (resource_address, mut bucket): (ResourceAddress, Bucket) = Runtime::call_function(
                RESOURCE_MANAGER_PACKAGE,
                FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT,
                scrypto_encode(
                    &FungibleResourceManagerCreateWithInitialSupplyAndAddressInput {
                        divisibility: 18,
                        metadata,
                        access_rules,
                        initial_supply,
                        resource_address: resource.address_bytes,
                    },
                )
                .unwrap(),
            );

            for (account_address, amount) in initial_allocation {
                let allocation_bucket = bucket.take(amount);
                let _: () = Runtime::call_method(
                    account_address,
                    "deposit",
                    scrypto_encode(&AccountDepositInput {
                        bucket: allocation_bucket,
                    })
                    .unwrap(),
                );
            }
            bucket.drop_empty();

            let address: GlobalAddress = resource_address.into();

            let metadata = borrow_resource_manager!(resource_address).metadata();
            metadata.set("owner_of", address);
            metadata.set_list("tags", vec![MetadataValue::String("badge".to_string())]);
        }
    }
}
