use scrypto::prelude::*;

blueprint! {
    struct UTCTimeOracle {
        // Used to update the time
        admin_badge: ResourceDef,
        
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        second_since_unix: u64
    }

    impl UTCTimeOracle {
        pub fn new(nb_admins: u32) -> (Component, Bucket) {
            // Create the admin badges
            let admin_badges = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .metadata("name", "UTCTimeOracle Admin Badge")
                                .initial_supply_fungible(nb_admins);

            let component = Self {
                admin_badge: admin_badges.resource_def(),
                year: 0,
                month: 0,
                day: 0,
                hour: 0,
                minute: 0,
                second: 0,
                second_since_unix: 0
            }
            .instantiate();

            // Return the component and the admin badges
            (component, admin_badges)
        }

        #[auth(admin_badge)]
        pub fn set_current_time(&mut self, year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8, second_since_unix: u64) {
            self.year = year;
            self.month = month;
            self.day = day;
            self.hour = hour;
            self.minute = minute;
            self.second = second;
            self.second_since_unix = second_since_unix;
        }

        pub fn get_time(&self) -> (u16, u8, u8, u8, u8, u8, u64) {
            // Return the datetime
            (self.year, self.month, self.day, self.hour, self.minute, self.second, self.second_since_unix)
        }
    }
}
