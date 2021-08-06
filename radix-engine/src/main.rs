mod cli;
mod execution;
mod ledger;
mod model;

use clap::{crate_version, App};
use cli::*;

pub fn main() {
    let matches = App::new("Radix Engine")
        .about("Build fast, reward everyone, and scale without friction")
        .version(crate_version!())
        .subcommand(prepare_show())
        .subcommand(prepare_publish())
        .subcommand(prepare_call())
        .get_matches();

    match matches.subcommand() {
        ("show", Some(m)) => handle_show(m),
        ("publish", Some(m)) => handle_publish(m),
        ("call", Some(m)) => handle_call(m),
        _ => {}
    }
}
