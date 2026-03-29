use thiserror::Error;

#[derive(Debug, Error)]
pub enum TepError {
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
}
