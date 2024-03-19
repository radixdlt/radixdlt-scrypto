#[cfg(windows)]
use colored::*;
use radix_cli::rtmd;

pub fn main() -> Result<(), rtmd::Error> {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    rtmd::run()
}
