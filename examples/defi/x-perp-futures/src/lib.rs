use sbor::*;
use scrypto::prelude::*;
use std::collections::hash_map::*;

#[derive(TypeId, Encode, Decode)]
struct VMM {
    k: Decimal,
    xrd: Decimal,
    usdt: Decimal,
}

impl VMM {
    pub fn take_call_on_xrd(&mut self, usd_pstn_amount: Decimal) -> Decimal {
        let initialxrd = self.xrd.clone();
        let initialusdt = self.usdt.clone();
        self.usdt = initialusdt + usd_pstn_amount;
        self.xrd = self.k.clone() / self.usdt.clone();
        let n_quantiy = initialxrd - self.xrd.clone();
        return n_quantiy;
    }

    pub fn settle_call_on_xrd(&mut self, settle_pstn: &Position) -> Decimal {
        let initialxrd = self.xrd.clone();
        let initialusdt = self.usdt.clone();
        self.xrd = initialxrd + settle_pstn.n_quantity.clone();
        let profit_n_loss =
            initialusdt - (self.k.clone() / self.xrd.clone()) - (settle_pstn.margin_amount.clone() * settle_pstn.leverage.clone());
        self.usdt = self.k.clone() / self.xrd.clone();
        return profit_n_loss;
    }

    pub fn take_put_on_xrd(&mut self, usd_pstn_amount: Decimal) -> Decimal {
        let initialxrd = self.xrd.clone();
        let initialusdt = self.usdt.clone();
        self.usdt = initialusdt - usd_pstn_amount;
        self.xrd = self.k.clone() / self.usdt.clone();
        let n_quantiy = self.xrd.clone() - initialxrd;
        return n_quantiy;
    }

    pub fn settle_put_on_xrd(&mut self, settle_pstn: &Position) -> Decimal {
        let initialxrd = self.xrd.clone();
        let initialusdt = self.usdt.clone();
        self.xrd = initialxrd - settle_pstn.n_quantity.clone();
        let profit_n_loss =
            initialusdt - (self.k.clone() / self.xrd.clone()) + settle_pstn.margin_amount.clone() * settle_pstn.leverage.clone();
        self.usdt = self.k.clone() / self.xrd.clone();
        return profit_n_loss;
    }

    pub fn running_xrd_price(&self) -> Decimal {
        return self.usdt.clone() / self.xrd.clone();
    }
}

#[derive(TypeId, Encode, Decode, Clone, Debug, Describe)]
struct Position {
    position_type: String,
    margin_amount: Decimal,
    wallet_id: Address,
    leverage: Decimal,
    n_quantity: Decimal,
}

impl Position {
    pub fn running_margin(position: &Position, running_xrd_price: Decimal) -> Decimal {
        info!("Running xrd price {}", running_xrd_price);
        return position.margin_amount.clone() / (running_xrd_price * position.n_quantity.clone());
    }
}

blueprint! {
    struct ClearingHouse {
        all_trader_pstns: HashMap<Address, Vec<Position>>,
        deposited_usd: Vault,
        mm: VMM,
        transfer_vault: Vault,
    }

    impl ClearingHouse {
        pub fn new(x_usd_addrs: Address) -> Component {
            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            Self {
                all_trader_pstns: HashMap::new(),
                deposited_usd: Vault::new(x_usd_addrs),
                mm: VMM {
                    k: 100000000.into(),
                    xrd: 1000.into(),
                    usdt: 100000.into(),
                },
                transfer_vault: Vault::new(x_usd_addrs),
            }
            .instantiate()
        }

        pub fn take_position(
            &mut self,
            margin_amount: Bucket,
            trader_account: Address,
            leverage: Decimal,
            position_type: String,
        ) {
            let margin_recieved: Decimal = margin_amount.amount();
            let pstn_amount = margin_recieved.clone() * leverage.clone();
            assert!(margin_recieved.clone() != 0.into());
            self.deposited_usd.put(margin_amount);
            let n_quatity;
            match position_type.as_ref() {
                "call" => n_quatity = self.mm.take_call_on_xrd(pstn_amount),
                "put" => n_quatity = self.mm.take_put_on_xrd(pstn_amount),
                _ => panic!("Invalid type of instrument. Either should be call or put "),
            }

            let new_pos = Position {
                position_type: position_type,
                margin_amount: margin_recieved,
                wallet_id: trader_account,
                leverage: leverage,
                n_quantity: n_quatity,
            };

            match self.all_trader_pstns.entry(trader_account) {
                Entry::Occupied(mut pstns) => pstns.get_mut().push(new_pos),
                Entry::Vacant(e) => {
                    let mut newpstns: Vec<Position> = Vec::new();
                    newpstns.push(new_pos);
                    self.all_trader_pstns.insert(trader_account, newpstns);
                }
            }
        }

        pub fn settle_position(
            &mut self,
            trader_account: Address,
            n_quantity: Decimal,
            position_type: String,
        ) -> Option<Bucket> {
            match self.all_trader_pstns.entry(trader_account) {
                Entry::Occupied(mut pstns) => {
                    let index = pstns
                        .get()
                        .iter()
                        .position(|x| x.wallet_id == trader_account && x.n_quantity == n_quantity);
                    match index {
                        Some(i) => {
                            let position: &Position = pstns.get().get(i).unwrap();
                            assert!(position.position_type == position_type);
                            let profit_n_loss;
                            match position_type.as_ref() {
                                "call" => {
                                    profit_n_loss =
                                        self.mm.settle_call_on_xrd(pstns.get().get(i).unwrap())
                                }
                                "put" => {
                                    profit_n_loss =
                                        self.mm.settle_put_on_xrd(pstns.get().get(i).unwrap())
                                }
                                _ => panic!(
                                    "Invalid type of instrument. Either should be call or put "
                                ),
                            }
                            let margin_amount = pstns.get().get(i).unwrap().margin_amount.clone();
                            assert!(margin_amount >= profit_n_loss);
                            let temp_bucket =
                                self.deposited_usd.take(margin_amount + profit_n_loss);
                            pstns.get_mut().remove(i);
                            return Some(temp_bucket);
                        }
                        _ => panic!("Error finding position for wallet "),
                    }
                }
                Entry::Vacant(e) => {
                    panic!("Error finding position for wallet ")
                }
            }
        }

        pub fn print_running_margins(&mut self) {
            for (address, positions) in &self.all_trader_pstns {
                for (i, position) in positions.iter().enumerate() {
                    let running_margin =
                        Position::running_margin(position, self.mm.running_xrd_price());
                    info!(
                        "Running margin {}% for account {:?}  ",
                        running_margin, position
                    );
                }
            }
        }

        pub fn liquidate(&mut self) {
            let positions = self.positions_to_liquidate();
            for position in positions {
                // TODO liquidate the i-th position of some account
            }
        }

        pub fn positions_to_liquidate(&self) -> Vec<(Address, usize)> {
            let mut result = Vec::new();

            for (address, positions) in &self.all_trader_pstns {
                for (i, position) in positions.iter().enumerate() {
                    let running_margin =
                        Position::running_margin(position, self.mm.running_xrd_price());
                    info!(
                        "Running margin {}% for account {:?}  ",
                        running_margin, position
                    );
                    if running_margin < Decimal::from_str("0.1").unwrap() {
                        result.push((*address, i));
                    }
                }
            }
            return result;
        }

        pub fn transfer_tokens_to_token_vault(&mut self, from: Address, amount: Bucket) {
            self.transfer_vault.put(amount);
        }

        pub fn transfer_tokens_from_token_vault(
            &mut self,
            to: Address,
            amount: u32,
        ) -> Option<Bucket> {
            return Some(self.transfer_vault.take(amount));
        }
    }
}
