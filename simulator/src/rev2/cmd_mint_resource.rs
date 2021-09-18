use clap::{crate_version, App, Arg, ArgMatches, SubCommand};
use radix_engine::execution::*;
use scrypto::buffer::*;
use scrypto::rust::str::FromStr;
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::ledger::*;
use crate::rev2::*;

const ARG_TRACE: &str = "TRACE";
const ARG_AMOUNT: &str = "AMOUNT";
const ARG_RESOURCE: &str = "RESOURCE";

/// Constructs a `mint-resource` subcommand.
pub fn make_mint_resource<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name(CMD_MINT_RESOURCE)
        .about("Mints resource")
        .version(crate_version!())
        .arg(
            Arg::with_name(ARG_TRACE)
                .short("t")
                .long("trace")
                .help("Turns on tracing."),
        )
        .arg(
            Arg::with_name(ARG_AMOUNT)
                .help("Specify the amount to mint.")
                .required(true),
        )
        .arg(
            Arg::with_name(ARG_RESOURCE)
                .help("Specify the resource address.")
                .required(true),
        )
}

/// Handles a `mint-resource` request.
pub fn handle_mint_resource(matches: &ArgMatches) -> Result<(), Error> {
    let trace = matches.is_present(ARG_TRACE);
    let amount = Amount::from_str(
        matches
            .value_of(ARG_AMOUNT)
            .ok_or_else(|| Error::MissingArgument(ARG_AMOUNT.to_owned()))?,
    )
    .map_err(|_| Error::InvalidAmount)?;
    let resource: Address = matches
        .value_of(ARG_RESOURCE)
        .ok_or_else(|| Error::MissingArgument(ARG_RESOURCE.to_owned()))?
        .parse()
        .map_err(Error::InvalidAddress)?;

    match get_config(CONF_DEFAULT_ACCOUNT)? {
        Some(a) => {
            let account: Address = a.as_str().parse().map_err(Error::InvalidAddress)?;
            let tx_hash = sha256(Uuid::new_v4().to_string());
            let mut ledger = FileBasedLedger::new(get_data_dir()?);
            let mut runtime = Runtime::new(tx_hash, &mut ledger);

            let mut process = Process::new(0, trace, &mut runtime);
            process
                .prepare_call_method(
                    account,
                    "mint_resource".to_owned(),
                    vec![scrypto_encode(&amount), scrypto_encode(&resource)],
                )
                .and_then(|target| process.run(target))
                .map_err(Error::TxnExecutionError)?;
            process.finalize().map_err(Error::TxnExecutionError)?;
            runtime.flush();

            println!("Done!");
            Ok(())
        }
        None => Err(Error::NoDefaultAccount),
    }
}
