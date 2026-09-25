use axum::{Json, extract::Path};
use invoice_decoder::{DecodedInvoice, decode};

use crate::http::error::ApiError;

pub async fn get(Path(invoice): Path<String>) -> Result<Json<DecodedInvoice>, ApiError> {
    let decoded = decode(&invoice)?;
    Ok(Json(decoded))
}
