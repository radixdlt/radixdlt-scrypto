use clap::{crate_version, App, Arg, ArgMatches};
use radix_engine::transaction::*;

use crate::ledger::*;
use crate::resim::*;

const ARG_RESOURCE: &str = "RESOURCE";
const ARG_RECIPIENT_ADDRESS: &str = "RECIPIENT_ADDRESS";

const ARG_TRACE: &str = "TRACE";
const ARG_SIGNERS: &str = "SIGNERS";

/// Constructs a `transfer` subcommand.
pub fn make_transfer<'a>() -> App<'a> {
    App::new(CMD_TRANSFER)
        .about("Transfers resource to another account")
        .version(crate_version!())
        .arg(
            Arg::new(ARG_RESOURCE)
                .help("Specify the resource to transfer, e.g. \"amount,resource_address\" or \"#nft_id1,#nft_id2,..,resource_address\".")
                .required(true),
        )
        .arg(
            Arg::new(ARG_RECIPIENT_ADDRESS)
                .help("Specify the recipient address.")
                .required(true),
        )
        // options
        .arg(
            Arg::new(ARG_TRACE)
                .long("trace")
                .help("Turn on tracing."),
        )
        .arg(
            Arg::new(ARG_SIGNERS)
                .long("signers")
                .takes_value(true)
                .help("Specify the transaction signers, separated by comma."),
        )
}

/// Handles a `transfer` request.
pub fn handle_transfer(matches: &ArgMatches) -> Result<(), Error> {
    let resource = match_resource(matches, ARG_RESOURCE)?;
    let recipient = match_address(matches, ARG_RECIPIENT_ADDRESS)?;
    let trace = matches.is_present(ARG_TRACE);
    let signers = match_signers(matches, ARG_SIGNERS)?;

    let mut configs = get_configs()?;
    let account = configs.default_account.ok_or(Error::NoDefaultAccount)?;
    let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
    let mut executor =
        TransactionExecutor::new(&mut ledger, configs.current_epoch, configs.nonce, trace);
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&resource, account.0)
        .call_method_with_all_resources(recipient, "deposit_batch")
        .build(signers)
        .map_err(Error::TransactionConstructionError)?;
    let receipt = executor.run(transaction).unwrap();

    println!("{:?}", receipt);
    if receipt.error.is_none() {
        configs.nonce = executor.nonce();
        set_configs(configs)?;
        Ok(())
    } else {
        Err(Error::TransactionFailed)
    }
}
