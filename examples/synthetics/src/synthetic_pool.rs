use scrypto::prelude::*;

import! {
r#"
{
    "package": "01024420d4c8749579abc13133bf07b0a4fc307aa0172f595a0245",
    "name": "PriceOracle",
    "functions": [
        {
            "name": "new",
            "inputs": [
                {
                    "type": "U8"
                },
                {
                    "type": "U32"
                }
            ],
            "output": {
                "type": "Tuple",
                "elements": [
                    {
                        "type": "Custom",
                        "name": "scrypto::core::Component",
                        "generics": []
                    },
                    {
                        "type": "Custom",
                        "name": "scrypto::resource::Bucket",
                        "generics": []
                    }
                ]
            }
        }
    ],
    "methods": [
        {
            "name": "get_price",
            "mutability": "Immutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::types::Address",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "scrypto::types::Address",
                    "generics": []
                }
            ],
            "output": {
                "type": "Option",
                "value": {
                    "type": "U128"
                }
            }
        },
        {
            "name": "update_price",
            "mutability": "Immutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::types::Address",
                    "generics": []
                },
                {
                    "type": "Custom",
                    "name": "scrypto::types::Address",
                    "generics": []
                },
                {
                    "type": "U128"
                },
                {
                    "type": "Custom",
                    "name": "scrypto::resource::BucketRef",
                    "generics": []
                }
            ],
            "output": {
                "type": "Unit"
            }
        },
        {
            "name": "decimals",
            "mutability": "Immutable",
            "inputs": [],
            "output": {
                "type": "U8"
            }
        },
        {
            "name": "admin",
            "mutability": "Immutable",
            "inputs": [],
            "output": {
                "type": "Custom",
                "name": "scrypto::types::Address",
                "generics": []
            }
        }
    ]
}
"#
}

blueprint! {
    struct SyntheticPool {
        // Parameters
        oracle: PriceOracle,
        collateralization_ratio_billionths: Amount, // Suggested ratio of 4, ie 4000000000
        collateral_resource_definition: ResourceDef,
        usd_resource_definition: ResourceDef,
        // State
        staked_collateral_vault_map: LazyMap<Address, Vault>,
        synthetic_token_minted_debt_by_exchange_ticker_code: HashMap<(String, String), Vault>,
        synthetic_token_resource_definitions_by_exchange_ticker_code: LazyMap<(String, String), ResourceDef>,
        synthetic_token_resource_definitions_to_exchange_ticker_code: LazyMap<ResourceDef, (String, String)>,
        // Oracle State
        off_ledger_exchange_ticker_code_prices_in_billionths_of_base: LazyMap<(String, String), u128>,
        unix_timestamp_oracle: u128
    }

    impl SyntheticPool {
        pub fn new(
            oracle_address: Address,
            collateral_token_address: Address,
            usd_token_address: Address,
            collateralization_ratio_billionths: u128
        ) -> Component {
            let oracle: PriceOracle = oracle_address.into();
            let collateral_resource_definition: ResourceDef = collateral_token_address.into();
            let usd_resource_definition: ResourceDef = usd_token_address.into();
            let synthetic_pool = Self {
                oracle,
                collateralization_ratio_billionths: collateralization_ratio_billionths.into(),
                collateral_resource_definition,
                usd_resource_definition,
                staked_collateral_vault_map: LazyMap::new(),
                synthetic_token_minted_debt_by_exchange_ticker_code: HashMap::new(),
                synthetic_token_resource_definitions_by_exchange_ticker_code: LazyMap::new(),
                synthetic_token_resource_definitions_to_exchange_ticker_code: LazyMap::new(),
                off_ledger_exchange_ticker_code_prices_in_billionths_of_base: LazyMap::new(),
                unix_timestamp_oracle: 0
            }.instantiate();

            synthetic_pool
        }

        pub fn stake_to_new_vault(&self, collateral: Bucket) -> Bucket {

            self.assert_collateral_correct(&collateral);

            let vault_owner_badge = ResourceBuilder::new()
                .metadata("name", "Vault Badge")
                .create_fixed(1);

            let vault_owner_badge_address = vault_owner_badge.resource_def().address();

            self.staked_collateral_vault_map.insert(vault_owner_badge_address, Vault::with_bucket(collateral));

            vault_owner_badge
        }

        pub fn stake_to_existing_vault(&self, vault_owner_badge: BucketRef, collateral: Bucket) {
            self.assert_collateral_correct(&collateral);

            let vault = self.get_vault_for_badgeref_safe(vault_owner_badge);

            vault.put(collateral);
        }

        // NB - I considered taking a badge, not a badge ref, because we might want to burn it if the vault is emptied
        //      But I preferred the option to explicitly dispose_badge if a user wished to close their vault
        pub fn unstake_from_vault(&self, vault_owner_badge: BucketRef, amount_to_unstake: Amount) -> Bucket {
            let vault = self.get_vault_for_badgeref_safe_no_use_ref(&vault_owner_badge);

            let unstaked_tokens = vault.take(amount_to_unstake); // Throws if not enough tokens

            // Rollback transaction if not sufficiently collateralised
            self.assert_sufficiently_collateralised(vault_owner_badge);

            unstaked_tokens
        }

        pub fn dispose_badge(&self, vault_owner_badge: Bucket) {
            let vault = self.get_vault_for_badge_safe(&vault_owner_badge);

            scrypto_assert!(
                vault.is_empty(),
                "You can't dispose of a badge if your vault is not empty"
            );

            // Lazy map doesn't at present support remove
            // self.staked_collateral_vault_map.remove(vault_owner_badge);
            vault_owner_badge.burn();
        }

        pub fn get_staked_balance(&self, vault_owner_badge: BucketRef) -> Amount {
            let vault = self.get_vault_for_badgeref_safe(vault_owner_badge);
            vault.amount()
        }

        pub fn mint_synthetic(&self, vault_owner_badge: BucketRef, exchange: String, ticker_code: String, quantity: Amount) -> Bucket {

            let new_synthetics = self.mint_synthetic_internal(exchange, ticker_code, quantity);

            // TODO - work out increase in vault owner's proportion of the debt pool - for now each vault owner is assumed to own the whole system debt!

            // Rollback transaction if not sufficiently collateralised
            self.assert_sufficiently_collateralised(vault_owner_badge);

            new_synthetics
        }

        fn assert_collateral_correct(&self, collateral: &Bucket) {
            scrypto_assert!(
                collateral.resource_def() == self.collateral_resource_definition,
                "You need to provide {} ({}) as collateral, but you provided {} ({})",
                self.collateral_resource_definition.metadata().get("symbol").unwrap_or(&"UNKNOWN_SYMBOL".to_string()),
                self.collateral_resource_definition.address().to_string(),
                collateral.resource_def().metadata().get("symbol").unwrap_or(&"UNKNOWN_SYMBOL".to_string()),
                collateral.resource_def().address().to_string()
            );
        }

        fn get_vault_for_badgeref_safe(&self, vault_owner_badge: BucketRef) -> Vault {
            scrypto_assert!(
                !vault_owner_badge.is_empty(),
                "The provided vault owner badge bucketref doesn't contain a badge"
            );

            let vault_owner_badge_address = vault_owner_badge.resource_def().address();
            vault_owner_badge.drop();

            let vault_map_contents = self.staked_collateral_vault_map.get(&vault_owner_badge_address);

            scrypto_assert!(
                vault_map_contents.is_some(),
                "The provided vault owner badge does not correspond to an active vault"
            );

            vault_map_contents.unwrap()
        }

        fn get_vault_for_badgeref_safe_no_use_ref(&self, vault_owner_badge: &BucketRef) -> Vault {
            scrypto_assert!(
                !vault_owner_badge.is_empty(),
                "The provided vault owner badge bucketref doesn't contain a badge"
            );

            let vault_owner_badge_address = vault_owner_badge.resource_def().address();

            let vault_map_contents = self.staked_collateral_vault_map.get(&vault_owner_badge_address);

            scrypto_assert!(
                vault_map_contents.is_some(),
                "The provided vault owner badge does not correspond to an active vault"
            );

            vault_map_contents.unwrap()
        }

        fn get_vault_for_badge_safe(&self, vault_owner_badge: &Bucket) -> Vault {
            scrypto_assert!(
                !vault_owner_badge.is_empty(),
                "The provided vault owner badge bucket doesn't contain a badge"
            );

            let vault_owner_badge_address = vault_owner_badge.resource_def().address();

            let vault_map_contents = self.staked_collateral_vault_map.get(&vault_owner_badge_address);

            scrypto_assert!(
                vault_map_contents.is_some(),
                "The provided vault owner badge does not correspond to an active vault"
            );

            vault_map_contents.unwrap()
        }

        // TODO - implement me!
        fn mint_synthetic_internal(&self, _exchange: String, _ticker_code: String, _quantity: Amount) -> Bucket {
            // TODO - fix me!
            Bucket::new(ResourceDef::from(self.collateral_resource_definition.address()))
        }

        // TODO - revisit this when the resource definition supports decimals
        fn get_total_system_debt_in_usd_billionths(&self) -> Amount {
            let mut total = Amount::zero();
            for (exchange_ticker_code_key, vault) in &self.synthetic_token_minted_debt_by_exchange_ticker_code {
                let oracle_price = self.get_off_ledger_usd_price_in_billionths(exchange_ticker_code_key.0.to_string(), exchange_ticker_code_key.1.to_string());
                scrypto_assert!(
                    oracle_price.is_some(),
                    "The oracle price for ({}, {}) has no value",
                    exchange_ticker_code_key.0, exchange_ticker_code_key.1
                );
                total = total + vault.amount() * oracle_price.unwrap();
            }
            total
        }

        fn get_collateral_price_in_usd_billionths(&self) -> Amount {
            let oracle_decimals = self.oracle.decimals();
            let oracle_price = self.oracle.get_price(self.collateral_resource_definition.address(), self.usd_resource_definition.address());
            scrypto_assert!(
                oracle_price.is_some(),
                "The oracle price for collateral {} ({}) has no value",
                self.collateral_resource_definition.metadata().get("symbol").unwrap_or(&"UNKNOWN_SYMBOL".to_string()),
                self.collateral_resource_definition.address().to_string()
            );
            let base: u128 = 10;
            if oracle_decimals >= 9 {
                (oracle_price.unwrap() / base.pow((oracle_decimals - 9).into())).into()
            } else {
                (oracle_price.unwrap() * base.pow((9 - oracle_decimals).into())).into()
            }
        }

        fn assert_sufficiently_collateralised(&self, vault_owner_badge: BucketRef) {
            let vault = self.get_vault_for_badgeref_safe(vault_owner_badge);
            let billion: Amount = (1000000000).into();

            let collateral_in_usd_billionths = vault.amount() * self.get_collateral_price_in_usd_billionths();

            // TODO - fix it so that the proportion of system debt is tracked - so that we don't assume each user is responsible for the whole system debt! 
            let proportion_of_system_debt_in_billionths = billion;
            let vault_owned_system_debt_in_usd_billionths = (self.get_total_system_debt_in_usd_billionths() * proportion_of_system_debt_in_billionths) / billion;

            if vault_owned_system_debt_in_usd_billionths == Amount::zero() {
                return
            }

            let vault_collat_ratio_in_billionths = collateral_in_usd_billionths * billion / vault_owned_system_debt_in_usd_billionths;

            scrypto_assert!(
                vault_collat_ratio_in_billionths >= self.collateralization_ratio_billionths,
                "Your (new) vault collateralisation ratio (in billionths) is {}, but needs to be at least {}",
                vault_collat_ratio_in_billionths, self.collateralization_ratio_billionths
            );
        }

        // SLIGHT HACK - WE ADD AN OFF-LEDGER PRICE ORACLE TO THIS COMPONENT - BUT IT COULD BE SEPARATE
        // TODO - Add badge auth

        /// Sets the price (in billionth) of pair BASE/QUOTE.
        pub fn get_off_ledger_usd_price_in_billionths(&self, exchange: String, ticker_code: String) -> Option<u128> {
            self.off_ledger_exchange_ticker_code_prices_in_billionths_of_base.get(&(exchange, ticker_code))
        }

        /// Updates the price (in billionth) of pair BASE/QUOTE.
        pub fn update_off_ledger_usd_price(&self, exchange: String, ticker_code: String, price_in_billionths: u128) {
            self.off_ledger_exchange_ticker_code_prices_in_billionths_of_base.insert((exchange, ticker_code), price_in_billionths);
        }

        pub fn get_unix_timestamp(&self) -> u128 {
            self.unix_timestamp_oracle
        }

        pub fn update_unix_timestamp(&mut self, timestamp_in_seconds: u128) {
            self.unix_timestamp_oracle = timestamp_in_seconds;
        }
    }
}
