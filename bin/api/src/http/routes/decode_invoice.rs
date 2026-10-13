use axum::{
    Json,
    extract::{Path, Query},
};
use invoice_decoder::{ExpectedNetwork, inspect};
use std::collections::HashMap;

use crate::http::{
    dto::{ApiErrorResponse, DecodedInvoiceResponse},
    error::ApiError,
};

#[utoipa::path(
    get,
    path = "/api/v1/invoices/decode/{invoice}",
    params(("invoice" = String, Path, description = "URL-encoded BOLT11 invoice"), ("expected_network" = Option<String>, Query, description = "Optional mainnet, testnet, signet, or regtest expectation")),
    responses(
        (status = 200, description = "Invoice decoded successfully", body = DecodedInvoiceResponse),
        (status = 400, description = "Empty or invalid invoice", body = ApiErrorResponse),
        (status = 414, description = "Invoice exceeds the maximum length", body = ApiErrorResponse),
        (status = 422, description = "Invoice expiry cannot be represented safely", body = ApiErrorResponse)
    ),
    tag = "Invoices"
)]
pub async fn get(
    Path(invoice): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<DecodedInvoiceResponse>, ApiError> {
    let expected = match query.get("expected_network").map(String::as_str) {
        None => None,
        Some("mainnet") => Some(ExpectedNetwork::Mainnet),
        Some("testnet") => Some(ExpectedNetwork::Testnet),
        Some("signet") => Some(ExpectedNetwork::Signet),
        Some("regtest") => Some(ExpectedNetwork::Regtest),
        Some(_) => return Err(ApiError::InvalidNetwork),
    };
    let (decoded, report) = inspect(&invoice, expected);
    let decoded = decoded.map_err(|error| ApiError::Validation(error, Box::new(report)))?;
    Ok(Json(decoded.into()))
}
