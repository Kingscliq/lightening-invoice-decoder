use std::{net::SocketAddr, str::FromStr};

use anyhow::Context;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub allowed_origins: Vec<String>,
    pub public_base_url: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let port = parse_env("PORT", 3_001_u16)?;
        let allowed_origins = std::env::var("ALLOWED_ORIGINS")
            .or_else(|_| std::env::var("ALLOWED_ORIGIN"))
            .unwrap_or_else(|_| "http://localhost:3000".to_owned());
        let allowed_origins = parse_allowed_origins(&allowed_origins)?;
        let public_base_url = std::env::var("PUBLIC_BASE_URL")
            .unwrap_or_else(|_| format!("http://localhost:{port}"))
            .trim_end_matches('/')
            .to_owned();

        Ok(Self {
            port,
            allowed_origins,
            public_base_url,
        })
    }

    pub fn socket_address(&self) -> SocketAddr {
        SocketAddr::from(([0, 0, 0, 0], self.port))
    }
}

fn parse_allowed_origins(value: &str) -> anyhow::Result<Vec<String>> {
    let origins = value
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();

    if origins.is_empty() {
        anyhow::bail!("ALLOWED_ORIGINS must contain at least one origin");
    }

    Ok(origins)
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

#[cfg(test)]
mod tests {
    use super::parse_allowed_origins;

    #[test]
    fn parses_multiple_allowed_origins() {
        let origins =
            parse_allowed_origins("http://localhost:3000, https://lightening-decoder.vercel.app")
                .expect("the origins should be valid");

        assert_eq!(origins.len(), 2);
        assert_eq!(origins[0], "http://localhost:3000");
        assert_eq!(origins[1], "https://lightening-decoder.vercel.app");
    }

    #[test]
    fn rejects_an_empty_allowed_origins_value() {
        assert!(parse_allowed_origins(" , ").is_err());
    }
}
