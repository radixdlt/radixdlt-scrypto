use scrypto::api::node_modules::metadata::MetadataValue;
use scrypto::blueprints::account::AccountDepositInput;
use scrypto::blueprints::epoch_manager::*;
use scrypto::prelude::*;

// Important: the types defined here must match those in bootstrap.rs

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisData {
    pub validators: Vec<GenesisValidator>,
    pub resources: Vec<GenesisResource>,
    pub accounts: Vec<ComponentAddress>,
    pub resource_balances: Vec<NonXrdResourceBalance>,
    pub xrd_balances: Vec<XrdBalance>,
    pub stakes: Vec<Stake>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisValidator {
    pub key: EcdsaSecp256k1PublicKey,
    pub component_address: ComponentAddress,
    pub allows_delegation: bool,
    pub is_registered: bool,
    pub metadata: Vec<(String, String)>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisResource {
    pub address_bytes: [u8; 26],
    pub metadata: Vec<(String, String)>,
    pub owner_account_index: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct NonXrdResourceBalance {
    pub resource_index: u32,
    pub account_index: u32,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct XrdBalance {
    pub account_index: u32,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct Stake {
    pub validator_index: u32,
    pub account_index: u32,
    pub xrd_amount: Decimal,
}

#[blueprint]
mod genesis_helper {
    struct GenesisHelper;

    impl GenesisHelper {
        pub fn init(
            genesis_data: GenesisData,
            mut whole_lotta_xrd: Bucket,
            validator_owner_token: [u8; 26], // TODO: Clean this up
            epoch_manager_component_address: [u8; 26], // TODO: Clean this up
            initial_epoch: u64,
            rounds_per_epoch: u64,
            num_unstake_epochs: u64,
        ) -> Bucket {
            // Create the resources
            let mut indexed_resource_balances = BTreeMap::new();
            for balance in genesis_data.resource_balances.into_iter() {
                *indexed_resource_balances
                    .entry(balance.resource_index)
                    .or_insert(BTreeMap::new())
                    .entry(balance.account_index)
                    .or_insert(Decimal::ZERO) += balance.amount;
            }

            for (resource_idx, resource) in genesis_data.resources.into_iter().enumerate() {
                let mut initial_supply = Decimal::ZERO;
                let mut initial_allocation = BTreeMap::new();
                for (account_idx, amount) in indexed_resource_balances
                    .remove(&(resource_idx as u32))
                    .unwrap_or(BTreeMap::new())
                {
                    // TODO: check for/handle overflows
                    initial_supply += amount;
                    let account_component_address = genesis_data.accounts[account_idx as usize].clone();
                    initial_allocation.insert(account_component_address, amount);
                }
                let owner = resource
                    .owner_account_index
                    .map(|idx| genesis_data.accounts[idx as usize].clone());
                Self::create_resource(resource, initial_supply, initial_allocation, owner);
            }

            // Create the epoch manager with initial validator set...
            let mut indexed_stakes = BTreeMap::new();
            for stake in genesis_data.stakes.into_iter() {
                *indexed_stakes
                    .entry(stake.validator_index)
                    .or_insert(BTreeMap::new())
                    .entry(stake.account_index)
                    .or_insert(Decimal::ZERO) += stake.xrd_amount;
            }
            let mut validators_with_initial_stake = vec![];
            for (validator_idx, validator) in genesis_data.validators.into_iter().enumerate() {
                let initial_stake_amount = indexed_stakes.get(&(validator_idx as u32))
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
                    rounds_per_epoch,
                    num_unstake_epochs,
                })
                .unwrap(),
            );

            // ...and distribute the LP tokens to stakers
            for (validator_idx, mut lp_bucket) in lp_buckets.into_iter().enumerate() {
                let stakes = indexed_stakes.remove(&(validator_idx as u32)).unwrap_or(BTreeMap::new());
                for (account_idx, stake_xrd_amount) in stakes {
                    // TODO: currently xrd amount matches stake tokens amount, but can this change later on?
                    let stake_bucket = lp_bucket.take(stake_xrd_amount);
                    let account_address = genesis_data.accounts[account_idx as usize];
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
            for XrdBalance { account_index, amount } in genesis_data.xrd_balances.into_iter() {
                let account_address = genesis_data.accounts[account_index as usize];
                let bucket = whole_lotta_xrd.take(amount);
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
            let resource_address = ResourceAddress::Fungible(resource.address_bytes.clone());
            let mut access_rules = BTreeMap::new();
            access_rules.insert(Deposit, (rule!(allow_all), rule!(deny_all)));
            access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));

            if let Some(owner) = owner_with_mint_and_burn_rights {
                // Note that we also set "tags" metadata later on
                let owner_badge = ResourceBuilder::new_fungible()
                    .divisibility(DIVISIBILITY_NONE)
                    .metadata(
                        "name",
                        format!("Resource Owner Badge ({})", "TODO"),
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

                let _: () = Runtime::call_method(
                    owner,
                    "deposit",
                    scrypto_encode(&AccountDepositInput {
                        bucket: owner_badge,
                    })
                    .unwrap(),
                );
            }

            let metadata = resource.metadata.into_iter().collect();

            let (_, mut bucket): (ResourceAddress, Bucket) = Runtime::call_function(
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

            borrow_resource_manager!(resource_address)
                .metadata()
                .set_list("tags", vec![MetadataValue::String("badge".to_string())]);
        }
    }
}
