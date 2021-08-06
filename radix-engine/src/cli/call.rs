use clap::{App, Arg, ArgMatches, SubCommand};
use scrypto::types::*;
use scrypto::utils::*;
use uuid::Uuid;

use crate::cli::*;
use crate::execution::*;
use crate::ledger::*;

pub fn prepare_call<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("call")
        .about("Call into a blueprint.")
        .version("1.0")
        .arg(
            Arg::with_name("BLUEPRINT")
                .help("Specify the blueprint address.")
                .required(true),
        )
        .arg(
            Arg::with_name("COMPONENT")
                .help("Specify the component name.")
                .required(true),
        )
        .arg(
            Arg::with_name("METHOD")
                .help("Specify the component method.")
                .required(true),
        )
        .arg(
            Arg::with_name("ARGS")
                .help("Specify the arguments.")
                .multiple(true),
        )
}

pub fn handle_call<'a>(matches: &ArgMatches<'a>) {
    let blueprint: Address = matches.value_of("BLUEPRINT").unwrap().into();
    let component = matches.value_of("COMPONENT").unwrap();
    let method = matches.value_of("METHOD").unwrap();
    let args = if let Some(x) = matches.values_of("ARGS") {
        x.map(|a| hex_decode(a).unwrap()).collect()
    } else {
        Vec::new()
    };
    println!("----");
    println!("Blueprint: {:?}", blueprint);
    println!("Component: {}", component);
    println!("Method: {}", method);
    println!("Arguments: {:?}", args);
    println!("----");

    let tx_hash = sha256(Uuid::new_v4().to_string());
    let ledger = FileBasedLedger::new(get_root_dir());
    let logger = Logger::new(Level::Info);
    let mut context = TransactionContext::new(tx_hash, ledger, logger);

    let code = context
        .get_blueprint(blueprint)
        .expect("Blueprint not found");
    let module = load_module(&code).expect("Unable to load blueprint");
    let (module_ref, memory_ref) =
        instantiate_module(&module).expect("Unable to instantiate module");
    let mut process = Process::new(
        &mut context,
        &module_ref,
        &memory_ref,
        blueprint,
        component.to_string(),
        method.to_string(),
        args,
        0,
    );

    let output = process.run();
    println!("Output: {:?}", output);
}
