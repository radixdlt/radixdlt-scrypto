use scrypto::prelude::*;
use crate::membership_system::*;

blueprint! {
    struct FundsSplitter {
        admin_badge: Vault,
        membership_system: MembershipSystem,
        member_nft_def: ResourceDef,
        funds: Vault
    }

    impl FundsSplitter {
        pub fn new() -> Component {
            // Instantiate the membership system component
            let (membership_system_component, admin_badge): (Component, Bucket) = MembershipSystem::new();
            let membership_system: MembershipSystem = membership_system_component.into();

            let member_nft_def = membership_system.get_member_nft_def();

            Self {
                admin_badge: Vault::with_bucket(admin_badge),
                membership_system: membership_system,
                member_nft_def: member_nft_def.into(),
                funds: Vault::new(RADIX_TOKEN)
            }
            .instantiate()
        }

        // Add funds to the DAO and split the amount between 
        // all members
        pub fn add_funds(&mut self, payment: Bucket) {
            let nb_members = self.membership_system.get_nb_members();
            assert!(nb_members > 0, "No members to give the funds to !");

            // Split the funds equally between all members
            for i in 1..=nb_members {
                let mut nft_data: MemberData = self.member_nft_def.get_nft_data(i as u128);

                // Update the shares on the NFT
                nft_data.fund_share += payment.amount() / nb_members;
                self.admin_badge.authorize(|badge| {
                    self.member_nft_def.update_nft_data(i as u128, nft_data, badge);
                });
            }

            // Store payment in DAO's fund vault
            self.funds.put(payment);
        }

        // Allow members to withdraw their share of the funds
        #[auth(member_nft_def)]
        pub fn withdraw(&mut self) -> Bucket {
            // Fetch data on the NFT
            let mut nft_data: MemberData = self.member_nft_def.get_nft_data(auth.get_nft_id());

            // Make a bucket with the XRD to return to the caller
            let shares_to_return: Bucket = self.funds.take(nft_data.fund_share);

            // Set the shares to 0 on the NFT
            nft_data.fund_share = Decimal::zero();
            self.admin_badge.authorize(|badge| { 
                self.member_nft_def.update_nft_data(auth.get_nft_id(), nft_data, badge);
            });

            shares_to_return
        }
    }
}
