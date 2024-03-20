#[cfg(windows)]
use colored::*;
use radix_clis::scrypto_bindgen;

pub fn main() -> Result<(), scrypto_bindgen::Error> {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    scrypto_bindgen::run()
}
