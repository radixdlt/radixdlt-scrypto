use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct Data {}

#[blueprint]
mod vault_test {
    struct NonFungibleVault {
        vault: Vault,
    }

    impl NonFungibleVault {
        fn create_singleton_non_fungible_vault() -> Vault {
            let bucket = ResourceBuilder::new_integer_non_fungible(OwnerRole::None)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply([(1u64.into(), Data {})]);
            Vault::with_bucket(bucket.into())
        }

        fn create_empty_non_fungible_vault() -> Vault {
            let resource_manager =
                ResourceBuilder::new_integer_non_fungible::<Data>(OwnerRole::None)
                    .metadata(metadata! {
                        init {
                            "name" => "TestToken".to_owned(), locked;
                        }
                    })
                    .create_with_no_initial_supply();
            resource_manager.create_empty_vault().into()
        }

        fn create_non_fungible_vault() -> Vault {
            let bucket = ResourceBuilder::new_integer_non_fungible(OwnerRole::None)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply([(1u64.into(), Data {}), (2u64.into(), Data {})]);
            Vault::with_bucket(bucket.into())
        }

        pub fn withdraw_one_from_empty() -> Global<NonFungibleVault> {
            let mut vault = Self::create_empty_non_fungible_vault();
            let bucket = vault.take(1);
            bucket.drop_empty();
            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn new_non_fungible_vault() -> Global<NonFungibleVault> {
            let vault = Self::create_non_fungible_vault();
            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn new_non_fungible_vault_with_take() -> Global<NonFungibleVault> {
            let mut vault = Self::create_non_fungible_vault();
            {
                let bucket = vault.take(1);
                assert_eq!(vault.amount(), Decimal::from(1));
                assert_eq!(bucket.amount(), Decimal::from(1));
                vault.put(bucket);
            }

            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn new_non_fungible_vault_with_take_twice() -> Global<NonFungibleVault> {
            let mut vault = Self::create_non_fungible_vault();
            {
                let bucket0 = vault.take(1);
                assert_eq!(bucket0.amount(), Decimal::from(1));
                assert_eq!(vault.amount(), Decimal::from(1));

                let bucket1 = vault.take(1);
                assert_eq!(bucket1.amount(), Decimal::from(1));
                assert_eq!(vault.amount(), Decimal::from(0));

                vault.put(bucket0);
                vault.put(bucket1);
                assert_eq!(vault.amount(), Decimal::from(2));
            }

            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn new_non_fungible_vault_with_take_non_fungible() -> Global<NonFungibleVault> {
            let mut vault = Self::create_non_fungible_vault();
            let bucket = vault
                .as_non_fungible()
                .take_non_fungible(&NonFungibleLocalId::integer(1));
            vault.put(bucket.into());
            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn new_vault_with_get_non_fungible_local_ids() -> Global<NonFungibleVault> {
            let vault = Self::create_non_fungible_vault();
            let _ids = vault.as_non_fungible().non_fungible_local_ids(100);
            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn new_vault_with_get_non_fungible_local_id() -> Global<NonFungibleVault> {
            let vault = Self::create_singleton_non_fungible_vault();
            let _id = vault.as_non_fungible().non_fungible_local_id();
            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn new_vault_with_get_amount() -> Global<NonFungibleVault> {
            let vault = Self::create_non_fungible_vault();
            let _amount = vault.amount();
            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn new_vault_with_get_resource_manager() -> Global<NonFungibleVault> {
            let vault = Self::create_non_fungible_vault();
            let _resource_manager = vault.resource_address();
            Self { vault }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn take(&mut self) {
            let bucket = self.vault.take(1);
            assert_eq!(bucket.amount(), Decimal::from(1));
            assert_eq!(self.vault.amount(), Decimal::from(1));
            self.vault.put(bucket);
            assert_eq!(self.vault.amount(), Decimal::from(2));
        }

        pub fn take_twice(&mut self) {
            let bucket0 = self.vault.take(1);
            assert_eq!(bucket0.amount(), Decimal::from(1));
            assert_eq!(self.vault.amount(), Decimal::from(1));

            let bucket1 = self.vault.take(1);
            assert_eq!(bucket1.amount(), Decimal::from(1));
            assert_eq!(self.vault.amount(), Decimal::from(0));

            self.vault.put(bucket0);
            self.vault.put(bucket1);
            assert_eq!(self.vault.amount(), Decimal::from(2));
        }
    }
}
