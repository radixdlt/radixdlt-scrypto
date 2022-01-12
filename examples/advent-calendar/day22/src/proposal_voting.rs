use scrypto::prelude::*;
use crate::membership_system::*;

#[derive(NftData)]
struct ProposalData {
    created_by_id: u128,
    title: String,
    description: String,
    created_at: u64,
    #[scrypto(mutable)]
    voted_by: Vec<Address>
}

blueprint! {
    struct ProposalVoting {
        proposals: Vault,
        proposal_minter: Vault,
        proposal_def: ResourceDef,
        nb_proposals: u128,
        membership_admin: Vault,
        membership_system: MembershipSystem,
        member_nft_def: ResourceDef
    }

    impl ProposalVoting {
        pub fn new() -> Component {
            // Badge allowed to mint new proposal NFTs
            let proposal_minter = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                    .initial_supply_fungible(1);

            // Create proposal NFT definition
            let proposal_definition = ResourceBuilder::new_non_fungible()
                                        .metadata("name", "Proposal")
                                        .flags(MINTABLE | INDIVIDUAL_METADATA_MUTABLE)
                                        .badge(proposal_minter.resource_def(), MAY_MINT | MAY_CHANGE_INDIVIDUAL_METADATA)
                                        .no_initial_supply();

            // Instantiate the membership system component
            let (membership_system_component, admin_badge): (Component, Bucket) = MembershipSystem::new();
            let membership_system: MembershipSystem = membership_system_component.into();
            let member_nft_def = membership_system.get_member_nft_def();

            Self {
                proposals: Vault::new(proposal_definition.address()),
                proposal_minter: Vault::with_bucket(proposal_minter),
                proposal_def: proposal_definition,
                nb_proposals: 0,
                membership_admin: Vault::with_bucket(admin_badge),
                membership_system: membership_system,
                member_nft_def: member_nft_def.into()
            }
            .instantiate()
        }

        // As a member, create a new proposal with
        // provided title and description
        #[auth(member_nft_def)]
        pub fn create_proposal(&mut self, title: String, description: String) {
            let proposal =self.proposal_minter.authorize(|badge| {
                self.proposal_def.mint_nft(self.nb_proposals, ProposalData {
                    created_by_id: auth.get_nft_id(),
                    title: title, 
                    description: description, 
                    voted_by: Vec::new(),
                    created_at: Context::current_epoch()
                }, badge)
            });

            self.nb_proposals += 1;
            self.proposals.put(proposal);
        }

        // As a member, vote for a proposal with
        // provided id
        #[auth(member_nft_def)]
        pub fn vote_on_proposal(&self, proposal_id: u128) {
            let mut nft_data: ProposalData = self.proposal_def.get_nft_data(proposal_id);

            // Make sure that the member voting is not he
            // one that created the proposal and that they have not already
            // voted on it.
            assert!(nft_data.created_by_id != auth.get_nft_id(), "You can't vote on your own proposal");
            assert!(!nft_data.voted_by.contains(&auth.resource_address()), "Already voted for that proposal !");

            // Add the member id to the list of votes
            nft_data.voted_by.push(auth.resource_address());

            // Update the NFT's data
            self.proposal_minter.authorize(|badge| {
                self.proposal_def.update_nft_data(proposal_id, nft_data, badge);
            })
        }

        // List all the proposals and the amount of 
        // votes they have
        pub fn list_proposals(&self) {
            info!("==== Proposals =====");
            for i in 0..self.nb_proposals {
                let data: ProposalData = self.proposal_def.get_nft_data(i);
                info!("Title: {}", data.title);
                info!("Description: {}", data.description);
                info!("Nb votes: {}", data.voted_by.len());
            }
        }
    }
}
