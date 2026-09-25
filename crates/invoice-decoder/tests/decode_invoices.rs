use invoice_decoder::{DecodeError, decode};

#[test]
fn rejects_a_non_bolt11_value() {
    assert!(matches!(
        decode("not-an-invoice"),
        Err(DecodeError::InvalidInvoice(_))
    ));
}
