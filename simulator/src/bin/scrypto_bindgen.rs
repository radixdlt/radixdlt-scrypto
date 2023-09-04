#[cfg(windows)]
use colored::*;
use simulator::scrypto_bindgen;

/// NSG = Native Stubs Generator
pub fn main() -> Result<(), scrypto_bindgen::Error> {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    scrypto_bindgen::run()
}
