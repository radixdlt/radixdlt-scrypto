use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use std::fs;
use std::path::*;

use crate::scrypto::*;

const ARG_NAME: &str = "NAME";

/// Constructs a `new-package` subcommand.
pub fn make_new_package_cmd<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_NEW_PACKAGE)
        .about("Creates an package")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_NAME)
                .help("Specifies the package name.")
                .required(true),
        )
}

/// Handles a `new-package` request.
pub fn handle_new_package(matches: &ArgMatches) -> Result<(), Error> {
    let name = matches
        .value_of(ARG_NAME)
        .ok_or_else(|| Error::MissingArgument(ARG_NAME.to_owned()))?;
    let simulator_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scrypto_dir = simulator_dir.parent().unwrap().to_string_lossy();

    if PathBuf::from(name).exists() {
        Err(Error::PackageAlreadyExists)
    } else {
        fs::create_dir_all(format!("{}/src", name)).map_err(Error::IOError)?;
        fs::create_dir_all(format!("{}/tests", name)).map_err(Error::IOError)?;

        fs::write(
            PathBuf::from(format!("{}/Cargo.toml", name)),
            include_str!("../../../assets/template/package/Cargo.toml")
                .replace("${package_name}", name)
                .replace("${scrypto_home}", &scrypto_dir),
        )
        .map_err(Error::IOError)?;

        fs::write(
            PathBuf::from(format!("{}/src/lib.rs", name)),
            include_str!("../../../assets/template/package/src/lib.rs"),
        )
        .map_err(Error::IOError)?;

        fs::write(
            PathBuf::from(format!("{}/tests/lib.rs", name)),
            include_str!("../../../assets/template/package/tests/lib.rs"),
        )
        .map_err(Error::IOError)?;

        Ok(())
    }
}
