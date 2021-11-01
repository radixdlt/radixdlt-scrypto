use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use colored::*;
use radix_engine::transaction::*;
use scrypto::types::*;

use crate::ledger::*;
use crate::resim::*;

const ARG_TRACE: &str = "TRACE";
const ARG_SIGNERS: &str = "SIGNERS";

/// Constructs a `new-account` subcommand.
pub fn make_new_account<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_NEW_ACCOUNT)
        .about("Creates an account")
        .version(crate_version!())
        // options
        .arg(
            Arg::with_name(ARG_TRACE)
                .long("trace")
                .help("Turn on tracing."),
        )
        .arg(
            Arg::with_name(ARG_SIGNERS)
                .long("signers")
                .takes_value(true)
                .help("Specify the transaction signers, separated by comma."),
        )
}

/// Handles a `new-account` request.
pub fn handle_new_account(matches: &ArgMatches) -> Result<(), Error> {
    let trace = matches.is_present(ARG_TRACE);
    let signers = match_signers(matches, ARG_SIGNERS)?;

    let mut configs = get_configs()?;
    let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
    let mut executor = TransactionExecutor::new(&mut ledger, configs.current_epoch, configs.nonce);
    let key = executor.new_public_key();
    let transaction = TransactionBuilder::new(&executor)
        .call_method(
            SYSTEM_COMPONENT,
            "free_xrd",
            vec!["1000000".to_owned()],
            None,
        )
        .new_account_with_resource(key, 1000000.into(), RADIX_TOKEN)
        .build(signers)
        .map_err(Error::TransactionConstructionError)?;
    let receipt = executor.run(transaction, trace).unwrap();
    println!("{:?}", receipt);

    if receipt.success {
        let account = receipt.component(0).unwrap();
        println!("{}", "=".repeat(80));
        println!("A new account has been created!");
        println!("Public key: {}", key.to_string().green());
        println!("Account address: {}", account.to_string().green());
        if configs.default_account.is_none() {
            println!("As this is the first account, it has been set as your default account.");
            configs.default_account = receipt.component(0);
        }
        println!("{}", "=".repeat(80));

        configs.nonce = executor.nonce();
        set_configs(configs)?;
        Ok(())
    } else {
        Err(Error::TransactionFailed)
    }
}
