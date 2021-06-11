use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    IO(ContextError<std::io::Error>)
}

#[derive(Debug)]
pub struct ContextError<T: Display + Debug> {
    context: String,
    error: T,
}

pub trait ContextTrait<T, E> where E: Display + Debug {
    fn context(self, context: impl Fn() -> String) -> Result<T, ContextError<E>>;
}

impl<E: Display + Debug, T> ContextTrait<T, E> for Result<T, E> {
    fn context(self, context: impl Fn() -> String) -> Result<T, ContextError<E>> {
        match self {
            Ok(x) => Ok(x),
            Err(error) => {
                let context = context();
                let wrapped = ContextError {
                    context,
                    error,
                };
                Err(wrapped)
            }
        }
    }
}

impl<T: Display + Debug> Display for ContextError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error {}: {}", self.context, self.error))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(err) => std::fmt::Display::fmt(err, f)
        }
    }
}

impl From<ContextError<std::io::Error>> for Error {
    fn from(err: ContextError<std::io::Error>) -> Self {
        Self::IO(err)
    }
}
