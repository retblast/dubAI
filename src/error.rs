use crate::{cli, dub, srt_ops, translate};
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum Error {
    Cli(cli::Error),
    Dub(dub::Error),
    Srt(srt_ops::Error),
    Translate(translate::Error),
    Config(llm_connect::config::Error),
    // TODO: Make agnostic
    Kobold(llm_connect::process::Error),
}

impl From<llm_connect::config::Error> for Error {
    fn from(error: llm_connect::config::Error) -> Self {
        Self::Config(error)
    }
}

impl From<llm_connect::process::Error> for Error {
    fn from(error: llm_connect::process::Error) -> Self {
        Self::Kobold(error)
    }
}

impl From<cli::Error> for Error {
    fn from(error: cli::Error) -> Self {
        Self::Cli(error)
    }
}

impl From<dub::Error> for Error {
    fn from(error: dub::Error) -> Self {
        Self::Dub(error)
    }
}

impl From<srt_ops::Error> for Error {
    fn from(error: srt_ops::Error) -> Self {
        Self::Srt(error)
    }
}

impl From<translate::Error> for Error {
    fn from(error: translate::Error) -> Self {
        Self::Translate(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Cli(error) => write!(f, "{error}"),
            Error::Dub(error) => write!(f, "{error}"),
            Error::Srt(error) => write!(f, "{error}"),
            Error::Translate(error) => write!(f, "{error}"),
            Error::Config(error) => write!(f, "{error}"),
            Error::Kobold(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Cli(error) => std::error::Error::source(error),
            Error::Dub(error) => std::error::Error::source(error),
            Error::Srt(error) => std::error::Error::source(error),
            Error::Translate(error) => std::error::Error::source(error),
            Error::Config(error) => std::error::Error::source(error),
            Error::Kobold(error) => std::error::Error::source(error),
        }
    }
}
