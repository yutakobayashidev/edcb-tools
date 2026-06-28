pub type Result<T> = std::result::Result<T, EdcbError>;

#[derive(Debug, thiserror::Error)]
pub enum EdcbError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("operation timed out")]
    Timeout,
    #[error("EDCB command returned status {0}")]
    CommandStatus(i32),
    #[error("decode error: {0}")]
    Decode(String),
}
