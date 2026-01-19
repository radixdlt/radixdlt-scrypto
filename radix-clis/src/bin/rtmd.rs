#[cfg(windows)]
use colored::*;
use radix_clis::error::exit_with_error;
use radix_clis::rtmd;

pub fn main() {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    if let Err(msg) = rtmd::run() {
        exit_with_error(msg, 1)
    }
}
