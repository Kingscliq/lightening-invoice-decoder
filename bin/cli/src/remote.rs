use std::time::Duration;

use anyhow::{Context, anyhow, bail};
use invoice_decoder::DecodedInvoice;
use reqwest::{Url, blocking::Client, header::ACCEPT};
use serde_json::Value;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

pub fn decode(api_url: &str, invoice: &str) -> anyhow::Result<DecodedInvoice> {
    let url = build_decode_url(api_url, invoice)?;
    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .context("failed to build the HTTP client")?;

    let response = client
        .get(url)
        .header(ACCEPT, "application/json")
        .send()
        .context("failed to reach the decoder API")?;
    let status = response.status();

    if !status.is_success() {
        let body = response
            .text()
            .context("failed to read the decoder API error response")?;
        let message = api_error_message(&body);
        bail!("decoder API returned {status}: {message}");
    }

    response
        .json::<DecodedInvoice>()
        .context("decoder API returned an invalid response")
}

fn build_decode_url(api_url: &str, invoice: &str) -> anyhow::Result<Url> {
    let mut url = Url::parse(api_url).context("API URL is invalid")?;
    url.set_query(None);
    url.set_fragment(None);

    let mut segments = url
        .path_segments_mut()
        .map_err(|()| anyhow!("API URL cannot be used as a base URL"))?;
    segments.pop_if_empty();
    segments.extend(["api", "v1", "invoices", "decode", invoice]);
    drop(segments);

    Ok(url)
}

fn api_error_message(body: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| {
            value
                .pointer("/error/message")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| body.chars().take(500).collect())
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread::{self, JoinHandle},
    };

    use super::{api_error_message, build_decode_url, decode};

    fn serve_once(body: &'static str) -> (String, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("the test server should bind");
        let address = listener
            .local_addr()
            .expect("the test server should have an address");

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("the test server should accept");
            let mut request = [0_u8; 4_096];
            let bytes_read = stream
                .read(&mut request)
                .expect("the test request should be readable");
            let request = String::from_utf8_lossy(&request[..bytes_read]);
            assert!(request.contains("/api/v1/invoices/decode/lnbcrt-test"));

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("the test response should be writable");
        });

        (format!("http://{address}"), handle)
    }

    #[test]
    fn builds_and_encodes_the_decode_endpoint() {
        let url = build_decode_url("https://example.com/", "invoice/with space")
            .expect("the URL should be valid");

        assert_eq!(
            url.as_str(),
            "https://example.com/api/v1/invoices/decode/invoice%2Fwith%20space"
        );
    }

    #[test]
    fn extracts_the_api_error_message() {
        let body = r#"{"error":{"code":"INVALID_INVOICE","message":"invalid invoice"}}"#;

        assert_eq!(api_error_message(body), "invalid invoice");
    }

    #[test]
    fn decodes_a_remote_api_response() {
        let body = r#"{
            "network":"regtest",
            "amount_msat":1000,
            "description":"test",
            "description_hash":null,
            "payment_hash":"hash",
            "payee_public_key":"key",
            "created_at_unix":0,
            "expires_at_unix":3600,
            "expiry_seconds":3600,
            "expired":false,
            "signature_valid":true,
            "min_final_cltv_expiry_delta":18,
            "fallback_addresses":[],
            "route_hints":[]
        }"#;
        let (api_url, server) = serve_once(body);

        let invoice = decode(&api_url, "lnbcrt-test")
            .expect("the remote response should deserialize successfully");
        server.join().expect("the test server should finish");

        assert_eq!(invoice.network, "regtest");
        assert_eq!(invoice.amount_msat, Some(1_000));
    }
}
