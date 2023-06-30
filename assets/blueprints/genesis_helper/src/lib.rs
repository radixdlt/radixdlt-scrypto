use native_sdk::account::*;
use native_sdk::consensus_manager::*;
use native_sdk::resource::ResourceManager;
use scrypto::api::node_modules::metadata::*;
use scrypto::api::object_api::ObjectModuleId;
use scrypto::api::ClientObjectApi;
use scrypto::prelude::scrypto_env::ScryptoEnv;
use scrypto::prelude::*;

// Important: the types defined here must match those in bootstrap.rs
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisValidator {
    pub key: Secp256k1PublicKey,
    pub accept_delegated_stake: bool,
    pub is_registered: bool,
    pub fee_factor: Decimal,
    pub metadata: Vec<(String, MetadataValue)>,
    pub owner: ComponentAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisStakeAllocation {
    pub account_index: u32,
    pub xrd_amount: Decimal,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisResource {
    pub address_reservation: GlobalAddressReservation,
    pub metadata: Vec<(String, MetadataValue)>,
    pub owner: Option<ComponentAddress>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisResourceAllocation {
    pub account_index: u32,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum GenesisDataChunk {
    Validators(Vec<GenesisValidator>),
    Stakes {
        accounts: Vec<ComponentAddress>,
        allocations: Vec<(Secp256k1PublicKey, Vec<GenesisStakeAllocation>)>,
    },
    Resources(Vec<GenesisResource>),
    ResourceBalances {
        accounts: Vec<ComponentAddress>,
        allocations: Vec<(ResourceAddress, Vec<GenesisResourceAllocation>)>,
    },
    XrdBalances(Vec<(ComponentAddress, Decimal)>),
}

#[blueprint]
mod genesis_helper {
    enable_function_auth! {
        new => rule!(deny_all); // Genesis skips this
    }

    enable_method_auth! {
        roles {
            system => updatable_by: [system];
        },
        methods {
            ingest_data_chunk => restrict_to: [system];
            wrap_up => restrict_to: [system];
        }
    }

    struct GenesisHelper {
        consensus_manager: ComponentAddress,
        validators: KeyValueStore<Secp256k1PublicKey, ComponentAddress>,
    }

    impl GenesisHelper {
        pub fn new(
            address_reservation: GlobalAddressReservation,
            consensus_manager: ComponentAddress,
            system_role: NonFungibleGlobalId,
        ) -> Global<GenesisHelper> {
            Self {
                consensus_manager,
                validators: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(system_role.clone()))))
            .with_address(address_reservation)
            .metadata(metadata! {
                init {
                    "name" => "Genesis Helper".to_owned(), locked;
                    "description" => "A component with various utility and helper methods used in the creation of the Babylon Genesis.".to_owned(), locked;
                }
            })
            .globalize()
        }

        pub fn ingest_data_chunk(&mut self, chunk: GenesisDataChunk) {
            match chunk {
                GenesisDataChunk::Validators(validators) => self.create_validators(validators),
                GenesisDataChunk::Stakes {
                    accounts,
                    allocations,
                } => self.allocate_stakes(accounts, allocations),
                GenesisDataChunk::Resources(resources) => self.create_resources(resources),
                GenesisDataChunk::ResourceBalances {
                    accounts,
                    allocations,
                } => self.allocate_resources(accounts, allocations),
                GenesisDataChunk::XrdBalances(allocations) => self.allocate_xrd(allocations),
            }
        }

        fn create_validators(&mut self, validators: Vec<GenesisValidator>) {
            for validator in validators.into_iter() {
                self.create_validator(validator);
            }
        }

        fn create_validator(&mut self, validator: GenesisValidator) {
            let xrd_payment = ResourceManager(XRD)
                .new_empty_bucket(&mut ScryptoEnv)
                .unwrap();
            let (validator_address, owner_token_bucket) = ConsensusManager(self.consensus_manager)
                .create_validator(
                    validator.key,
                    validator.fee_factor,
                    xrd_payment,
                    &mut ScryptoEnv,
                )
                .unwrap();

            // Deposit the badge to the owner account
            Account(validator.owner)
                .deposit(owner_token_bucket, &mut ScryptoEnv)
                .unwrap();

            if validator.is_registered {
                Validator(validator_address)
                    .register(&mut ScryptoEnv)
                    .unwrap();
            }

            Validator(validator_address)
                .update_accept_delegated_stake(validator.accept_delegated_stake, &mut ScryptoEnv)
                .unwrap();

            for (key, value) in validator.metadata {
                ScryptoEnv
                    .call_method_advanced(
                        &validator_address.into_node_id(),
                        false,
                        ObjectModuleId::Metadata,
                        METADATA_SET_IDENT,
                        scrypto_encode(&MetadataSetInput { key, value }).unwrap(),
                    )
                    .expect("Failed to set validator metadata");
            }

            self.validators.insert(validator.key, validator_address);
        }

        fn allocate_stakes(
            &mut self,
            accounts: Vec<ComponentAddress>,
            allocations: Vec<(Secp256k1PublicKey, Vec<GenesisStakeAllocation>)>,
        ) {
            let xrd_needed: Decimal = allocations
                .iter()
                .flat_map(|(_, allocations)| {
                    allocations.iter().map(|alloc| alloc.xrd_amount.clone())
                })
                .sum();
            let mut xrd_bucket = ResourceManager(RADIX_TOKEN)
                .mint_fungible(xrd_needed, &mut ScryptoEnv)
                .expect("XRD mint for genesis stake allocation failed");

            for (validator_key, stake_allocations) in allocations.into_iter() {
                let validator_address = self.validators.get(&validator_key).unwrap();
                let validator = Validator(validator_address.clone());

                // Enable staking temporarily for genesis delegators
                let accepts_delegated_stake =
                    validator.accepts_delegated_stake(&mut ScryptoEnv).unwrap();
                if !accepts_delegated_stake {
                    validator
                        .update_accept_delegated_stake(true, &mut ScryptoEnv)
                        .unwrap();
                }

                for GenesisStakeAllocation {
                    account_index,
                    xrd_amount,
                } in stake_allocations.into_iter()
                {
                    let staker_account_address = accounts[account_index as usize].clone();
                    let stake_xrd_bucket = xrd_bucket.take(xrd_amount);
                    let stake_unit_bucket =
                        validator.stake(stake_xrd_bucket, &mut ScryptoEnv).unwrap();
                    let _: () = Account(staker_account_address)
                        .deposit(stake_unit_bucket, &mut ScryptoEnv)
                        .unwrap();
                }

                // Restore original delegated stake flag
                if !accepts_delegated_stake {
                    validator
                        .update_accept_delegated_stake(accepts_delegated_stake, &mut ScryptoEnv)
                        .unwrap();
                }
            }

            xrd_bucket.drop_empty();
        }

        fn create_resources(&mut self, resources: Vec<GenesisResource>) {
            for resource in resources {
                Self::create_resource(resource);
            }
        }

        fn create_resource(resource: GenesisResource) -> () {
            let metadata: BTreeMap<String, MetadataValue> = resource.metadata.into_iter().collect();

            let owner_badge_address = if let Some(owner) = resource.owner {
                // TODO: Should we use securify style non fungible resource for the owner badge?
                let owner_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                    .divisibility(DIVISIBILITY_NONE)
                    .metadata(metadata! {
                        init {
                            "name" => format!(
                                "Resource Owner Badge ({})",
                                String::from_metadata_value(metadata.get("symbol").unwrap().clone())
                                    .unwrap()
                            ), locked;
                        }
                    })
                    .mint_initial_supply(1);

                let owner_badge_address = owner_badge.resource_address();

                let resource_mgr = owner_badge.resource_manager();
                resource_mgr.set_metadata("tags", vec!["badge".to_string()]);

                let _: () = Account(owner)
                    .deposit(owner_badge, &mut ScryptoEnv)
                    .unwrap();

                Some(owner_badge_address)
            } else {
                None
            };

            let owner_role = match owner_badge_address {
                None => OwnerRole::None,
                Some(owner_badge_address) => OwnerRole::Fixed(rule!(require(owner_badge_address))),
            };

            let builder = ResourceBuilder::new_fungible(owner_role)
                .metadata(ModuleConfig {
                    init: metadata.into(),
                    roles: RolesInit::default(),
                })
                .with_address(resource.address_reservation);

            if let Some(address) = owner_badge_address {
                builder
                    .mintable(rule!(require(address)), rule!(deny_all))
                    .burnable(rule!(require(address)), rule!(deny_all))
                    .create_with_no_initial_supply();
            } else {
                builder.create_with_no_initial_supply();
            }
        }

        fn allocate_resources(
            &mut self,
            accounts: Vec<ComponentAddress>,
            allocations: Vec<(ResourceAddress, Vec<GenesisResourceAllocation>)>,
        ) {
            for (resource_address, allocations) in allocations.into_iter() {
                let amount_needed = allocations.iter().map(|alloc| alloc.amount.clone()).sum();
                let mut resource_bucket = ResourceManager(resource_address)
                    .mint_fungible(amount_needed, &mut ScryptoEnv)
                    .expect("Resource mint for genesis allocation failed");

                for GenesisResourceAllocation {
                    account_index,
                    amount,
                } in allocations.into_iter()
                {
                    let account_address = accounts[account_index as usize].clone();
                    let allocation_bucket = resource_bucket.take(amount);
                    let _: () = Account(account_address)
                        .deposit(allocation_bucket, &mut ScryptoEnv)
                        .unwrap();
                }
                resource_bucket.drop_empty();
            }
        }

        fn allocate_xrd(&mut self, allocations: Vec<(ComponentAddress, Decimal)>) {
            let xrd_needed = allocations.iter().map(|(_, amount)| amount.clone()).sum();
            let mut xrd_bucket = ResourceManager(RADIX_TOKEN)
                .mint_fungible(xrd_needed, &mut ScryptoEnv)
                .expect("XRD mint for genesis allocation failed");

            for (account_address, amount) in allocations.into_iter() {
                let bucket = xrd_bucket.take(amount);
                let _: () = Account(account_address)
                    .deposit(bucket, &mut ScryptoEnv)
                    .unwrap();
            }

            xrd_bucket.drop_empty();
        }

        pub fn wrap_up(&mut self) -> () {
            ConsensusManager(self.consensus_manager)
                .start(&mut ScryptoEnv)
                .unwrap();
        }
    }
}
