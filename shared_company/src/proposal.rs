use scrypto::prelude::*;
 // A proposal that unlocks funds (XRD). 
        //This fund can then be used to pay for services and products of other firms
blueprint! {      
            struct Proposal{cost: Bucket, destination_adress_funds: String, reason: String,
                        company_voting_token: Vault,
                        replacement_tokens: Vault,
                        yes_counter: Decimal,
                        no_counter: Decimal,
                        proposal_admin: Vault,
                        needed_votes: Decimal,
                        end_epoch: u32,
                    admin_adress: String}
            
                        impl Proposal{
             
            
                            pub fn new(destination_adress_funds: String, reason: String, admin_adress: String)-> Component {
            
                                  // The token that the user gets in exchange for their voting.
                                let replacement_token_resource_def = ResourceBuilder::new()
                                   .metadata("name", "Replacement token").metadata("symbol", "RT")
                                .new_token_fixed(1_000_000);
            
                                let proposal_admin_badge = ResourceBuilder::new().new_badge_mutable();
            
                              Self {
                                cost: cost, destination_adress_funds: destination_adress_funds, reason: reason,
                                    company_voting_token : Vault.with_bucket(),
                                    replacement_tokens: Vault::with_bucket(replacement_token_resource_def),
                                    yes_counter: 0.0 as i128,
                                    no_counter: 0.0,
                                    proposal_admin: Vault.with_bucket(proposal_admin_badge),
                                    needed_votes: needed_votes,
                                    admin_adress: admin_adress,
                                    end_epoch: end_epoch,
                            }.instantiate()
                            }
            
                            pub fn retrive_voting_tokens(destination_adress_funds: String, replacement_tokens: Bucket){
                                // burn replacement tokens
                                // send shares from vault to destination adress 
                            }
            
                            pub fn vote(vote: bool, voting_tokens: Bucket, destination_adress: String){
                                // Increase yes_counter by voting_tokens.amount()
                                // Send replacement_tokens to adress
                            }
            
                            pub fn trySuccess(){
                                //If yes > 51
                                // if no >51
                                // if time is over (like no)
                                // (Either way terminate struct)
                                // Otherwise: nothing happens
                            }
                            //Additional needed Methods
                            //on_failure (auto-send-back. Not needed for prototype)
                            //(update_needed_votes = to fix vulnerability "buy a lot of shares to win vote"
                            
                        }
            
                    }
                