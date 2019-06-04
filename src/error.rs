#[derive(Debug, Fail)]
/// Errors that might happen in the crate
pub enum InfluxDbError {
    #[fail(display = "query must contain at least one field")]
    /// Error happens when query has zero fields
    InvalidQueryError,

    #[fail(
        display = "an error happened: \"{}\". this case should be handled better, please file an issue.",
        error
    )]
    /// todo: Error which is a placeholder for more meaningful errors. This should be refactored away.
    UnspecifiedError { error: String },

    #[fail(display = "InfluxDB encountered the following error: {}", error)]
    /// Error which has happened inside InfluxDB
    DatabaseError { error: String },
}