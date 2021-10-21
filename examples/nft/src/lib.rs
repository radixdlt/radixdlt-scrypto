use sbor::*;
use scrypto::prelude::*;

#[derive(TypeId, Encode, Decode)]
pub enum Colour {
    Green, Red, Yellow, Gray, Black, White
}

#[derive(TypeId, Encode, Decode)]
pub enum Skill {
    Jump, Fly, EatScorpion, Swim
}

blueprint! {
    struct Radragon {
        owner_badge_addr: Address,
        level: u32,
        skin_colour: Colour,
        hair_colour: Colour,
        weight_kg: u32,
        height_cm: u32,
        skills: Vec<Skill>,
        owner: BucketRef,
    }

    impl Radragon {
        pub fn new(to_burn: Bucket) {
            
        }
    }
}
