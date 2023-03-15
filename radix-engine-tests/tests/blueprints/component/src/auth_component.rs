use scrypto::prelude::*;

#[blueprint]
mod auth_component {
    struct AuthComponent {
        some_non_fungible: NonFungibleGlobalId,
    }

    impl AuthComponent {
        pub fn create_component(some_non_fungible: NonFungibleGlobalId) -> ComponentAddress {
            let component = Self { some_non_fungible }.instantiate();
            component.globalize_with_access_rules(
                AccessRulesConfig::new()
                    .method(
                        "get_secret",
                        rule!(require("some_non_fungible")),
                        rule!(deny_all),
                    )
                    .default(rule!(allow_all), AccessRule::DenyAll),
            )
        }

        pub fn get_secret(&self) -> String {
            "Secret".to_owned()
        }

        pub fn update_auth(&mut self, some_non_fungible: NonFungibleGlobalId) {
            self.some_non_fungible = some_non_fungible;
        }
    }
}
