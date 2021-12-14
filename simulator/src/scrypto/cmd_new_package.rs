use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use std::fs;
use std::path::*;

use crate::scrypto::*;

const ARG_NAME: &str = "NAME";
const ARG_PATH: &str = "PATH";
const ARG_LOCAL: &str = "TRACE";

/// Constructs a `new-package` subcommand.
pub fn make_new_package<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_NEW_PACKAGE)
        .about("Creates a package")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_NAME)
                .help("Specifies the package name.")
                .required(true),
        )
        // options
        .arg(
            Arg::with_name(ARG_PATH)
                .long("path")
                .takes_value(true)
                .help("Specifies the package dir.")
                .required(false),
        )
        .arg(
            Arg::with_name(ARG_LOCAL)
                .long("local")
                .help("Uses local Scrypto as dependency."),
        )
}

/// Handles a `new-package` request.
pub fn handle_new_package(matches: &ArgMatches) -> Result<(), Error> {
    let pkg_name = matches
        .value_of(ARG_NAME)
        .ok_or_else(|| Error::MissingArgument(ARG_NAME.to_owned()))?;
    let lib_name = pkg_name.replace("-", "_");
    let pkg_dir = matches.value_of(ARG_PATH).unwrap_or(pkg_name);
    let simulator_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let (sbor, scrypto, radix_engine) = if matches.is_present(ARG_LOCAL) {
        let scrypto_dir = simulator_dir
            .parent()
            .unwrap()
            .to_string_lossy()
            .replace("\\", "/");
        (
            format!("{{ path = \"{}/sbor\" }}", scrypto_dir),
            format!("{{ path = \"{}/scrypto\" }}", scrypto_dir),
            format!("{{ path = \"{}/radix-engine\" }}", scrypto_dir),
        )
    } else {
        let s = format!(
            "{{ git = \"https://github.com/radixdlt/radixdlt-scrypto\", tag = \"v{}\" }}",
            env!("CARGO_PKG_VERSION")
        );
        (s.clone(), s.clone(), s)
    };

    if PathBuf::from(pkg_dir).exists() {
        Err(Error::PackageAlreadyExists)
    } else {
        fs::create_dir_all(format!("{}/src", pkg_dir)).map_err(Error::IOError)?;
        fs::create_dir_all(format!("{}/tests", pkg_dir)).map_err(Error::IOError)?;

        fs::write(
            PathBuf::from(format!("{}/Cargo.toml", pkg_dir)),
            include_str!("../../../assets/template/Cargo.toml")
                .replace("${package_name}", pkg_name)
                .replace("${sbor}", &sbor)
                .replace("${scrypto}", &scrypto)
                .replace("${radix-engine}", &radix_engine),
        )
        .map_err(Error::IOError)?;

        fs::write(
            PathBuf::from(format!("{}/src/lib.rs", pkg_dir)),
            include_str!("../../../assets/template/src/lib.rs"),
        )
        .map_err(Error::IOError)?;

        fs::write(
            PathBuf::from(format!("{}/tests/lib.rs", pkg_dir)),
            include_str!("../../../assets/template/tests/lib.rs").replace("${lib_name}", &lib_name),
        )
        .map_err(Error::IOError)?;

        Ok(())
    }
}
