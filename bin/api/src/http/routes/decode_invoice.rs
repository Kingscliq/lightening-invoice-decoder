use axum::{Json, extract::Path};
use invoice_decoder::{DecodedInvoice, decode};

use crate::http::error::ApiError;

pub async fn get(Path(invoice): Path<String>) -> Result<Json<DecodedInvoice>, ApiError> {
    // TODO(lesson 6): Once the decoder crate is implemented, add API-level
    // tests for successful responses and every error mapping.
    let decoded = decode(&invoice)?;
    Ok(Json(decoded))
}
