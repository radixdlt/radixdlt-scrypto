use sbor::*;
use scrypto::prelude::*;

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
        let nQuantiy = initialxrd - self.xrd;
        return nQuantiy;
    }

    pub fn settle_call_on_xrd(&mut self, settlePstn: Position) -> u32 {
        let initialxrd = self.xrd;
        let initialusdt = self.usdt;
        self.xrd = initialxrd + settlePstn.nQuantity;
        let PnL =
            initialusdt - (self.k / self.xrd) - (settlePstn.marginAmount * settlePstn.leverage);
        self.usdt = self.k / self.xrd;
        return PnL;
    }
}

#[derive(TypeId, Encode, Decode, Clone)]
struct Position {
    positionType: String,
    marginAmount: u32,
    wallet_id: Address,
    leverage: u32,
    nQuantity: u32,
}

impl Position {
    pub fn addTraderPstns(
        traderPstns: &mut Vec<Position>,
        marginRecieved: u32,
        traderAccount: Address,
        leverage: u32,
        positionType: String,
        nQuatity: u32,
    ) -> &mut Vec<Position> {
        let newPos = Position {
            positionType: String::from("call"),
            marginAmount: marginRecieved,
            wallet_id: traderAccount,
            leverage: leverage,
            nQuantity: nQuatity,
        };
        traderPstns.push(newPos);
        return traderPstns;
    }

    pub fn getTraderExistingPstns(
        wallet_id: Address,
        allTraderPstns:  HashMap<Address, Vec<Position>>,
    ) ->  Vec<Position> {
        let trader = wallet_id;
        match allTraderPstns.get(&trader) {
            Some(pstns) => return *pstns,
            _ => {
                let mut traderPstns: Vec<Position> = Vec::new();
                return traderPstns;
            }
        }
    }

    pub fn findTraderPstn(
        wallet_id: Address,
        nQuantity: u32,
        allTraderPstns: &mut HashMap<Address, Vec<Position>>,
    ) -> Option<&mut Position> {
        let trader = wallet_id;
        let mut traderPstns: Vec<Position>;
        match allTraderPstns.get(&trader) {
            Some(custPstns) => {
                let toReturn = custPstns
                    .iter_mut()
                    .find(|x| x.wallet_id == trader && x.nQuantity == nQuantity);
                return toReturn;
            }
            _ => return None,
        }
    }
}

blueprint! {
    struct ClearingHouse {
        allTraderPstns: HashMap<Address,Vec<Position>>,
        depositedUsd: Vault,
        mm: VMM
    }

    impl PerpF {
        pub fn new(xUsdAddrs : Address ) -> Component {

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            Self {
                allTraderPstns: HashMap::new(),
                depositedUsd: Vault::new(xUsdAddrs),
                mm: VMM {
                    k: 1000000,
                    xrd: 100,
                    usdt: 10000,
                }
            }
            .instantiate()
        }


        pub fn take_call_position(&mut self,
             marginAmount:Bucket,
             traderAccount: Address,
             leverage:u32,positionType:String) {
                let marginRecieved : Amount = marginAmount.amount();
                let pstnAmount = marginRecieved * leverage;
                assert!(marginRecieved!= 0.into());
                self.depositedUsd.put(marginAmount);
                let nQuatity = self.mm.take_call_on_xrd(pstnAmount.as_u32());
                let mut traderPstns = Position::getTraderExistingPstns(traderAccount,self.allTraderPstns.clone());
                let pstns = Position::addTraderPstns(&mut traderPstns, marginRecieved.as_u32(), traderAccount, leverage, positionType, nQuatity);
                self.allTraderPstns.insert(traderAccount,pstns.to_vec());
        }

        pub fn settle_call_position(&mut self,traderAccount: Address,nQuantity:u32){
            let psnt_to_settle = Position::findTraderPstn(traderAccount,nQuantity,&mut self.allTraderPstns);
            match psnt_to_settle{
                Some(pstn)=> {            
                    self.mm.settle_call_on_xrd(pstn.clone());
                },
                _ => {
                    println!("Position not found")
                }
            }
        }

    }
}
