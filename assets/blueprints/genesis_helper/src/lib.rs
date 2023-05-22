use native_sdk::account::*;
use native_sdk::epoch_manager::*;
use scrypto::prelude::scrypto_env::ScryptoEnv;
use scrypto::prelude::*;

// Important: the types defined here must match those in bootstrap.rs
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisValidator {
    pub key: EcdsaSecp256k1PublicKey,
    pub accept_delegated_stake: bool,
    pub is_registered: bool,
    pub metadata: Vec<(String, String)>,
    pub owner: ComponentAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisStakeAllocation {
    pub account_index: u32,
    pub xrd_amount: Decimal,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct GenesisResource {
    pub address_bytes_without_entity_id: [u8; NodeId::UUID_LENGTH],
    pub initial_supply: Decimal,
    pub metadata: Vec<(String, String)>,
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
        allocations: Vec<(EcdsaSecp256k1PublicKey, Vec<GenesisStakeAllocation>)>,
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
    struct GenesisHelper {
        epoch_manager: ComponentAddress,
        xrd_vault: Vault,
        resource_vaults: KeyValueStore<ResourceAddress, Vault>,
        validators: KeyValueStore<EcdsaSecp256k1PublicKey, ComponentAddress>,
    }

    impl GenesisHelper {
        pub fn new(
            preallocated_address_bytes: [u8; 30],
            whole_lotta_xrd: Bucket,
            epoch_manager: ComponentAddress,
            system_role: NonFungibleGlobalId,
        ) -> Global<GenesisHelper> {
            Self {
                epoch_manager,
                xrd_vault: Vault::with_bucket(whole_lotta_xrd),
                resource_vaults: KeyValueStore::new(),
                validators: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize()
            .define_roles(roles! {
                "system" => rule!(require(system_role.clone())), ["system"];
            })
            .methods(methods! {
                ingest_data_chunk => ["system"];
                wrap_up => ["system"];
            })
            .with_address(ComponentAddress::new_or_panic(preallocated_address_bytes))
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

        pub fn wrap_up(&mut self) -> Bucket {
            EpochManager(self.epoch_manager)
                .start(&mut ScryptoEnv)
                .unwrap();

            // TODO: assert all resource vaults are empty
            // i.e. that for all resources: initial_supply == sum(allocations)

            // return any unused XRD
            self.xrd_vault.take_all()
        }

        fn create_validators(&mut self, validators: Vec<GenesisValidator>) {
            for validator in validators.into_iter() {
                self.create_validator(validator);
            }
        }

        fn create_validator(&mut self, validator: GenesisValidator) {
            let (validator_address, owner_token_bucket) = EpochManager(self.epoch_manager)
                .create_validator(validator.key, &mut ScryptoEnv)
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

            self.validators.insert(validator.key, validator_address);
        }

        fn allocate_stakes(
            &mut self,
            accounts: Vec<ComponentAddress>,
            allocations: Vec<(EcdsaSecp256k1PublicKey, Vec<GenesisStakeAllocation>)>,
        ) {
            for (validator_key, stake_allocations) in allocations.into_iter() {
                let validator_address = self.validators.get(&validator_key).unwrap();
                for GenesisStakeAllocation {
                    account_index,
                    xrd_amount,
                } in stake_allocations.into_iter()
                {
                    let staker_account_address = accounts[account_index as usize].clone();
                    let stake_xrd_bucket = self.xrd_vault.take(xrd_amount);
                    let stake_unit_bucket = Validator(validator_address.clone())
                        .stake(stake_xrd_bucket, &mut ScryptoEnv)
                        .unwrap();
                    let _: () = Account(staker_account_address)
                        .deposit(stake_unit_bucket, &mut ScryptoEnv)
                        .unwrap();
                }
            }
        }

        fn create_resources(&mut self, resources: Vec<GenesisResource>) {
            for resource in resources {
                let (resource_address, initial_supply_bucket) = Self::create_resource(resource);
                self.resource_vaults
                    .insert(resource_address, Vault::with_bucket(initial_supply_bucket));
            }
        }

        fn create_resource(resource: GenesisResource) -> (ResourceAddress, Bucket) {
            let metadata: BTreeMap<String, String> = resource.metadata.into_iter().collect();

            let address_bytes = NodeId::new(
                EntityType::GlobalFungibleResourceManager as u8,
                &resource.address_bytes_without_entity_id,
            )
            .0;
            let resource_address = ResourceAddress::new_or_panic(address_bytes.clone());
            let mut access_rules = BTreeMap::new();
            access_rules.insert(Deposit, (rule!(allow_all), rule!(deny_all)));
            access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));

            if let Some(owner) = resource.owner {
                // TODO: Should we use securify style non fungible resource for the owner badge?
                let owner_badge = ResourceBuilder::new_fungible()
                    .divisibility(DIVISIBILITY_NONE)
                    .metadata(
                        "name",
                        format!("Resource Owner Badge ({})", metadata.get("symbol").unwrap()),
                    )
                    .mint_initial_supply(1);

                owner_badge
                    .resource_manager()
                    .metadata()
                    .set("tags", vec!["badge".to_string()]);

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

                let _: () = Account(owner)
                    .deposit(owner_badge, &mut ScryptoEnv)
                    .unwrap();
            }

            let (_, initial_supply_bucket): (ResourceAddress, Bucket) = Runtime::call_function(
                RESOURCE_PACKAGE,
                FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT,
                scrypto_encode(
                    &FungibleResourceManagerCreateWithInitialSupplyAndAddressInput {
                        divisibility: 18,
                        metadata,
                        access_rules,
                        initial_supply: resource.initial_supply,
                        resource_address: address_bytes,
                    },
                )
                .unwrap(),
            );

            (resource_address, initial_supply_bucket)
        }

        fn allocate_resources(
            &mut self,
            accounts: Vec<ComponentAddress>,
            allocations: Vec<(ResourceAddress, Vec<GenesisResourceAllocation>)>,
        ) {
            for (resource_address, allocations) in allocations.into_iter() {
                let mut resource_vault = self.resource_vaults.get_mut(&resource_address).unwrap();
                for GenesisResourceAllocation {
                    account_index,
                    amount,
                } in allocations.into_iter()
                {
                    let account_address = accounts[account_index as usize].clone();
                    let allocation_bucket = resource_vault.take(amount);
                    let _: () = Account(account_address)
                        .deposit(allocation_bucket, &mut ScryptoEnv)
                        .unwrap();
                }
            }
        }

        fn allocate_xrd(&mut self, allocations: Vec<(ComponentAddress, Decimal)>) {
            for (account_address, amount) in allocations.into_iter() {
                let bucket = self.xrd_vault.take(amount);
                let _: () = Account(account_address)
                    .deposit(bucket, &mut ScryptoEnv)
                    .unwrap();
            }
        }
    }
}
