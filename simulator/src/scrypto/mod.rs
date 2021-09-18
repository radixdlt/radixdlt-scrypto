mod cmd_build;
mod cmd_new_package;
mod cmd_test;
mod error;

pub use cmd_build::*;
pub use cmd_new_package::*;
pub use cmd_test::*;
pub use error::*;

pub const CMD_NEW_PACKAGE: &str = "new-package";
pub const CMD_BUILD: &str = "build";
pub const CMD_TEST: &str = "test";

pub fn run<I, T>(args: I) -> Result<(), Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let app = clap::App::new("Scrypto")
        .name("scrypto")
        .about("Create, build and test Scrypto code.")
        .version(clap::crate_version!())
        .subcommand(make_new_package())
        .subcommand(make_build())
        .subcommand(make_test());
    let matches = app.get_matches_from(args);

    match matches.subcommand() {
        (CMD_NEW_PACKAGE, Some(m)) => handle_new_package(m),
        (CMD_BUILD, Some(m)) => handle_build(m),
        (CMD_TEST, Some(m)) => handle_test(m),
        _ => Err(Error::MissingSubCommand),
    }
}
