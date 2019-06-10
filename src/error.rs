//! Errors that might happen in the crate

#[derive(Debug, Fail)]
pub enum InfluxDbError {
    #[fail(display = "query is invalid: {}", error)]
    /// Error happens when a query is invalid
    InvalidQueryError { error: String },

    #[fail(display = "http protocol error: {}", error)]
    /// Error happens when a query is invalid
    ProtocolError { error: String },

    #[fail(display = "http protocol error: {}", error)]
    /// Error happens when Serde cannot deserialize the response
    DeserializationError { error: String },

    #[fail(display = "InfluxDB encountered the following error: {}", error)]
    /// Error which has happened inside InfluxDB
    DatabaseError { error: String },
}
