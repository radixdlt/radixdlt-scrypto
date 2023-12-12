use reqwest::{blocking, header::*};
use serde::{Deserialize, Serialize};
use transaction::prelude::*;

#[derive(Debug)]
pub struct GatewayApiClient {
    url: String,
    client: blocking::Client,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReleaseInfo {
    pub release_version: String,
    pub open_api_schema_version: String,
    pub image_tag: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LedgerState {
    pub network: String,
    pub state_version: u32,
    pub proposer_round_timestamp: String,
    pub epoch: u64,
    pub round: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GatewayStatus {
    pub ledger_state: LedgerState,
    pub release_info: ReleaseInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionSubmit {
    pub duplicate: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KnownPayloads {
    pub payload_hash: String,
    pub status: String,
    pub payload_status: String,
    pub payload_status_description: String,
    pub handling_status: String,
    pub handling_status_reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionStatus {
    pub status: String,
    pub intent_status: String,
    pub ledger_state: LedgerState,
    pub intent_status_description: String,
    pub known_payloads: Vec<KnownPayloads>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionOutput {
    pub hex: String,
    pub programmatic_json: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionReceipt {
    pub status: String,
    pub output: Vec<TransactionOutput>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionDetailsStatus {
    pub transaction_status: String,
    pub state_version: u32,
    pub epoch: u32,
    pub round: u32,
    pub round_timestamp: String,
    pub payload_hash: String,
    pub intent_hash: String,
    pub fee_paid: String,
    pub confirmed_at: String,
    pub receipt: TransactionReceipt,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionDetails {
    pub ledger_state: LedgerState,
    pub transaction: TransactionDetailsStatus,
}

impl TransactionDetails {
    pub fn get_output(&self, idx: usize) -> String {
        self.transaction
            .receipt
            .output
            .get(idx)
            .unwrap()
            .hex
            .clone()
    }
}

impl GatewayApiClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: blocking::Client::new(),
        }
    }

    pub fn gateway_status(&self) -> GatewayStatus {
        let resp = self
            .client
            .post(self.url.clone() + "/status/gateway-status")
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .send()
            .unwrap()
            .text()
            .unwrap();

        let status: GatewayStatus = serde_json::from_str(&resp).unwrap();
        status
    }

    pub fn current_epoch(&self) -> u64 {
        self.gateway_status().ledger_state.epoch
    }

    pub fn transaction_submit(&self, transaction: NotarizedTransactionV1) -> TransactionSubmit {
        let notarized_transaction_bytes = transaction.to_payload_bytes().unwrap();
        let notarized_transaction_hex = hex::encode(&notarized_transaction_bytes);

        let mut map = HashMap::new();
        map.insert("notarized_transaction_hex", notarized_transaction_hex);

        let resp = self
            .client
            .post(self.url.clone() + "/transaction/submit")
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&map)
            .send()
            .unwrap()
            .text()
            .unwrap();
        let status: TransactionSubmit = serde_json::from_str(&resp).unwrap();
        status
    }

    pub fn transaction_status(&self, intent_hash: &str) -> TransactionStatus {
        let mut map = HashMap::new();
        map.insert("intent_hash", intent_hash);

        let resp = self
            .client
            .post(self.url.clone() + "/transaction/status")
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&map)
            .send()
            .unwrap()
            .text()
            .unwrap();

        let status: TransactionStatus = serde_json::from_str(&resp).unwrap();
        status
    }

    pub fn transaction_details(&self, intent_hash: &str) -> TransactionDetails {
        let mut map = HashMap::new();
        map.insert("intent_hash", intent_hash);

        let resp = self
            .client
            .post(self.url.clone() + "/transaction/committed-details")
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .json(&map)
            .send()
            .unwrap()
            .text()
            .unwrap();
        let status: TransactionDetails = serde_json::from_str(&resp).unwrap();
        status
    }
}
