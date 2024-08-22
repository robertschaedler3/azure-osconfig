use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Unable to find component: {0}")]
    ComponentNotFound(String),

    #[error(transparent)]
    Library(#[from] libloading::Error),

    #[error(transparent)]
    Errno(#[from] errno::Errno),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Null(#[from] std::ffi::NulError),
}
