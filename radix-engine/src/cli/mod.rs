mod export_abi;
mod invoke;
mod invoke_component;
mod publish;
mod reset;
mod show;
mod utils;

const CMD_EXPORT_ABI: &'static str = "export-abi";
const CMD_INVOKE: &'static str = "invoke";
const CMD_INVOKE_COMPONENT: &'static str = "invoke-component";
const CMD_PUBLISH: &'static str = "publish";
const CMD_RESET: &'static str = "reset";
const CMD_SHOW: &'static str = "show";

pub use export_abi::*;
pub use invoke::*;
pub use invoke_component::*;
pub use publish::*;
pub use reset::*;
pub use show::*;
pub use utils::*;

/// Runs Radix Engine CLI.
pub fn run() {
    let matches = clap::App::new("Radix Engine")
        .about("Build fast, reward everyone, and scale without friction")
        .version(clap::crate_version!())
        .subcommand(make_export_abi_cmd())
        .subcommand(make_invoke_cmd())
        .subcommand(make_invoke_component_cmd())
        .subcommand(make_publish_cmd())
        .subcommand(make_reset_cmd())
        .subcommand(make_show_cmd())
        .get_matches();

    match matches.subcommand() {
        (CMD_EXPORT_ABI, Some(m)) => handle_export_abi(m),
        (CMD_INVOKE, Some(m)) => handle_invoke(m),
        (CMD_INVOKE_COMPONENT, Some(m)) => handle_invoke_component(m),
        (CMD_PUBLISH, Some(m)) => handle_publish(m),
        (CMD_RESET, Some(m)) => handle_reset(m),
        (CMD_SHOW, Some(m)) => handle_show(m),
        _ => {}
    }
}
