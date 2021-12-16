use scrypto::prelude::*;

blueprint! {
    struct Hello {
        // Define what resources and data will be managed by Hello components
        sample_vault: Vault
    }

    impl Hello {
        // Implement the functions and methods which will manage those resources and data

        // This is a function, and can be called directly on the blueprint once deployed
        pub fn new() -> Component {
            // Create a new token called "HelloToken," with a fixed supply of 1000, and put that supply into a bucket
            let my_bucket: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "HelloToken")
                .metadata("symbol", "HT")
                .initial_supply_fungible(1000);

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            Self {
                sample_vault: Vault::with_bucket(my_bucket)
            }
            .instantiate()
        }

        // works (assuming the tx includes a DropAllBucketRefs instruction)
        fn non_pub_show(&self, amount: Decimal) -> BucketRef {
            self.sample_vault.take(amount).present()
        }

        // works (assuming the tx includes a DropAllBucketRefs instruction)
        pub fn show(&self, amount: Decimal) -> BucketRef {
            self.sample_vault.take(amount).present()
        }

        // but this fails ... I guess because the BucketRef is not dropped before the function ends.
        // ok... but this is counterintuative and problematic ... see all the following examples
        pub fn show_amount(&self, amount: Decimal) -> Decimal {
            self.sample_vault.take(amount).present().amount()
        }

        // also works, the Bucket is not left dangling even when assigned to a local variable
        // which is weird because teh BucketRef is left dangling in the above `show_amount` example
        pub fn show_a(&self, amount: Decimal) -> BucketRef {
            let bucket = self.sample_vault.take(amount);
            bucket.present()
        }

        // also works (without a bucket local variable)
        pub fn show_b(&self, amount: Decimal) -> BucketRef {
            let bucket_ref = self.sample_vault.take(amount).present();
            bucket_ref
        }

        // works, the "right" way to do this, but very very verbose
        pub fn have(&self, amount: Decimal) -> bool {
            let bucket = self.sample_vault.take(amount);
            let bucket_ref = bucket.present();
            let result = bucket_ref.amount() == amount;
            bucket_ref.drop(); // don't let the bucket ref dangle
            self.sample_vault.put(bucket); // don't let the bucket dangle
            result
        }

        // broken, but "expected" since the bucket becomes dangling.  this is pretty subtle compared to show()
        // why does .amount() make the bucket become dnagling??? - compare with .show_b
        // fails with dangling BucketRef
        pub fn have_b(&self, amount: Decimal) -> bool {
            let bucket_ref = self.sample_vault.take(amount).present();
            bucket_ref.amount() == amount
        }

        // still broken, dropping the BucketRef doesn't fix the dangling Bucket (unsurprisingly)
        // now fails with dangling Bucket
        pub fn have_b2(&self, amount: Decimal) -> bool {
            let bucket_ref = self.sample_vault.take(amount).present();
            let result = bucket_ref.amount() == amount;
            bucket_ref.drop(); // manually drop the BucketRef
            result
        }

        // broken, probably for teh same reason show_amount is broken
        // fails with dangling BucketRef
        pub fn have_c(&self, amount: Decimal) -> bool {
            let bucket_ref = self.show(amount);
            bucket_ref.amount() == amount
        }

        // still broken, even when manually dropping the BucketRef
        // now fails with dangling bucket (not dangling bucket ref)
        // this can't be fixed, don't have the bucket to drop
        pub fn have_c2(&self, amount: Decimal) -> bool {
            let bucket_ref = self.show(amount);
            let result: bool = bucket_ref.amount() == amount;
            bucket_ref.drop(); // manually drop the BucketRef
            result
        }

        // no temporary variables, still broken.  would really really like this to "just work"
        // fails with dangling BucketRef
        pub fn have_d(&self, amount: Decimal) -> bool {
            self.show(amount).amount() == amount
        }

        // still fails when using a non-pub version of show
        pub fn have_e(&self, amount: Decimal) -> bool {
            self.non_pub_show(amount).amount() == amount
        }

        // fails, understandably, no way to actually fix the dangling bucket and still return the BucketRef
        // This is probably the case for needing to have vault.present(amount) -> VaultRef and it might fix everything else
        pub fn show_not_fixed(&self, amount: Decimal) -> BucketRef {
            let bucket = self.sample_vault.take(amount);
            let bucket_ref = bucket.present();
            self.sample_vault.put(bucket); // don't let the bucket dangle -- fails, can't have an outstanding ref to this bucket
            bucket_ref
        }

    }
}
