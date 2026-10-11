//! Regenerate the public demo QR fixtures used by both clients.
use anyhow::Context;
use image::Luma;
use qrcode::QrCode;

fn main() -> anyhow::Result<()> {
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
