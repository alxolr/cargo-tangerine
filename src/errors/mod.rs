use std::fmt::Display;

use derive_more::From;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    Io(std::io::Error),
    Clap(clap::Error),
    Toml(toml::de::Error),
    Str(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "I/O error: {}", err),
            Error::Clap(err) => write!(f, "clap error: {}", err),
            Error::Toml(err) => write!(f, "TOML error: {}", err),
            Error::Str(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}
