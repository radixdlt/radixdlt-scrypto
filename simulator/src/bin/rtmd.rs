#[cfg(windows)]
use colored::*;
use simulator::rtmd;

pub fn main() -> Result<(), rtmd::Error> {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    rtmd::run()
}
