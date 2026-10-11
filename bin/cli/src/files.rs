use std::{fs::OpenOptions, io::Write, path::PathBuf};

use anyhow::{Context, bail};
use invoice_decoder::DecodedInvoice;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Sample {
    pub label: String,
    pub detail: String,
    pub invoice: String,
}

pub fn samples() -> Vec<Sample> {
    serde_json::from_str(include_str!("../../../frontend/data/samples.json"))
        .expect("bundled demo invoices must be valid JSON")
}

pub fn normalize_invoice(value: &str) -> &str {
    let value = value.trim();
    if value
        .get(..10)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("lightning:"))
    {
        value[10..].trim()
    } else {
        value
    }
}

pub fn input_path(value: &str) -> PathBuf {
    let value = value.trim();
    let value = value
        .strip_prefix('\'')
        .and_then(|v| v.strip_suffix('\''))
        .or_else(|| value.strip_prefix('"').and_then(|v| v.strip_suffix('"')))
        .unwrap_or(value);
    if let Some(relative) = value.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home).join(relative);
    }
    PathBuf::from(value)
}

pub fn read_qr(path: &str) -> anyhow::Result<String> {
    let path = input_path(path);
    let metadata = std::fs::metadata(&path).context("Cannot open image file")?;
    if !metadata.is_file() || metadata.len() > 10 * 1024 * 1024 {
        bail!("Choose a PNG, JPEG, or WebP file smaller than 10 MB");
    }
    let mut reader = image::ImageReader::open(&path)?.with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .context("Cannot read image; use PNG, JPEG, or WebP up to 4096 × 4096")?
        .to_luma8();
    let mut image = rqrr::PreparedImage::prepare(image);
    let grids = image.detect_grids();
    if grids.len() > 1 {
        bail!("Multiple QR codes found. Crop the image to one invoice");
    }
    let grid = grids
        .first()
        .context("No QR code found. Try a sharper image with the entire code visible")?;
    let (_, content) = grid.decode().context("The QR code could not be read")?;
    let invoice = normalize_invoice(&content);
    // Parsing here also prevents URLs or other QR content being sent to a remote API.
    invoice_decoder::parse_invoice(invoice)
        .context("QR code does not contain a valid BOLT11 invoice")?;
    Ok(invoice.to_owned())
}

pub fn export_json(path: &str, invoice: &DecodedInvoice) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(invoice)? + "\n";
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(input_path(path))
        .context("Cannot create file. Choose a writable path that does not already exist")?;
    file.write_all(json.as_bytes())
        .context("Could not finish writing JSON")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_decode_and_cover_fixed_and_open_amounts() {
        let decoded: Vec<_> = samples()
            .iter()
            .map(|s| invoice_decoder::decode(&s.invoice).unwrap())
            .collect();
        assert_eq!(decoded[0].amount_msat, Some(250_000_000));
        assert_eq!(decoded[1].amount_msat, None);
        assert!(decoded.iter().all(|i| i.expired && i.signature_valid));
    }

    #[test]
    fn reads_a_real_qr_image_with_lightning_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let sample = &samples()[0].invoice;
        let code = qrcode::QrCode::new(format!("LIGHTNING:{sample}")).unwrap();
        let image = code
            .render::<image::Luma<u8>>()
            .min_dimensions(800, 800)
            .build();
        let path = dir.path().join("invoice.png");
        image.save(&path).unwrap();
        assert_eq!(read_qr(path.to_str().unwrap()).unwrap(), *sample);
    }

    #[test]
    fn rejects_non_invoice_qr_and_blank_images() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("image.png");
        qrcode::QrCode::new("https://example.com")
            .unwrap()
            .render::<image::Luma<u8>>()
            .build()
            .save(&path)
            .unwrap();
        assert!(
            read_qr(path.to_str().unwrap())
                .unwrap_err()
                .to_string()
                .contains("BOLT11")
        );
        image::GrayImage::from_pixel(100, 100, image::Luma([255]))
            .save(&path)
            .unwrap();
        assert!(
            read_qr(path.to_str().unwrap())
                .unwrap_err()
                .to_string()
                .contains("No QR")
        );
    }

    #[test]
    fn exports_json_without_overwriting_existing_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("invoice.json");
        let invoice = invoice_decoder::decode(&samples()[0].invoice).unwrap();
        export_json(path.to_str().unwrap(), &invoice).unwrap();
        let first = std::fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&first).unwrap();
        assert_eq!(value["payment_hash"], invoice.payment_hash);
        assert!(export_json(path.to_str().unwrap(), &invoice).is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), first);
    }
}
