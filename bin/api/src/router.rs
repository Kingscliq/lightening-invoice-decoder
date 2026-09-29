use anyhow::Context;

use axum::{
    Router,
    body::Body,
    http::{HeaderValue, Method, Request, header},
    routing::get,
};

use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    http::routes::{decode_invoice, health},
    openapi::ApiDoc,
};

pub fn create_router(allowed_origins: &[String]) -> anyhow::Result<Router> {
    let allowed_origins = allowed_origins
        .iter()
        .map(|origin| {
            HeaderValue::from_str(origin)
                .with_context(|| format!("`{origin}` is not a valid HTTP origin"))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([Method::GET])
        .allow_headers([header::ACCEPT, header::CONTENT_TYPE]);

    let decode_routes = Router::new()
        .route(
            "/api/v1/invoices/decode/{invoice}",
            get(decode_invoice::get),
        )
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        ));

    Ok(Router::new()
        .route("/health", get(health::get))
        .merge(decode_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .fallback(crate::http::routes::not_found)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                // Deliberately omit the URI because it contains the BOLT11 invoice for security reasons
                tracing::info_span!("http_request", method = %request.method())
            }),
        ))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use invoice_decoder::MAX_INVOICE_LENGTH;
    use serde_json::Value;
    use tower::ServiceExt;

    use super::create_router;

    const VALID_EXPIRED_INVOICE: &str = "lnbc25m1pvjluezpp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqdq5vdhkven9v5sxyetpdeessp5zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygs9q5sqqqqqqqqqqqqqqqpqsq67gye39hfg3zd8rgc80k32tvy9xk2xunwm5lzexnvpx6fd77en8qaq424dxgt56cag2dpt359k3ssyhetktkpqh24jqnjyw6uqd08sgptq44qu";

    fn app() -> axum::Router {
        create_router(&[
            "http://localhost:3000".to_owned(),
            "https://lightening-decoder.vercel.app".to_owned(),
        ])
        .expect("test origins should be valid")
    }

    async fn json(response: axum::response::Response) -> Value {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should be readable");
        serde_json::from_slice(&bytes).expect("response should contain JSON")
    }

    #[tokio::test]
    async fn health_check_succeeds() {
        let response = app()
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn decodes_and_reports_an_expired_invoice() {
        let response = app()
            .oneshot(
                Request::get(format!("/api/v1/invoices/decode/{VALID_EXPIRED_INVOICE}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        let body = json(response).await;
        assert_eq!(body["network"], "mainnet");
        assert_eq!(body["expired"], true);
        assert_eq!(body["signature_valid"], true);
    }

    #[tokio::test]
    async fn rejects_a_malformed_invoice() {
        let response = app()
            .oneshot(
                Request::get("/api/v1/invoices/decode/not-an-invoice")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(json(response).await["error"]["code"], "INVALID_INVOICE");
    }

    #[tokio::test]
    async fn rejects_an_oversized_invoice() {
        let invoice = "x".repeat(MAX_INVOICE_LENGTH + 1);
        let response = app()
            .oneshot(
                Request::get(format!("/api/v1/invoices/decode/{invoice}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::URI_TOO_LONG);
        assert_eq!(json(response).await["error"]["code"], "INVOICE_TOO_LONG");
    }

    #[tokio::test]
    async fn unknown_routes_return_json_not_found() {
        let response = app()
            .oneshot(Request::get("/missing").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(json(response).await["error"]["code"], "NOT_FOUND");
    }

    #[tokio::test]
    async fn allows_the_configured_frontend_origin() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri(format!("/api/v1/invoices/decode/{VALID_EXPIRED_INVOICE}"))
                    .header(header::ORIGIN, "http://localhost:3000")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
            "http://localhost:3000"
        );
    }

    #[tokio::test]
    async fn allows_a_second_configured_frontend_origin() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri(format!("/api/v1/invoices/decode/{VALID_EXPIRED_INVOICE}"))
                    .header(header::ORIGIN, "https://lightening-decoder.vercel.app")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
            "https://lightening-decoder.vercel.app"
        );
    }

    #[tokio::test]
    async fn exposes_the_openapi_document() {
        let response = app()
            .oneshot(
                Request::get("/api-docs/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = json(response).await;
        assert!(body["paths"]["/health"].is_object());
        assert!(body["paths"]["/api/v1/invoices/decode/{invoice}"].is_object());
    }

    #[tokio::test]
    async fn exposes_swagger_ui() {
        let response = app()
            .oneshot(Request::get("/swagger-ui/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response.headers()[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with("text/html")
        );
    }
}
