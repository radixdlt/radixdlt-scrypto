use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use std::fs;
use std::path::*;

use crate::cli::*;

const ARG_NAME: &'static str = "NAME";

/// Constructs a `new-package` subcommand.
pub fn make_new_package_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_NEW_PACKAGE)
        .about("Creates an package")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_NAME)
                .long("name")
                .takes_value(true)
                .help("Specifies the package name.")
                .required(true),
        )
}

/// Handles a `new-package` request.
pub fn handle_new_package<'a>(matches: &ArgMatches<'a>) -> Result<(), Error> {
    let name = matches.value_of(ARG_NAME).unwrap();
    let simulator_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scrypto_dir = simulator_dir.parent().unwrap().to_string_lossy();

    if PathBuf::from(name).exists() {
        Err(Error::PackageAlreadyExists)
    } else {
        fs::create_dir_all(format!("{}/.cargo", name)).map_err(|e| Error::IOError(e))?;
        fs::create_dir_all(format!("{}/src", name)).map_err(|e| Error::IOError(e))?;

        fs::write(
            PathBuf::from(format!("{}/Cargo.toml", name)),
            include_str!("../../../assets/template/package/Cargo.toml")
                .replace("${package_name}", name)
                .replace("${scrypto_home}", &scrypto_dir),
        )
        .map_err(|e| Error::IOError(e))?;

        fs::write(
            PathBuf::from(format!("{}/.cargo/config.toml", name)),
            include_str!("../../../assets/template/package/.cargo/config.toml"),
        )
        .map_err(|e| Error::IOError(e))?;

        fs::write(
            PathBuf::from(format!("{}/src/lib.rs", name)),
            include_str!("../../../assets/template/package/src/lib.rs"),
        )
        .map_err(|e| Error::IOError(e))?;

        Ok(())
    }
}
