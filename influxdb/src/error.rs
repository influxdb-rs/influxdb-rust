//! Errors that might happen in the crate

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "query is invalid: {}", error)]
    /// Error happens when a query is invalid
    InvalidQueryError { error: String },

    #[fail(display = "Failed to build URL: {}", error)]
    /// Error happens when a query is invalid
    UrlConstructionError { error: String },

    #[fail(display = "http protocol error: {}", error)]
    /// Error happens when a query is invalid
    ProtocolError { error: String },

    #[fail(display = "http protocol error: {}", error)]
    /// Error happens when Serde cannot deserialize the response
    DeserializationError { error: String },

    #[fail(display = "InfluxDB encountered the following error: {}", error)]
    /// Error which has happened inside InfluxDB
    DatabaseError { error: String },

    #[fail(display = "authentication error. No or incorrect credentials")]
    /// Error happens when no or incorrect credentials are used. `HTTP 401 Unauthorized`
    AuthenticationError,

    #[fail(display = "authorization error. User not authorized")]
    /// Error happens when the supplied user is not authorized. `HTTP 403 Forbidden`
    AuthorizationError,

    #[fail(display = "connection error: {}", error)]
    /// Error happens when reqwest fails
    ConnectionError {
        #[fail(cause)]
        error: reqwest::Error,
    },
}
