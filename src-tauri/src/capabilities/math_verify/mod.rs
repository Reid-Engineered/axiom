mod error;
mod provider;
mod types;

pub use error::MathVerifyError;
pub use provider::MathVerifyProvider;
pub use types::{VerifyRequest, VerifyResult};

/// The embedded first-party manifest for this capability.
pub const MANIFEST_TOML: &str = include_str!("module.toml");

#[cfg(test)]
mod tests;
