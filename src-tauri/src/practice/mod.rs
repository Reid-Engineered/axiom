mod error;
mod provider;
mod store;
mod types;

pub use error::PracticeError;
pub use provider::PracticeProvider;
pub use store::PracticeStore;
pub use types::{
    AttemptStatus, DescribeRequest, DescribeResponse, EvaluateRequest, EvaluateResponse,
    GenerateRequest, GenerateResponse, HintRequest, HintResponse, ResponseValue, StartRequest,
    StartResponse,
};

/// The embedded first-party manifest for this module.
pub const MANIFEST_TOML: &str = include_str!("module.toml");

#[cfg(test)]
mod tests;
