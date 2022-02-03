mod cmd_build;
mod cmd_fmt;
mod cmd_new_package;
mod cmd_test;
mod error;

pub use cmd_build::*;
pub use cmd_fmt::*;
pub use cmd_new_package::*;
pub use cmd_test::*;
pub use error::*;

use clap::{Parser, Subcommand};

/// Create, build and test Scrypto code
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, name = "scrypto")]
pub struct ScryptoCli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Build(Build),
    Fmt(Fmt),
    NewPackage(NewPackage),
    Test(Test),
}

pub fn run() -> Result<(), Error> {
    let cli = ScryptoCli::parse();

    match cli.command {
        Command::Build(cmd) => cmd.run(),
        Command::Fmt(cmd) => cmd.run(),
        Command::NewPackage(cmd) => cmd.run(),
        Command::Test(cmd) => cmd.run(),
    }
}
