#[cfg(windows)]
use colored::*;
use radix_cli::replay;

pub fn main() -> Result<(), replay::Error> {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    replay::run()
}
