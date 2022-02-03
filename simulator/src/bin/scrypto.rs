#[cfg(windows)]
use colored::*;
use simulator::scrypto;

pub fn main() -> Result<(), scrypto::Error> {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    scrypto::run()
}
