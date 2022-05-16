use clap::Parser;
use regex::{Captures, Regex};
use std::env;
use std::path::PathBuf;

use crate::resim::*;

/// Compiles, signs and runs a transaction manifest
#[derive(Parser, Debug)]
pub struct Run {
    /// The path to a transaction manifest file
    path: PathBuf,

    /// The private keys used for signing, separated by comma
    #[clap(short, long)]
    signing_keys: Option<String>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl Run {
    pub fn pre_process_manifest(manifest: &str) -> String {
        let re = Regex::new(r"\$\{(.+?)\}").unwrap();
        re.replace_all(manifest, |caps: &Captures| {
            env::var(&caps[1].trim()).unwrap_or_default()
        })
        .into()
    }

    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let mut substate_store = RadixEngineDB::new(get_data_dir()?);
        let mut wasm_engine = default_wasm_engine();
        let mut executor =
            TransactionExecutor::new(&mut substate_store, &mut wasm_engine, self.trace);
        let manifest = std::fs::read_to_string(&self.path).map_err(Error::IOError)?;
        let pre_processed_manifest = Self::pre_process_manifest(&manifest);
        let transaction =
            transaction_manifest::compile(&pre_processed_manifest).map_err(Error::CompileError)?;
        process_transaction(&mut executor, transaction, &self.signing_keys, &None, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_process_manifest() {
        temp_env::with_vars(
            vec![
                (
                    "system",
                    Some("020000000000000000000000000000000000000000000000000002"),
                ),
                (
                    "xrd",
                    Some("030000000000000000000000000000000000000000000000000004"),
                ),
            ],
            || {
                let manifest = r#"CALL_METHOD ComponentAddress("${  system  }") "free_xrd";\nTAKE_FROM_WORKTOP ResourceAddress("${xrd}") Bucket("bucket1");\n"#;
                let after = r#"CALL_METHOD ComponentAddress("020000000000000000000000000000000000000000000000000002") "free_xrd";\nTAKE_FROM_WORKTOP ResourceAddress("030000000000000000000000000000000000000000000000000004") Bucket("bucket1");\n"#;
                assert_eq!(Run::pre_process_manifest(manifest), after);
            },
        );
    }
}
