#![allow(dead_code)]

pub struct FlashError {
    pub error: FlashErrorType,
    pub traceback: String,
}
pub enum FlashErrorType {
    EnvVar(String),
    ResourceError,
    FileReadError(String),
}
