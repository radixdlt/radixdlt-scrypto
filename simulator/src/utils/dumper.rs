use colored::*;
use radix_engine::ledger::*;
use radix_engine::model::*;
use scrypto::types::*;

use crate::txn::*;
use crate::utils::*;

pub fn dump_package<T: Ledger>(address: Address, ledger: &T) {
    let package = ledger.get_package(address);
    match package {
        Some(b) => {
            println!("{}: {}", "Package".green().bold(), address.to_string());
            println!("{}: {} bytes", "Code size".green().bold(), b.code().len());
        }
        None => {
            println!("{}", "Package not found".red());
        }
    }
}

pub fn dump_component<T: Ledger>(address: Address, ledger: &T) {
    let component = ledger.get_component(address);
    match component {
        Some(c) => {
            println!("{}: {}", "Component".green().bold(), address.to_string());

            println!(
                "{}: {}::{}",
                "Blueprint".green().bold(),
                c.package(),
                c.blueprint()
            );

            println!("{}: {:02x?}", "State".green().bold(), c.state());

            let mut res = Vec::new();
            println!(
                "{}: {}",
                "State Parsed".green().bold(),
                format_sbor(c.state(), ledger, &mut res)
                    .unwrap_or("Failed to parse data".to_owned())
            );

            println!("{}:", "Resources".green().bold());
            for (last, b) in res.iter().identify_last() {
                println!(
                    "{} {{ amount: {}, resource: {} }}",
                    list_item_prefix(last),
                    b.amount(),
                    b.resource(),
                );
            }
        }
        None => {
            println!("{}", "Component not found".red());
        }
    }
}

pub fn dump_resource<T: Ledger>(address: Address, ledger: &T) {
    let resource = ledger.get_resource(address);
    match resource {
        Some(r) => {
            println!("{}: {}", "Resource".green().bold(), address.to_string());
            println!("{}: {}", "Symbol".green().bold(), r.symbol);
            println!("{}: {}", "Name".green().bold(), r.name);
            println!("{}: {}", "Description".green().bold(), r.description);
            println!("{}: {}", "URL".green().bold(), r.url);
            println!("{}: {}", "Icon URL".green().bold(), r.icon_url);
            println!("{}: {:?}", "Minter".green().bold(), r.minter);
            println!("{}: {:?}", "supply".green().bold(), r.supply);
        }
        None => {
            println!("{}", "Resource not found".red());
        }
    }
}

pub fn dump_receipt(receipt: TransactionReceipt) {
    println!(
        "{} {}",
        "Transaction Status:".bold().green(),
        if receipt.success {
            "SUCCESS".blue()
        } else {
            "FAILURE".red()
        }
        .bold()
    );

    println!("{}", "Instructions:".bold().green());
    for (last, inst) in receipt.transaction.instructions.iter().identify_last() {
        println!("{} {:02x?}", list_item_prefix(last), inst);
    }

    println!("{}", "Results:".bold().green());
    for (last, result) in receipt.results.iter().identify_last() {
        println!("{} {:02x?}", list_item_prefix(last), result);
    }

    println!("{}", "Logs:".bold().green());
    for (last, (level, msg)) in receipt.logs.iter().identify_last() {
        let (l, m) = match level {
            Level::Error => ("ERROR".red(), msg.red()),
            Level::Warn => ("WARN".yellow(), msg.yellow()),
            Level::Info => ("INFO".green(), msg.green()),
            Level::Debug => ("DEBUG".cyan(), msg.cyan()),
            Level::Trace => ("TRACE".normal(), msg.normal()),
        };
        println!("{} [{:5}] {}", list_item_prefix(last), l, m);
    }

    println!(
        "{} {} ms",
        "Execution Time:".bold().green(),
        receipt.execution_time
    );
}
