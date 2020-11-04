//! Errors that might happen in the crate

use thiserror::Error;

#[derive(Debug, Error)]
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

    #[error("authentication error. No or incorrect credentials")]
    /// Error happens when no or incorrect credentials are used. `HTTP 401 Unauthorized`
    AuthenticationError,

    #[error("authorization error. User not authorized")]
    /// Error happens when the supplied user is not authorized. `HTTP 403 Forbidden`
    AuthorizationError,

    #[error("connection error: {error}")]
    /// Error happens when HTTP request fails
    ConnectionError { error: String },
}
