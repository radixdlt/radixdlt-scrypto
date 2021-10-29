use scrypto::prelude::*;

import! {
r#"
{    
  "package": "01024420d4c8749579abc13133bf07b0a4fc307aa0172f595a0245",
  "name": "PriceOracle",
  "functions": [
    {
      "name": "new",
      "inputs": [],
      "output": {
        "type": "Custom",
        "name": "scrypto::core::Component",
        "generics": []
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
        }
      ],
      "output": {
        "type": "Unit"
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
        minting_collateralization_ratio_billionths: u128, // Suggested ratio of 4, ie 4000000000
        collateral_resource_definition: ResourceDef,
        // State
        staked_collateral_vault_map: LazyMap<Address, Vault>,
        synthetic_token_resource_definitions_by_exchange_ticker_code: LazyMap<(String, String), ResourceDef>,
        synthetic_token_resource_definitions_to_exchange_ticker_code: LazyMap<ResourceDef, (String, String)>,
        // Oracle State
        off_ledger_exchange_ticker_code_prices_in_billionths_of_base: LazyMap<(String, String, Address), u128>,
        unix_timestamp_oracle: u128
    }

    impl SyntheticPool {
        pub fn new(
            oracle_address: Address,
            collateral_token_address: Address,
            minting_collateralization_ratio_billionths: u128
        ) -> Component {
            let oracle: PriceOracle = oracle_address.into();
            let collateral_resource_definition: ResourceDef = collateral_token_address.into();
            let synthetic_pool = Self {
                oracle,
                minting_collateralization_ratio_billionths,
                collateral_resource_definition,
                staked_collateral_vault_map: LazyMap::new(),
                synthetic_token_resource_definitions_by_exchange_ticker_code: LazyMap::new(),
                synthetic_token_resource_definitions_to_exchange_ticker_code: LazyMap::new(),
                off_ledger_exchange_ticker_code_prices_in_billionths_of_base: LazyMap::new(),
                unix_timestamp_oracle: 0
            }.instantiate();

            synthetic_pool
        }

        pub fn stake_to_new_vault(&self, collateral: Bucket) -> Bucket {

            scrypto_assert!(
                collateral.resource_def() == self.collateral_resource_definition,
                "You need to provide {} ({}) as collateral, but you provided {} ({})",
                self.collateral_resource_definition.metadata().get("symbol").unwrap_or(&"UNKNOWN_SYMBOL".to_string()),
                self.collateral_resource_definition.address().to_string(),
                collateral.resource_def().metadata().get("symbol").unwrap_or(&"UNKNOWN_SYMBOL".to_string()),
                collateral.resource_def().address().to_string()
            );

            let vault_owner_badge = ResourceBuilder::new()
                .metadata("name", "Vault Badge")
                .create_fixed(1);

            let vault_owner_badge_address = vault_owner_badge.resource_def().address();

            self.staked_collateral_vault_map.insert(vault_owner_badge_address, Vault::with_bucket(collateral));

            vault_owner_badge
        }

        pub fn stake_to_existing_vault(&self, vault_owner_badge: BucketRef, collateral: Bucket) {
            scrypto_assert!(
                vault_owner_badge.is_empty(),
                "Your badge bucket doesn't contain a badge"
            );

            scrypto_assert!(
                collateral.resource_def() == self.collateral_resource_definition,
                "You need to provide {} ({}) as collateral, but you provided {} ({})",
                self.collateral_resource_definition.metadata().get("symbol").unwrap_or(&"UNKNOWN_SYMBOL".to_string()),
                self.collateral_resource_definition.address().to_string(),
                collateral.resource_def().metadata().get("symbol").unwrap_or(&"UNKNOWN_SYMBOL".to_string()),
                collateral.resource_def().address().to_string()
            );

            let vault_owner_badge_address = vault_owner_badge.resource_def().address();
            vault_owner_badge.drop();

            let vault_map_contents = self.staked_collateral_vault_map.get(&vault_owner_badge_address);

            scrypto_assert!(
                vault_map_contents.is_some(),
                "The provided badge does not correspond to an active vault"
            );

            let vault = vault_map_contents.unwrap();

            vault.put(collateral);
        }
/*
        pub fn mint_synthetic(&self, exchange: String, ticker_code: String, amount_in_billionths_to_mint: u128, collateral: Bucket) -> Bucket {

            // Assert collateral is of right type(!)
            // Get price of underlying asset
            // 
            // Assert over <minting_collateralization_ratio_billionths> collateral compared to underlying price
            // Record something TBC to determine pool share later?, and to thank over-collateralization
            // Return synthetic token (also maybe pool share token?)
        }

        pub fn redeem_for_collateral(&self, synthetic_tokens: Bucket) -> Bucket {
            // Assert synthetic tokens are of created token type
            // Calculate amount of, minus fees
            // Burn synthetic tokens
            // Return collateral
        }

        pub fn issuer_repayment(&self, returned_synthetic_tokens: Bucket, pool_share_tokens: Bucket) -> Bucket {

            // Assert synthetic tokens are of valid type
            // Assert pool share tokens are of valid type
            // Some complex calculation to work out collateral
            // - Ideally, if reverted immediately, can receive roughly equal to the amount of collateral redeemed, minus fees
            // - If redeemed later, can redeem some interest (?)
            // Return collateral
        }

        pub fn pay_interest(&self) {
            // This interface possibly needs to change - or interest gets issued at repayment time or something
            // Who knows how this works!? Can read from the unix timestamp oracle - but what impact does interest have?
            // Notes: We could store a last interest paid timestamp; do we store the date of issuance into the pool share tokens somehow??
            // How do we store interest into the 
        }
        */

        // SLIGHT HACK - WE ADD AN OFF-LEDGER PRICE ORACLE TO THIS COMPONENT - BUT IT COULD BE SEPARATE

        /// Sets the price (in billionth) of pair BASE/QUOTE.
        pub fn get_off_ledger_price_in_billionths(&self, exchange: String, ticker_code: String, quote_token_definition: Address) -> Option<u128> {
            self.off_ledger_exchange_ticker_code_prices_in_billionths_of_base.get(&(exchange, ticker_code, quote_token_definition))
        }

        /// Updates the price (in billionth) of pair BASE/QUOTE.
        pub fn update_off_ledger_price(&self, exchange: String, ticker_code: String, quote_token_definition: Address, price_in_billionths: u128) {
            self.off_ledger_exchange_ticker_code_prices_in_billionths_of_base.insert((exchange, ticker_code, quote_token_definition), price_in_billionths);
        }

        pub fn get_unix_timestamp(&self) -> u128 {
            self.unix_timestamp_oracle
        }

        pub fn update_unix_timestamp(&mut self, timestamp_in_seconds: u128) {
            self.unix_timestamp_oracle = timestamp_in_seconds;
        }
    }
}
