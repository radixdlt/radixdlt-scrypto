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
                "{}: {{ package: {}, name: {:?} }}",
                "Blueprint".green().bold(),
                c.package(),
                c.blueprint()
            );
            let mut vaults = vec![];
            println!(
                "{}: {}",
                "State".green().bold(),
                format_sbor_with_ledger(c.state(), ledger, &mut vaults).unwrap()
            );

            println!("{}:", "Resources".green().bold());
            for (last, vid) in vaults.iter().identify_last() {
                let vault = ledger.get_vault(*vid).unwrap();
                println!(
                    "{} {{ amount: {}, resource: {} }}",
                    list_item_prefix(last),
                    vault.amount(),
                    vault.resource(),
                );
            }
        }
        None => {
            println!("{}", "Component not found".red());
        }
    }
}

pub fn dump_resource<T: Ledger>(address: Address, ledger: &T) {
    let resource = ledger.get_resource_def(address);
    match resource {
        Some(r) => {
            for (k, v) in r.metadata {
                println!("{}: {}", k.green().bold(), v);
            }
            println!("{}: {:?}", "Minter".green().bold(), r.minter);
            println!("{}: {:?}", "supply".green().bold(), r.supply);
        }
        None => {
            println!("{}", "Resource not found".red());
        }
    }
}

pub fn dump_receipt(receipt: &TransactionReceipt) {
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
        println!("{} {:?}", list_item_prefix(last), inst);
    }

    println!("{}", "Results:".bold().green());
    for (last, result) in receipt.results.iter().identify_last() {
        let msg = match result {
            Ok(r) => match r {
                Some(rtn) => {
                    format!("Ok({})", format_sbor(rtn).unwrap())
                }
                None => "Ok".to_string(),
            },
            Err(err) => format!("Err({:?})", err),
        };
        println!("{} {}", list_item_prefix(last), msg);
    }

    println!("{} {}", "Logs:".bold().green(), receipt.logs.len());
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
        "{} {}",
        "New Addresses:".bold().green(),
        receipt.new_addresses.len()
    );
    for (last, address) in receipt.new_addresses.iter().identify_last() {
        let ty = match address {
            Address::Package(_) => "Package",
            Address::Component(_) => "Component",
            Address::Resource(_) => "Resource",
            _ => "Other",
        };
        println!("{} {}: {}", list_item_prefix(last), ty, address);
    }

    println!(
        "{} {} ms",
        "Execution Time:".bold().green(),
        receipt.execution_time
    );
}
