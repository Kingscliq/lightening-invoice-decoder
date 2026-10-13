mod cli;
mod files;
mod remote;
mod terminal_ui;

use anyhow::Context;
use clap::Parser;
use cli::{Cli, Command};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let api_url = cli.api_url;

    match cli.command.unwrap_or(Command::Tui) {
        Command::Tui => terminal_ui::run(api_url, cli.expected_network),
        Command::Decode { invoice, image } => {
            let invoice = if let Some(path) = image {
                files::read_qr(&path)?
            } else {
                files::normalize_invoice(invoice.as_deref().unwrap_or_default()).to_owned()
            };
            let (local, report) = invoice_decoder::inspect(&invoice, cli.expected_network);
            let mut decoded = match local {
                Ok(decoded) => decoded,
                Err(error) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(
                            &serde_json::json!({"error": error.to_string(), "validation": report})
                        )?
                    );
                    return Err(error.into());
                }
            };
            if let Some(api_url) = api_url {
                decoded =
                    remote::decode(&api_url, &invoice).context("remote invoice decoding failed")?;
                decoded.validation = report;
            }
            println!("{}", serde_json::to_string_pretty(&decoded)?);
            Ok(())
        }
    }
}
