mod cli;
mod transaction;

pub fn main() -> Result<(), cli::Error> {
    cli::run(std::env::args())
}
