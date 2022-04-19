use scrypto::prelude::*;

blueprint! {
    struct Chess {
        players: [NonFungibleAddress; 2],
    }

    impl Chess {
        pub fn create_game(players: [NonFungibleAddress; 2]) -> ComponentAddress {
            let access_rules = AccessRules::new().method("make_move", auth!(require("players/0")));

            Self { players }.instantiate().add_access_check(access_rules).globalize()
        }

        pub fn make_move(&mut self) {
            // Swap
            let current_player = self.players[0].clone();
            self.players[0] = self.players[1].clone();
            self.players[1] = current_player;
        }
    }
}
