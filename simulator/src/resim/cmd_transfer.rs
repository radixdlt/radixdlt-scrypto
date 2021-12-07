use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::transaction::*;

use crate::ledger::*;
use crate::resim::*;

const ARG_RESOURCE: &str = "RESOURCE";
const ARG_RECIPIENT_ADDRESS: &str = "RECIPIENT_ADDRESS";

const ARG_TRACE: &str = "TRACE";
const ARG_SIGNERS: &str = "SIGNERS";

/// Constructs a `transfer` subcommand.
pub fn make_transfer<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_TRANSFER)
        .about("Transfers resource to another account")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_RESOURCE)
                .help("Specify the resource to transfer, e.g. \"amount,resource_address\" or \"#nft_id1,#nft_id2,..,resource_address\".")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_RECIPIENT_ADDRESS)
                .help("Specify the recipient address.")
                .required(true),
        )
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

/// Handles a `transfer` request.
pub fn handle_transfer(matches: &ArgMatches) -> Result<(), Error> {
    let resource = match_resource(matches, ARG_RESOURCE)?;
    let recipient = match_address(matches, ARG_RECIPIENT_ADDRESS)?;
    let trace = matches.is_present(ARG_TRACE);
    let signers = match_signers(matches, ARG_SIGNERS)?;

    let mut configs = get_configs()?;
    let account = configs.default_account.ok_or(Error::NoDefaultAccount)?;
    let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
    let mut executor = TransactionExecutor::new(&mut ledger, configs.current_epoch, configs.nonce);
    let transaction = TransactionBuilder::new(&executor)
        .withdraw_from_account(&resource, account.0)
        .drop_all_bucket_refs()
        .deposit_all_buckets(recipient)
        .build(signers)
        .map_err(Error::TransactionConstructionError)?;
    let receipt = executor.run(transaction, trace).unwrap();

    println!("{:?}", receipt);
    if receipt.success {
        configs.nonce = executor.nonce();
        set_configs(configs)?;
        Ok(())
    } else {
        Err(Error::TransactionFailed)
    }
}
