use utoipa::OpenApi;

use crate::http::{
    dto::{
        ApiErrorBody, ApiErrorResponse, DecodedInvoiceResponse, HealthResponse,
        RouteHintHopResponse, RouteHintResponse,
    },
    routes::{decode_invoice, health},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Lightening Decoder API",
        version = "0.1.0",
        description = "Decode and validate BOLT11 Lightning invoices"
    ),
    paths(health::get, decode_invoice::get),
    components(schemas(
        HealthResponse,
        ApiErrorBody,
        ApiErrorResponse,
        DecodedInvoiceResponse,
        RouteHintResponse,
        RouteHintHopResponse
    )),
    tags(
        (name = "System", description = "Service health"),
        (name = "Invoices", description = "BOLT11 invoice decoding")
    )
)]
pub struct ApiDoc;
