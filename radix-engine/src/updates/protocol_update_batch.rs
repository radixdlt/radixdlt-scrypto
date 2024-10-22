use crate::internal_prelude::*;
use radix_transactions::model::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolUpdateTransaction {
    FlashTransactionV1(FlashTransactionV1),
    SystemTransactionV1(ProtocolSystemTransactionV1),
}

impl From<FlashTransactionV1> for ProtocolUpdateTransaction {
    fn from(value: FlashTransactionV1) -> Self {
        Self::FlashTransactionV1(value)
    }
}

impl From<ProtocolSystemTransactionV1> for ProtocolUpdateTransaction {
    fn from(value: ProtocolSystemTransactionV1) -> Self {
        Self::SystemTransactionV1(value)
    }
}

impl ProtocolUpdateTransaction {
    pub fn flash(name: impl Into<String>, state_updates: StateUpdates) -> Self {
        let name = name.into();
        if name != name.to_ascii_lowercase().as_str() {
            panic!("Protocol transaction names should be in kebab-case for consistency");
        }
        Self::FlashTransactionV1(FlashTransactionV1 {
            name: name.into(),
            state_updates,
        })
    }

    pub fn genesis_transaction(name: impl Into<String>, transaction: SystemTransactionV1) -> Self {
        let name = name.into();
        if name != name.to_ascii_lowercase().as_str() {
            panic!("Protocol transaction names should be in kebab-case for consistency");
        }
        Self::SystemTransactionV1(ProtocolSystemTransactionV1 {
            name: name.into(),
            disable_auth: true,
            transaction,
        })
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            ProtocolUpdateTransaction::FlashTransactionV1(tx) => Some(tx.name.as_str()),
            ProtocolUpdateTransaction::SystemTransactionV1(tx) => Some(tx.name.as_str()),
        }
    }
}

/// At present, this isn't actually saved in the node - instead just the
/// SystemTransactionV1 is saved.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct ProtocolSystemTransactionV1 {
    pub name: String,
    pub disable_auth: bool,
    pub transaction: SystemTransactionV1,
}

/// A set of transactions which all get committed together with the same proof.
/// To avoid memory overflows, this should be kept small enough to comfortably fit into
/// memory (e.g. one transaction per batch).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolUpdateBatch {
    pub transactions: Vec<ProtocolUpdateTransaction>,
}

impl ProtocolUpdateBatch {
    pub fn empty() -> Self {
        Self {
            transactions: vec![],
        }
    }

    pub fn new(transactions: impl IntoIterator<Item = ProtocolUpdateTransaction>) -> Self {
        Self {
            transactions: transactions.into_iter().collect(),
        }
    }

    pub fn add_flash(mut self, name: impl Into<String>, updates: StateUpdates) -> Self {
        self.mut_add_flash(name, updates);
        self
    }

    pub fn mut_add_flash(&mut self, name: impl Into<String>, updates: StateUpdates) {
        self.mut_add(ProtocolUpdateTransaction::flash(name, updates))
    }

    pub fn add(mut self, transaction: ProtocolUpdateTransaction) -> Self {
        self.mut_add(transaction);
        self
    }

    pub fn mut_add(&mut self, transaction: ProtocolUpdateTransaction) {
        self.transactions.push(transaction);
    }

    pub fn single(single_transaction: ProtocolUpdateTransaction) -> Self {
        Self {
            transactions: vec![single_transaction],
        }
    }
}
