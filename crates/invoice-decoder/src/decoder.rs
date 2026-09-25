use lightning_invoice::{Bolt11Invoice, Bolt11InvoiceDescriptionRef};

use crate::{DecodeError, DecodeResult, DecodedInvoice, RouteHint, RouteHintHop};

pub const MAX_INVOICE_LENGTH: usize = 8_192;

/// Parses and validates a BOLT11 invoice string.
///
/// # Errors
///
/// Returns [`DecodeError`] when the input is empty, too long, or not a valid
/// BOLT11 invoice.
pub fn parse_invoice(input: &str) -> DecodeResult<Bolt11Invoice> {
    let input = input.trim();

    if input.is_empty() {
        return Err(DecodeError::EmptyInvoice);
    }

    if input.len() > MAX_INVOICE_LENGTH {
        return Err(DecodeError::InvoiceTooLong {
            actual: input.len(),
            maximum: MAX_INVOICE_LENGTH,
        });
    }

    input
        .parse::<Bolt11Invoice>()
        .map_err(|error| DecodeError::InvalidInvoice(error.to_string()))
}

/// Entry point shared by the API and CLI.
///
/// # Errors
///
/// Returns the fields decoded from a valid BOLT11 invoice.
pub fn decode(input: &str) -> DecodeResult<DecodedInvoice> {
    let invoice = parse_invoice(input)?;
    let (description, description_hash) = match invoice.description() {
        Bolt11InvoiceDescriptionRef::Direct(value) => (Some(value.to_string()), None),
        Bolt11InvoiceDescriptionRef::Hash(value) => (None, Some(value.0.to_string())),
    };

    let network = match invoice.network().to_string().as_str() {
        "bitcoin" => "mainnet".to_owned(),
        network => network.to_owned(),
    };

    let amount_msat = invoice.amount_milli_satoshis();
    let payment_hash = invoice.payment_hash().to_string();

    let created_at_unix = invoice.duration_since_epoch().as_secs();
    let expiry_seconds = invoice.expiry_time().as_secs();
    let expires_at_unix = invoice
        .expires_at()
        .ok_or(DecodeError::ExpiryOverflow)?
        .as_secs();
    let min_final_cltv_expiry_delta = invoice.min_final_cltv_expiry_delta();

    let fallback_addresses = invoice
        .fallback_addresses()
        .into_iter()
        .map(|address| address.to_string())
        .collect::<Vec<_>>();

    let route_hints = invoice
        .route_hints()
        .into_iter()
        .map(|hint| RouteHint {
            hops: hint
                .0
                .into_iter()
                .map(|hop| RouteHintHop {
                    source_node_id: hop.src_node_id.to_string(),
                    short_channel_id: hop.short_channel_id,
                    base_fee_msat: hop.fees.base_msat,
                    proportional_fee_millionths: hop.fees.proportional_millionths,
                    cltv_expiry_delta: hop.cltv_expiry_delta,
                    htlc_minimum_msat: hop.htlc_minimum_msat,
                    htlc_maximum_msat: hop.htlc_maximum_msat,
                })
                .collect(),
        })
        .collect::<Vec<_>>();

    let payee_public_key = invoice.get_payee_pub_key().to_string();
    let signature_valid = invoice.check_signature().is_ok();
    let expired = invoice.is_expired();

    Ok(DecodedInvoice {
        network,
        amount_msat,
        description,
        description_hash,
        payment_hash,
        payee_public_key,
        created_at_unix,
        expires_at_unix,
        expiry_seconds,
        expired,
        signature_valid,
        min_final_cltv_expiry_delta,
        fallback_addresses,
        route_hints,
        features: None,
    })
}

#[cfg(test)]
mod tests {
    use super::{MAX_INVOICE_LENGTH, decode, parse_invoice};
    use crate::DecodeError;

    const VALID_INVOICE: &str = "lnbc25m1pvjluezpp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqdq5vdhkven9v5sxyetpdeessp5zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygs9q5sqqqqqqqqqqqqqqqpqsq67gye39hfg3zd8rgc80k32tvy9xk2xunwm5lzexnvpx6fd77en8qaq424dxgt56cag2dpt359k3ssyhetktkpqh24jqnjyw6uqd08sgptq44qu";

    #[test]
    fn rejects_empty_input() {
        assert!(matches!(decode("  "), Err(DecodeError::EmptyInvoice)));
    }

    #[test]
    fn rejects_input_over_the_configured_limit() {
        let input = "x".repeat(MAX_INVOICE_LENGTH + 1);

        assert!(matches!(
            parse_invoice(&input),
            Err(DecodeError::InvoiceTooLong { .. })
        ));
    }

    #[test]
    fn parses_a_valid_bolt11_invoice() {
        assert!(parse_invoice(VALID_INVOICE).is_ok());
    }

    #[test]
    fn rejects_a_malformed_invoice() {
        assert!(matches!(
            parse_invoice("not-an-invoice"),
            Err(DecodeError::InvalidInvoice(_))
        ));
    }

    #[test]
    fn decodes_a_valid_invoice() {
        let decoded = decode(VALID_INVOICE).expect("the invoice should decode");

        assert_eq!(decoded.network, "mainnet");
        assert_eq!(decoded.amount_msat, Some(2_500_000_000));
        assert_eq!(decoded.description.as_deref(), Some("coffee beans"));
        assert!(decoded.signature_valid);
    }
}
