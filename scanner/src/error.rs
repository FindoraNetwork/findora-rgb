use bitcoin::Block;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum ScannerError {
    Custom(String),
    DBError(sqlx::Error),
    JoinError(tokio::task::JoinError),
    ParseUrlError(url::ParseError),
    ReqwestError(reqwest::Error),
    ParseIntError(ParseIntError),
    DecodeError(base64::DecodeError),
    SerdeJsonError(serde_json::Error),
    BlockNotFound(u64),
    HexError(rustc_hex::FromHexError),
    SenderError(crossbeam_channel::SendError<Block>),
    BtcRpcError(bitcoincore_rpc::Error),
}
impl From<crossbeam_channel::SendError<Block>> for ScannerError {
    fn from(e: crossbeam_channel::SendError<Block>) -> Self {
        ScannerError::SenderError(e)
    }
}
impl From<bitcoincore_rpc::Error> for ScannerError {
    fn from(e: bitcoincore_rpc::Error) -> Self {
        ScannerError::BtcRpcError(e)
    }
}
impl From<String> for ScannerError {
    fn from(e: String) -> Self {
        ScannerError::Custom(e)
    }
}

impl From<sqlx::Error> for ScannerError {
    fn from(e: sqlx::Error) -> Self {
        ScannerError::DBError(e)
    }
}

impl From<tokio::task::JoinError> for ScannerError {
    fn from(e: tokio::task::JoinError) -> Self {
        ScannerError::JoinError(e)
    }
}

impl From<url::ParseError> for ScannerError {
    fn from(e: url::ParseError) -> Self {
        ScannerError::ParseUrlError(e)
    }
}

impl From<reqwest::Error> for ScannerError {
    fn from(e: reqwest::Error) -> Self {
        ScannerError::ReqwestError(e)
    }
}

impl From<ParseIntError> for ScannerError {
    fn from(e: ParseIntError) -> Self {
        ScannerError::ParseIntError(e)
    }
}

impl From<base64::DecodeError> for ScannerError {
    fn from(e: base64::DecodeError) -> Self {
        ScannerError::DecodeError(e)
    }
}

impl From<serde_json::Error> for ScannerError {
    fn from(e: serde_json::Error) -> Self {
        ScannerError::SerdeJsonError(e)
    }
}

impl From<rustc_hex::FromHexError> for ScannerError {
    fn from(e: rustc_hex::FromHexError) -> Self {
        ScannerError::HexError(e)
    }
}

pub type Result<T> = core::result::Result<T, ScannerError>;
