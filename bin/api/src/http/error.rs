use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use invoice_decoder::DecodeError;

use crate::http::dto::{ApiErrorBody, ApiErrorResponse};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    Decode(#[from] DecodeError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::Decode(DecodeError::EmptyInvoice) => (StatusCode::BAD_REQUEST, "EMPTY_INVOICE"),
            Self::Decode(DecodeError::InvoiceTooLong { .. }) => {
                (StatusCode::URI_TOO_LONG, "INVOICE_TOO_LONG")
            }
            Self::Decode(DecodeError::InvalidInvoice(_)) => {
                (StatusCode::BAD_REQUEST, "INVALID_INVOICE")
            }
            Self::Decode(DecodeError::ExpiryOverflow) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "EXPIRY_OVERFLOW")
            }
        };

        let body = ApiErrorResponse {
            error: ApiErrorBody {
                code,
                message: self.to_string(),
            },
        };

        (status, Json(body)).into_response()
    }
}
