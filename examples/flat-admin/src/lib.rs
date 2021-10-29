use scrypto::prelude::*;

blueprint! {
    struct FlatAdmin {
        admin_mint_auth: Vault,
        admin_badge: ResourceDef
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
                admin_badge: first_admin_badge.resource_def()
            }
            .instantiate();
            (component, first_admin_badge)
        }

        // Any existing admin may create another admin token
        #[auth(admin_badge)]
        pub fn create_additional_admin(&self) -> Bucket {
            self.admin_mint_auth.authorize(|badge| { self.admin_badge.mint(1, badge) })            
        }

        // Not needed at the moment when anyone can burn, leaving here as a TODO for when burning is restricted
        pub fn destroy_admin_badge(&self, to_destroy: Bucket) {
            scrypto_assert!(to_destroy.resource_def().address() == self.admin_badge.address(), "Can not destroy the contents of this bucket!");
            to_destroy.burn();
        }

        pub fn get_admin_badge_address(&self) -> Address {
            self.admin_badge.address()
        }
    }
}
