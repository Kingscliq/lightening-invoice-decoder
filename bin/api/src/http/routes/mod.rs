pub mod decode_invoice;
pub mod health;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::http::dto::{ApiErrorBody, ApiErrorResponse};

pub async fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorResponse {
            error: ApiErrorBody {
                code: "NOT_FOUND",
                message: "The requested endpoint does not exist.".to_owned(),
            },
        }),
    )
        .into_response()
}
