mod cli;
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

    match cli.command.unwrap_or(Command::Tui) {
        Command::Tui => terminal_ui::run(),
        Command::Decode { invoice, api_url } => {
            // TODO(lesson 7): Add CLI-focused tests, choose the final output
            // format, and explain how `anyhow::Context` builds an error chain.
            let decoded = if let Some(api_url) = api_url {
                remote::decode(&api_url, &invoice).context("remote invoice decoding failed")?
            } else {
                invoice_decoder::decode(&invoice).context("local invoice decoding failed")?
            };

            println!("{}", serde_json::to_string_pretty(&decoded)?);
            Ok(())
        }
    }
}
