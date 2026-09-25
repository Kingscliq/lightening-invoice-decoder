use axum::{Json, extract::Path};
use invoice_decoder::decode;

use crate::http::{
    dto::{ApiErrorResponse, DecodedInvoiceResponse},
    error::ApiError,
};

#[utoipa::path(
    get,
    path = "/api/v1/invoices/decode/{invoice}",
    params(("invoice" = String, Path, description = "URL-encoded BOLT11 invoice")),
    responses(
        (status = 200, description = "Invoice decoded successfully", body = DecodedInvoiceResponse),
        (status = 400, description = "Empty or invalid invoice", body = ApiErrorResponse),
        (status = 414, description = "Invoice exceeds the maximum length", body = ApiErrorResponse),
        (status = 422, description = "Invoice expiry cannot be represented safely", body = ApiErrorResponse)
    ),
    tag = "Invoices"
)]
pub async fn get(Path(invoice): Path<String>) -> Result<Json<DecodedInvoiceResponse>, ApiError> {
    let decoded = decode(&invoice)?;
    Ok(Json(decoded.into()))
}
