use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "bolt11-decoder",
    version,
    about = "Decode and validate BOLT11 Lightning invoices"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Decode an invoice locally, or through the hosted API when --api-url is supplied.
    Decode {
        /// Complete BOLT11 invoice string.
        invoice: String,

        /// Base URL of the hosted API, for example <https://example.onrender.com>.
        #[arg(long, env = "LIGHTNING_API_URL")]
        api_url: Option<String>,
    },
}
