use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;

pub mod concept;
pub mod goal;
pub mod material;
pub mod module;
pub mod note;
pub mod seed;
pub mod session;
pub mod workspace;

mod models;

#[cfg(test)]
mod tests;

pub use models::*;

pub type CommandResult<T> = Result<T, String>;

pub struct Database(Mutex<Connection>);

impl Database {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        crate::db::open(path).map(|connection| Self(Mutex::new(connection)))
    }

    #[cfg(test)]
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        crate::db::open_in_memory().map(|connection| Self(Mutex::new(connection)))
    }

    pub(crate) fn connection(&self) -> CommandResult<MutexGuard<'_, Connection>> {
        self.0
            .lock()
            .map_err(|_| "Database connection lock is poisoned".to_owned())
    }
}

pub(crate) fn database_error(error: rusqlite::Error) -> String {
    error.to_string()
}

pub(crate) fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

pub(crate) fn new_id(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4())
}
