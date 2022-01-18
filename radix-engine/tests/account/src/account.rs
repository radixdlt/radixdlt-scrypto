use scrypto::prelude::*;

blueprint! {
    struct AccountTest {
    }

    impl AccountTest {
	pub fn account_withdraw(acct_addr: Address) {
	    let account = Account::from(acct_addr);
	    let bucket = account.withdraw(Decimal::from(1), RADIX_TOKEN);
	    account.deposit(bucket);
	}
    }
}
