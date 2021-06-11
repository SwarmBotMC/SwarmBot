use std::fmt::{Display, Formatter};

pub type Res<T = ()> = Result<T, Error>;
pub type ResContext<T = ()> = Result<T, Context<Error>>;

pub enum Error {
    IO(std::io::Error),
    CSV(csv::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(io) => std::fmt::Display::fmt(&io, f),
            Error::CSV(csv) => std::fmt::Debug::fmt(&csv, f)
        }
    }
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

impl<T, E : Into<Error>> HasContext<T, Error> for Result<T, E> {
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
