//! Storage backends for analytics data.

pub mod port;
pub mod sqlite;

#[cfg(feature = "postgres")]
pub mod postgres;

pub use port::StoragePort;

#[cfg(test)]
mod sqlite_test;
