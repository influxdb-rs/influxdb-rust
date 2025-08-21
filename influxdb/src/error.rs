//! Errors that might happen in the crate

use thiserror::Error;

#[derive(Debug, Eq, PartialEq, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("query is invalid: {error}")]
    /// Error happens when a query is invalid
    InvalidQueryError { error: String },

    #[error("Failed to build URL: {error}")]
    /// Error happens when a query is invalid
    UrlConstructionError { error: String },

    #[error("http protocol error: {error}")]
    /// Error happens when a query is invalid
    ProtocolError { error: String },

    #[error("http protocol error: {error}")]
    /// Error happens when Serde cannot deserialize the response
    DeserializationError { error: String },

    #[error("InfluxDB encountered the following error: {error}")]
    /// Error which has happened inside InfluxDB
    DatabaseError { error: String },

    #[error("API error with a status code: {0}")]
    /// Error happens when API returns non 2xx status code.
    ApiError(u16),

    #[error("connection error: {error}")]
    /// Error happens when HTTP request fails
    ConnectionError { error: String },
}
