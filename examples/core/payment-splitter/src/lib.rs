use scrypto::prelude::*;

/// Defines the data of the shareholders badge which is an NFT.
#[derive(NonFungibleData)]
pub struct ShareHolder {
    /// Defines the account address of the shareholder. This address is almost never used by the this blueprint but is
    /// kept just as a symbol and not for any real use.
    pub address: Address,

    /// The number of shares that this shareholder owns.
    pub shares: Decimal,
}

blueprint! {
    /// A PaymentSplitter is a Scrypto blueprint which allows for a way for funds to be distributed among shareholders
    /// in a project depending on the amount of shares that each of the shareholders own.
    struct PaymentSplitter {
        /// This is a HashMap that maps the IDs of the shareholder NFTs to vaults that contain the funds that are owed
        /// to them.
        xrd_vaults: HashMap<NonFungibleKey, Vault>,

        /// The resource definition of the admin badge. This admin badge is used to mint the NFTs which give the 
        /// shareholders to authenticate them and keep track of how much they took so far from the splitter.
        admin_badge_def: ResourceDef,

        /// The resource definition of the shareholders NFT. This NFT is used to authenticate shareholders to allow
        /// for the withdrawal of funds from the payment splitter and is also used to keep track of information about
        /// this shareholder such as the number of shares that they own.
        shareholder_def: ResourceDef,

        /// This decimal number is used to keep track of the number of shareholders that we have so far under this
        /// payment splitter. This is used to allow for us to easily index all of the Shareholder NFTs.
        number_of_shareholders: u128,

        /// This describes the total amount of shares across all of the shareholders 
        total_quantity_of_shares: Decimal
    }

    impl PaymentSplitter {
        /// Creates a new Payment Splitter and returns it along with the admin badge back to the creator of the
        /// payment splitter.
        /// 
        /// # Returns
        /// 
        /// * `Component` - A PaymentSplitter component
        /// * `Bucket` - A bucket of the admin badge. This badge may be used to add shareholders to the PaymentSplitter.
        pub fn new() -> (Component, Bucket) {
            // Creating the admin badge
            let admin_badge: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Admin Badge")
                .metadata("symbol", "adm")
                .initial_supply_fungible(1);

            // Creating the shareholders NFT Resource Definition. We will not mint any of them at this point
            // of time as the minting and creation of these tokens happens when the `add_shareholder` function
            // is called
            let shareholder_def: ResourceDef = ResourceBuilder::new_non_fungible()
                .metadata("name", "Shareholder")
                .metadata("symbol", "shr")
                .flags(MINTABLE)
                .badge(admin_badge.resource_address(), MAY_MINT)
                .no_initial_supply();

            // Creating the payment splitter component
            let payment_splitter: Component = Self {
                xrd_vaults: HashMap::new(),
                admin_badge_def: admin_badge.resource_def(),
                shareholder_def: shareholder_def,
                number_of_shareholders: 0,
                total_quantity_of_shares: Decimal::zero(),
            }.instantiate();

            // Returning the PaymentSplitter component and the admin badge
            return (payment_splitter, admin_badge);
        }

        /// Adds a shareholder to the PaymentSplitter
        /// 
        /// This is an authenticated method which needs to be called and presented the admin badge (obtained from creating the component).
        /// This method is used to add a shareholder to the PaymentSplitter by minting an NFT to authenticate and keep track of the shareholder
        /// data.
        /// 
        /// # Arguments
        /// 
        /// * `shareholder_address` - The address of the shareholder. This address will be airdropped the NFT.
        /// * `shareholder_shares` - The amount of shares that belong to this shareholder.
        #[auth(admin_badge_def)]
        pub fn add_shareholder(
            &mut self,
            shareholder_address: Address,
            shareholder_shares: Decimal
        ) -> Bucket {
            // Creating this NFT for this specific shareholder and storing it in a bucket
            let nft_bucket: Bucket = self.shareholder_def.mint_non_fungible(
                &NonFungibleKey::from(self.number_of_shareholders),
                ShareHolder {
                    address: shareholder_address,
                    shares: shareholder_shares,
                },
                auth
            );
            self.number_of_shareholders += 1;

            // Incrementing the total amount of shares by the amount of shares added with this 
            // method call
            self.total_quantity_of_shares += shareholder_shares;

            // Adding the ID of the newly minted NFT to the hashmap of vaults and creating a new vault for this shareholder
            self.xrd_vaults.insert(nft_bucket.get_non_fungible_key(), Vault::new(RADIX_TOKEN));

            // Logging the addition of the shareholder to the payment splitter
            info!("Added shareholder {} with shares {} to the splitter. Number of current shareholders: {}", shareholder_address, shareholder_shares, self.number_of_shareholders);
            
            // Now that we have created the shareholder NFTs, there two two main ways we can get these NFTs to the shareholders:
            //
            // 1- We could airdrop them to the shareholders through the account component.
            // 2- We could return the shareholder's badge as a bucket and trust that the caller would send them their badges.
            //
            // Method 1 seems somewhat better as there is no need to trust the admin to send the badges when they're created and 
            // admin can't go back on their word one the badges are created. However, this would go against the asset oriented 
            // style of coding used in Radix. So, if you want to directly airdrop the tokens instead of returning them in a bucket
            // then you can use the line of code below:
            // Component::from(shareholder_address).call::<()>("deposit", vec![scrypto_encode(&nft_bucket)]);

            // Returning the bucket of shareholder NFTs back to the caller
            return nft_bucket;
        }

        /// Used to deposit XRD into the XRD vault controlled by this component.
        /// 
        /// This method is used to deposit XRD into the vault that this component controls. When XRD is deposited we split
        /// the XRD according to the number of shares and put the XRD in the vault associated with each of the shareholders
        /// 
        /// # Arguments
        /// 
        /// * `xrd_bucket` - A bucket of XRD that we're depositing into our component.
        /// 
        /// # Returns
        /// 
        /// * `Bucket` - A bucket of the XRD that will be returned back to the caller after the XRD has been split 
        /// across the shareholders. 
        pub fn deposit_xrd(&mut self, mut xrd_bucket: Bucket) -> Bucket {
            // Getting the amount of XRD that passed in the XRD bucket
            let xrd_amount: Decimal = xrd_bucket.amount();
            info!("Depositing XRD of amount: {}", xrd_amount);

            // Calculating how much of the XRD deposited is owed to each of the shareholders and then adding it to their
            // respective vault.
            for i in 0..self.number_of_shareholders {
                // Creating a non fungible key from the variable i
                let nft_id: NonFungibleKey = NonFungibleKey::from(i);
                
                // Adding the amount of XRD owed to this shareholder to their respective vault
                let shareholder: ShareHolder = self.shareholder_def.get_non_fungible_data(&nft_id);
                let xrd_owed_to_shareholder: Decimal = xrd_amount * shareholder.shares / self.total_quantity_of_shares;

                self.xrd_vaults.get_mut(&nft_id).unwrap().put(xrd_bucket.take(xrd_owed_to_shareholder));
            }

            return xrd_bucket;
        }

        /// Withdraws the amount of XRD that is owed to the shareholder.
        /// 
        /// This is an authenticated method that only a valid shareholder can call by presenting their shareholder badge when calling
        /// this method. This method withdraws the total amount that we currently owe the shareholder from the XRD vault and returns
        /// it back to them in a bucket. If this specific shareholder has already withdrawn all of their owed amount then an empty 
        /// bucket of XRD is returned.
        /// 
        /// # Returns
        /// 
        /// * `Bucket` - A bucket containing the XRD owed to the shareholder.
        #[auth(shareholder_def)]
        pub fn withdraw_xrd(&mut self) -> Bucket {
            // Getting the ID of the NFT passed in the badge and then withdrawing all of the XRD owed to this shareholder
            let nft_id: NonFungibleKey = auth.get_non_fungible_key(); 
            return self.xrd_vaults.get_mut(&nft_id).unwrap().take_all();
        }
    }
}