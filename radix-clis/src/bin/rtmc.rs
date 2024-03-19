#[cfg(windows)]
use colored::*;
use radix_cli::rtmc;

pub fn main() -> Result<(), rtmc::Error> {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    rtmc::run()
}
