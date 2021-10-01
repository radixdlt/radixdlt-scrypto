mod cmd_call_function;
mod cmd_call_method;
mod cmd_export_abi;
mod cmd_mint;
mod cmd_new_account;
mod cmd_new_resource_fixed;
mod cmd_new_resource_mutable;
mod cmd_publish;
mod cmd_reset;
mod cmd_set_current_epoch;
mod cmd_set_default_account;
mod cmd_show;
mod cmd_show_configs;
mod cmd_transfer;
mod config;
mod error;
mod utils;

pub use cmd_call_function::*;
pub use cmd_call_method::*;
pub use cmd_export_abi::*;
pub use cmd_mint::*;
pub use cmd_new_account::*;
pub use cmd_new_resource_fixed::*;
pub use cmd_new_resource_mutable::*;
pub use cmd_publish::*;
pub use cmd_reset::*;
pub use cmd_set_current_epoch::*;
pub use cmd_set_default_account::*;
pub use cmd_show::*;
pub use cmd_show_configs::*;
pub use cmd_transfer::*;
pub use config::*;
pub use error::*;
pub use utils::*;

pub const CMD_EXPORT_ABI: &str = "export-abi";
pub const CMD_CALL_FUNCTION: &str = "call-function";
pub const CMD_CALL_METHOD: &str = "call-method";
pub const CMD_NEW_ACCOUNT: &str = "new-account";
pub const CMD_NEW_RESOURCE_FIXED: &str = "new-resource-fixed";
pub const CMD_NEW_RESOURCE_MUTABLE: &str = "new-resource-mutable";
pub const CMD_MINT: &str = "mint";
pub const CMD_TRANSFER: &str = "transfer";
pub const CMD_PUBLISH: &str = "publish";
pub const CMD_RESET: &str = "reset";
pub const CMD_SET_DEFAULT_ACCOUNT: &str = "set-default-account";
pub const CMD_SET_CURRENT_EPOCH: &str = "set-current-epoch";
pub const CMD_SHOW: &str = "show";
pub const CMD_SHOW_CONFIGS: &str = "show-configs";

pub fn run<I, T>(args: I) -> Result<(), Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let app = clap::App::new("Radix Engine Simulator")
        .name("rev2")
        .about("Build fast, reward everyone, and scale without friction")
        .version(clap::crate_version!())
        .subcommand(make_export_abi())
        .subcommand(make_call_function())
        .subcommand(make_call_method())
        .subcommand(make_new_resource_fixed())
        .subcommand(make_new_resource_mutable())
        .subcommand(make_mint())
        .subcommand(make_transfer())
        .subcommand(make_new_account())
        .subcommand(make_publish())
        .subcommand(make_reset())
        .subcommand(make_set_default_account())
        .subcommand(make_set_current_epoch())
        .subcommand(make_show())
        .subcommand(make_show_configs());
    let matches = app.get_matches_from(args);

    match matches.subcommand() {
        (CMD_EXPORT_ABI, Some(m)) => handle_export_abi(m),
        (CMD_CALL_FUNCTION, Some(m)) => handle_call_function(m),
        (CMD_CALL_METHOD, Some(m)) => handle_call_method(m),
        (CMD_NEW_RESOURCE_FIXED, Some(m)) => handle_new_resource_fixed(m),
        (CMD_NEW_RESOURCE_MUTABLE, Some(m)) => handle_new_resource_mutable(m),
        (CMD_MINT, Some(m)) => handle_mint(m),
        (CMD_TRANSFER, Some(m)) => handle_transfer(m),
        (CMD_NEW_ACCOUNT, Some(m)) => handle_new_account(m),
        (CMD_PUBLISH, Some(m)) => handle_publish(m),
        (CMD_RESET, Some(m)) => handle_reset(m),
        (CMD_SET_DEFAULT_ACCOUNT, Some(m)) => handle_set_default_account(m),
        (CMD_SET_CURRENT_EPOCH, Some(m)) => handle_set_current_epoch(m),
        (CMD_SHOW, Some(m)) => handle_show(m),
        (CMD_SHOW_CONFIGS, Some(m)) => handle_show_configs(m),
        _ => Err(Error::MissingSubCommand),
    }
}
