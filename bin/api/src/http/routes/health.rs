use axum::Json;

use crate::http::dto::HealthResponse;

pub async fn get() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
