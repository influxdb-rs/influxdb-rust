//! Used to create queries of type [`ReadQuery`](crate::query::read_query::ReadQuery) or
//! [`WriteQuery`](crate::query::write_query::WriteQuery) which can be executed in InfluxDB
//!
//! # Examples
//!
//! ```rust
//! use influxdb::{Query, Timestamp};
//!
//! let write_query = Query::write_query(Timestamp::Now, "measurement")
//!     .add_field("field1", 5)
//!     .add_tag("author", "Gero")
//!     .build();
//!
//! assert!(write_query.is_ok());
//!
//! let read_query = Query::raw_read_query("SELECT * FROM weather")
//!     .build();
//!
//! assert!(read_query.is_ok());
//! ```

pub mod read_query;
pub mod write_query;

use std::fmt;

use crate::{Error, ReadQuery, WriteQuery};

#[derive(PartialEq)]
pub enum Timestamp {
    Now,
    Nanoseconds(usize),
    Microseconds(usize),
    Milliseconds(usize),
    Seconds(usize),
    Minutes(usize),
    Hours(usize),
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Timestamp::*;
        match self {
            Now => write!(f, ""),
            Nanoseconds(ts) | Microseconds(ts) | Milliseconds(ts) | Seconds(ts) | Minutes(ts)
            |   Hours(ts) => write!(f, "{}", ts),
        }
    }
}

/// Internal enum used to represent either type of query.
pub enum QueryTypes<'a> {
    Read(&'a ReadQuery),
    Write(&'a WriteQuery),
}

impl<'a> From<&'a ReadQuery> for QueryTypes<'a> {
    fn from(query: &'a ReadQuery) -> Self {
        Self::Read(query)
    }
}

impl<'a> From<&'a WriteQuery> for QueryTypes<'a> {
    fn from(query: &'a WriteQuery) -> Self {
        Self::Write(query)
    }
}

pub trait Query {
    /// Builds valid InfluxSQL which can be run against the Database.
    /// In case no fields have been specified, it will return an error,
    /// as that is invalid InfluxSQL syntax.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::{Query, Timestamp};
    ///
    /// let invalid_query = Query::write_query(Timestamp::Now, "measurement").build();
    /// assert!(invalid_query.is_err());
    ///
    /// let valid_query = Query::write_query(Timestamp::Now, "measurement").add_field("myfield1", 11).build();
    /// assert!(valid_query.is_ok());
    /// ```
    fn build(&self) -> Result<ValidQuery, Error>;

    fn get_type(&self) -> QueryType;
}

impl dyn Query {
    /// Returns a [`WriteQuery`](crate::WriteQuery) builder.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::{Query, Timestamp};
    ///
    /// Query::write_query(Timestamp::Now, "measurement"); // Is of type [`WriteQuery`](crate::WriteQuery)
    /// ```
    pub fn write_query<S>(timestamp: Timestamp, measurement: S) -> WriteQuery
    where
        S: Into<String>,
    {
        WriteQuery::new(timestamp, measurement)
    }

    /// Returns a [`ReadQuery`](crate::ReadQuery) builder.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::Query;
    ///
    /// Query::raw_read_query("SELECT * FROM weather"); // Is of type [`ReadQuery`](crate::ReadQuery)
    /// ```
    pub fn raw_read_query<S>(read_query: S) -> ReadQuery
    where
        S: Into<String>,
    {
        ReadQuery::new(read_query)
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
impl<T> From<T> for ValidQuery
where
    T: Into<String>,
{
    fn from(string: T) -> Self {
        Self(string.into())
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
#[derive(PartialEq, Debug)]
pub enum QueryType {
    ReadQuery,
    WriteQuery,
}

#[cfg(test)]
mod tests {
    use crate::query::{Timestamp, ValidQuery};

    #[test]
    fn test_equality_str() {
        assert_eq!(ValidQuery::from("hello"), "hello");
    }

    #[test]
    fn test_equality_string() {
        assert_eq!(
            ValidQuery::from(String::from("hello")),
            String::from("hello")
        );
    }

    #[test]
    fn test_format_for_timestamp_now() {
        assert!(format!("{}", Timestamp::Now) == "");
    }

    #[test]
    fn test_format_for_timestamp_else() {
        assert!(format!("{}", Timestamp::Nanoseconds(100)) == "100");
    }
}
