use scrypto::prelude::*;

blueprint! {

    // create a crowdfunded component
    // the component implements a fund for some purpose
    // component distributes units of the fund for a price
    // if time is up, then close the fund component 
    // if some units are left then return funds with a premium from the insurance = FAILURE


    struct InsuredCrowdFund {
        is_open: bool,
        is_success: bool,
        current_time: u32,
        time_limit: u32,

        unit_price: u32,
        unit_insurance: u32,

        fund_manager: Address,

        //founders: Vec<Address>, 
        fund_units: Vault,
        insurance_xrd: Vault,
        collected_xrd: Vault

    }

    impl InsuredCrowdFund {
        pub fn new() -> Address {

            let address = Address::from_str("06f53b41fc782ecda3ace4c0e1a154a47732e736f11980c3738f4b").unwrap();

            Self {
                is_open: true,
                is_success: false,
                
                current_time: 0,
                time_limit: 5,

                unit_price: 200,
                unit_insurance: 2,

                fund_manager: address,
                
                //founders: vec![],
                fund_units: Vault::wrap(ResourceBuilder::new().metadata("symbol", "unit").create_fixed(100)),
                collected_xrd: Vault::new(Address::RadixToken),
                insurance_xrd: Vault::new(Address::RadixToken)
            }
            .instantiate()
        }

        // simulate progression of rounds or hight
        pub fn tick(&mut self) {
            if self.current_time < self.time_limit {
                self.current_time += 1;
            } else {
                self.evaluate()
            }
            self.get_current_height();
        }

        pub fn get_current_height(&mut self) -> u32 {
            info!("Current height is now: {}", self.current_time);
            return self.current_time;
        }

        pub fn get_remaining_number_of_units(&mut self) -> u32 {
            let amount = self.fund_units.amount().as_u32();
            info!("There is {} units left in the fund", amount);
            return amount;
        }

        fn evaluate(&mut self) {
            if self.fund_units.amount().is_zero() {
                self.is_open = false;
                self.is_success = true
            } else {
                self.is_open = false;
                self.is_success = false
            }
            info!("The time is up! is_open: {}, is_success: {}", self.is_open, self.is_success);
        }



// comments:
// enough units available?
            // enough tokens in the Bucket?
            // save new owner for potential insurance pay out
            // ...or we can have reverse flow: if is_success = false & open = false one can send back units and get insurance
            // ...there can be another wrapping component to manage ownership and automate payouts if the buyer wishes so
            // ...this can be generalized, e.g. for the order book. For unfulfiled orders we get special Token that can be redeemed
            

            

        pub fn buy_fund_units(&mut self, payment: Bucket) -> (Option<Bucket>, Option<Bucket>) {
            let ret; // (Units, Change)
            let final_payment;
            let final_units;
            let current_units_number = self.get_remaining_number_of_units();
            let payment_input = payment.amount().as_u32();
            info!("Compontent received {} xrd", payment_input);

            if self.is_open {
                let reqested_units = payment_input/self.unit_price; // this is math.floor
                if reqested_units > current_units_number {
                    final_units = current_units_number;
                    final_payment = current_units_number * self.unit_price;
                } else {
                    final_units = reqested_units;
                    final_payment = reqested_units * self.unit_price;
                }
                let exact_payment = payment.take(final_payment);
                info!("Current value of exact_payment: {}, change: {}, final_units: {}", exact_payment.amount(), payment.amount(), final_units);
                self.collected_xrd.put(exact_payment);
                ret = (Some(self.fund_units.take(final_units)), Some(payment));
                info!("Current value of collected_xrd: {}, fund_units: {}", self.collected_xrd.amount(), self.fund_units.amount());
            } else {
                ret = (None, Some(payment));
            }
            return ret;
        }

        // this is for founders
        pub fn cashout_fund(&mut self) -> Option<Bucket> {
            // check owner
            // good for badges, classic authentication
            if self.is_open == false && self.is_success == true {
                return Some(self.collected_xrd.take(self.collected_xrd.amount())); // howto unwrap to take all?
            } else {
                return None;
            }
            
        }

        pub fn cashout_insurance(&mut self, fund_units: Bucket) -> Option<Bucket> {
            // check amount and address of tokens in the bucket
            // payback price and add insurance
            return None;
        }

        pub fn is_defualt (&mut self) -> bool {
            // shall we check if collected_xrd + insurance_xrd = fund_units.amout * (unit_price + unit_insurance)
            // ^^ bugs, hacks etc...
            return false; 
        }
    }
}
