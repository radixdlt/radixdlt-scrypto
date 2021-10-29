use scrypto::prelude::*;

blueprint! {
    struct FlatAdmin {
        admin_mint_auth: Vault,
        admin_badge_def: ResourceDef,
        admin_badge_address: Address
    }

    impl FlatAdmin {
        pub fn new(badge_name: String) -> (Component, Bucket) {
            // Instantiate the component, create the first admin badge, and return both
            let mint_auth = ResourceBuilder::new().create_fixed(1);
            let admin_badge_def = ResourceBuilder::new()
                .metadata("name", badge_name)
                .create_mutable(mint_auth.resource_def());
            let first_admin_badge = admin_badge_def.mint(1, mint_auth.borrow());
            let component = Self {
                admin_mint_auth: Vault::with_bucket(mint_auth),
                admin_badge_def: first_admin_badge.resource_def(),
                admin_badge_address: first_admin_badge.resource_def().address()
            }
            .instantiate();
            (component, first_admin_badge)
        }

        // Any existing admin may create another admin token
        #[auth(admin_badge_address)]
        pub fn create_additional_admin(&self) -> Bucket {
            self.admin_badge_def.mint(1, self.admin_mint_auth.take(1).borrow())
        }

        pub fn destroy_admin_badge(&self, to_destroy: Bucket) {
            scrypto_assert!(to_destroy.resource_def().address() == self.admin_badge_address, "Can not destroy the contents of this bucket");
            to_destroy.burn();
        }
    }
}
