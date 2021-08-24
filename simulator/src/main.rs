mod cli;
mod ledger;
mod transaction;

pub fn main() {
    cli::run(std::env::args());
}
