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
    pub fn take_call_on_xrd(&mut self, additional_usd: u32) -> u32 {
        let initialxrd = self.xrd;
        let initialusdt = self.usdt;
        self.usdt = initialusdt + additional_usd;
        self.xrd = self.k / self.usdt;
        let n_quantiy = initialxrd - self.xrd;
        return n_quantiy;
    }

    pub fn settle_call_on_xrd(&mut self, settle_pstn: &Position) -> u32 {
        let initialxrd = self.xrd;
        let initialusdt = self.usdt;
        self.xrd = initialxrd + settle_pstn.n_quantity;
        let profit_n_loss =
            initialusdt - (self.k / self.xrd) - (settle_pstn.margin_amount * settle_pstn.leverage);
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
        mm: VMM
    }

    impl PerpF {
        pub fn new(x_usd_addrs : Address ) -> Component {

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            Self {
                all_trader_pstns: HashMap::new(),
                deposited_usd: Vault::new(x_usd_addrs),
                mm: VMM {
                    k: 1000000,
                    xrd: 100,
                    usdt: 10000,
                }
            }
            .instantiate()
        }


        pub fn take_call_position(&mut self,
             margin_amount:Bucket,
             trader_account: Address,
             leverage:u32) {
                let margin_recieved : Amount = margin_amount.amount();
                let pstn_amount = margin_recieved * leverage;
                assert!(margin_recieved!= 0.into());
                self.deposited_usd.put(margin_amount);
                let n_quatity = self.mm.take_call_on_xrd(pstn_amount.as_u32());
                let new_pos = Position {
                    position_type: String::from("call"),
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

        pub fn settle_call_position(&mut self,trader_account: Address,n_quantity:u32){


            match self.all_trader_pstns.entry(trader_account) {
                Entry::Occupied(mut pstns) =>  {
                    let index = pstns.get().iter().position(|x| x.wallet_id == trader_account && x.n_quantity == n_quantity);
                    match index{
                        Some(i) => {
                            self.mm.settle_call_on_xrd(pstns.get().get(i).unwrap());
                            pstns.get_mut().remove(i);},
                        _=> println!("Error finding position for wallet ")
                    }
                }
                Entry::Vacant(e) => {
                    println!("Error finding position for wallet ")
                }
            }     
        }

    }
}
