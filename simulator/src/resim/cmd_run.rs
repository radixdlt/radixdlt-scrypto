use clap::{crate_version, App, Arg, ArgMatches};
use radix_engine::model::*;
use radix_engine::transaction::*;

use crate::ledger::*;
use crate::resim::*;

const ARG_TRANSACTION_MANIFEST: &str = "MANIFEST";

const ARG_TRACE: &str = "TRACE";
const ARG_SIGNERS: &str = "SIGNERS";

/// Constructs a `run` subcommand.
pub fn make_run<'a>() -> App<'a> {
    App::new(CMD_RUN)
        .about("Runs a transaction manifest")
        .version(crate_version!())
        .arg(
            Arg::new(ARG_TRANSACTION_MANIFEST)
                .help("Specify the transaction manifest path.")
                .required(true),
        )
        // options
        .arg(Arg::new(ARG_TRACE).long("trace").help("Turn on tracing."))
        .arg(
            Arg::new(ARG_SIGNERS)
                .long("signers")
                .takes_value(true)
                .help("Specify the transaction signers, separated by comma."),
        )
}

/// Handles a `run` request.
pub fn handle_run(matches: &ArgMatches) -> Result<(), Error> {
    let manifest_path = match_path(matches, ARG_TRANSACTION_MANIFEST)?;
    let trace = matches.is_present(ARG_TRACE);
    let signers = match_signers(matches, ARG_SIGNERS)?;

    let manifest = std::fs::read_to_string(manifest_path).map_err(Error::IOError)?;
    let mut transaction = transaction_manifest::compile(&manifest).map_err(Error::CompileError)?;
    transaction.instructions.push(Instruction::End {
        signatures: signers,
    });

    let mut configs = get_configs()?;
    let mut ledger = FileBasedLedger::with_bootstrap(get_data_dir()?);
    let mut executor =
        TransactionExecutor::new(&mut ledger, configs.current_epoch, configs.nonce, trace);
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
