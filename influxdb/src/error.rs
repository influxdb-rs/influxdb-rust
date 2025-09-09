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

#[cfg(feature = "chrono")]
#[derive(Clone, Copy, Debug, Error)]
#[error("The timestamp is too large to fit into an i64.")]
pub struct TimestampTooLargeError(pub(crate) ());

#[cfg(any(feature = "chrono", feature = "time"))]
#[derive(Clone, Copy, Debug, Error)]
pub enum TimeTryFromError<T, I> {
    TimeError(#[source] T),
    IntError(#[source] I),
}
