use axum::{Router, routing::get};

use crate::http::routes::{decode_invoice, health};

/// Creates all API routes.
pub fn create_router() -> Router {
    // No AppState is needed while invoice decoding remains stateless.
    // TODO(lesson 6): Add and explain CORS for the separate Next.js origin.
    // TODO(lesson 6): Add `Cache-Control: no-store` without logging invoices.
    // TODO(lesson 6): Add request tracing that omits the sensitive URL path.
    Router::new()
        .route("/health", get(health::get))
        .route(
            "/api/v1/invoices/decode/{invoice}",
            get(decode_invoice::get),
        )
        .fallback(crate::http::routes::not_found)
}
