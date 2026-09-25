use std::{net::SocketAddr, str::FromStr};

use anyhow::Context;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub allowed_origin: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let port = parse_env("PORT", 3_001_u16)?;
        let allowed_origin =
            std::env::var("ALLOWED_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_owned());

        Ok(Self {
            port,
            allowed_origin,
        })
    }

    pub fn socket_address(&self) -> SocketAddr {
        SocketAddr::from(([0, 0, 0, 0], self.port))
    }
}

fn parse_env<T>(name: &str, default: T) -> anyhow::Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match std::env::var(name) {
        Ok(value) => value
            .parse::<T>()
            .with_context(|| format!("{name} has an invalid value `{value}`")),
        Err(std::env::VarError::NotPresent) => Ok(default),
        Err(error) => Err(error).with_context(|| format!("could not read {name}")),
    }
}
