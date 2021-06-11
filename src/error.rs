use std::fmt::{Display, Formatter};

pub type Res<T = ()> = Result<T, Error>;
pub type ResContext<T = ()> = Result<T, Context<Error>>;

pub enum Error {
    IO(std::io::Error),
    CSV(csv::Error),
    Socks5(tokio_socks::Error),
    Tokio(tokio::io::Error),
    Simple(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(io) => io.fmt(f),
            Error::CSV(csv) => csv.fmt(f),
            Error::Simple(str) => f.write_str(str),
            Error::Socks5(socks) => socks.fmt(f),
            Error::Tokio(tokio) => tokio.fmt(f)
        }
    }
}

pub fn err(str: String) -> Result<(), Context<Error>> {
    Err(Context {
        inner: Error::Simple(str),
        context: "".to_string(),
    })
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Self::CSV(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
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

pub struct Context<T> {
    inner: T,
    context: String,
}

impl<T: Display> Display for Context<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error {} : {}", self.context, self.inner))
    }
}

pub trait HasContext<T, E> {
    fn context(self, f: impl Fn() -> String) -> Result<T, Context<E>>;
}

impl<T, E: Into<Error>> HasContext<T, Error> for Result<T, E> {
    fn context(self, f: impl Fn() -> String) -> Result<T, Context<Error>> {
        match self {
            Ok(res) => Ok(res),
            Err(inner) => {
                let inner = inner.into();
                let context = f();
                Err(Context {
                    inner,
                    context,
                })
            }
        }
    }
}
