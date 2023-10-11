#[cfg(windows)]
use colored::*;
use simulator::tx_replay;

pub fn main() -> Result<(), tx_replay::Error> {
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();
    tx_replay::run()
}
