use sbor::*;
use scrypto::prelude::*;

#[derive(TypeId, Decode, Encode, Describe, NftData)]
pub struct Vaccine {
    name: String,
    epoch_taken: u64
}

#[derive(NftData)]
pub struct Passport {
    #[scrypto(mutable)]
    vaccines: Vec<Vaccine>
}

blueprint! {
    struct VaccinePassport {
        admin_def: ResourceDef,
        // Token that the this component use to mint and update
        // vaccine passports NFTs
        passport_manager_badge: Vault,
        // Resource definition of the NFTs
        passport_def: ResourceDef,
        // Number of passports minted
        nb_passports: u128
    }

    impl VaccinePassport {
        pub fn new() -> (Component, Bucket) {
            let passport_manager_badge: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                                    .metadata("name", "Vaccine Passport Manager")
                                                    .initial_supply_fungible(1);

            // Define the admin badge
            let admin_badge: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "VaccinePassport Admin Badge")
                .initial_supply_fungible(1);

            // Define the VaccinePassport NFT.
            // Specify that the admin_badge can mint, burn and update the metadata of the tokens
            let passport: ResourceDef = ResourceBuilder::new_non_fungible()
                            .metadata("name", "Vaccine Passport")
                            .flags(MINTABLE | BURNABLE | INDIVIDUAL_METADATA_MUTABLE | RECALLABLE)
                            .badge(
                                passport_manager_badge.resource_def(),
                                MAY_MINT | MAY_BURN | MAY_CHANGE_INDIVIDUAL_METADATA | MAY_RECALL
                            )
                            .no_initial_supply();

            let component = Self {
                admin_def: admin_badge.resource_def(),
                passport_manager_badge: Vault::with_bucket(passport_manager_badge),
                passport_def: passport,
                nb_passports: 0
            }.instantiate();

            (component, admin_badge)
        }

        #[auth(admin_def)]
        pub fn cancel_badge(&self, id: u128) {
            // todo: We can't recall badges yet but this could be a cool feature for this example.
        }

        // Allow people to create a new empty vaccine passport
        pub fn get_new_passport(&mut self) -> Bucket {
            // Mint a new NFT with empty array of vaccines
            let passport = self.passport_manager_badge.authorize(|badge| {
                self.passport_def.mint_nft(self.nb_passports, Passport{vaccines: Vec::new()}, badge)
            });
            
            self.nb_passports += 1;

            passport
        }

        // Update the provided passport NFT with the vaccine data
        pub fn get_vaccine(&self, passport: Bucket) -> Bucket {
            // Make sure the passed bucket is valid
            assert!(passport.amount() > Decimal::zero(), "Missing passport");
            assert!(passport.resource_def() == self.passport_def, "Wrong passport. Create one with `get_new_passport`");

            // Add the vaccine data to the passport
            let mut data: Passport = passport.get_nft_data(passport.get_nft_id());
            data.vaccines.push(Vaccine{
                name: "ScryptoZeneca".to_owned(),
                epoch_taken: Context::current_epoch()
            });

            // Update the NFT data with the new array of vaccines
            self.passport_manager_badge.authorize(|badge| {
                passport.update_nft_data(passport.get_nft_id(), data, badge);
            });

            // Return the passport back to the caller
            passport
        }

        // Display the information on the taken vaccines
        pub fn display_vaccine_data(&self, passport: BucketRef) {
            // Make sure the passed bucket is valid
            assert!(passport.amount() > Decimal::zero(), "Missing passport");
            assert!(passport.resource_def() == self.passport_def, "Wrong passport. Create one with `get_new_passport`");

            let data: Passport = self.passport_def.get_nft_data(passport.get_nft_id());
            passport.drop();

            info!("Vaccines you have taken:");
            for vaccine in data.vaccines {
                info!("Vaccine {} taken on epoch {}", vaccine.name, vaccine.epoch_taken);
            }
        }
    }
}
