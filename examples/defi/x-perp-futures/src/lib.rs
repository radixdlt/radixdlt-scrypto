use sbor::*;
use scrypto::prelude::*;

blueprint! {
    struct ClearingHouse {
        /// All traders' positions
        trader_positions: LazyMap<Address, Vec<Position>>,
        /// Deposit vault
        deposits_in_quote: Vault,
        /// Liquidation threshold
        liquidation_threshold: Decimal,
        /// Virtual AMM
        amm: AMM,
    }

    impl ClearingHouse {
        pub fn instantiate_clearing_house(
            quote_address: Address,
            base_init_supply: Decimal,
            quote_init_supply: Decimal,
        ) -> Component {
            Self {
                trader_positions: LazyMap::new(),
                deposits_in_quote: Vault::new(quote_address),
                liquidation_threshold: "0.06".parse().unwrap(),
                amm: AMM {
                    base_supply: base_init_supply,
                    quote_supply: quote_init_supply,
                },
            }
            .instantiate()
        }

        /// Creates a position.
        pub fn new_position(
            &mut self,
            user_auth: BucketRef,
            margin: Bucket,
            leverage: Decimal,
            position_type: String, // TODO: make CLI support enum
        ) {
            assert!(leverage >= 1.into() && leverage <= 16.into());
            let user_id = Self::get_user_id(user_auth);
            let position_type = match position_type.as_str() {
                "Long" => PositionType::Long,
                "Short" => PositionType::Short,
                _ => panic!("Invalid position type"),
            };

            let margin_amount = margin.amount();
            let position = self
                .amm
                .new_position(margin_amount, leverage, position_type);
            let mut positions = self.trader_positions.get(&user_id).unwrap_or(Vec::new());
            positions.push(position);

            self.trader_positions.insert(user_id, positions);
            self.deposits_in_quote.put(margin);
        }

        /// Settles a position.
        pub fn settle_position(&mut self, user_auth: BucketRef, nth: usize) -> Bucket {
            let user_id = Self::get_user_id(user_auth);
            self.settle_internal(user_id, nth)
        }

        /// Liquidate a position.
        pub fn liquidate(&mut self, user_id: Address, nth: usize) -> Bucket {
            assert!(
                self.get_margin_ratio(user_id, nth) <= self.liquidation_threshold,
                "Position can't be liquidated"
            );

            self.settle_internal(user_id, nth)
        }

        /// Returns the running price.
        pub fn get_price(&self) -> Decimal {
            self.amm.get_price()
        }

        /// Returns the n-th position of a user
        pub fn get_position(&self, user_id: Address, nth: usize) -> Position {
            let positions = self.trader_positions.get(&user_id).unwrap();
            positions.get(nth).unwrap().clone()
        }

        /// Returns the margin ratio of a specific position
        pub fn get_margin_ratio(&self, user_id: Address, nth: usize) -> Decimal {
            let position = self.get_position(user_id, nth);
            self.amm.get_margin_ratio(&position)
        }

        /// Donates into this protocol.
        pub fn donate(&mut self, donation: Bucket) {
            self.deposits_in_quote.put(donation);
        }

        /// Registers a new user
        pub fn new_user(&self) -> Bucket {
            ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "xPerpFutures User Badge")
                .initial_supply_fungible(1)
        }

        /// Parse user id from a bucket ref.
        fn get_user_id(user_auth: BucketRef) -> Address {
            assert!(user_auth.amount() > 0.into(), "Invalid user proof");
            user_auth.resource_address()
        }

        fn settle_internal(&mut self, user_id: Address, nth: usize) -> Bucket {
            let mut positions = self.trader_positions.get(&user_id).unwrap();
            let position = positions.get(nth).unwrap();

            let pnl = self.amm.settle_position(position);
            debug!(
                "Margin: {}, PnL: {}, Vault balance: {}",
                position.margin_in_quote,
                pnl,
                self.deposits_in_quote.amount()
            );
            let to_return = position.margin_in_quote + pnl;

            positions.swap_remove(nth);
            self.trader_positions.insert(user_id, positions);
            self.deposits_in_quote.take(to_return)
        }
    }
}

#[derive(Debug, Clone, TypeId, Encode, Decode, Describe, PartialEq, Eq)]
pub enum PositionType {
    Long,
    Short,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, Describe, PartialEq, Eq)]
pub struct Position {
    /// The position type, either long or short
    pub position_type: PositionType,
    /// The initial margin in quote, always positive
    pub margin_in_quote: Decimal,
    /// The leverage
    pub leverage: Decimal,
    /// The position in base, positive for long and negative for short
    pub position_in_base: Decimal,
}

#[derive(TypeId, Encode, Decode)]
struct AMM {
    /// Supply of base asset
    base_supply: Decimal,
    /// Supply of quote asset
    quote_supply: Decimal,
}

impl AMM {
    /// Creates a new position.
    pub fn new_position(
        &mut self,
        margin_in_quote: Decimal,
        leverage: Decimal,
        position_type: PositionType,
    ) -> Position {
        // Calculate the new quote & base supply
        let k = self.base_supply * self.quote_supply;
        let new_quote_supply = if position_type == PositionType::Long {
            self.quote_supply + margin_in_quote * leverage
        } else {
            self.quote_supply - margin_in_quote * leverage
        };
        let new_base_supply = k / new_quote_supply;

        // Calculate the position received and commit changes
        let position_in_base = self.base_supply - new_base_supply;
        self.quote_supply = new_quote_supply;
        self.base_supply = new_base_supply;

        Position {
            position_type,
            margin_in_quote,
            leverage,
            position_in_base,
        }
    }

    /// Settles a position and returns the PnL
    pub fn settle_position(&mut self, position: &Position) -> Decimal {
        let pnl = self.get_pnl(position);

        let k = self.base_supply * self.quote_supply;
        self.base_supply += position.position_in_base;
        self.quote_supply = k / self.base_supply;

        pnl
    }

    /// Returns the current price of pair BASE/QUOTE
    pub fn get_price(&self) -> Decimal {
        self.quote_supply / self.base_supply
    }

    /// Returns the margin ratio of a position
    pub fn get_margin_ratio(&self, position: &Position) -> Decimal {
        (position.margin_in_quote + self.get_pnl(position))
            / (self.get_price() * position.position_in_base.abs())
    }

    /// Returns the profit and loss of a position
    pub fn get_pnl(&self, position: &Position) -> Decimal {
        // Calculate the new quote & base supply
        let k = self.base_supply * self.quote_supply;
        let new_base_supply = self.base_supply + position.position_in_base;
        let new_quote_supply = k / new_base_supply;

        // Calculate PnL
        let delta_in_quote = self.quote_supply - new_quote_supply;
        if position.position_type == PositionType::Long {
            delta_in_quote - position.margin_in_quote * position.leverage
        } else {
            delta_in_quote + position.margin_in_quote * position.leverage
        }
    }
}
