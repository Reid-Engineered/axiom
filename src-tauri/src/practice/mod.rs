mod error;
mod provider;
mod store;
mod types;

pub use error::PracticeError;
pub use provider::PracticeProvider;
pub use types::{
    AttemptStatus, EvaluateRequest, EvaluateResponse, GenerateRequest, GenerateResponse,
    HintRequest, HintResponse, ResponseValue,
};

/// The embedded first-party manifest for this module.
pub const MANIFEST_TOML: &str = include_str!("module.toml");
