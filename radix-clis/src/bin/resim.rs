#[cfg(windows)]
use colored::*;
use radix_cli::resim;

pub fn main() -> Result<(), resim::Error> {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    resim::run()
}