use sbor::*;
use scrypto::prelude::*;

#[derive(TypeId, Encode, Decode, Describe)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

#[derive(TypeId, Encode, Decode, Describe)]
pub enum Class {
    Land,
    Creature,
    Artifact,
    Enchantment,
    Planeswalker,
    Sorcery,
    Instant,
}

#[derive(TypeId, Encode, Decode, Describe)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    MythicRare,
}

#[derive(TypeId, Encode, Decode, Describe)]
pub struct MagicCard {
    color: Color,
    class: Class,
    rarity: Rarity,
}

blueprint! {
    struct HelloNft {
        /// A vault that holds all our special cards
        special_cards: Vault,
        /// The price for each special card
        special_card_prices: HashMap<u128, Decimal>,
        /// A vault that holds the mint badge
        random_card_mint_badge: Vault,
        /// The resource definition of all random cards
        random_card_resource_def: ResourceDef,
        /// The price of each random card
        random_card_price: Decimal,
        /// A counter for ID generation
        random_card_id_counter: u128,
        /// A vault that collects all XRD payments
        collected_xrd: Vault,
    }

    impl HelloNft {
        pub fn new() -> Component {
            // Creates a fixed set of NFTs
            let special_cards_bucket = ResourceBuilder::new()
                .metadata("name", "Russ' Magic Card Collection")
                .new_nft_fixed(BTreeMap::from([
                    (
                        1,
                        MagicCard {
                            color: Color::Black,
                            class: Class::Sorcery,
                            rarity: Rarity::MythicRare,
                        },
                    ),
                    (
                        2,
                        MagicCard {
                            color: Color::Green,
                            class: Class::Planeswalker,
                            rarity: Rarity::Rare,
                        },
                    ),
                    (
                        3,
                        MagicCard {
                            color: Color::Red,
                            class: Class::Creature,
                            rarity: Rarity::Uncommon,
                        },
                    ),
                ]));

            // Create an NFT resource with mutable supply
            let random_card_mint_badge_badge = ResourceBuilder::new()
                .metadata("name", "Random Cards Mint Badge")
                .new_badge_fixed(1);
            let random_card_resource_def = ResourceBuilder::new()
                .metadata("name", "Random Cards")
                .new_nft_mutable(ResourceAuthConfigs::new(
                    random_card_mint_badge_badge.resource_def(),
                ));

            // Instantiate our component
            Self {
                special_cards: Vault::with_bucket(special_cards_bucket),
                special_card_prices: HashMap::from([
                    (1, 500.into()),
                    (2, 666.into()),
                    (3, 123.into()),
                ]),
                random_card_mint_badge: Vault::with_bucket(random_card_mint_badge_badge),
                random_card_resource_def,
                random_card_price: 50.into(),
                random_card_id_counter: 0,
                collected_xrd: Vault::new(RADIX_TOKEN),
            }
            .instantiate()
        }

        pub fn buy_special_card(&mut self, id: u128, payment: Bucket) -> (Bucket, Bucket) {
            // Take our price out of the payment bucket
            let price = self.special_card_prices.remove(&id).unwrap();
            self.collected_xrd.put(payment.take(price));

            // Take the requested NFT
            let nft = self.special_cards.take_nft(id);

            // Return the NFT and change
            (nft, payment)
        }

        pub fn buy_random_card(&mut self, payment: Bucket) -> (Bucket, Bucket) {
            // Take our price out of the payment bucket
            self.collected_xrd.put(payment.take(self.random_card_price));

            // Mint a new card
            let random_seed = 100; // TODO: obtain from oracle
            let new_card = MagicCard {
                color: Self::random_color(random_seed),
                class: Self::random_class(random_seed),
                rarity: Self::random_rarity(random_seed),
            };
            let nft = self.random_card_mint_badge.authorize(|auth| {
                self.random_card_resource_def
                    .mint_nft(self.random_card_id_counter, new_card, auth)
            });
            self.random_card_id_counter += 1;

            // Return the NFT and change
            (nft, payment)
        }

        pub fn get_available_special_cards(&self) -> BTreeSet<u128> {
            self.special_cards.get_nft_ids()
        }

        pub fn fuse_my_cards(&self, nfts: Bucket) -> Bucket {
            assert!(
                nfts.amount() == 2.into(),
                "You need to pass 2 NFTs for fusion"
            );
            assert!(
                nfts.resource_def() == self.random_card_resource_def,
                "Only random cards can be fused"
            );

            // Get the NFT IDs
            let nft_ids: Vec<u128> = nfts.get_nft_ids().iter().cloned().collect();

            // Generate a new card based on the provided two.
            let card1: MagicCard = nfts.get_nft_data(nft_ids[0]);
            let card2: MagicCard = nfts.get_nft_data(nft_ids[1]);
            let new_card = Self::fuse_magic_cards(card1, card2);

            // Burn the second card
            self.random_card_mint_badge.authorize(|auth| {
                nfts.take_nft(nft_ids[1]).burn(auth);
            });

            // Update the first card
            self.random_card_mint_badge.authorize(|auth| {
                nfts.update_nft_data(nft_ids[0], new_card, auth);
            });

            nfts
        }

        pub fn fuse_magic_cards(card1: MagicCard, card2: MagicCard) -> MagicCard {
            // TODO introduce some cool fusion algorithm
            MagicCard {
                color: card1.color,
                class: card2.class,
                rarity: Rarity::MythicRare,
            }
        }

        fn random_color(seed: u64) -> Color {
            match seed % 5 {
                0 => Color::White,
                1 => Color::Blue,
                2 => Color::Black,
                3 => Color::Red,
                4 => Color::Green,
                _ => panic!(),
            }
        }

        fn random_class(seed: u64) -> Class {
            match seed % 7 {
                0 => Class::Land,
                1 => Class::Creature,
                2 => Class::Artifact,
                3 => Class::Enchantment,
                4 => Class::Planeswalker,
                5 => Class::Sorcery,
                6 => Class::Instant,
                _ => panic!(),
            }
        }

        fn random_rarity(seed: u64) -> Rarity {
            match seed % 4 {
                0 => Rarity::Common,
                1 => Rarity::Uncommon,
                2 => Rarity::Rare,
                3 => Rarity::MythicRare,
                _ => panic!(),
            }
        }
    }
}
