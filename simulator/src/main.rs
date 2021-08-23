mod cli;
mod invoke;
mod ledger;

pub fn main() {
    cli::run(std::env::args());
}
