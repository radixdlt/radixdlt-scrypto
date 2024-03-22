#[cfg(windows)]
use colored::*;
use radix_clis::error::exit_with_error;
use radix_clis::rtmd;

pub fn main() {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    match rtmd::run() {
        Err(msg) => exit_with_error(msg, 1),
        _ => {}
    }
}
