use clap::{crate_version, App, ArgMatches};
use colored::*;

use crate::ledger::*;
use crate::resim::*;
use crate::utils::*;

/// Constructs a `show-ledger` subcommand.
pub fn make_show_ledger<'a>() -> App<'a> {
    App::new(CMD_SHOW_LEDGER)
        .about("Displays ledger summary")
        .version(crate_version!())
}

/// Handles a `show-ledger` request.
pub fn handle_show_ledger(_matches: &ArgMatches) -> Result<(), Error> {
    let ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);

    println!("{}:", "Packages".green().bold());
    for (last, address) in ledger.list_packages().iter().identify_last() {
        println!("{} {}", list_item_prefix(last), address,);
    }

    println!("{}:", "Components".green().bold());
    for (last, address) in ledger.list_components().iter().identify_last() {
        println!("{} {}", list_item_prefix(last), address,);
    }

    println!("{}:", "Resource Definitions".green().bold());
    for (last, address) in ledger.list_resource_defs().iter().identify_last() {
        println!("{} {}", list_item_prefix(last), address,);
    }

    Ok(())
}
