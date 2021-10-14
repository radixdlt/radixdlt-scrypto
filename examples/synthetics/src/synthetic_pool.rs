use scrypto::prelude::*;

import! {
r#"
{    
  "package": "01806c33ab58c922240ce20a5b697546cc84aaecdf1b460a42c425",
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
         ) -> Component {
            let oracle: PriceOracle = oracle_address.into();
            let synthetic_pool = Self {
                oracle: oracle,
                collateral_token_symbol: collateral_token_symbol,
                underlying_asset_token_symbol: underlying_asset_token_symbol,
                synthetic_asset_token_symbol: synthetic_asset_token_symbol,
            }.instantiate();

            synthetic_pool
        }

    }
}
