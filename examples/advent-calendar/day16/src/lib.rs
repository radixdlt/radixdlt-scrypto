use scrypto::prelude::*;
use sbor::*;
use std::fmt;

// Define the attributes of the NFT
#[derive(TypeId, Encode, Decode, Describe, Debug)]
pub enum Head {
    None,
    Cap,
    ChristmasHat,
    Crown
}

#[derive(TypeId, Encode, Decode, Describe, Debug)]
pub enum Clothing {
    WhiteShirt,
    Hoodie,
    WinterCoat,
    SantaCoat
}

#[derive(TypeId, Encode, Decode, Describe, Debug)]
pub enum Mouth {
    Smile,
    TongueOut,
    Sad,
    GoldenTeeths,
    DiamondTeeths
}

#[derive(TypeId, Encode, Decode, Describe, Debug)]
pub enum Nose {
    Regular,
    Clown,
    Runny
}

#[derive(TypeId, Encode, Decode, Describe, Debug)]
pub enum Eyewear {
    None,
    ReadingGlasses,
    SunGlasses,
    EyePatch
}

#[derive(TypeId, Encode, Decode, Describe, Debug)]
pub enum Background {
    White,
    Blue,
    Gold,
    Diamond,
    Space
}

#[derive(NftData)]
pub struct DegenerateElf {
    head: Head,
    clothing: Clothing,
    mouth: Mouth,
    nose: Nose,
    eye_wear: Eyewear,
    background: Background,
    color: usize
}

impl fmt::Display for DegenerateElf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Head: {:?}\nClothing: {:?}\nMouth: {:?}\nNose: {:?}\nEyes: {:?}\nBackground: {:?}\nColor: {:?}\n", self.head, self.clothing, self.mouth, self.nose, self.eye_wear, self.background, self.color)
    }
}

blueprint! {
    struct DegenerateElves {
        // Store the badge that allows
        // the component to mint new elves
        mint_badge: Vault,
        // Resource definition of the elf NFT
        elf_def: ResourceDef,
        // Vault to store the payments
        payment_vault: Vault,
        // Cost of minting one NFT
        mint_price: Decimal,
        // Maximum supply of the nfts
        max_supply: u128,
        // Keep track of the number of elves minted
        nb_minted: u128,

        // Used for randomness
        random_seed: u64
    }

    impl DegenerateElves {
        // Instantiate a new DegenerateElves component with a
        // minting cost and a max supply
        pub fn new(mint_price: Decimal, max_supply: u128) -> Component {
            // Create the elf minting badge
            let mint_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .metadata("name", "DegenerateElf Minting Badge")
                                .initial_supply_fungible(1);

            // Create the elf NFT definition
            let elf_def = ResourceBuilder::new_non_fungible()
                            .metadata("name", "Degenerate Elves")
                            .flags(MINTABLE)
                            .badge(mint_badge.resource_def(), MAY_MINT)
                            .no_initial_supply();

            Self {
                mint_badge: Vault::with_bucket(mint_badge),
                elf_def: elf_def,
                payment_vault: Vault::new(RADIX_TOKEN),
                mint_price: mint_price,
                max_supply: max_supply,
                nb_minted: 0,
                random_seed: Self::generate_seed()
            }.instantiate()
        }

        // Mint a new elf NFT. Requires a payment
        // Return the NFT and the change (if payment > mint_cost)
        pub fn mint(&mut self, payment: Bucket) -> (Bucket, Bucket) {
            assert!(payment.amount() >= self.mint_price, "Minting costs {}", self.mint_price);
            assert!(payment.resource_def() == RADIX_TOKEN.into(), "You can only pay in XRD");
            assert!(self.nb_minted <= self.max_supply, "Max supply reached !");

            self.payment_vault.put(payment.take(self.mint_price));

            // Mint a random Elf
            let elf_attributes = DegenerateElf{
                head: self.random_head(),
                clothing: self.random_clothing(),
                eye_wear: self.random_eye(),
                background: self.random_background(),
                nose: self.random_nose(),
                mouth: self.random_mouth(),
                color: self.random_color()
            };

            let elf = self.mint_badge.authorize(|badge| {
                self.elf_def.mint_nft(self.nb_minted, elf_attributes, badge)
            });

            self.nb_minted += 1;

            // Return the change and NFT back
            (payment, elf)
        }

        // Used to display information about your elf NFT
        pub fn display_info(&self, elves: BucketRef) {
            assert!(elves.amount() > Decimal::zero(), "Missing NFT !");
            assert!(elves.resource_def() == self.elf_def, "NFT definition not matching");

            for nft_id in elves.get_nft_ids() {
                let data: DegenerateElf = self.elf_def.get_nft_data(nft_id);
                info!("========");
                info!("{}", data)
            }
            
            elves.drop();
        }

        // The following methods are used to randomly generate an elf

        fn random_head(&mut self) -> Head {
            match self.random_number(0, 3) {
                0 => Head::Cap,
                1 => Head::ChristmasHat,
                2 => Head::Crown,
                3 => Head::None,
                _ => panic!()
            }
        }

        fn random_clothing(&mut self) -> Clothing {
            match self.random_number(0, 3) {
                0 => Clothing::Hoodie,
                1 => Clothing::SantaCoat,
                2 => Clothing::WhiteShirt,
                3 => Clothing::WinterCoat,
                _ => panic!()
            }
        }

        fn random_mouth(&mut self) -> Mouth {
            match self.random_number(0, 4) {
                0 => Mouth::Smile,
                1 => Mouth::Sad,
                2 => Mouth::TongueOut,
                3 => Mouth::GoldenTeeths,
                4 => Mouth::DiamondTeeths,
                _ => panic!()
            }
        }

        fn random_nose(&mut self) -> Nose {
            match self.random_number(0, 2) {
                0 => Nose::Regular,
                1 => Nose::Clown,
                2 => Nose::Runny,
                _ => panic!()
            }
        }

        fn random_eye(&mut self) -> Eyewear {
            match self.random_number(0, 3) {
                0 => Eyewear::None,
                1 => Eyewear::EyePatch,
                2 => Eyewear::ReadingGlasses,
                3 => Eyewear::SunGlasses,
                _ => panic!()
            }
        }

        fn random_background(&mut self) -> Background {
            match self.random_number(0, 4) {
                0 => Background::White,
                1 => Background::Blue,
                2 => Background::Gold,
                3 => Background::Space,
                4 => Background::Diamond,
                _ => panic!()
            }
        }

        fn random_color(&mut self) -> usize {
            self.random_number(0, 16777215)
        }

        // Generate the seed for random number generation
        fn generate_seed() -> u64 {
            let mut seed: u64 = 1;
            for byte in Context::transaction_signers()[0].to_vec().iter() {
                if (seed * *byte as u64) != 0 {
                    seed *= *byte as u64;
                }
            }
            seed
        }

        // Generate a random number
        // WARNING: DON'T USE THIS IN PRODUCTION !
        fn random_number(&mut self, min: i32, max: i32) -> usize {
            self.random_seed = ( ( 1664525 * self.random_seed ) + 1013904223 ) % 4294967296;
            let range : u64 = (max - min + 1).try_into().unwrap();
            let shift : u64 = min.try_into().unwrap();
            (self.random_seed % range + shift).try_into().unwrap()
        }
    }
}