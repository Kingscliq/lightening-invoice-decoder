use std::time::{SystemTime, UNIX_EPOCH};

use lightning_invoice::{Bolt11Invoice, SignedRawBolt11Invoice};
use serde::{Deserialize, Serialize};

use crate::{DecodeResult, DecodedInvoice, MAX_INVOICE_LENGTH};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExpectedNetwork {
    Mainnet,
    Testnet,
    Signet,
    Regtest,
}

impl ExpectedNetwork {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
            Self::Signet => "signet",
            Self::Regtest => "regtest",
        }
    }
}

impl std::str::FromStr for ExpectedNetwork {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            "signet" => Ok(Self::Signet),
            "regtest" => Ok(Self::Regtest),
            _ => Err("Choose mainnet, testnet, signet, or regtest".to_owned()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Passed,
    Warning,
    Failed,
    Info,
    NotChecked,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ValidationCheck {
    pub id: String,
    pub label: String,
    pub status: CheckStatus,
    pub message: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ValidationReport {
    pub checked_at_unix: u64,
    pub expected_network: Option<ExpectedNetwork>,
    pub checks: Vec<ValidationCheck>,
}

/// Inspect an invoice, retaining diagnostic checks even when decoding fails.
pub fn inspect(
    input: &str,
    expected: Option<ExpectedNetwork>,
) -> (DecodeResult<DecodedInvoice>, ValidationReport) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    inspect_at(input, expected, now)
}

fn inspect_at(
    input: &str,
    expected: Option<ExpectedNetwork>,
    now: u64,
) -> (DecodeResult<DecodedInvoice>, ValidationReport) {
    use CheckStatus::{Failed, Info, NotChecked, Passed, Warning};
    let mut result = crate::decoder::decode_fields(input);
    let mut report = ValidationReport {
        checked_at_unix: now,
        expected_network: expected,
        checks: [
            ("format", "Invoice format"),
            ("signature", "Signature"),
            ("requirements", "Required invoice fields"),
            ("expiry", "Expiry"),
            ("network", "Network"),
            ("amount", "Amount"),
        ]
        .into_iter()
        .map(|(id, label)| ValidationCheck {
            id: id.to_owned(),
            label: label.to_owned(),
            status: NotChecked,
            message: "Not checked because the invoice could not be fully validated.".to_owned(),
        })
        .collect(),
    };
    let raw = if input.trim().len() <= MAX_INVOICE_LENGTH {
        input.trim().parse::<SignedRawBolt11Invoice>().ok()
    } else {
        None
    };
    if let Some(raw) = raw {
        report.set(0, Passed, "BOLT11 encoding and checksum are valid.");
        if raw.check_signature() {
            report.set(1, Passed, "The cryptographic signature matches the invoice signing public key. This does not verify the recipient's identity.");
            let requirements_valid = Bolt11Invoice::from_signed(raw).is_ok();
            report.set(2, if requirements_valid { Passed } else { Failed }, if requirements_valid {
                "Required fields and semantic rules were accepted by the BOLT11 parser. Wallet feature compatibility is not assessed."
            } else { "The invoice could not be decoded after signature verification. Required fields or semantic rules may be invalid; see the decoding error." });
        } else {
            report.set(1, Failed, "The invoice signature could not be verified.");
        }
    } else {
        report.set(0, Failed, "The input is empty, too long, or has invalid BOLT11 encoding/checksum. Check that the complete invoice was pasted.");
    }
    if matches!(result, Err(crate::DecodeError::ExpiryOverflow)) {
        report.set(3, Failed, "The expiry value cannot be represented safely.");
    }
    if let Ok(invoice) = &mut result {
        invoice.expired = now >= invoice.expires_at_unix;
        report.set(3, if invoice.expired { Warning } else { Passed }, if invoice.expired {
            "The expiry deadline has passed. This invoice should no longer be used."
        } else { "The invoice has not expired at the time of this check. This does not establish whether it has already been paid." });
        let (status, message) = match expected {
            Some(network) if network.as_str() == invoice.network => (
                Passed,
                format!(
                    "The invoice uses {}, matching your expected network.",
                    invoice.network
                ),
            ),
            Some(network) => (
                Warning,
                format!(
                    "Network mismatch: the invoice uses {}, but you selected {}.",
                    invoice.network,
                    network.as_str()
                ),
            ),
            None => (
                Info,
                format!(
                    "The invoice uses {}. Select an expected network to check for a mismatch.",
                    invoice.network
                ),
            ),
        };
        report.set(4, status, &message);
        let message = invoice.amount_msat.map_or_else(|| "No amount is specified. The payer must choose the payment amount.".to_owned(), |amount| format!("The invoice specifies {amount} millisatoshis. This check does not assess affordability or payment limits."));
        report.set(5, Info, &message);
        invoice.validation = report.clone();
    }
    (result, report)
}

impl ValidationReport {
    fn set(&mut self, index: usize, status: CheckStatus, message: &str) {
        self.checks[index].status = status;
        message.clone_into(&mut self.checks[index].message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> String {
        // Shared public fixtures also used by the web and terminal clients.
        let samples: serde_json::Value =
            serde_json::from_str(include_str!("../../../frontend/data/samples.json")).unwrap();
        samples[0]["invoice"].as_str().unwrap().to_owned()
    }
    #[test]
    fn malformed_input_does_not_claim_downstream_checks_passed() {
        let (result, report) = inspect("not an invoice", None);
        assert!(result.is_err());
        assert_eq!(report.checks[0].status, CheckStatus::Failed);
        assert!(
            report.checks[1..]
                .iter()
                .all(|check| check.status == CheckStatus::NotChecked)
        );
    }
    #[test]
    fn checks_expiry_boundary_and_network_mismatch() {
        let invoice = sample();
        let expiry = crate::decoder::decode_fields(&invoice)
            .unwrap()
            .expires_at_unix;
        let (_, before) = inspect_at(&invoice, Some(ExpectedNetwork::Mainnet), expiry - 1);
        assert_eq!(before.checks[3].status, CheckStatus::Passed);
        assert_eq!(before.checks[4].status, CheckStatus::Passed);
        let (decoded, at) = inspect_at(&invoice, Some(ExpectedNetwork::Testnet), expiry);
        assert_eq!(at.checks[3].status, CheckStatus::Warning);
        assert_eq!(at.checks[4].status, CheckStatus::Warning);
        assert_eq!(decoded.unwrap().validation, at);
    }
    #[test]
    fn distinguishes_signature_failure_from_required_field_failure() {
        let (result, report) = inspect(
            include_str!("../tests/fixtures/invalid-signature.txt"),
            None,
        );
        assert!(result.is_err());
        assert_eq!(report.checks[0].status, CheckStatus::Passed);
        assert_eq!(report.checks[1].status, CheckStatus::Failed);
        assert!(
            report.checks[2..]
                .iter()
                .all(|check| check.status == CheckStatus::NotChecked)
        );
        let (result, report) = inspect(include_str!("../tests/fixtures/missing-secret.txt"), None);
        assert!(result.is_err());
        assert_eq!(report.checks[0].status, CheckStatus::Passed);
        assert_eq!(report.checks[1].status, CheckStatus::Passed);
        assert_eq!(report.checks[2].status, CheckStatus::Failed);
        assert_eq!(report.checks[3].status, CheckStatus::NotChecked);
    }

    #[test]
    fn explains_open_amount_without_claiming_a_network_match() {
        let samples: serde_json::Value =
            serde_json::from_str(include_str!("../../../frontend/data/samples.json")).unwrap();
        let (result, report) = inspect(samples[1]["invoice"].as_str().unwrap(), None);
        assert!(result.is_ok());
        assert_eq!(report.checks[4].status, CheckStatus::Info);
        assert_eq!(report.checks[5].status, CheckStatus::Info);
        assert!(report.checks[5].message.contains("payer must choose"));
    }
}
