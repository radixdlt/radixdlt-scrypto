mod cmd_call_function;
mod cmd_call_method;
mod cmd_config;
mod cmd_create_resource;
mod cmd_export_abi;
mod cmd_new_account;
mod cmd_new_package;
mod cmd_publish;
mod cmd_reset;
mod cmd_show;
mod config;
mod error;

pub use cmd_call_function::*;
pub use cmd_call_method::*;
pub use cmd_config::*;
pub use cmd_create_resource::*;
pub use cmd_export_abi::*;
pub use cmd_new_account::*;
pub use cmd_new_package::*;
pub use cmd_publish::*;
pub use cmd_reset::*;
pub use cmd_show::*;
pub use config::*;
pub use error::*;

pub const CONF_DEFAULT_ACCOUNT: &str = "default.account";

pub const CMD_EXPORT_ABI: &str = "export-abi";
pub const CMD_CALL_FUNCTION: &str = "call-function";
pub const CMD_CALL_METHOD: &str = "call-method";
pub const CMD_CREATE_RESOURCE: &str = "create-resource";
pub const CMD_NEW_ACCOUNT: &str = "new-account";
pub const CMD_NEW_PACKAGE: &str = "new-package";
pub const CMD_PUBLISH: &str = "publish";
pub const CMD_RESET: &str = "reset";
pub const CMD_CONFIG: &str = "config";
pub const CMD_SHOW: &str = "show";

pub fn run<I, T>(args: I) -> Result<(), Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let app = clap::App::new("Radix Engine Simulator")
        .name("rev2")
        .about("Build fast, reward everyone, and scale without friction")
        .version(clap::crate_version!())
        .subcommand(make_export_abi_cmd())
        .subcommand(make_call_function_cmd())
        .subcommand(make_call_method_cmd())
        .subcommand(make_create_resource_cmd())
        .subcommand(make_new_account_cmd())
        .subcommand(make_new_package_cmd())
        .subcommand(make_publish_cmd())
        .subcommand(make_reset_cmd())
        .subcommand(make_config_cmd())
        .subcommand(make_show_cmd());
    let matches = app.get_matches_from(args);

    match matches.subcommand() {
        (CMD_EXPORT_ABI, Some(m)) => handle_export_abi(m),
        (CMD_CALL_FUNCTION, Some(m)) => handle_call_function(m),
        (CMD_CALL_METHOD, Some(m)) => handle_call_method(m),
        (CMD_CREATE_RESOURCE, Some(m)) => handle_create_resource(m),
        (CMD_NEW_ACCOUNT, Some(m)) => handle_new_account(m),
        (CMD_NEW_PACKAGE, Some(m)) => handle_new_package(m),
        (CMD_PUBLISH, Some(m)) => handle_publish(m),
        (CMD_RESET, Some(m)) => handle_reset(m),
        (CMD_CONFIG, Some(m)) => handle_config(m),
        (CMD_SHOW, Some(m)) => handle_show(m),
        _ => Err(Error::MissingSubCommand),
    }
}
