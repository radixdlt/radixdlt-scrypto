use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct LoanDue {
    pub amount_due: Decimal,
}

#[blueprint]
#[types(LoanDue)]
mod basic_flash_loan {
    struct BasicFlashLoan {
        loan_vault: Vault,
        auth_vault: Vault,
        transient_resource_manager: ResourceManager,
    }

    impl BasicFlashLoan {
        /// The most elementary possible flash loan.  Creates a loan pool from whatever is initially supplied,
        /// provides loans with a .1% fee, and lets anyone freely add liquidity.
        ///
        /// Does NOT reward liquidity providers in any way or provide a way to remove liquidity from the pool.
        /// Minting LP tokens for rewards, and removing liquidity, is covered in other examples, such as:
        /// https://github.com/radixdlt/scrypto-examples/tree/main/defi/radiswap
        pub fn instantiate_default(
            initial_liquidity: Bucket,
        ) -> (Global<BasicFlashLoan>, ResourceManager) {
            let auth_token = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata! {
                    init {
                        "name" => "Admin authority for BasicFlashLoan".to_string(), locked;
                    }
                })
                .mint_initial_supply(1);

            // Define a "transient" resource which can never be deposited once created, only burned
            let transient_resource_manager = ResourceBuilder::new_ruid_non_fungible_with_registered_type::<LoanDue>(OwnerRole::Fixed(rule!(require(auth_token.resource_address()))))
                .metadata(metadata! {
                    init {
                        "name" => "Promise token for BasicFlashLoan - must be returned to be burned!".to_string(), locked;
                    }
                })
                .mint_roles(mint_roles! {
                    minter => OWNER;
                    minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                    burner => OWNER;
                    burner_updater => rule!(deny_all);
                })
                .deposit_roles(deposit_roles! {
                    depositor => rule!(deny_all);
                    depositor_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply();

            let global_component = Self {
                loan_vault: Vault::with_bucket(initial_liquidity),
                auth_vault: Vault::with_bucket(auth_token.into()),
                transient_resource_manager,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            (global_component, transient_resource_manager)
        }

        pub fn available_liquidity(&self) -> Decimal {
            self.loan_vault.amount()
        }

        pub fn add_liquidity(&mut self, tokens: Bucket) {
            self.loan_vault.put(tokens);
        }

        pub fn take_loan(&mut self, loan_amount: Decimal) -> (Bucket, Bucket) {
            assert!(
                loan_amount <= self.loan_vault.amount(),
                "Not enough liquidity to supply this loan!"
            );

            // Calculate how much we must be repaid
            let amount_due = loan_amount.checked_mul(dec!("1.001")).unwrap();

            // Mint an NFT with the loan terms.  Remember that this resource previously had rules defined which
            // forbid it from ever being deposited in any vault.  Thus, once it is present in the transaction
            // the only way for the TX to complete is to remove this "dangling" resource by burning it.
            //
            // Our component will control the only badge with the authority to burn the resource, so anyone taking
            // a loan must call our repay_loan() method with an appropriate reimbursement, at which point we will
            // burn the NFT and allow the TX to complete.
            let loan_terms = self
                .auth_vault
                .as_fungible()
                .authorize_with_amount(dec!(1), || {
                    self.transient_resource_manager
                        .mint_ruid_non_fungible(LoanDue {
                            amount_due: amount_due,
                        })
                });
            (self.loan_vault.take(loan_amount), loan_terms)
        }

        pub fn repay_loan(&mut self, loan_repayment: Bucket, loan_terms: Bucket) {
            assert!(
                loan_terms.resource_address() == self.transient_resource_manager.address(),
                "Incorrect resource passed in for loan terms"
            );

            // Verify we are being sent at least the amount due
            let terms: LoanDue = loan_terms.as_non_fungible().non_fungible().data();
            assert!(
                loan_repayment.amount() >= terms.amount_due,
                "Insufficient repayment given for your loan!"
            );

            // We could also verify that the resource being repaid is of the correct kind, and give a friendly
            // error message if not. For this example we'll just let the engine handle that when we try to deposit
            self.loan_vault.put(loan_repayment);

            // We have our payment; we can now burn the transient token
            self.auth_vault
                .as_fungible()
                .authorize_with_amount(dec!(1), || loan_terms.burn());
        }
    }
}
