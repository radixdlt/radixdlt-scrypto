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
                    "type": "U32"
                }
            ],
            "output": {
                "type": "Tuple",
                "elements": [
                    {
                        "type": "Custom",
                        "name": "scrypto::resource::Bucket",
                        "generics": []
                    },
                    {
                        "type": "Custom",
                        "name": "scrypto::core::Component",
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
                    "type": "Custom",
                    "name": "scrypto::types::Decimal",
                    "generics": []
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
                    "type": "Custom",
                    "name": "scrypto::types::Decimal",
                    "generics": []
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
            "name": "admin_badge_address",
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
        collateralization_ratio: Decimal,
        collateral_resource_definition: ResourceDef,
        usd_resource_definition: ResourceDef,
        // State
        synthetics_minter_auth: Vault,
        staked_collateral_vault_map: LazyMap<Address, Vault>,
        synthetic_token_minted_debt_by_ticker_code: HashMap<String, Vault>,
        synthetic_token_and_debt_rdefs_by_ticker_code: LazyMap<String, (ResourceDef, ResourceDef)>,
        synthetic_token_rdef_to_ticker_code: LazyMap<ResourceDef, String>,
        // Oracle State
        off_ledger_ticker_code_prices_in_usd: LazyMap<String, Decimal>,
        unix_timestamp_oracle: u128,
        // Trash
        trash: Vec<Vault>,
    }

    /// Implements a barebones synthetics system for a single user.
    /// Does not currently implement:
    /// * Proportional debt ownership
    /// * Burning synths
    /// * Trading synths
    /// * Fees or rewards
    impl SyntheticPool {
        pub fn new(
            oracle_address: Address,
            collateral_token_address: Address,
            usd_token_address: Address,
            collateralization_ratio: Decimal,
        ) -> Component {
            let oracle: PriceOracle = oracle_address.into();
            let collateral_resource_definition: ResourceDef = collateral_token_address.into();
            let usd_resource_definition: ResourceDef = usd_token_address.into();
            let synthetics_mint_auth_badge: Bucket = ResourceBuilder::new().metadata("name", "synthetics_mint_auth").new_token_fixed(1);
            let synthetic_pool = Self {
                oracle,
                collateralization_ratio,
                collateral_resource_definition,
                usd_resource_definition,
                synthetics_minter_auth: Vault::with_bucket(synthetics_mint_auth_badge),
                staked_collateral_vault_map: LazyMap::new(),
                synthetic_token_minted_debt_by_ticker_code: HashMap::new(),
                synthetic_token_and_debt_rdefs_by_ticker_code: LazyMap::new(),
                synthetic_token_rdef_to_ticker_code: LazyMap::new(),
                off_ledger_ticker_code_prices_in_usd: LazyMap::new(),
                unix_timestamp_oracle: 0,
                trash: Vec::new(),
            }
            .instantiate();

            synthetic_pool
        }

        pub fn stake_to_new_vault(&self, collateral: Bucket) -> Bucket {
            self.assert_collateral_correct(&collateral);

            let vault_owner_badge = ResourceBuilder::new()
                .metadata("name", "Vault Badge")
                .new_badge_fixed(1);

            let vault_owner_badge_address = vault_owner_badge.resource_def().address();

            self.staked_collateral_vault_map
                .insert(vault_owner_badge_address, Vault::with_bucket(collateral));

            vault_owner_badge
        }

        pub fn stake_to_existing_vault(&self, vault_owner_badge: BucketRef, collateral: Bucket) {
            self.assert_collateral_correct(&collateral);

            let vault = self.get_vault_for_badgeref_safe(vault_owner_badge);

            vault.put(collateral);
        }

        // NB - I considered taking a badge, not a badge ref, because we might want to burn it if the vault is emptied
        //      But I preferred the option to explicitly dispose_badge if a user wished to close their vault
        pub fn unstake_from_vault(
            &self,
            vault_owner_badge: BucketRef,
            amount_to_unstake: Decimal,
        ) -> Bucket {
            let vault = self.get_vault_for_badgeref_safe_no_use_ref(&vault_owner_badge);

            let unstaked_tokens = vault.take(amount_to_unstake); // Throws if not enough tokens

            // Rollback transaction if not sufficiently collateralised
            self.assert_sufficiently_collateralised(vault_owner_badge);

            unstaked_tokens
        }

        pub fn dispose_badge(&mut self, vault_owner_badge: Bucket) {
            let vault = self.get_vault_for_badge_safe(&vault_owner_badge);

            scrypto_assert!(
                vault.is_empty(),
                "You can't dispose of a badge if your vault is not empty"
            );

            // Lazy map doesn't at present support remove
            // self.staked_collateral_vault_map.remove(vault_owner_badge);

            // Burn the badge
            // TODO - make the resource def mutable / actually burn the badge(?)
            self.trash.push(Vault::with_bucket(vault_owner_badge));
        }

        pub fn get_staked_balance(&self, vault_owner_badge: BucketRef) -> Decimal {
            let vault = self.get_vault_for_badgeref_safe(vault_owner_badge);
            vault.amount()
        }

        pub fn mint_synthetic(
            &mut self,
            vault_owner_badge: BucketRef,
            ticker_code: String,
            quantity: Decimal,
        ) -> Bucket {
            let new_synthetics = self.mint_synthetic_internal(&ticker_code, quantity);

            // TODO - work out increase in vault owner's proportion of the debt pool - for now each vault owner is assumed to own the whole system debt!

            // Rollback transaction if not sufficiently collateralised
            self.assert_sufficiently_collateralised(vault_owner_badge);

            new_synthetics
        }

        fn assert_collateral_correct(&self, collateral: &Bucket) {
            scrypto_assert!(
                collateral.resource_def() == self.collateral_resource_definition,
                "You need to provide {} ({}) as collateral, but you provided {} ({})",
                self.collateral_resource_definition
                    .metadata()
                    .get("symbol")
                    .unwrap_or(&"UNKNOWN_SYMBOL".to_string()),
                self.collateral_resource_definition.address().to_string(),
                collateral
                    .resource_def()
                    .metadata()
                    .get("symbol")
                    .unwrap_or(&"UNKNOWN_SYMBOL".to_string()),
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

            let vault_map_contents = self
                .staked_collateral_vault_map
                .get(&vault_owner_badge_address);

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

            let vault_map_contents = self
                .staked_collateral_vault_map
                .get(&vault_owner_badge_address);

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

            let vault_map_contents = self
                .staked_collateral_vault_map
                .get(&vault_owner_badge_address);

            scrypto_assert!(
                vault_map_contents.is_some(),
                "The provided vault owner badge does not correspond to an active vault"
            );

            vault_map_contents.unwrap()
        }

        // TODO - implement me!
        fn mint_synthetic_internal(&mut self, ticker_code: &String, quantity: Decimal) -> Bucket {
            let (synthetic_resource_def, debt_resource_def) = self.get_resource_defs_for_synthetic_and_debt(ticker_code);
            
            let minted_token = self.synthetics_minter_auth.authorize(|badge| synthetic_resource_def.mint(quantity.to_owned(), badge));
            let minted_debt = self.synthetics_minter_auth.authorize(|badge| debt_resource_def.mint(quantity.to_owned(), badge));
            let debt_vault_option = self.synthetic_token_minted_debt_by_ticker_code.get(ticker_code);
            if debt_vault_option.is_some() {
                debt_vault_option.unwrap().put(minted_debt);
            } else {
                self.synthetic_token_minted_debt_by_ticker_code.insert(ticker_code.to_owned(), Vault::with_bucket(minted_debt));
            }
            minted_token
        }

        fn get_resource_defs_for_synthetic_and_debt(&self, ticker_code: &String) -> (ResourceDef, ResourceDef) {
            let resource_defs_option = self.synthetic_token_and_debt_rdefs_by_ticker_code.get(&ticker_code);

            if resource_defs_option.is_some() {
                return resource_defs_option.unwrap();
            }        
            
            let resource_defs = (
                ResourceBuilder::new()
                    .metadata("name".to_owned(), format!("Synthetic {}", ticker_code))
                    .metadata("symbol".to_owned(), format!("s{}", ticker_code))
                    .new_token_mutable(self.synthetics_minter_auth.resource_def()),
                ResourceBuilder::new()
                    .metadata("name".to_owned(), format!("Synthetic {} system debt", ticker_code))
                    .metadata("symbol".to_owned(), format!("s{}-DEBT", ticker_code))
                    .new_token_mutable(self.synthetics_minter_auth.resource_def())
            );

            self.synthetic_token_and_debt_rdefs_by_ticker_code.insert(ticker_code.to_owned(), resource_defs.to_owned());

            resource_defs
        }

        fn get_total_system_debt_in_usd(&self) -> Decimal {
            let mut total = Decimal::zero();
            for (ticker_code, vault) in &self.synthetic_token_minted_debt_by_ticker_code {
                let oracle_price = self.get_off_ledger_usd_price(ticker_code.to_string());
                scrypto_assert!(
                    oracle_price.is_some(),
                    "The oracle price for ({}) has no value",
                    ticker_code
                );
                total = total + vault.amount() * oracle_price.unwrap();
            }
            total
        }

        fn get_collateral_price(&self) -> Decimal {
            let oracle_price = self.oracle.get_price(
                self.collateral_resource_definition.address(),
                self.usd_resource_definition.address(),
            );
            scrypto_assert!(
                oracle_price.is_some(),
                "The oracle price for collateral {} ({}) has no value",
                self.collateral_resource_definition
                    .metadata()
                    .get("symbol")
                    .unwrap_or(&"UNKNOWN_SYMBOL".to_string()),
                self.collateral_resource_definition.address().to_string()
            );
            oracle_price.unwrap()
        }

        fn assert_sufficiently_collateralised(&self, vault_owner_badge: BucketRef) {
            let vault = self.get_vault_for_badgeref_safe(vault_owner_badge);

            let collateral_in_usd = vault.amount() * self.get_collateral_price();

            // TODO - fix it so that the proportion of system debt is tracked - so that we don't assume each user is responsible for the whole system debt!
            let vault_owned_system_debt_in_usd = self.get_total_system_debt_in_usd();

            if vault_owned_system_debt_in_usd == Decimal::zero() {
                return;
            }

            let vault_collat_ratio = collateral_in_usd / vault_owned_system_debt_in_usd;

            scrypto_assert!(
                vault_collat_ratio >= self.collateralization_ratio,
                "Your (new) vault collateralisation ratio is {}, but needs to be at least {}",
                vault_collat_ratio,
                self.collateralization_ratio
            );
        }

        // SLIGHT HACK - WE ADD AN OFF-LEDGER PRICE ORACLE TO THIS COMPONENT - BUT THIS SHOULD BE SEPARATE AND AUTHORISED

        /// Sets the price of pair BASE/QUOTE.
        pub fn get_off_ledger_usd_price(&self, ticker_code: String) -> Option<Decimal> {
            self.off_ledger_ticker_code_prices_in_usd.get(&ticker_code)
        }

        /// Updates the price of pair BASE/QUOTE.
        pub fn update_off_ledger_usd_price(&self, ticker_code: String, price: Decimal) {
            self.off_ledger_ticker_code_prices_in_usd
                .insert(ticker_code, price);
        }

        pub fn get_unix_timestamp(&self) -> u128 {
            self.unix_timestamp_oracle
        }

        pub fn update_unix_timestamp(&mut self, timestamp_in_seconds: u128) {
            self.unix_timestamp_oracle = timestamp_in_seconds;
        }
    }
}
