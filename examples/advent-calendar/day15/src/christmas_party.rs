use sbor::*;
use scrypto::prelude::*;

#[derive(TypeId, Decode, Encode, Describe, NftData)]
pub struct Vacine {
    name: String,
    epoch_taken: u64
}

#[derive(NftData)]
pub struct Passport {
    #[scrypto(mutable)]
    vacines: Vec<Vacine>
}

blueprint! {
    struct ChristmasParty {
        // Resource Definition of the passport NFT
        passport_nft_def: ResourceDef
    }

    impl ChristmasParty {
        pub fn new(passport_nft_def: Address) -> Component {
            Self {
                passport_nft_def: passport_nft_def.into()
            }.instantiate()
        }

        pub fn enter_party(&self, vacine_passport: BucketRef) {
            assert!(vacine_passport.amount() > Decimal::zero(), "Missing passport");
            assert!(vacine_passport.resource_def() == self.passport_nft_def, "Wrong passport !");

            let data: Passport = self.passport_nft_def.get_nft_data(vacine_passport.get_nft_id());
            vacine_passport.drop();
            
            if data.vacines.len() > 0 {
                info!("Come in !");
            } else {
                info!("You are not authorized to come in.")
            }
        }
    }
}
