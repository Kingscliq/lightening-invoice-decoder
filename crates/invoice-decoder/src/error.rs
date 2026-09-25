pub type DecodeResult<T> = Result<T, DecodeError>;

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("the invoice must not be empty")]
    EmptyInvoice,

    #[error("the invoice is {actual} bytes, exceeding the {maximum}-byte limit")]
    InvoiceTooLong { actual: usize, maximum: usize },

    #[error("invalid BOLT11 invoice: {0}")]
    InvalidInvoice(String),

    #[error("the invoice expiry timestamp overflows the supported range")]
    ExpiryOverflow,
}
