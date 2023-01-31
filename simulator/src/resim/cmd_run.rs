use clap::Parser;
use regex::{Captures, Regex};
use std::env;
use std::path::PathBuf;

use crate::resim::*;

/// Compiles, signs and runs a transaction manifest
#[derive(Parser, Debug)]
pub struct Run {
    /// The path to a transaction manifest file
    pub path: PathBuf,

    /// The network to use when outputting manifest, [simulator | adapanet | nebunet | mainnet]
    #[clap(short, long)]
    pub network: Option<String>,

    /// The paths to blobs
    #[clap(short, long, multiple = true)]
    pub blobs: Option<Vec<String>>,

    /// The private keys used for signing, separated by comma
    #[clap(short, long)]
    pub signing_keys: Option<String>,

    /// Turn on tracing
    #[clap(short, long)]
    pub trace: bool,
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
        let manifest = std::fs::read_to_string(&self.path).map_err(Error::IOError)?;
        let pre_processed_manifest = Self::pre_process_manifest(&manifest);
        let network = match &self.network {
            Some(n) => NetworkDefinition::from_str(&n).map_err(Error::ParseNetworkError)?,
            None => NetworkDefinition::simulator(),
        };
        let mut blobs = Vec::new();
        if let Some(paths) = &self.blobs {
            for path in paths {
                blobs.push(std::fs::read(path).map_err(Error::IOError)?);
            }
        }
        let compiled_manifest =
            transaction::manifest::compile(&pre_processed_manifest, &network, blobs)
                .map_err(Error::CompileError)?;
        handle_manifest(
            compiled_manifest,
            &self.signing_keys,
            &self.network,
            &None,
            self.trace,
            true,
            out,
        )
        .map(|_| ())
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
                    "faucet",
                    Some("system_sim1qsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpql4sktx"),
                ),
                (
                    "xrd",
                    Some("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag"),
                ),
            ],
            || {
                let manifest = r#"CALL_METHOD ComponentAddress("${  faucet  }") "free";\nTAKE_FROM_WORKTOP ResourceAddress("${xrd}") Bucket("bucket1");\n"#;
                let after = r#"CALL_METHOD ComponentAddress("system_sim1qsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpql4sktx") "free";\nTAKE_FROM_WORKTOP ResourceAddress("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag") Bucket("bucket1");\n"#;
                assert_eq!(Run::pre_process_manifest(manifest), after);
            },
        );
    }
}
