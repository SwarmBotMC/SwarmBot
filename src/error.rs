/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::fmt::{Debug, Display, Formatter};

use swarm-bot-packets::types::PacketState;
use reqwest::StatusCode;

pub type Res<T = ()> = Result<T, Error>;
pub type ResContext<T = ()> = Result<T, ErrorContext<Error>>;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Csv(csv::Error),
    Socks5(tokio_socks::Error),
    Serde(serde_json::Error),
    Reqwest(reqwest::Error),
    Resolve(Box<trust_dns_resolver::error::ResolveError>),
    WrongPacket {
        state: PacketState,
        expected: u32,
        actual: u32,
    },
    Simple(String),
    Mojang(MojangErr),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Serde(err)
    }
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
        match self {
            MojangErr::InvalidCredentials { error_code, info } => {
                f.write_fmt(format_args!("mojang err #{} info {}", error_code, info.clone().unwrap_or_default()))
            }
        }
    }
}


impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(io) => std::fmt::Display::fmt(io, f),
            Error::Csv(csv) => std::fmt::Display::fmt(csv, f),
            Error::Simple(str) => f.write_str(str),
            Error::Reqwest(inner) => std::fmt::Display::fmt(inner, f),
            Error::Socks5(socks) => std::fmt::Display::fmt(socks, f),
            Error::Mojang(inner) => std::fmt::Display::fmt(inner, f),
            Error::WrongPacket { state, actual, expected } => f.write_fmt(format_args!("wrong packet. Expected ID {}, got {} in state {}", expected, actual, state)),
            Error::Resolve(r) => std::fmt::Display::fmt(r, f),
            Error::Serde(s) => std::fmt::Display::fmt(s, f)
        }
    }
}

pub fn err(str: &str) -> Error {
    Error::Simple(str.to_string())
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Self::Csv(err)
    }
}

impl From<trust_dns_resolver::error::ResolveError> for Error {
    fn from(err: trust_dns_resolver::error::ResolveError) -> Self {
        Self::Resolve(box err)
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

#[derive(Debug)]
pub struct ErrorContext<T: Debug> {
    inner: T,
    context: String,
}

impl<T: Display + Debug> Display for ErrorContext<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error {} : {}", self.context, self.inner))
    }
}

pub trait HasContext<T, E: Debug>: Sized {
    fn context(self, f: impl Fn() -> String) -> Result<T, ErrorContext<E>>;
    fn context_str(self, str: &str) -> Result<T, ErrorContext<E>> {
        self.context(|| str.to_string())
    }
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
