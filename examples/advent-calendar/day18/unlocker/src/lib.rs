use sbor::*;
use scrypto::prelude::*;

// Import the price oracle blueprint
// Change the "change_me" text with the price oracle package address.
// Example:
// {
//   "package": "013fa22e238526e9c82376d2b4679a845364243bf970e5f783d13f"
//   "name": "PriceOracle"
//   ...
import! {
r#"
    {
      "package": "01e2df51eb999d85f29fd3d92bc4be9fec7119f3408ebbd7db91ae",
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
              "type": "Custom",
              "name": "scrypto::types::Decimal",
              "generics": []
            }
          }
        },
        {
          "name": "get_usd_address",
          "mutability": "Immutable",
          "inputs": [],
          "output": {
            "type": "Custom",
            "name": "scrypto::types::Address",
            "generics": []
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

#[derive(Debug, TypeId, Encode, Decode, Describe, PartialEq, Eq)]
struct PercentageData {
  percentage: Decimal,
  nb_admin_approved: u32,
}

#[derive(NftData)]
struct RecipientData {
  amount: Decimal,
  #[scrypto(mutable)]
  percentage_unlocked: Decimal,
}

blueprint! {
  struct PriceBasedUnlockScheduler {
    // Used to have a reference to the price oracle
    // to get the price of the token
    price_oracle: PriceOracle,
    // Keep track of the different unlock steps
    token_percentage_unlocked: HashMap<Decimal, PercentageData>,
    // Badge definition used to protect methods on the component
    admin_def: ResourceDef,
    // Vault to store the tokens
    tokens: Vault,
    // Maps recipient badges to the amount left to withdraw
    recipients: HashMap<Address, Decimal>,
    minter_badge: Vault,
    recipient_def: ResourceDef,
    nb_recipients: u128,
    percentage_unlocked: Decimal
  }

  impl PriceBasedUnlockScheduler {
    pub fn new(token_def: Address, price_oracle_address: Address) -> (Component, Bucket) {
      // Create an admin badge used for
      // authorization to call methods on the component
      let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                          .metadata("name", "Unlock Scheduler")
                          .initial_supply_fungible(1);

      // Define a minter badge, used to mint recipient NFTs
      // and update their individual metadata
      let minter_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                          .metadata("name", "Unlock Scheduler")
                          .initial_supply_fungible(1);

      // Define the recipient NFT, used to keep track
      // of how many tokens a user have left to unlock
      let recipient_def = ResourceBuilder::new_non_fungible()
                            .metadata("name", "Recipient Data")
                            .flags(MINTABLE | INDIVIDUAL_METADATA_MUTABLE)
                            .badge(minter_badge.resource_def(), MAY_MINT | MAY_CHANGE_INDIVIDUAL_METADATA)
                            .no_initial_supply();

      // Define the different unlocking steps
      let mut token_percentage_unlocked = HashMap::new();
      token_percentage_unlocked.insert(10.into(), PercentageData { percentage: 10.into(), nb_admin_approved: 0 }); // At 10$, unlock 10% of the supply
      token_percentage_unlocked.insert(20.into(), PercentageData { percentage: 30.into(), nb_admin_approved: 0 }); // At 20$, unlock 30% of the supply
      token_percentage_unlocked.insert(50.into(), PercentageData { percentage: 60.into(), nb_admin_approved: 0 }); // At 50$, unlock 60% of the supply
      token_percentage_unlocked.insert(60.into(), PercentageData { percentage: 100.into(), nb_admin_approved: 0 }); // At 60$, unlock 100% of the supply

      // Store all required information on the component's state
      let component = Self {
        price_oracle: price_oracle_address.into(),
        token_percentage_unlocked: token_percentage_unlocked,
        admin_def: admin_badge.resource_def(),
        tokens: Vault::new(token_def),
        recipients: HashMap::new(),
        minter_badge: Vault::with_bucket(minter_badge),
        recipient_def: recipient_def,
        nb_recipients: 0,
        percentage_unlocked: Decimal::zero()
      }.instantiate();

      // Return the component and admin badge to the caller
      (component, admin_badge)
    }

    #[auth(admin_def)]
    pub fn add_recipient(&mut self, recipient: Address, tokens: Bucket) {
      // Mint a new NFT for the recipient
      let recipient_nft = self.minter_badge.authorize(|badge| {
        // Keep track of how much that account owns by
        // inserting it in the NFT metadata
        self.recipient_def.mint_nft(self.nb_recipients, RecipientData {amount: tokens.amount(), percentage_unlocked: Decimal::zero()}, badge)
      });

      // Store the user's token in the component's vault
      self.tokens.put(tokens);

      self.nb_recipients += 1;

      // Send the NFT to the account
      Account::from(recipient).deposit(recipient_nft);
    }

    #[auth(admin_def)]
    pub fn do_unlock(&mut self) {
      // Get the current price of the asset
      let current_price = match self.price_oracle.get_price(self.tokens.resource_address(), self.price_oracle.get_usd_address()) {
        Some(price) => price,
        None => {
          info!("No price found for {}", self.tokens.resource_def().metadata().get("name").unwrap());
          std::process::abort();
        }
      };

      // Update the percentage unlocked
      for (price, data) in self.token_percentage_unlocked.iter() {
        if *price <= current_price && data.percentage > self.percentage_unlocked {
          self.percentage_unlocked = data.percentage;
        }
      }

      info!("percentage_unlocked: {}", self.percentage_unlocked);
    }

    // Allow a recipient to withdraw the unlocked
    // tokens that have not been withdrawn yet.
    pub fn withdraw(&mut self, recipient_nft: BucketRef) -> Bucket {
      let nft_id = recipient_nft.get_nft_id();
      assert!(recipient_nft.amount() > Decimal::zero(), "Need to provide a badge");
      assert!(recipient_nft.resource_def() == self.recipient_def, "Wrong token");

      // Fetch the metadata of the NFT
      let mut nft_data: RecipientData = self.recipient_def.get_nft_data(recipient_nft.get_nft_id());
      let amount = nft_data.amount;
      recipient_nft.drop();

      let to_unlock = self.percentage_unlocked - nft_data.percentage_unlocked;

      // Set the total_unlocked on the NFT.
      // This is necessesary to make sure the recipient
      // can only withdraw what they haven't withdrawn yet
      nft_data.percentage_unlocked = self.percentage_unlocked;

      // Insert the new metadata on the NFT
      self.minter_badge.authorize(|badge| {
        self.recipient_def.update_nft_data(nft_id, nft_data, badge);
      });

      self.tokens.take(amount * (to_unlock / 100))
    }
  }
}
