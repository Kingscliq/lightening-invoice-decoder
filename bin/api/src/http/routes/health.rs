use axum::Json;

use crate::http::dto::HealthResponse;

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "API is ready", body = HealthResponse)),
    tag = "System"
)]
pub async fn get() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}
