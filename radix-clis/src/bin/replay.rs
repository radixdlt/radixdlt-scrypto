#[cfg(windows)]
use colored::*;
use radix_clis::error::exit_with_error;
use radix_clis::replay;

pub fn main() {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    if let Err(msg) = replay::run() {
        exit_with_error(msg, 1)
    }
}
