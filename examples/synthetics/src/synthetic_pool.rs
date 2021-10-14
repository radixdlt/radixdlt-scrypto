use scrypto::prelude::*;

use crate::oracle::PriceOracle;

import! {
r#"
    {
        "package": "01ca59a8d6ea4f7efa1765cef702d14e47570c079aedd44992dd09",
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
                "type": "String"
              }
            ],
            "output": {
              "type": "Option",
              "value": {
                "type": "U64"
              }
            }
          },
          {
            "name": "put_price",
            "mutability": "Immutable",
            "inputs": [
              {
                "type": "String"
              },
              {
                "type": "U64"
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
        oracle: PriceOracle,
        collateral_token_symbol: String,
        underlying_asset_token_symbol: String,
        synthetic_asset_token_symbol: String,
    }

    impl SyntheticPool {
        pub fn new(
            oracle_address: Address,
            collateral_token_symbol: String,
            underlying_asset_token_symbol: String,
            synthetic_asset_token_symbol: String,
        } -> (Component) {
            let oracle: PriceOracle = oracle_address.into();
            let synthetic_pool = Self {
                oracle: oracle,
                collateral_token_symbol: collateral_token_symbol,
                underlying_asset_token_symbol: underlying_asset_token_symbol,
                synthetic_asset_token_symbol: synthetic_asset_token_symbol,
            }.instantiate();

            synthetic_pool
        }

        pub fn get_price(&self, pair: String) -> Bucket {
            self.vendor.get_price(pair)
        }
    }
}
