use std::fmt::{Display, Formatter};
use reqwest::StatusCode;
use packets::types::PacketState;

pub type Res<T = ()> = Result<T, Error>;
pub type ResContext<T = ()> = Result<T, ErrorContext<Error>>;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    CSV(csv::Error),
    Socks5(tokio_socks::Error),
    Reqwest(reqwest::Error),
    WrongPacket {
        state: PacketState,
        expected: u32,
        actual: u32,
    },
    Simple(String),
    Mojang(MojangErr)
}

#[derive(Debug)]
pub enum MojangErr {
    InvalidCredentials {
        error_code: StatusCode,
        info: Option<String>,
    },
}

impl Display for MojangErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("yes")
    }
}


impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(io) => io.fmt(f),
            Error::CSV(csv) => csv.fmt(f),
            Error::Simple(str) => f.write_str(str),
            Error::Reqwest(inner) => inner.fmt(f),
            Error::Socks5(socks) => socks.fmt(f),
            Error::Mojang(inner) => inner.fmt(f),
            Error::WrongPacket { state, actual, expected } => f.write_fmt(format_args!("wrong packet. Expected ID {}, got {} in state {}", expected, actual, state)),
        }
    }
}

pub fn err(str: String) -> Result<(), ErrorContext<Error>> {
    Err(ErrorContext {
        inner: Error::Simple(str),
        context: "".to_string(),
    })
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Self::CSV(err)
    }
}

impl From<MojangErr> for Error {
    fn from(err: MojangErr) -> Self {
        Self::Mojang(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

// impl From<tokio::io::Error> for Error {
//     fn from(err: tokio::io::Error) -> Self {
//         Self::Tokio(err)
//     }
// }

impl From<tokio_socks::Error> for Error {
    fn from(err: tokio_socks::Error) -> Self {
        Self::Socks5(err)
    }
}

// impl From<tokio::io::Error> for Error {
//     fn from(err: tokio_socks::Error) -> Self {
//         Self::Socks5(err)
//     }
// }

pub struct ErrorContext<T> {
    inner: T,
    context: String,
}

impl<T: Display> Display for ErrorContext<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error {} : {}", self.context, self.inner))
    }
}

pub trait HasContext<T, E> {
    fn context(self, f: impl Fn() -> String) -> Result<T, ErrorContext<E>>;
}

impl<T, E: Into<Error>> HasContext<T, Error> for Result<T, E> {
    fn context(self, f: impl Fn() -> String) -> Result<T, ErrorContext<Error>> {
        match self {
            Ok(res) => Ok(res),
            Err(inner) => {
                let inner = inner.into();
                let context = f();
                Err(ErrorContext {
                    inner,
                    context,
                })
            }
        }
    }
}
