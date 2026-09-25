use invoice_decoder::{DecodedInvoice, RouteHint, RouteHintHop};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiErrorResponse {
    pub error: ApiErrorBody,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiErrorBody {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DecodedInvoiceResponse {
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
    pub route_hints: Vec<RouteHintResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RouteHintResponse {
    pub hops: Vec<RouteHintHopResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RouteHintHopResponse {
    pub source_node_id: String,
    pub short_channel_id: u64,
    pub base_fee_msat: u32,
    pub proportional_fee_millionths: u32,
    pub cltv_expiry_delta: u16,
    pub htlc_minimum_msat: Option<u64>,
    pub htlc_maximum_msat: Option<u64>,
}

impl From<DecodedInvoice> for DecodedInvoiceResponse {
    fn from(invoice: DecodedInvoice) -> Self {
        Self {
            network: invoice.network,
            amount_msat: invoice.amount_msat,
            description: invoice.description,
            description_hash: invoice.description_hash,
            payment_hash: invoice.payment_hash,
            payee_public_key: invoice.payee_public_key,
            created_at_unix: invoice.created_at_unix,
            expires_at_unix: invoice.expires_at_unix,
            expiry_seconds: invoice.expiry_seconds,
            expired: invoice.expired,
            signature_valid: invoice.signature_valid,
            min_final_cltv_expiry_delta: invoice.min_final_cltv_expiry_delta,
            fallback_addresses: invoice.fallback_addresses,
            route_hints: invoice
                .route_hints
                .into_iter()
                .map(RouteHintResponse::from)
                .collect(),
        }
    }
}

impl From<RouteHint> for RouteHintResponse {
    fn from(route_hint: RouteHint) -> Self {
        Self {
            hops: route_hint
                .hops
                .into_iter()
                .map(RouteHintHopResponse::from)
                .collect(),
        }
    }
}

impl From<RouteHintHop> for RouteHintHopResponse {
    fn from(hop: RouteHintHop) -> Self {
        Self {
            source_node_id: hop.source_node_id,
            short_channel_id: hop.short_channel_id,
            base_fee_msat: hop.base_fee_msat,
            proportional_fee_millionths: hop.proportional_fee_millionths,
            cltv_expiry_delta: hop.cltv_expiry_delta,
            htlc_minimum_msat: hop.htlc_minimum_msat,
            htlc_maximum_msat: hop.htlc_maximum_msat,
        }
    }
}
