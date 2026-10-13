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
    #[error("{0}")]
    Validation(DecodeError, Box<invoice_decoder::ValidationReport>),
    #[error("Expected network must be mainnet, testnet, signet, or regtest")]
    InvalidNetwork,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if matches!(self, Self::InvalidNetwork) {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiErrorResponse {
                    validation: None,
                    error: ApiErrorBody {
                        code: "INVALID_NETWORK",
                        message: self.to_string(),
                    },
                }),
            )
                .into_response();
        }
        let (error, validation) = match self {
            Self::Decode(error) => (error, None),
            Self::Validation(error, report) => (error, Some((*report).into())),
            Self::InvalidNetwork => unreachable!(),
        };
        let (status, code) = match &error {
            DecodeError::EmptyInvoice => (StatusCode::BAD_REQUEST, "EMPTY_INVOICE"),
            DecodeError::InvoiceTooLong { .. } => (StatusCode::URI_TOO_LONG, "INVOICE_TOO_LONG"),
            DecodeError::InvalidInvoice(_) => (StatusCode::BAD_REQUEST, "INVALID_INVOICE"),
            DecodeError::ExpiryOverflow => (StatusCode::UNPROCESSABLE_ENTITY, "EXPIRY_OVERFLOW"),
        };

        let body = ApiErrorResponse {
            validation,
            error: ApiErrorBody {
                code,
                message: error.to_string(),
            },
        };

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::to_bytes,
        http::StatusCode,
        response::{IntoResponse, Response},
    };
    use invoice_decoder::DecodeError;
    use serde_json::Value;

    use super::ApiError;

    async fn assert_mapping(error: DecodeError, expected_status: StatusCode, expected_code: &str) {
        let response: Response = ApiError::from(error).into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("error response body should be readable");
        let body: Value =
            serde_json::from_slice(&bytes).expect("error response should contain JSON");

        assert_eq!(status, expected_status);
        assert_eq!(body["error"]["code"], expected_code);
    }

    #[tokio::test]
    async fn maps_every_decoder_error() {
        assert_mapping(
            DecodeError::EmptyInvoice,
            StatusCode::BAD_REQUEST,
            "EMPTY_INVOICE",
        )
        .await;
        assert_mapping(
            DecodeError::InvoiceTooLong {
                actual: 8_193,
                maximum: 8_192,
            },
            StatusCode::URI_TOO_LONG,
            "INVOICE_TOO_LONG",
        )
        .await;
        assert_mapping(
            DecodeError::InvalidInvoice("malformed".to_owned()),
            StatusCode::BAD_REQUEST,
            "INVALID_INVOICE",
        )
        .await;
        assert_mapping(
            DecodeError::ExpiryOverflow,
            StatusCode::UNPROCESSABLE_ENTITY,
            "EXPIRY_OVERFLOW",
        )
        .await;
    }
}
