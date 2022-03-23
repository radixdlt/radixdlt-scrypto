use scrypto::prelude::*;

blueprint! {
    struct Chess {
        players: [NonFungibleAddress; 2]
    }

    impl Chess {
        pub fn create_game(players: [NonFungibleAddress; 2]) -> ComponentId {
            Self {
                players
            }
            .instantiate_with_auth(component_authorization! {
                "make_move" => any_of!(vec![0, 0]),
            })
        }

        pub fn make_move(&mut self) {
            // Swap
            let current_player = self.players[0].clone();
            self.players[0] = self.players[1].clone();
            self.players[1] = current_player;
        }
    }
}
