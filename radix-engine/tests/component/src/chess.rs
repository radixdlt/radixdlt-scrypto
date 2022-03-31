use scrypto::prelude::*;

blueprint! {
    struct Chess {
        players: [NonFungibleAddress; 2],
    }

    impl Chess {
        pub fn create_game(players: [NonFungibleAddress; 2]) -> ComponentId {
            let mut component: LocalComponent = Self { players }.into();
            component.auth("make_move", this!(SchemaPath::new().field("players").index(0)));
            component.globalize()
        }

        pub fn make_move(&mut self) {
            // Swap
            let current_player = self.players[0].clone();
            self.players[0] = self.players[1].clone();
            self.players[1] = current_player;
        }
    }
}
