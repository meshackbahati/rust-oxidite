use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Server error: {0}")]
    Server(String),
    #[error("Not found")]
    NotFound,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
