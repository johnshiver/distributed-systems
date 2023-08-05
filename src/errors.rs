use crate::in_memory_cluster::NetworkRequest;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkErrors {
    #[error("serializing request")]
    Serialization(#[from] serde_json::Error),
    // #[error("the data for key `{0}` is not available")]
    // Redaction(String),
    // #[error("invalid header (expected {expected:?}, found {found:?})")]
    // InvalidHeader {
    //     expected: String,
    //     found: String,
    // },
    #[error("unknown data store error")]
    Unknown,
    #[error("network timed out")]
    Timeout,
    #[error("error on channel send")]
    TokioChannel(#[from] tokio::sync::mpsc::error::SendError<NetworkRequest>),
}
