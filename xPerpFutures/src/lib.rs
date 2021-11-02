use sbor::*;
use scrypto::prelude::*;
use std::collections::hash_map::*;

#[derive(TypeId, Encode, Decode)]
struct VMM {
    k: u32,
    xrd: u32,
    usdt: u32,
}

impl VMM {
    pub fn take_call_on_xrd(&mut self, usd_pstn_amount: u32) -> u32 {
        let initialxrd = self.xrd;
        let initialusdt = self.usdt;
        self.usdt = initialusdt + usd_pstn_amount;
        self.xrd = self.k / self.usdt;
        let n_quantiy = initialxrd - self.xrd;
        return n_quantiy;
    }

    pub fn settle_call_on_xrd(&mut self, settle_pstn: &Position) -> i32 {
        let initialxrd = self.xrd;
        let initialusdt = self.usdt;
        self.xrd = initialxrd + settle_pstn.n_quantity;
        let profit_n_loss = initialusdt as i32
            - ((self.k / self.xrd) as i32)
            - (settle_pstn.margin_amount * settle_pstn.leverage) as i32;
        self.usdt = self.k / self.xrd;
        return profit_n_loss;
    }

    pub fn take_put_on_xrd(&mut self, usd_pstn_amount: u32) -> u32 {
        let initialxrd = self.xrd;
        let initialusdt = self.usdt;
        self.usdt = initialusdt - usd_pstn_amount;
        self.xrd = self.k / self.usdt;
        let n_quantiy = self.xrd - initialxrd;
        return n_quantiy;
    }

    pub fn settle_put_on_xrd(&mut self, settle_pstn: &Position) -> i32 {
        let initialxrd = self.xrd;
        let initialusdt = self.usdt;
        self.xrd = initialxrd - settle_pstn.n_quantity;
        let profit_n_loss = initialusdt as i32 - ((self.k / self.xrd) as i32)
            + (settle_pstn.margin_amount * settle_pstn.leverage) as i32;
        self.usdt = self.k / self.xrd;
        return profit_n_loss;
    }
}

#[derive(TypeId, Encode, Decode, Clone)]
struct Position {
    position_type: String,
    margin_amount: u32,
    wallet_id: Address,
    leverage: u32,
    n_quantity: u32,
}

blueprint! {
    struct ClearingHouse {
        all_trader_pstns: HashMap<Address,Vec<Position>>,
        deposited_usd: Vault,
        mm: VMM,
        transfer_vault: Vault
    }

    impl PerpF {
        pub fn new(x_usd_addrs : Address ) -> Component {

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            Self {
                all_trader_pstns: HashMap::new(),
                deposited_usd: Vault::new(x_usd_addrs),
                mm: VMM {
                    k: 100000000,
                    xrd: 1000,
                    usdt: 100000,
                },
                transfer_vault: Vault::new(x_usd_addrs)
            }
            .instantiate()
        }


        pub fn take_position(&mut self,
             margin_amount:Bucket,
             trader_account: Address,
             leverage:u32, position_type:String) {
                let margin_recieved : Amount = margin_amount.amount();
                let pstn_amount = margin_recieved * leverage;
                assert!(margin_recieved!= 0.into());
                self.deposited_usd.put(margin_amount);
                let n_quatity;
                match position_type.as_ref() {
                    "call" => n_quatity = self.mm.take_call_on_xrd(pstn_amount.as_u32()),
                    "put" => n_quatity = self.mm.take_put_on_xrd(pstn_amount.as_u32()),
                    _=> panic!("Invalid type of instrument. Either should be call or put ")
                }

                let new_pos = Position {
                    position_type: position_type,
                    margin_amount: margin_recieved.as_u32(),
                    wallet_id: trader_account,
                    leverage: leverage,
                    n_quantity: n_quatity,
                };

                match self.all_trader_pstns.entry(trader_account) {
                    Entry::Occupied(mut pstns) =>  {
                        pstns.get_mut().push(new_pos)
                    }
                    Entry::Vacant(e) => {
                        let mut newpstns: Vec<Position> = Vec::new();
                        newpstns.push(new_pos);
                        self.all_trader_pstns.insert(trader_account,newpstns);
                    }
                }
        }

        pub fn settle_position(&mut self,trader_account: Address,n_quantity:u32,position_type:String) -> Option<Bucket>  {


            match self.all_trader_pstns.entry(trader_account) {
                Entry::Occupied(mut pstns) =>  {
                    let index = pstns.get().iter().position(|x| x.wallet_id == trader_account && x.n_quantity == n_quantity);
                    match index{
                        Some(i) => {
                            let position : &Position = pstns.get().get(i).unwrap();
                            assert!(position.position_type == position_type);
                            let profit_n_loss;
                            match position_type.as_ref() {
                                "call" =>  profit_n_loss = self.mm.settle_call_on_xrd(pstns.get().get(i).unwrap()),
                                "put" =>   profit_n_loss= self.mm.settle_put_on_xrd(pstns.get().get(i).unwrap()),
                                _=> panic!("Invalid type of instrument. Either should be call or put ")
                            }
                            let margin_amount = pstns.get().get(i).unwrap().margin_amount as i32;
                            assert!(margin_amount >= profit_n_loss);
                            let temp_bucket = self.deposited_usd.take(margin_amount + profit_n_loss);
                            pstns.get_mut().remove(i);
                            return Some(temp_bucket);
                        },
                        _=> panic!("Error finding position for wallet ")
                    }
                }
                Entry::Vacant(e) => {
                    panic!("Error finding position for wallet ")
                }
            }
        }

        pub fn transfer_tokens_to_token_vault(&mut self, from: Address, amount:Bucket ) {
            self.transfer_vault.put(amount);
        }

        pub fn transfer_tokens_from_token_vault(&mut self, to: Address, amount: u32 ) -> Option<Bucket>  {
            return Some(self.transfer_vault.take(amount));
        }
    }
}
