use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Unable to find component: {0}")]
    ComponentNotFound(String),

    #[error("Module failed to load: {0}")]
    ModuleFailedToLoad(String),

    #[error(transparent)]
    Library(#[from] libloading::Error),

    #[error(transparent)]
    Errno(#[from] errno::Errno),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Null(#[from] std::ffi::NulError),
}