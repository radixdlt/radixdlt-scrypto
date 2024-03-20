#[cfg(windows)]
use colored::*;
use radix_clis::rtmc;

pub fn main() -> Result<(), rtmc::Error> {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    rtmc::run()
}
