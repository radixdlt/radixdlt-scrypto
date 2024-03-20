#[cfg(windows)]
use colored::*;
use radix_clis::rtmd;

pub fn main() -> Result<(), rtmd::Error> {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    rtmd::run()
}
