use scrypto::prelude::*;
use sbor::*;

// Used to keep track of the user's stake
#[derive(Debug, TypeId, Encode, Decode, Describe, PartialEq, Eq)]
pub struct StakerData {
    // Define when the user staked
    started_at: u64,
    // Defines the amount that the user staked
    amount: Decimal
}

blueprint! {
    struct CoalYieldFarming {
        // Will hold a badge allowing the component to
        // mint Coal tokens and burn staker badges
        minter: Vault,
        
        // Will hold the staked Coal tokens
        stake_pool: Vault,

        stakers: HashMap<Address, StakerData>
    }

    impl CoalYieldFarming {
        pub fn new() -> Component {
            // Create the minter badge.
            // this badge will be owned by the component and will
            // allow it to mint new coal tokens and burn staker's badge
            let minter = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .metadata("name", "Coal Minter Badge")
                                .initial_supply_fungible(1);

            // Define the coal resource
            let coal = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                        .metadata("name", "Coal")
                        .flags(MINTABLE)
                        .badge(minter.resource_def(), MAY_MINT)
                        .no_initial_supply();

            Self {
                minter: Vault::with_bucket(minter),
                stake_pool: Vault::new(coal),
                stakers: HashMap::new()
            }.instantiate()
        }

        // Allow caller to stake their coal tokens.
        // This method sends a badge allowing the user to withdraw their funds later
        pub fn stake(&mut self, coal: Bucket) -> Bucket {
            assert!(coal.resource_def() == self.stake_pool.resource_def(), "You can only stake coal !");

            // Create the badge used to withdraw the tokens in the futur
            let staker_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                    .metadata("name", "Coal Staker Badge")
                    .flags(MINTABLE | BURNABLE)
                    .badge(self.minter.resource_def(), MAY_MINT | MAY_BURN)
                    .initial_supply_fungible(1);

            // Save the stake's data on the component's state
            self.stakers.insert(staker_badge.resource_address(), StakerData { started_at: Context::current_epoch(), amount: coal.amount() });
            self.stake_pool.put(coal);

            // Return the staker badge to the caller
            staker_badge
        }

        // Withdraw the staked tokens and rewards received.
        pub fn withdraw(&mut self, staker_badge: Bucket) -> (Bucket, Bucket) {
            let staker_data = match self.stakers.get(&staker_badge.resource_address()) {
                Some(staker) => staker,
                None => {
                    info!("No entries found for this badge !");
                    std::process::abort();
                }
            };

            // Burn the staker badge so that it cannot be used again
            self.minter.authorize(|minter| {
                staker_badge.burn_with_auth(minter)
            });

            // Mint coal depending on how long the user staked
            let reward = self.minter.authorize(|minter| {
                let epochs_staked = Context::current_epoch() - staker_data.started_at;
                self.stake_pool.resource_def().mint(10 * epochs_staked, minter)
            });
            
            // Return the staked amount + newly minted tokens
            (self.stake_pool.take(staker_data.amount), reward)
        }

        // Send 1000 Coal tokens to the caller
        // to help you test this component
        pub fn faucet(&self) -> Bucket {
            self.minter.authorize(|minter| {
                self.stake_pool.resource_def().mint(1000, minter)
            })
        }
    }
}
