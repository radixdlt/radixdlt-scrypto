use scrypto::prelude::*;

#[blueprint]
mod chess {
    struct Chess {
        players: [NonFungibleGlobalId; 2],
    }

    impl Chess {
        pub fn create_game(players: [NonFungibleGlobalId; 2]) -> ComponentAddress {
            let access_rules = AccessRulesConfig::new().method(
                "make_move",
                rule!(require("players/0")),
                rule!(deny_all),
            );

            let component = Self { players }.instantiate();
            component.globalize_with_access_rules(access_rules)
        }

        pub fn make_move(&mut self) {
            // Swap
            let current_player = self.players[0].clone();
            self.players[0] = self.players[1].clone();
            self.players[1] = current_player;
        }
    }
}
