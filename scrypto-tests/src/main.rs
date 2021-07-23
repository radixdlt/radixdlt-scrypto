use scrypto::component;

component! {
    struct Test {
        x: u32,
    }

    impl Test {
        pub fn simple() -> u32 {
            1
        }

        pub fn with_return(&self, x: u32) -> u32 {
            x * 2
        }

        pub fn no_return(&mut self, _x: u32) {
        }
    }
}

fn main() {}
