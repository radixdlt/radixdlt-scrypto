use scrypto::prelude::*;

blueprint! {
    struct RegulatedToken {        
        token_supply: Vault,
        internal_authority: Vault,
        collected_xrd: Vault,
        current_stage: u8,
        admin_badge_def: ResourceDef,
        freeze_badge_def: ResourceDef,
    }

    impl RegulatedToken {
        
        pub fn instantiate_regulated_token() -> (Component, Bucket, Bucket) {                        
            // We will start by creating two tokens we will use as badges and return to our instantiator
            let general_admin: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name","RegulatedToken general admin badge")
                .flags(BURNABLE | FREELY_BURNABLE)
                .initial_supply_fungible(1);

            let freeze_admin: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name","RegulatedToken freeze-only badge")
                .flags(BURNABLE | FREELY_BURNABLE)
                .initial_supply_fungible(1);

            // Next we will create a badge we'll hang on to for minting & transfer authority
            let internal_admin: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name","RegulatedToken internal authority badge")
                .flags(BURNABLE | FREELY_BURNABLE)
                .initial_supply_fungible(1);

            // Next we will create our regulated token with an initial fixed supply of 100 and the appropriate flags and permissions
            let my_bucket: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "Regulo")
                .metadata("symbol", "REG")
                .metadata("stage", "Stage 1 - Fixed supply, may be restricted transfer")
                .flags(SHARED_METADATA_MUTABLE | RESTRICTED_TRANSFER)
                .mutable_flags(MINTABLE | SHARED_METADATA_MUTABLE | RESTRICTED_TRANSFER)
                .badge(
                    general_admin.resource_def(),
                    ALL_PERMISSIONS
                )
                .badge(
                    freeze_admin.resource_def(),
                    MAY_MANAGE_RESOURCE_FLAGS
                )
                .badge(
                    internal_admin.resource_def(),
                    MAY_MINT | MAY_TRANSFER
                )
                .initial_supply_fungible(100);

            let component = Self {
                token_supply: Vault::with_bucket(my_bucket),
                internal_authority: Vault::with_bucket(internal_admin),
                collected_xrd: Vault::new(RADIX_TOKEN),
                current_stage: 1,
                admin_badge_def: general_admin.resource_def(),
                freeze_badge_def: freeze_admin.resource_def(),
            }
            .instantiate();

            // Note that the freeze badge actually has the ability to modify *all* of the mutable flags, not just RESTRICTED_TRANSFER.
            // In a real system, if we wanted the recipient to only have the ability to modify a single flag, we could hang on to the real badge
            // within our component, and issue something that grants the bearer the right to call a method which uses the real badge to modify the flag
            (component, general_admin, freeze_admin)
        }

        /// Either the general admin or freeze admin badge may be used to freeze or unfreeze consumer transfers of the supply
        #[auth(admin_badge_def, freeze_badge_def)]
        pub fn toggle_transfer_freeze(&self, set_frozen: bool) {
            // We can refer to the incoming badge as "auth"
            // Note that this operation will fail if the token has reached stage 3 and the RESTRICTED_TRANSFER flag has become immutably disabled
            let mut token_def = self.token_supply.resource_def();
            if set_frozen {
                token_def.enable_flags(RESTRICTED_TRANSFER, auth);
                info!("Token transfer is now RESTRICTED");
            }
            else {
                token_def.disable_flags(RESTRICTED_TRANSFER, auth);
                info!("Token is now freely transferrable");
            }     
        }

        pub fn get_current_stage(&self) -> u8 {
            info!("Current stage is {}", self.current_stage);
            self.current_stage
        }

        /// Permit the proper authority to withdraw our collected XRD
        #[auth(admin_badge_def)]
        pub fn collect_payments(&mut self) -> Bucket {
            self.collected_xrd.take_all()
        }
        
        #[auth(admin_badge_def)]
        pub fn advance_stage(&mut self) {            
            assert!(self.current_stage <= 2, "Already at final stage");

            if self.current_stage == 1 {
                // Advance to stage 2                
                // Token will still be restricted transfer upon admin demand, but we will mint beyond the initial supply as required                
                self.current_stage = 2;                
                let mut token_def = self.token_supply.resource_def();

                // Update token's metadata to reflect the current stage
                let mut metadata = token_def.metadata();
                metadata.insert("stage".into(), "Stage 2 - Unlimited supply, may be restricted transfer".into());
                token_def.update_metadata(metadata, auth.clone());

                // Enable minting for the token                
                token_def.enable_flags(MINTABLE, auth.clone());
                info!("Advanced to stage 2");
            }
            else {
                // Advance to stage 3
                // Token will no longer be regulated
                // Restricted transfer will be permanently turned off, supply will be made permanently immutable
                self.current_stage = 3;
                let mut token_def = self.token_supply.resource_def();

                // Update token's metadata to reflect the final stage
                let mut metadata = token_def.metadata();                
                metadata.insert("stage".into(), "Stage 3 - Unregulated token, fixed supply".into());
                token_def.update_metadata(metadata, auth.clone());

                // Set our flags appropriately now that the regulated period has ended
                token_def.disable_flags(MINTABLE | RESTRICTED_TRANSFER | SHARED_METADATA_MUTABLE, auth.clone());

                // Permanently prevent the flags from changing
                token_def.lock_flags(ALL_FLAGS, auth.clone());

                // With the resource flags all forever disabled and locked, our internal authority badge no longer has any use
                // We will burn our internal badge, and the holders of the other badges may burn them at will
                // Our badge has the FREELY_BURNABLE flag, so there's no need to provide a burning authority           
                self.internal_authority.take_all().burn();
                info!("Advanced to stage 3");
            }
        }

        /// Buy a quantity of tokens, if the supply on-hand is sufficient, or if current rules permit minting additional supply.
        /// The system will *always* allow buyers to purchase available tokens, even when the token transfers are otherwise frozen
        pub fn buy_token(&mut self, quantity: Decimal, mut payment: Bucket) -> (Bucket, Bucket) {
            assert!(quantity > 0.into(), "Can't sell you nothing or less than nothing");
            // Early birds who buy during stage 1 get a discounted rate
            let price: Decimal = if self.current_stage == 1 { 50.into() } else { 100.into() };
            
            // Take what we're owed
            self.collected_xrd.put(payment.take(price * quantity));

            // Can we fill the desired quantity from current supply?
            let extra_demand = quantity - self.token_supply.amount();
            if extra_demand <= 0.into() {
                // Take the required quantity, and return it along with any change
                // The token may currently be under restricted transfer, so we will authorize our withdrawal
                let tokens = self.internal_authority.authorize(
                    |auth| self.token_supply.take_with_auth(quantity, auth)
                );
                return (tokens, payment);                
            }
            else {
                // We will attempt to mint the shortfall
                // If we are in stage 1 or 3, this action will fail, and it would probably be a good idea to tell the user this
                // For the purposes of example, we will blindly attempt to mint
                let mut tokens = self.internal_authority.authorize(
                    |auth| self.token_supply.resource_def().mint(extra_demand, auth)
                );
                
                // Combine the new tokens with whatever was left in supply to meet the full quantity
                let existing_tokens = self.internal_authority.authorize(
                    |auth| self.token_supply.take_all_with_auth(auth)
                );
                tokens.put(existing_tokens);

                // Return the tokens, along with any change
                return (tokens, payment);
            }
        }
    }
}
