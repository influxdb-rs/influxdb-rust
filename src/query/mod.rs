//! Used to create queries of type [`InfluxDbReadQuery`](crate::query::read_query::InfluxDbReadQuery) or
//! [`InfluxDbWriteQuery`](crate::query::write_query::InfluxDbWriteQuery) which can be executed in InfluxDB
//!
//! # Examples
//!
//! ```rust
//! use influxdb::query::InfluxDbQuery;
//!
//! let write_query = InfluxDbQuery::write_query("measurement")
//!     .add_field("field1", 5)
//!     .add_tag("author", "Gero")
//!     .build();
//!
//! assert!(write_query.is_ok());
//!
//! let read_query = InfluxDbQuery::raw_read_query("SELECT * FROM weather")
//!     .build();
//!
//! assert!(read_query.is_ok());
//! ```

pub mod read_query;
pub mod write_query;

use crate::error::InfluxDbError;
use crate::query::read_query::InfluxDbReadQuery;
use crate::query::write_query::InfluxDbWriteQuery;

pub trait InfluxDbQuery {
    /// Builds valid InfluxSQL which can be run against the Database.
    /// In case no fields have been specified, it will return an error,
    /// as that is invalid InfluxSQL syntax.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::query::InfluxDbQuery;
    ///
    /// let invalid_query = InfluxDbQuery::write_query("measurement").build();
    /// assert!(invalid_query.is_err());
    ///
    /// let valid_query = InfluxDbQuery::write_query("measurement").add_field("myfield1", 11).build();
    /// assert!(valid_query.is_ok());
    /// ```
    fn build(self) -> Result<ValidQuery, InfluxDbError>;

    fn get_type(&self) -> QueryType;
}

impl InfluxDbQuery {
    /// Returns a [`InfluxDbWriteQuery`](crate::query::write_query::InfluxDbWriteQuery) builder.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::query::InfluxDbQuery;
    ///
    /// InfluxDbQuery::write_query("measurement"); // Is of type [`InfluxDbWriteQuery`](crate::query::write_query::InfluxDbWriteQuery)
    /// ```
    pub fn write_query<S>(measurement: S) -> InfluxDbWriteQuery
    where
        S: ToString,
    {
        InfluxDbWriteQuery::new(measurement)
    }

    /// Returns a [`InfluxDbReadQuery`](crate::query::read_query::InfluxDbReadQuery) builder.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::query::InfluxDbQuery;
    ///
    /// InfluxDbQuery::raw_read_query("SELECT * FROM weather"); // Is of type [`InfluxDbReadQuery`](crate::query::read_query::InfluxDbReadQuery)
    /// ```
    pub fn raw_read_query<S>(read_query: S) -> InfluxDbReadQuery
    where
        S: ToString,
    {
        InfluxDbReadQuery::new(read_query)
    }
}

#[derive(Debug)]
#[doc(hidden)]
pub struct ValidQuery(String);
impl ValidQuery {
    pub fn get(self) -> String {
        self.0
    }
}
impl PartialEq<String> for ValidQuery {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}
impl PartialEq<&str> for ValidQuery {
    fn eq(&self, other: &&str) -> bool {
        &self.0 == other
    }
}

/// Internal Enum used to decide if a `POST` or `GET` request should be sent to InfluxDB. See [InfluxDB Docs](https://docs.influxdata.com/influxdb/v1.7/tools/api/#query-http-endpoint).
pub enum QueryType {
    ReadQuery,
    WriteQuery,
}