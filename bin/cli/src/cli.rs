use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "bolt11-decoder",
    version,
    about = "Decode and validate BOLT11 Lightning invoices"
)]
pub struct Cli {
    /// Decode through this API instead of using the local Rust library.
    #[arg(long, global = true, env = "LIGHTNING_API_URL")]
    pub api_url: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Open the interactive terminal interface. This is the default command.
    Tui,

    /// Decode an invoice and print JSON.
    Decode {
        /// Complete BOLT11 invoice string.
        invoice: String,
    },
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command};

    #[test]
    fn accepts_remote_mode_for_the_default_tui() {
        let cli = Cli::try_parse_from(["bolt11-decoder", "--api-url", "http://localhost:3001"])
            .expect("the CLI arguments should be valid");

        assert_eq!(cli.api_url.as_deref(), Some("http://localhost:3001"));
        assert!(cli.command.is_none());
    }

    #[test]
    fn accepts_remote_mode_for_json_output() {
        let cli = Cli::try_parse_from([
            "bolt11-decoder",
            "decode",
            "lnbcrt...",
            "--api-url",
            "http://localhost:3001",
        ])
        .expect("the CLI arguments should be valid");

        assert_eq!(cli.api_url.as_deref(), Some("http://localhost:3001"));
        assert!(matches!(cli.command, Some(Command::Decode { .. })));
    }
}
