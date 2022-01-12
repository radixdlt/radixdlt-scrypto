use scrypto::prelude::*;

#[derive(NftData)]
struct MemberData {
    name: String,
    #[scrypto(mutable)]
    good_member_points: Decimal,
    #[scrypto(mutable)]
    is_banned: bool
}

blueprint! {
    struct MembershipSystem {
        minter: Vault,
        contributions: Vault,
        member_nft_def: ResourceDef,
        nb_members: u64,
    }

    impl MembershipSystem {
        pub fn new() -> Component {
            // Minter badge, kept by the component
            // to mint/burn/update new member NFTs
            let minter = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                            .initial_supply_fungible(1);

            // Create the definition of the member NFT.
            // Declare the NFT as mintable, burnable, recallable and updatable by 
            // the minter
            let member_nft_def = ResourceBuilder::new_non_fungible()
                                .metadata("name", "Member NFT")
                                .flags(MINTABLE | BURNABLE | RECALLABLE | INDIVIDUAL_METADATA_MUTABLE)
                                .badge(minter.resource_def(), MAY_MINT | MAY_BURN | MAY_CHANGE_INDIVIDUAL_METADATA)
                                .no_initial_supply();

            Self {
                minter: Vault::with_bucket(minter),
                contributions: Vault::new(RADIX_TOKEN),
                member_nft_def: member_nft_def,
                nb_members: 0
            }
            .instantiate()
        }

        // Allow anyone to become a member of the DAO.
        // The component mints a badge representing the user.
        pub fn become_member(&mut self, name: String) -> Bucket {
            self.nb_members += 1;

            self.minter.authorize(|badge| {
                self.member_nft_def.mint_nft(self.nb_members.into(), MemberData{
                    name: name, 
                    good_member_points: Decimal::zero(),
                    is_banned: false
                }, badge)
            })
        }

        // Allow members to contribute XRD to the
        // DAO's vault. The member receive points based on
        // how much they give
        #[auth(member_nft_def)]
        pub fn contribute(&self, payment: Bucket) {
            let points = payment.amount();
            self.contributions.put(payment);

            // Add points to the nft metadata
            let mut nft_data: MemberData = self.member_nft_def.get_nft_data(auth.get_nft_id());
            assert!(!nft_data.is_banned, "You are banned from the DAO !");  
            nft_data.good_member_points += points;

            self.minter.authorize(|badge| {
                self.member_nft_def.update_nft_data(auth.get_nft_id(), nft_data, badge);
            });

            info!("Thank you ! You received {} points !", points);
        }

        // Allow members with more than 10000 points
        // to ban another member
        #[auth(member_nft_def)]
        pub fn ban_member(&mut self, nft_id: u128) {      
            let nft_data: MemberData = self.member_nft_def.get_nft_data(auth.get_nft_id());
            assert!(!nft_data.is_banned, "You are banned from the DAO !");
            assert!(nft_data.good_member_points >= 10000.into(), "You do not have enough points to ban another member !");

            let mut other_member_nft_data: MemberData = self.member_nft_def.get_nft_data(nft_id);
            other_member_nft_data.is_banned = true;
            self.minter.authorize(|badge| {
                self.member_nft_def.update_nft_data(nft_id, other_member_nft_data, badge);
            });
        }

        // Will be used by other components of the DAO to
        // know if a member is banned
        pub fn is_banned(&self, nft: Bucket) -> bool {
            assert!(nft.amount() > Decimal::zero(), "Missing nft");
            assert!(nft.resource_def() == self.member_nft_def, "Wrong nft");

            let data: MemberData = self.member_nft_def.get_nft_data(nft.get_nft_id());
            data.is_banned
        }

        // Will be used by other components of the DAO to
        // get the member NFT resource definition
        pub fn get_member_nft_def(&self) -> Address {
            self.member_nft_def.address()
        }
    }
}
