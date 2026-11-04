mod error;
mod provider;
mod types;

pub use error::MathVerifyError;
pub use provider::{verify_v2, MathVerifyProvider};
pub use types::{
    ContentRefInput, ObjectiveVerifyResult, VerifyMethod, VerifyRequest, VerifyResult,
    VerifySample, VerifyTerm, VerifyV2Request, VerifyV2Result, VerifyVerdict,
};

/// The embedded first-party manifest for this capability.
pub const MANIFEST_TOML: &str = include_str!("module.toml");

#[cfg(test)]
mod tests;
