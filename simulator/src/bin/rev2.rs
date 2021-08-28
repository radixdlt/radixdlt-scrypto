use simulator::cli;

pub fn main() -> Result<(), cli::Error> {
    cli::run(std::env::args())
}
