use scrypto::prelude::*;

#[derive(NftData)]
struct SubscriptionData {
    amount: Decimal,
    recurrence: u64,
    last_payment_at: u64,
    destination: Address
}

blueprint! {
    struct RecurrentPayment {
        admin_def: ResourceDef,
        token_authority_badge: Vault,
        user_payments: Vault,
        payment_token_def: ResourceDef,
        user_nft_def: ResourceDef,
        nb_users: u128
    }

    impl RecurrentPayment {
        pub fn new() -> (Component, Bucket) {
            // Admin badge used to protect methods
            let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .metadata("name", "RecurrentPayment Admin")
                                .initial_supply_fungible(1);

            // Badge used to manage the payment tokens and
            // the user NFT
            let token_authority_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                    .initial_supply_fungible(1);


            // Create a payment token that this component can take from
            // the accounts
            let payment_token_def = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                                        .metadata("name", "Payment Token")
                                        .flags(MINTABLE | RECALLABLE)
                                        .badge(token_authority_badge.resource_def(), MAY_MINT | MAY_RECALL)
                                        .no_initial_supply();

            // NFT definition that will represent individual subscriptions                     
            let user_nft_def = ResourceBuilder::new_non_fungible()
                                .metadata("name", "RecurrentPayment user")
                                .flags(MINTABLE | BURNABLE | INDIVIDUAL_METADATA_MUTABLE)
                                .badge(token_authority_badge.resource_def(), MAY_MINT | MAY_BURN | MAY_CHANGE_INDIVIDUAL_METADATA)
                                .no_initial_supply();

            let component = Self {
                admin_def: admin_badge.resource_def(),
                payment_token_def: payment_token_def,
                user_payments: Vault::new(RADIX_TOKEN),
                token_authority_badge: Vault::with_bucket(token_authority_badge),
                user_nft_def: user_nft_def,
                nb_users: 0
            }
            .instantiate();

            (component, admin_badge)
        }

        // Allow users to refill their Payment tokens
        pub fn buy_payment_tokens(&self, payment: Bucket) -> Bucket {
            assert!(payment.resource_address() == RADIX_TOKEN, "Payment must be XRD tokens");
            let amount = payment.amount();
            self.user_payments.put(payment);

            // Mint new payment tokens and return them to the user
            self.token_authority_badge.authorize(|badge| {
                self.payment_token_def.mint(amount, badge)
            })
        }

        // Allow a user to swap their Payment tokens to XRD
        pub fn sell_payment_tokens(&self, payment: Bucket) -> Bucket {
            assert!(payment.resource_address() == self.payment_token_def.address(), "Payment must be Payment tokens");
            let amount = payment.amount();

            // Burn the Payment tokens
            self.token_authority_badge.authorize(|badge| {
                payment.burn_with_auth(badge);
            });

            // Return the same amount of XRD
            self.user_payments.take(amount)
        }

        #[auth(admin_def)]
        pub fn take_payments(&self) {
            for n in 0..self.nb_users {
                let mut data: SubscriptionData = self.user_nft_def.get_nft_data(n);
                // Check if it's time to pay
                if data.last_payment_at + data.recurrence >= Context::current_epoch() {
                    // Take the payment (Payment tokens) from the user.
                    // Please note that it is not yet possible to recall tokens with Scrypto (18/12/2021)
                    // This is just a proof of concept.
                    let payment_tokens = user.recall(self.payment_token_def, data.amount);
                    let payment_xrd = self.user_payments.take(data.amount);

                    // Send the XRD tokens to the service 
                    Account::from(data.destination).deposit(payment_xrd);

                    // Burn the payment tokens and
                    // update the NFT last payment epoch
                    self.token_authority_badge.authorize(|badge| {
                        payment_tokens.burn_with_auth(badge);

                        data.last_payment_at = Context::current_epoch();
                        self.user_nft_def.update_nft_data(n, data, badge);
                    })
                }
            }
        }

        // Allow services to create subscriptions on behalf of their users
        pub fn setup_subscription(&mut self, amount: Decimal, recurrence: u64, destination: Address) -> Bucket {
            self.nb_users += 1;
            
            // Mint a new user NFT and return it to the caller
            self.token_authority_badge.authorize(|badge| {
                self.user_nft_def.mint_nft(self.nb_users, SubscriptionData{
                    amount: amount, 
                    recurrence: recurrence, 
                    last_payment_at: 0, 
                    destination: destination
                }, badge)
            })
        }
    }
}
