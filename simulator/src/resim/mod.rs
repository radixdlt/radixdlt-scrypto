mod cmd_call_function;
mod cmd_call_method;
mod cmd_export_abi;
mod cmd_mint;
mod cmd_new_account;
mod cmd_new_badge_fixed;
mod cmd_new_badge_mutable;
mod cmd_new_token_fixed;
mod cmd_new_token_mutable;
mod cmd_publish;
mod cmd_reset;
mod cmd_run;
mod cmd_set_current_epoch;
mod cmd_set_default_account;
mod cmd_show;
mod cmd_show_configs;
mod cmd_show_ledger;
mod cmd_transfer;
mod config;
mod error;
mod utils;

pub use cmd_call_function::*;
pub use cmd_call_method::*;
pub use cmd_export_abi::*;
pub use cmd_mint::*;
pub use cmd_new_account::*;
pub use cmd_new_badge_fixed::*;
pub use cmd_new_badge_mutable::*;
pub use cmd_new_token_fixed::*;
pub use cmd_new_token_mutable::*;
pub use cmd_publish::*;
pub use cmd_reset::*;
pub use cmd_run::*;
pub use cmd_set_current_epoch::*;
pub use cmd_set_default_account::*;
pub use cmd_show::*;
pub use cmd_show_configs::*;
pub use cmd_show_ledger::*;
pub use cmd_transfer::*;
pub use config::*;
pub use error::*;
pub use utils::*;

pub const CMD_EXPORT_ABI: &str = "export-abi";
pub const CMD_CALL_FUNCTION: &str = "call-function";
pub const CMD_CALL_METHOD: &str = "call-method";
pub const CMD_NEW_ACCOUNT: &str = "new-account";
pub const CMD_NEW_TOKEN_FIXED: &str = "new-token-fixed";
pub const CMD_NEW_TOKEN_MUTABLE: &str = "new-token-mutable";
pub const CMD_NEW_BADGE_FIXED: &str = "new-badge-fixed";
pub const CMD_NEW_BADGE_MUTABLE: &str = "new-badge-mutable";
pub const CMD_MINT: &str = "mint";
pub const CMD_TRANSFER: &str = "transfer";
pub const CMD_PUBLISH: &str = "publish";
pub const CMD_RESET: &str = "reset";
pub const CMD_RUN: &str = "run";
pub const CMD_SET_DEFAULT_ACCOUNT: &str = "set-default-account";
pub const CMD_SET_CURRENT_EPOCH: &str = "set-current-epoch";
pub const CMD_SHOW: &str = "show";
pub const CMD_SHOW_CONFIGS: &str = "show-configs";
pub const CMD_SHOW_LEDGER: &str = "show-ledger";

/// Runs resim CLI.
pub fn run<I, T>(args: I) -> Result<(), Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let app = clap::App::new("Radix Engine Simulator")
        .name("resim")
        .about("Build fast, reward everyone, and scale without friction")
        .version(clap::crate_version!())
        .subcommand(make_export_abi())
        .subcommand(make_call_function())
        .subcommand(make_call_method())
        .subcommand(make_new_token_fixed())
        .subcommand(make_new_token_mutable())
        .subcommand(make_new_badge_fixed())
        .subcommand(make_new_badge_mutable())
        .subcommand(make_mint())
        .subcommand(make_transfer())
        .subcommand(make_new_account())
        .subcommand(make_publish())
        .subcommand(make_reset())
        .subcommand(make_run())
        .subcommand(make_set_default_account())
        .subcommand(make_set_current_epoch())
        .subcommand(make_show())
        .subcommand(make_show_configs())
        .subcommand(make_show_ledger());
    let matches = app.get_matches_from(args);

    match matches.subcommand() {
        Some((CMD_EXPORT_ABI, m)) => handle_export_abi(m),
        Some((CMD_CALL_FUNCTION, m)) => handle_call_function(m),
        Some((CMD_CALL_METHOD, m)) => handle_call_method(m),
        Some((CMD_NEW_TOKEN_FIXED, m)) => handle_new_token_fixed(m),
        Some((CMD_NEW_TOKEN_MUTABLE, m)) => handle_new_token_mutable(m),
        Some((CMD_NEW_BADGE_FIXED, m)) => handle_new_badge_fixed(m),
        Some((CMD_NEW_BADGE_MUTABLE, m)) => handle_new_badge_mutable(m),
        Some((CMD_MINT, m)) => handle_mint(m),
        Some((CMD_TRANSFER, m)) => handle_transfer(m),
        Some((CMD_NEW_ACCOUNT, m)) => handle_new_account(m),
        Some((CMD_PUBLISH, m)) => handle_publish(m),
        Some((CMD_RESET, m)) => handle_reset(m),
        Some((CMD_RUN, m)) => handle_run(m),
        Some((CMD_SET_DEFAULT_ACCOUNT, m)) => handle_set_default_account(m),
        Some((CMD_SET_CURRENT_EPOCH, m)) => handle_set_current_epoch(m),
        Some((CMD_SHOW, m)) => handle_show(m),
        Some((CMD_SHOW_CONFIGS, m)) => handle_show_configs(m),
        Some((CMD_SHOW_LEDGER, m)) => handle_show_ledger(m),
        _ => Err(Error::MissingSubCommand),
    }
}
