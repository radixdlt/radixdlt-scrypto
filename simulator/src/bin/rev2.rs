#[cfg(windows)]
use colored::*;
use simulator::rev2;

pub fn main() -> Result<(), rev2::Error> {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    rev2::run(std::env::args())
}
