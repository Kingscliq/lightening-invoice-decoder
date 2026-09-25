use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DecodedInvoice {
    pub network: String,
    pub amount_msat: Option<u64>,
    pub description: Option<String>,
    pub description_hash: Option<String>,
    pub payment_hash: String,
    pub payee_public_key: String,
    pub created_at_unix: u64,
    pub expires_at_unix: u64,
    pub expiry_seconds: u64,
    pub expired: bool,
    pub signature_valid: bool,
    pub min_final_cltv_expiry_delta: u64,
    pub fallback_addresses: Vec<String>,
    pub route_hints: Vec<RouteHint>,
    pub features: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RouteHint {
    pub hops: Vec<RouteHintHop>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RouteHintHop {
    pub source_node_id: String,
    pub short_channel_id: u64,
    pub base_fee_msat: u32,
    pub proportional_fee_millionths: u32,
    pub cltv_expiry_delta: u16,
    pub htlc_minimum_msat: Option<u64>,
    pub htlc_maximum_msat: Option<u64>,
}
