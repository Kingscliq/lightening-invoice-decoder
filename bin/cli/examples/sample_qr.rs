//! Generate a QR from an invoice, or regenerate the public demo QR fixtures.
use anyhow::Context;
use image::Luma;
use qrcode::QrCode;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        anyhow::ensure!(
            args.len() == 2,
            "Usage: cargo run -p lightning-cli --example sample_qr -- '<invoice>' output.png"
        );
        let input = args[0].trim();
        let invoice = match input.get(..10) {
            Some(prefix) if prefix.eq_ignore_ascii_case("lightning:") => input[10..].trim(),
            _ => input,
        };
        invoice_decoder::decode(invoice).context("invalid BOLT11 invoice")?;
        let output = std::path::Path::new(&args[1]);
        let image = QrCode::new(format!("lightning:{invoice}"))?
            .render::<Luma<u8>>()
            .min_dimensions(800, 800)
            .build();
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output)
            .context("could not create output file (choose a new filename if it exists)")?;
        image.write_to(&mut std::io::BufWriter::new(file), image::ImageFormat::Png)?;
        println!("Saved QR image to {}", output.display());
        return Ok(());
    }
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../frontend/public/samples");
    std::fs::create_dir_all(&directory)?;
    let samples: serde_json::Value =
        serde_json::from_str(include_str!("../../../frontend/data/samples.json"))?;
    for (index, name) in ["coffee", "donation"].iter().enumerate() {
        let invoice = samples[index]["invoice"]
            .as_str()
            .context("missing sample invoice")?;
        QrCode::new(format!("lightning:{invoice}"))?
            .render::<Luma<u8>>()
            .min_dimensions(800, 800)
            .build()
            .save(directory.join(format!("{name}.png")))?;
    }
    Ok(())
}
