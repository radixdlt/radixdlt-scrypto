use scrypto::prelude::*;
use crate::membership_system::*;

blueprint! {
    struct ElectionSystem {
        membership_admin_badge: Vault,
        membership_component: MembershipSystem,
        membership_nft_def: ResourceDef,
        current_leader_id: Option<u128>,
        votes: HashMap<u128, u64>,
        election_duration: u64,
        election_deadline: u64,
        who_voted: Vec<u128>,
        election_decided: bool
    }

    impl ElectionSystem {
    
        pub fn new(election_duration: u64) -> Component {
            // Setup the membership system component
            let (membership_system_component, admin_badge): (Component, Bucket) = MembershipSystem::new();
            let membership_system: MembershipSystem = membership_system_component.into();
            let member_nft_def = membership_system.get_member_nft_def();
            
            Self {
                membership_admin_badge: Vault::with_bucket(admin_badge),
                membership_component: membership_system,
                membership_nft_def: member_nft_def.into(),
                current_leader_id: None,
                votes: HashMap::new(),
                election_duration: election_duration,
                election_deadline: 0,
                who_voted: Vec::new(),
                election_decided: true
            }
            .instantiate()
        }

        // Start a new election
        #[auth(membership_nft_def)]
        pub fn start_election(&mut self) {
            assert!(Context::current_epoch() >= self.election_deadline && self.election_decided, "An election is already ongoing");
            self.election_deadline = Context::current_epoch() + self.election_duration;
            self.election_decided = false;
        }

        // As a member of the DAO, vote for who
        // should be the leader
        #[auth(membership_nft_def)]
        pub fn vote(&mut self, member_id: u128) {
            assert!(Context::current_epoch() < self.election_deadline, "Election not yet started !");
            assert!(!self.who_voted.contains(&auth.get_nft_id()), "You already voted !");

            // Make sure NFT with member_id exists
            self.membership_nft_def.get_nft_data::<MemberData>(member_id);

            // Increase the number of votes by one
            let existing_votes = *self.votes.entry(member_id).or_insert(0);
            self.votes.insert(member_id, existing_votes + 1);

            self.who_voted.push(auth.get_nft_id());
        }

        // Close the election and find the member with the
        // highest vote
        #[auth(membership_nft_def)]
        pub fn close_election(&mut self) {
            assert!(Context::current_epoch() >= self.election_deadline  && !self.election_decided, "The election has not ended yet.");
            // Find who won
            let mut highest_votes = -1;
            let mut highest_votes_member_id: Option<u128> = None;

            for (id, nb_votes) in self.votes.iter() {
                if *nb_votes as i128 > highest_votes {
                    highest_votes = *nb_votes as i128;
                    highest_votes_member_id = Some(*id);
                }
            }

            self.current_leader_id = highest_votes_member_id;

            // Clear data for next election
            self.who_voted.clear();
            self.votes.clear();
            self.election_decided = true;
        }

        // Display the current leader of the DAO
        pub fn get_current_leader(&self) {
            let member_data: MemberData = match self.current_leader_id {
                Some(id) => self.membership_nft_def.get_nft_data(id),
                None => {
                    info!("Not current leader !");
                    std::process::abort();
                }
            };
            
            info!("Current leader: {}", member_data.name);
        }
    }
}
