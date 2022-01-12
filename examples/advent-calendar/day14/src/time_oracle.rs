use scrypto::prelude::*;

blueprint! {
    struct TimeOracle {
        // Badge used to update the time
        admin_badge: ResourceDef,
        second_since_unix: u64
    }

    impl TimeOracle {
        pub fn new(nb_admins: u32) -> (Component, Bucket) {
            // Create the admin badges
            let admin_badges = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .metadata("name", "UTCTimeOracle Admin Badge")
                                .initial_supply_fungible(nb_admins);

            let component = Self {
                admin_badge: admin_badges.resource_def(),
                second_since_unix: 0
            }
            .instantiate();

            // Return the component and the admin badges
            (component, admin_badges)
        }

        #[auth(admin_badge)]
        pub fn set_current_time(&mut self, second_since_unix: u64) {
            self.second_since_unix = second_since_unix;
        }

        pub fn get_time(&self) -> u64 {
            // Return the datetime
            self.second_since_unix
        }
    }
}
