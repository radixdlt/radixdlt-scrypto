use scrypto::prelude::*;

// Important: the types defined here must match those in bootstrap.rs
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisValidator {
    pub key: Secp256k1PublicKey,
    pub accept_delegated_stake: bool,
    pub is_registered: bool,
    pub fee_factor: Decimal,
    pub metadata: Vec<(String, MetadataValue)>,
    pub owner: Global<Account>,
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
    pub owner: Option<Global<Account>>,
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
        accounts: Vec<Global<Account>>,
        allocations: Vec<(Secp256k1PublicKey, Vec<GenesisStakeAllocation>)>,
    },
    Resources(Vec<GenesisResource>),
    ResourceBalances {
        accounts: Vec<Global<Account>>,
        allocations: Vec<(ResourceManager, Vec<GenesisResourceAllocation>)>,
    },
    XrdBalances(Vec<(Global<Account>, Decimal)>),
}

#[blueprint]
#[types(Secp256k1PublicKey, Global<Validator>)]
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

    const XRD_MGR: ResourceManager = resource_manager!(XRD);

    struct GenesisHelper {
        consensus_manager: Global<ConsensusManager>,
        validators: KeyValueStore<Secp256k1PublicKey, Global<Validator>>,
    }

    impl GenesisHelper {
        pub fn new(
            address_reservation: GlobalAddressReservation,
            consensus_manager: Global<ConsensusManager>,
            system_role: NonFungibleGlobalId,
        ) -> Global<GenesisHelper> {
            Self {
                consensus_manager,
                validators: KeyValueStore::new_with_registered_type(),
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

        fn create_validator(&mut self, mut validator: GenesisValidator) {
            let xrd_payment = XRD_MGR.create_empty_bucket();
            let (mut validator_component, owner_token_bucket, change) = self
                .consensus_manager
                .create_validator(validator.key, validator.fee_factor, xrd_payment);

            change.drop_empty();

            // Deposit the badge to the owner account
            validator.owner.deposit(owner_token_bucket);

            if validator.is_registered {
                validator_component.register();
            }

            validator_component.update_accept_delegated_stake(validator.accept_delegated_stake);

            for (key, value) in validator.metadata {
                ScryptoVmV1Api::object_call_module(
                    &validator_component.address().into_node_id(),
                    AttachedModuleId::Metadata,
                    METADATA_SET_IDENT,
                    scrypto_encode(&MetadataSetInput { key, value }).unwrap(),
                );
            }

            self.validators.insert(validator.key, validator_component);
        }

        fn allocate_stakes(
            &mut self,
            accounts: Vec<Global<Account>>,
            allocations: Vec<(Secp256k1PublicKey, Vec<GenesisStakeAllocation>)>,
        ) {
            let xrd_needed: Decimal = {
                let mut sum = Decimal::ZERO;
                for v in allocations.iter().flat_map(|(_, allocations)| {
                    allocations.iter().map(|alloc| alloc.xrd_amount.clone())
                }) {
                    sum = sum
                        .checked_add(v)
                        .expect("Overflow because resource limit exceeded");
                }
                sum
            };
            let mut xrd_bucket = XRD_MGR.mint(xrd_needed);

            for (validator_key, stake_allocations) in allocations.into_iter() {
                let mut validator = self.validators.get_mut(&validator_key).unwrap();

                // Enable staking temporarily for genesis delegators
                let accepts_delegated_stake = validator.accepts_delegated_stake();
                if !accepts_delegated_stake {
                    validator.update_accept_delegated_stake(true);
                }

                for GenesisStakeAllocation {
                    account_index,
                    xrd_amount,
                } in stake_allocations.into_iter()
                {
                    let mut staker_account = accounts[account_index as usize].clone();
                    let stake_xrd_bucket = xrd_bucket.take(xrd_amount);
                    let stake_unit_bucket = validator.stake(stake_xrd_bucket);
                    staker_account.deposit(stake_unit_bucket);
                }

                // Restore original delegated stake flag
                if !accepts_delegated_stake {
                    validator.update_accept_delegated_stake(accepts_delegated_stake);
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

            if let Some(mut owner) = resource.owner {
                // create owner badge
                let owner_badge = ResourceBuilder::new_fungible(OwnerRole::None)
                    .divisibility(DIVISIBILITY_NONE)
                    .metadata(metadata! {
                        init {
                            "name" => format!(
                                "Resource Owner Badge ({})",
                                String::from_metadata_value(metadata.get("symbol").unwrap().clone())
                                    .unwrap()
                            ), locked;
                            "tags" => vec!["badge".to_string()], locked;
                        }
                    })
                    .mint_initial_supply(1);

                let owner_badge_address = owner_badge.resource_address();
                owner.deposit(owner_badge.into());

                // create resource
                ResourceBuilder::new_fungible(OwnerRole::Fixed(rule!(require(
                    owner_badge_address
                ))))
                .mint_roles(mint_roles! {
                    minter => OWNER;
                    minter_updater => OWNER;
                })
                .burn_roles(burn_roles! {
                    burner => OWNER;
                    burner_updater => OWNER;
                })
                .metadata(ModuleConfig {
                    init: metadata.into(),
                    roles: RoleAssignmentInit::default(),
                })
                .with_address(resource.address_reservation)
                .create_with_no_initial_supply();
            } else {
                // create resource
                ResourceBuilder::new_fungible(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => rule!(deny_all);
                        minter_updater => rule!(deny_all);
                    })
                    .metadata(ModuleConfig {
                        init: metadata.into(),
                        roles: RoleAssignmentInit::default(),
                    })
                    .with_address(resource.address_reservation)
                    .create_with_no_initial_supply();
            };
        }

        fn allocate_resources(
            &mut self,
            accounts: Vec<Global<Account>>,
            allocations: Vec<(ResourceManager, Vec<GenesisResourceAllocation>)>,
        ) {
            for (resource_manager, allocations) in allocations.into_iter() {
                let amount_needed = {
                    let mut sum = Decimal::ZERO;
                    for v in allocations.iter().map(|alloc| alloc.amount.clone()) {
                        sum = sum
                            .checked_add(v)
                            .expect("Overflow because resource limit exceeded");
                    }
                    sum
                };
                let mut resource_bucket = resource_manager.mint(amount_needed);

                for GenesisResourceAllocation {
                    account_index,
                    amount,
                } in allocations.into_iter()
                {
                    let mut account = accounts[account_index as usize].clone();
                    let allocation_bucket = resource_bucket.take(amount);
                    account.deposit(allocation_bucket);
                }
                resource_bucket.drop_empty();
            }
        }

        fn allocate_xrd(&mut self, allocations: Vec<(Global<Account>, Decimal)>) {
            let xrd_needed = {
                let mut sum = Decimal::ZERO;
                for v in allocations.iter().map(|(_, amount)| amount.clone()) {
                    sum = sum
                        .checked_add(v)
                        .expect("Overflow because resource limit exceeded");
                }
                sum
            };
            let mut xrd_bucket = XRD_MGR.mint(xrd_needed);

            for (mut account, amount) in allocations.into_iter() {
                let bucket = xrd_bucket.take(amount);
                account.deposit(bucket);
            }

            xrd_bucket.drop_empty();
        }

        pub fn wrap_up(&mut self) -> () {
            self.consensus_manager.start();
        }
    }
}
