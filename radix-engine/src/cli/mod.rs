mod call;
mod clear;
mod publish;
mod show;
mod utils;

pub use call::{handle_call, prepare_call};
pub use clear::{handle_clear, prepare_clear};
pub use publish::{handle_publish, prepare_publish};
pub use show::{handle_show, prepare_show};
pub use utils::get_root_dir;

use clap::{crate_version, App};

pub fn run() {
    let matches = App::new("Radix Engine")
        .about("Build fast, reward everyone, and scale without friction")
        .version(crate_version!())
        .subcommand(prepare_show())
        .subcommand(prepare_publish())
        .subcommand(prepare_call())
        .subcommand(prepare_clear())
        .get_matches();

    match matches.subcommand() {
        ("show", Some(m)) => handle_show(m),
        ("publish", Some(m)) => handle_publish(m),
        ("call", Some(m)) => handle_call(m),
        ("clear", Some(m)) => handle_clear(m),
        _ => {}
    }
}
