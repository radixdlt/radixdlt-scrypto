use scrypto::prelude::*;
use crate::time_oracle::TimeOracle;

blueprint! {
    struct AlarmClock {
        // Admin badge of the TimeOracle
        // used to protect the "try_trigger" method
        admin_badge: ResourceDef,
        // Reference to the TimeOracle component
        time_oracle: TimeOracle,
        // Unix time when the AlarmClock should call the other component
        call_at: u64,

        // Component and method to call when the time is greater
        // or equal to the "call_at" variable
        component: Component,
        method_to_call: String,

        // Specifies whether the AlarmClock already 
        // called the component or not.
        done: bool
    }

    impl AlarmClock {
        pub fn new(component_address: Address, method_to_call: String, call_at: u64) -> (Component, Bucket) {
            // Instantiate the TimeOracle component
            let (time_oracle_component, admin_badge): (Component, Bucket) = TimeOracle::new(1);
            let time_oracle: TimeOracle = time_oracle_component.into();

            // Set the time to 2021-12-24 00:00:00
            time_oracle.set_current_time(1640322000, admin_badge.present());

            let component = Self{
                time_oracle: time_oracle.into(),
                admin_badge: admin_badge.resource_def(),
                call_at: call_at,
                component: component_address.into(),
                method_to_call: method_to_call,
                done: false
            }.instantiate();

            // We will use the same admin badge as the one used
            // in the TimeOracle component for simplicity.
            (component, admin_badge)
        }

        #[auth(admin_badge)]
        pub fn try_trigger(&mut self) {
            assert!(!self.done, "Already triggered !");
            let current_time = self.time_oracle.get_time();
            if current_time >= self.call_at {
                // Call the method of the specified component
                self.component.call::<()>(&self.method_to_call, vec![]);
                self.done = true;
            } else {
                info!("Not ready yet !");
            }
        }
    }
}