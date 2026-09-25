use anyhow::bail;
use invoice_decoder::DecodedInvoice;

pub fn decode(api_url: &str, invoice: &str) -> anyhow::Result<DecodedInvoice> {
    let _ = (api_url, invoice);

    // TODO(lesson 8): Build the URL safely with `reqwest::Url`.
    // TODO(lesson 8): Send a GET request and deserialize `DecodedInvoice`.
    // TODO(lesson 8): Turn non-success HTTP responses into useful CLI errors.
    bail!("remote CLI decoding is not implemented yet; see remote.rs")
}
