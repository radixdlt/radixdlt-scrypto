use scrypto::prelude::*;
// A proposal that unlocks funds (XRD).
//This fund can then be used to pay for services and products of other firms
blueprint! {
struct Proposal{
            cost_vault: Vault,
            destination_adress_funds: String,
            reason: String,
            company_voting_token: Vault,
            replacement_tokens_type_yes: Vault,
            replacement_tokens_type_no: Vault,
            yes_counter: Decimal,
            no_counter: Decimal,
            proposal_admin: Vault,
            needed_votes: Decimal,
            end_epoch: u32,
              admin_adress: String,
    }

            impl Proposal{

                // creates a new instance
                pub fn new(cost: Bucket, destination_adress_funds: String, reason: String, admin_adress: String, end_epoch: u32,
                    needed_votes: Decimal, company_voting_token_resource_def: ResourceDef)-> Component {

                    // The token that the user gets in exchange for their voting.
                    let replacement_token_yes_resource_def = ResourceBuilder::new()
                       .metadata("name", "Replacement token yes").metadata("symbol", "RTY")
                    .new_token_fixed(1_000_000);

                     // The token that the user gets in exchange for their voting.
                     let replacement_token_no_resource_def = ResourceBuilder::new()
                     .metadata("name", "Replacement token no").metadata("symbol", "RTN")
                  .new_token_fixed(1_000_000);


                    let proposal_admin_badge = ResourceBuilder::new().new_badge_fixed(1);
                    // the vault that holds the costs that are associated with the proposal
                    let cost_vault = Vault::new(cost.resource_def());
                    // fills the cost vault
                    cost_vault.put(cost);

                  Self {
                    cost_vault: cost_vault,
                    destination_adress_funds: destination_adress_funds, reason: reason,
                    company_voting_token : Vault::new(company_voting_token_resource_def),
                    replacement_tokens_type_yes: Vault::with_bucket(replacement_token_yes_resource_def),
                    replacement_tokens_type_no: Vault::with_bucket(replacement_token_no_resource_def),
                    yes_counter: Decimal(0.0 as i128),
                    no_counter: Decimal(0.0 as i128),
                    proposal_admin: Vault::with_bucket(proposal_admin_badge),
                    needed_votes: needed_votes,
                    admin_adress: admin_adress,
                    end_epoch: end_epoch,
                }.instantiate()
                }

                pub fn retrive_voting_tokens(&mut self, replacement_tokens: Bucket) -> Bucket {
                    //TODO burn replacement tokes instead of putting them back
                    let amount = replacement_tokens.amount();
                    self.replacement_tokens_type_yes.put(replacement_tokens);
                    // send shares from vault
                    self.company_voting_token.take(amount)

                }

                /// Allows the user to vote on an issue using his voting tokens which will be locked
                pub fn vote(&mut self, vote: bool, voting_tokens: Bucket) -> Bucket {
                    // get amount of voting tokens
                    let amount = voting_tokens.amount();
                    // Lock voting tokens in Vault
                    self.company_voting_token.put(voting_tokens);
                    // Increase correct counter & prepare replacement_tokens
                    let replacement_tokens;
                    if vote {self.yes_counter += amount; replacement_tokens =  self.replacement_tokens_type_yes.take(amount);}
                    else {self.no_counter += amount; replacement_tokens =  self.replacement_tokens_type_no.take(amount);};
                    // Send replacement_tokens to adress
                    replacement_tokens

                }

               /*  pub fn trySuccess(){
                    //If yes > 51
                    // if no >51
                    // if time is over (like no)
                    // (Either way terminate struct)
                    // Otherwise: nothing happens
                }
                //Additional needed Methods
                //on_failure (auto-send-back. Not needed for prototype)
                //(update_needed_votes = to fix vulnerability "buy a lot of shares to win vote" */

            }

        }
