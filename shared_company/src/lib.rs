use scrypto::prelude::*;
mod proposal;
use crate::proposal::Proposal;

/*
Missing functionality (for now):
- Burn --> How to? Seems like badges are needed but clear example is missing. Waiting for video explaination
- Send_to_adress --> Approach by Rock should work but unclear how to pass the adress to Account::FromStr
- many more, especially regarding security, testing, updates etc.
*/

/*
A shared company is a company owned by its shareholders.
Shares can be bought (fixed rate for now).
Funds can be spend using [Proposal] if accepted by a majority of shareweight.
This is only a demonstration. There are known security flaws in this design
(like: Buy a lot of shares, make proposal to send you all compnay_radix, vote for it)
*/
blueprint! {

struct SharedCompany {
    company_radix:  Vault,
    company_shares: Vault,
    company_voting_token: Vault,
    share_counter: Decimal,
    price_share: Decimal,
    share_burn_badge: Vault,
}

impl SharedCompany {
    pub fn new(price_share: Decimal) -> Component{

        //Create a badge that will be locked in a vault and can later be user to burn shares
        let share_burn_badge = ResourceBuilder::new().new_badge_fixed(1);

        // create a new company_share resource,
        let shared_company_share_resource_def = ResourceBuilder::new()
        .metadata("name", "SharedCompany share").metadata("symbol", "SC")
        .new_token_fixed(1_000_000);

        // create a new company_share resource,
        let shared_company_voting_token_resource_def = ResourceBuilder::new()
        .metadata("name", "SharedCompany voting token").metadata("symbol", "SCVT")
        .new_token_fixed(1_000_000);



    //populate the SharedCompany struct and instantiate a new component
    Self {
        company_shares: Vault::with_bucket(shared_company_share_resource_def),
        company_voting_token: Vault::with_bucket(shared_company_voting_token_resource_def),
        company_radix: Vault::new(RADIX_TOKEN),
        share_counter: Decimal(0.0 as i128),
        price_share: price_share,
        share_burn_badge: Vault::with_bucket(share_burn_badge),
    }.instantiate()

    }
    // Returns the price per share
    pub fn get_price(&self) -> Decimal {
        self.price_share
    }

    /// buys an amount of shares and returns change
    pub fn buy_shares(&mut self, payment: Bucket) -> (Bucket, Bucket, Bucket) {
        let max_share_buy_power = payment.amount() / self.price_share;
        // Increase the share_counter so the amount of shares that are outstanding is tracked
        self.share_counter += max_share_buy_power;
        // take our price in XRD out of the payment and store it in the company vault
        let collected_xrd = payment.take(self.price_share * max_share_buy_power);
        self.company_radix.put(collected_xrd);
        // return the share(s) and change
        (self.company_shares.take(max_share_buy_power), payment, self.company_voting_token.take(max_share_buy_power))
    }

     /// sells an amount of shares and for a part of the companies xrd
     pub fn sell_shares(&mut self, shares: Bucket, voting_token: Bucket) -> Bucket {
        // calculates the percentage of all shares
        let percentage_of_all_shares = shares.amount() / self.share_counter;
        // Decreases the counter
        self.share_counter -= shares.amount();
        //ToDoBurns the shares
        //ToDO Burn the voting_token

        // returns the same percentage of the company xrd
        self.company_radix.take(self.company_radix.amount() * percentage_of_all_shares)
    }

    // A proposal that if it is accepted sends funds away from the company
    pub fn make_proposal(&self,cost_as_number: u32, destination_adress: String,reason: String,
        admin_adress: String, end_epoch: u64,){
        //ToDo change this to smt variable, but fails method call for some reason
            let cost = self.company_radix.take(cost_as_number);
        Proposal::new(cost, destination_adress, reason, admin_adress, end_epoch, self.share_counter / 2 + 1, self.company_voting_token.resource_def());
    }





}}
