#[cfg(windows)]
use colored::*;
use radix_cli::scrypto;

pub fn main() -> Result<(), scrypto::Error> {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    scrypto::run()
}
