use scrypto::prelude::*;
// A proposal that unlocks funds (XRD).
//This fund can then be used to pay for services and products of other firms
blueprint! {
struct Proposal{
            // Fund which are looked for the vote
            cost_vault: Vault,
            // If proposal is accepted: Funds are send to this adress
            destination_adress_funds: String,
            // What are users voting on?
            reason: String,
            // The token that is needed to vote.
            company_voting_token: Vault,
            // Material for voting. If the vault could count, yes and no counter wouldnt be needed
            replacement_tokens_type_yes: Vault,
            replacement_tokens_type_no: Vault,
            yes_counter: Decimal,
            no_counter: Decimal,
            // the admin can stop the proposal
            proposal_admin: Vault,
            // The votes needed for the vote to succeed
            needed_votes: Decimal,
            // When this epoch is reached and try_solve() is called,
            //no more voting will be possible and cost_vault will be send to owners address (not implemented yet)
            end_epoch: u64,
            fund_owner_adress: String,
    }

            impl Proposal{

                /// creates a new instance of a proposal
                pub fn new(cost: Bucket, destination_adress_funds: String, reason: String, admin_adress: String, end_epoch: u64,
                    needed_votes: Decimal, company_voting_token_resource_def: ResourceDef)-> Component {

                    // The token that the user gets in exchange for their "yes" voting.
                    let replacement_token_yes_resource_def = ResourceBuilder::new()
                       .metadata("name", "Replacement token yes").metadata("symbol", "RTY")
                    .new_token_fixed(1_000_000);

                     // The token that the user gets in exchange for their "no" voting.
                     let replacement_token_no_resource_def = ResourceBuilder::new()
                     .metadata("name", "Replacement token no").metadata("symbol", "RTN")
                  .new_token_fixed(1_000_000);

                    // This badge can be theoretically be used for burns
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
                    fund_owner_adress: admin_adress,
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
                /// Checks if a finish condition is reached
               pub fn try_solve(&mut self){
                   if self.yes_counter > self.needed_votes {
                    // ToDo send tokens to destination_adress.
                    //Problem: Account::from Str not working, cant find the correct type
                    /*  let acc = Account::from(self.destination_adress_funds);
                    acc.deposit(self.cost_vault.take_all()) */
                    let string = "testset";
                    let my_addr: Address::from_str(&string);
                    let acc = Account::from(my_addr);
                   }
                   if self.no_counter > self.needed_votes {
                    //ToDo send tokens to initial address
                   }
                   if Context::current_epoch() > self.end_epoch {
                    // Optimal: Send all tokens back to their owners.
                    // For now: Send funds back to owner, user can retrive their tokens with retrive_voting_tokens()
                   }
                }
                //Additional needed Methods
                //on_failure (auto-send-back. Not needed for prototype)
                //(update_needed_votes = to fix vulnerability "buy a lot of shares to win vote" */

            }

        }
