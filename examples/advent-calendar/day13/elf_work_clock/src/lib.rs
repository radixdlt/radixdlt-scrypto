use scrypto::prelude::*;
use scrypto::types::Decimal;

// Import the time oracle blueprint
// Change the "change_me" text with the time oracle package address.
// Example:
// {
//   "package": "013fa22e238526e9c82376d2b4679a845364243bf970e5f783d13f"
//   "name": "UTCTimeOracle"
//   ...
import! {
    r#"
    {
      "package": "change_me",
      "name": "UTCTimeOracle",
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
          "name": "set_current_time",
          "mutability": "Mutable",
          "inputs": [
            {
              "type": "U16"
            },
            {
              "type": "U8"
            },
            {
              "type": "U8"
            },
            {
              "type": "U8"
            },
            {
              "type": "U8"
            },
            {
              "type": "U8"
            },
            {
              "type": "U64"
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
          "name": "get_time",
          "mutability": "Immutable",
          "inputs": [],
          "output": {
            "type": "Tuple",
            "elements": [
              {
                "type": "U16"
              },
              {
                "type": "U8"
              },
              {
                "type": "U8"
              },
              {
                "type": "U8"
              },
              {
                "type": "U8"
              },
              {
                "type": "U8"
              },
              {
                "type": "U64"
              }
            ]
          }
        }
      ]
    }"#
  }

blueprint! {
    struct WorkClock {
        time_oracle: UTCTimeOracle,
        elf_badge: ResourceDef,
        time_sheet_minter: Vault,
        salary_vault: Vault,
        hour_salary: Decimal
    }

    impl WorkClock {
        pub fn new(nb_workers: u32, hour_salary: Decimal, time_oracle_address: Address) -> (Component, Bucket) {
            let elf_badges = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                    .metadata("name", "Elf Badge")
                                    .initial_supply_fungible(nb_workers);

            // Used to create and burn time sheets
            let time_sheet_minter = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                        .metadata("name", "TimeSheet minter")
                                        .initial_supply_fungible(1);

            // Create the tokens that will be used to pay the elfs
            let salary_tokens = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                                    .metadata("name", "Elf Salary")
                                    .initial_supply_fungible(100000000000_u64);

            let component = Self {
                time_oracle: time_oracle_address.into(),
                elf_badge: elf_badges.resource_def(),
                time_sheet_minter: Vault::with_bucket(time_sheet_minter),
                salary_vault: Vault::with_bucket(salary_tokens),
                hour_salary: hour_salary
            }
            .instantiate();

            (component, elf_badges)
        }

        #[auth(elf_badge)]
        pub fn start_work(&self) -> Bucket {
            // Get the time. Send empty bucket as fee
            let (year, month, day, hour, minute, second, unix_time) = self.time_oracle.get_time();

            // Create a timesheet token
            ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                    .metadata("name", format!("TimeSheet {}/{}/{} {}:{}:{}", year, month, day, hour, minute, second))
                    .metadata("date", format!("{}", unix_time))
                    .flags(BURNABLE)
                    .badge(self.time_sheet_minter.resource_def(), MAY_BURN)
                    .initial_supply_fungible(1)
        }

        pub fn end_work(&self, timesheet: Bucket) -> Bucket {
            assert!(timesheet.amount() > Decimal::zero(), "Missing timesheet");

            // Get the current time
            let (_, _, _, _, _, _, unix_time) = self.time_oracle.get_time();

            // No checks here when unwrapping to keep it simple.
            // Keep in mind that anyone could create their own badge to fake the time
            // they started working. I didn't want to do fix this since NFTs have not been implemented yet
            // and they could easily fix this issue.
            let start_time: u64 = timesheet.resource_def().metadata().get("date").unwrap().parse().unwrap();

            // Burn the timesheet
            self.time_sheet_minter.authorize(|minter| {
                timesheet.burn_with_auth(minter);
            });

            let hours_worked = (unix_time - start_time) / 3600;
            // Send the salary
            self.salary_vault.take(self.hour_salary * hours_worked)
        }
    }
}
