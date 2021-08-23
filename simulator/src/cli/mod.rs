mod export_abi;
mod invoke_blueprint;
mod invoke_component;
mod new_account;
mod publish;
mod reset;
mod show;

const CMD_EXPORT_ABI: &'static str = "export-abi";
const CMD_INVOKE_BLUEPRINT: &'static str = "invoke-blueprint";
const CMD_INVOKE_COMPONENT: &'static str = "invoke-component";
const CMD_NEW_ACCOUNT: &'static str = "new-account";
const CMD_PUBLISH: &'static str = "publish";
const CMD_RESET: &'static str = "reset";
const CMD_SHOW: &'static str = "show";

pub use export_abi::*;
pub use invoke_blueprint::*;
pub use invoke_component::*;
pub use new_account::*;
pub use publish::*;
pub use reset::*;
pub use show::*;

pub fn run<I, T>(itr: I)
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let matches = clap::App::new("Radix Engine Simulator")
        .about("Build fast, reward everyone, and scale without friction")
        .version(clap::crate_version!())
        .subcommand(make_export_abi_cmd())
        .subcommand(make_invoke_blueprint_cmd())
        .subcommand(make_invoke_component_cmd())
        .subcommand(make_new_account_cmd())
        .subcommand(make_publish_cmd())
        .subcommand(make_reset_cmd())
        .subcommand(make_show_cmd())
        .get_matches_from(itr);

    match matches.subcommand() {
        (CMD_EXPORT_ABI, Some(m)) => handle_export_abi(m),
        (CMD_INVOKE_BLUEPRINT, Some(m)) => handle_invoke_blueprint(m),
        (CMD_INVOKE_COMPONENT, Some(m)) => handle_invoke_component(m),
        (CMD_NEW_ACCOUNT, Some(m)) => handle_new_account(m),
        (CMD_PUBLISH, Some(m)) => handle_publish(m),
        (CMD_RESET, Some(m)) => handle_reset(m),
        (CMD_SHOW, Some(m)) => handle_show(m),
        _ => {}
    }
}
