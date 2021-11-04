use scrypto::prelude::*;

import! {
r#"
{
    "package": "01d25d4eab30b60d9951f3433b35cff52a48f8cf163b66c0a16677",
    "name": "AutoLend",
    "functions": [
        {
        "name": "new",
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
            "type": "Custom",
            "name": "scrypto::core::Component",
            "generics": []
        }
        }
    ],
    "methods": [
        {
        "name": "deposit",
        "mutability": "Mutable",
        "inputs": [
            {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
            }
        ],
        "output": {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
        }
        },
        {
        "name": "redeem",
        "mutability": "Mutable",
        "inputs": [
            {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
            }
        ],
        "output": {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
        }
        },
        {
        "name": "borrow",
        "mutability": "Mutable",
        "inputs": [
            {
            "type": "Custom",
            "name": "scrypto::types::Decimal",
            "generics": []
            },
            {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
            }
        ],
        "output": {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
        }
        },
        {
        "name": "repay",
        "mutability": "Mutable",
        "inputs": [
            {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
            }
        ],
        "output": {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
        }
        },
        {
        "name": "get_collateral_ratio",
        "mutability": "Immutable",
        "inputs": [],
        "output": {
            "type": "U32"
        }
        },
        {
        "name": "a_b_tokens_supply",
        "mutability": "Mutable",
        "inputs": [],
        "output": {
            "type": "Custom",
            "name": "scrypto::types::Decimal",
            "generics": []
        }
        },
        {
        "name": "b_tokens_liquidity",
        "mutability": "Mutable",
        "inputs": [],
        "output": {
            "type": "Custom",
            "name": "scrypto::types::Decimal",
            "generics": []
        }
        }
    ]
    }      
"#
}

import! {
r#"
{
    "package": "01a78cfec3dac583cc2394d14452099892a5af4a5201d771d918a2",
    "name": "SyntheticPool",
    "functions": [
      {
        "name": "new",
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
            "name": "scrypto::types::Address",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "scrypto::types::Decimal",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "scrypto::core::Component",
          "generics": []
        }
      }
    ],
    "methods": [
      {
        "name": "stake_to_new_vault",
        "mutability": "Immutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "scrypto::resource::Bucket",
          "generics": []
        }
      },
      {
        "name": "stake_to_existing_vault",
        "mutability": "Immutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::BucketRef",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
          }
        ],
        "output": {
          "type": "Unit"
        }
      },
      {
        "name": "unstake_from_vault",
        "mutability": "Immutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::BucketRef",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "scrypto::types::Decimal",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "scrypto::resource::Bucket",
          "generics": []
        }
      },
      {
        "name": "dispose_badge",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
          }
        ],
        "output": {
          "type": "Unit"
        }
      },
      {
        "name": "get_staked_balance",
        "mutability": "Immutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::BucketRef",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "scrypto::types::Decimal",
          "generics": []
        }
      },
      {
        "name": "mint_synthetic",
        "mutability": "Mutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::BucketRef",
            "generics": []
          },
          {
            "type": "String"
          },
          {
            "type": "Custom",
            "name": "scrypto::types::Decimal",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "scrypto::resource::Bucket",
          "generics": []
        }
      },
      {
        "name": "get_off_ledger_usd_price",
        "mutability": "Immutable",
        "inputs": [
          {
            "type": "String"
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
        "name": "update_off_ledger_usd_price",
        "mutability": "Immutable",
        "inputs": [
          {
            "type": "String"
          },
          {
            "type": "Custom",
            "name": "scrypto::types::Decimal",
            "generics": []
          }
        ],
        "output": {
          "type": "Unit"
        }
      },
      {
        "name": "get_unix_timestamp",
        "mutability": "Immutable",
        "inputs": [],
        "output": {
          "type": "U128"
        }
      },
      {
        "name": "update_unix_timestamp",
        "mutability": "Mutable",
        "inputs": [
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

import! {
r#"
{
    "package": "01205eedae4ac21cbdc07728bf934d6c0b253cdec0439f867e6bee",
    "name": "Radiswap",
    "functions": [
      {
        "name": "new",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "scrypto::types::Decimal",
            "generics": []
          },
          {
            "type": "String"
          },
          {
            "type": "String"
          },
          {
            "type": "String"
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
        "name": "add_liquidity",
        "mutability": "Immutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
          },
          {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
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
              "name": "scrypto::resource::Bucket",
              "generics": []
            }
          ]
        }
      },
      {
        "name": "remove_liquidity",
        "mutability": "Immutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
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
              "name": "scrypto::resource::Bucket",
              "generics": []
            }
          ]
        }
      },
      {
        "name": "swap",
        "mutability": "Immutable",
        "inputs": [
          {
            "type": "Custom",
            "name": "scrypto::resource::Bucket",
            "generics": []
          }
        ],
        "output": {
          "type": "Custom",
          "name": "scrypto::resource::Bucket",
          "generics": []
        }
      }
    ]
  }  
"#
}

blueprint! {
    struct MutualFarm {
        auto_lend: AutoLend, /* AutoLend contract for borrowing USDC with XRD collateral */
        synthetic_pool: SyntheticPool, /* Synthetic pool that accepts USDC */
        radiswap: Radiswap, /* Radiswap contract with USDC/<synthetic_ticker> pool */
        radiswap_lp_vault: Vault,
        synthetic_ticker: String,
        synthetic_vault_badge_vault: Vault
    }

    impl MutualFarm {
        pub fn new(
            auto_lend_address: Address,
            synthetic_pool_address: Address,
            radiswap_address: Address,
            radiswap_lp_address: Address,
            synthetic_ticker: String
        ) -> Component {
            Self {
                auto_lend: auto_lend_address.into(),
                synthetic_pool: synthetic_pool_address.into(),
                radiswap: radiswap_address.into(),
                radiswap_lp_vault: Vault::new(radiswap_lp_address),
                synthetic_ticker: synthetic_ticker,
                synthetic_vault_badge_vault: Vault::new(SYSTEM_PACKAGE) /* TODO: using SYSTEM_PACKAGE as a vault placeholder; it gets replaced in deposit() */
            }
            .instantiate()
        }

        pub fn deposit(
            &mut self,
            xrd_bucket: Bucket
        ) -> Bucket {
            let amount_usdc_to_borrow: Decimal = xrd_bucket.amount() / self.auto_lend.get_collateral_ratio();
            let usdc_bucket = self.auto_lend.borrow(amount_usdc_to_borrow, xrd_bucket);
            let usdc_bucket_remainder_half = usdc_bucket.take(usdc_bucket.amount() / 2);

            let synthetic_price = self.synthetic_pool.get_off_ledger_usd_price(self.synthetic_ticker.clone()).unwrap();
            let synthetic_quantity = usdc_bucket.amount() / synthetic_price; // TODO: is this correct?
            let synthetic_vault_badge = self.synthetic_pool.stake_to_new_vault(usdc_bucket);
            let tsla_bucket = self.synthetic_pool.mint_synthetic(synthetic_vault_badge.borrow(), self.synthetic_ticker.clone(), synthetic_quantity);
            self.synthetic_vault_badge_vault = Vault::new(synthetic_vault_badge.resource_def());
            self.synthetic_vault_badge_vault.put(synthetic_vault_badge);

            let (radiswap_lp_bucket, remainder_bucket) = self.radiswap.add_liquidity(usdc_bucket_remainder_half, tsla_bucket);
            self.radiswap_lp_vault.put(radiswap_lp_bucket);

            return remainder_bucket;
        }

        pub fn withdraw(&mut self) -> (Bucket, Bucket) {
            let (usdc_bucket, tsla_bucket) = self.radiswap.remove_liquidity(self.radiswap_lp_vault.take_all());
            let synthetic_badge = self.synthetic_vault_badge_vault.take_all();
            let synthetic_stake_balance = self.synthetic_pool.get_staked_balance(synthetic_badge.borrow());
            let usdc_from_synthetic = self.synthetic_pool.unstake_from_vault(synthetic_badge.borrow(), synthetic_stake_balance);
            self.synthetic_pool.dispose_badge(synthetic_badge);
            usdc_bucket.put(usdc_from_synthetic);
            let xrd_bucket = self.auto_lend.repay(usdc_bucket); // TODO: make sure it's enough to repay(?)
            return (xrd_bucket, tsla_bucket); // return the xrd and synthetic back to the user
        }
    }
}
