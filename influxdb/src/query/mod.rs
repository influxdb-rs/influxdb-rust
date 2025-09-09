//! Used to create queries of type [`ReadQuery`](crate::query::read_query::ReadQuery) or
//! [`WriteQuery`](crate::query::write_query::WriteQuery) which can be executed in InfluxDB
//!
//! # Examples
//!
//! ```rust
//! use influxdb::{InfluxDbWriteable, Query as _, ReadQuery, Timestamp};
//!
//! let write_query = Timestamp::Nanoseconds(0)
//!     .into_query("measurement")
//!     .add_field("field1", 5)
//!     .add_tag("author", "Gero")
//!     .build();
//!
//! assert!(write_query.is_ok());
//!
//! let read_query = ReadQuery::new("SELECT * FROM weather").build();
//!
//! assert!(read_query.is_ok());
//! ```

pub mod consts;
mod line_proto_term;
pub mod read_query;
pub mod write_query;
use std::convert::Infallible;
use std::fmt;

use crate::{Error, WriteQuery};
use consts::{
    MILLIS_PER_SECOND, MINUTES_PER_HOUR, NANOS_PER_MICRO, NANOS_PER_MILLI, SECONDS_PER_MINUTE,
};

#[cfg(feature = "derive")]
pub use influxdb_derive::InfluxDbWriteable;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Timestamp {
    Nanoseconds(u128),
    Microseconds(u128),
    Milliseconds(u128),
    Seconds(u128),
    Minutes(u128),
    Hours(u128),
}

impl Timestamp {
    pub fn nanos(&self) -> u128 {
        match self {
            Timestamp::Hours(h) => {
                h * MINUTES_PER_HOUR * SECONDS_PER_MINUTE * MILLIS_PER_SECOND * NANOS_PER_MILLI
            }
            Timestamp::Minutes(m) => m * SECONDS_PER_MINUTE * MILLIS_PER_SECOND * NANOS_PER_MILLI,
            Timestamp::Seconds(s) => s * MILLIS_PER_SECOND * NANOS_PER_MILLI,
            Timestamp::Milliseconds(millis) => millis * NANOS_PER_MILLI,
            Timestamp::Microseconds(micros) => micros * NANOS_PER_MICRO,
            Timestamp::Nanoseconds(nanos) => *nanos,
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Timestamp::*;
        match self {
            Nanoseconds(ts) | Microseconds(ts) | Milliseconds(ts) | Seconds(ts) | Minutes(ts)
            | Hours(ts) => write!(f, "{ts}"),
        }
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<Timestamp> for chrono::DateTime<chrono::Utc> {
    type Error = <i64 as TryFrom<u128>>::Error;

    fn try_from(ts: Timestamp) -> Result<Self, Self::Error> {
        use chrono::TimeZone as _;
        Ok(chrono::Utc.timestamp_nanos(ts.nanos().try_into()?))
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<chrono::DateTime<chrono::Utc>> for Timestamp {
    type Error = crate::error::TimeTryFromError<
        crate::error::TimestampTooLargeError,
        <u128 as TryFrom<i64>>::Error,
    >;

    fn try_from(dt: chrono::DateTime<chrono::Utc>) -> Result<Self, Self::Error> {
        // unfortunately chrono doesn't give us the nanos as i128, so we have to error
        // if it doesn't fit and then cast the i64 to u128 anyways
        let nanos = dt
            .timestamp_nanos_opt()
            .ok_or(Self::Error::TimeError(
                crate::error::TimestampTooLargeError(()),
            ))?
            .try_into()
            .map_err(Self::Error::IntError)?;
        Ok(Self::Nanoseconds(nanos))
    }
}

#[cfg(feature = "time")]
impl TryFrom<Timestamp> for time::UtcDateTime {
    type Error =
        crate::error::TimeTryFromError<time::error::ComponentRange, <i128 as TryFrom<u128>>::Error>;

    fn try_from(value: Timestamp) -> Result<Self, Self::Error> {
        let nanos = value.nanos().try_into().map_err(Self::Error::IntError)?;
        time::UtcDateTime::from_unix_timestamp_nanos(nanos).map_err(Self::Error::TimeError)
    }
}

#[cfg(feature = "time")]
impl TryFrom<time::UtcDateTime> for Timestamp {
    type Error = <u128 as TryFrom<i128>>::Error;

    fn try_from(value: time::UtcDateTime) -> Result<Self, Self::Error> {
        Ok(Timestamp::Nanoseconds(
            value.unix_timestamp_nanos().try_into()?,
        ))
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
    /// use influxdb::{InfluxDbWriteable, Query, Timestamp};
    ///
    /// let invalid_query = Timestamp::Nanoseconds(0).into_query("measurement").build();
    /// assert!(invalid_query.is_err());
    ///
    /// let valid_query = Timestamp::Nanoseconds(0)
    ///     .into_query("measurement")
    ///     .add_field("myfield1", 11)
    ///     .build();
    /// assert!(valid_query.is_ok());
    /// ```
    fn build(&self) -> Result<ValidQuery, Error>;

    /// Like [build] but with additional support for unsigned integers in the line protocol.
    /// Please note, this crate can only interact with InfluxDB 2.0 in compatibility mode
    /// and does not natively support InfluxDB 2.0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::{InfluxDbWriteable, Query, Timestamp};
    ///
    /// let use_v2 = true;
    ///
    /// let invalid_query = Timestamp::Nanoseconds(0)
    ///     .into_query("measurement")
    ///     .build_with_opts(use_v2);
    /// assert!(invalid_query.is_err());
    ///
    /// let valid_query = Timestamp::Nanoseconds(0)
    ///     .into_query("measurement")
    ///     .add_field("myfield1", 11)
    ///     .build_with_opts(use_v2);
    /// assert!(valid_query.is_ok());
    /// ```
    fn build_with_opts(&self, use_v2: bool) -> Result<ValidQuery, Error>;

    fn get_type(&self) -> QueryType;
}

impl<Q: Query> Query for &Q {
    fn build(&self) -> Result<ValidQuery, Error> {
        Q::build_with_opts(self, false)
    }

    fn build_with_opts(&self, use_v2: bool) -> Result<ValidQuery, Error> {
        Q::build_with_opts(self, use_v2)
    }

    fn get_type(&self) -> QueryType {
        Q::get_type(self)
    }
}

impl<Q: Query> Query for Box<Q> {
    fn build(&self) -> Result<ValidQuery, Error> {
        Q::build(self)
    }

    fn build_with_opts(&self, use_v2: bool) -> Result<ValidQuery, Error> {
        Q::build_with_opts(self, use_v2)
    }

    fn get_type(&self) -> QueryType {
        Q::get_type(self)
    }
}

pub trait InfluxDbWriteable {
    type Error;

    fn try_into_query<I: Into<String>>(self, name: I) -> Result<WriteQuery, Self::Error>;
}

impl InfluxDbWriteable for Timestamp {
    type Error = Infallible;

    fn try_into_query<I: Into<String>>(self, name: I) -> Result<WriteQuery, Infallible> {
        Ok(WriteQuery::new(self, name.into()))
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
#[derive(PartialEq, Eq, Debug)]
pub enum QueryType {
    ReadQuery,
    /// write query with precision
    WriteQuery(String),
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "chrono")]
    use super::consts::{
        MILLIS_PER_SECOND, MINUTES_PER_HOUR, NANOS_PER_MICRO, NANOS_PER_MILLI, SECONDS_PER_MINUTE,
    };
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
    fn test_format_for_timestamp_else() {
        assert!(format!("{}", Timestamp::Nanoseconds(100)) == "100");
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_chrono_datetime_from_timestamp_hours() {
        use chrono::prelude::*;
        let datetime_from_timestamp: DateTime<Utc> = Timestamp::Hours(2).try_into().unwrap();
        assert_eq!(
            Utc.timestamp_nanos(
                (2 * MINUTES_PER_HOUR * SECONDS_PER_MINUTE * MILLIS_PER_SECOND * NANOS_PER_MILLI)
                    .try_into()
                    .unwrap()
            ),
            datetime_from_timestamp
        )
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_chrono_datetime_from_timestamp_minutes() {
        use chrono::prelude::*;
        let datetime_from_timestamp: DateTime<Utc> = Timestamp::Minutes(2).try_into().unwrap();
        assert_eq!(
            Utc.timestamp_nanos(
                (2 * SECONDS_PER_MINUTE * MILLIS_PER_SECOND * NANOS_PER_MILLI)
                    .try_into()
                    .unwrap()
            ),
            datetime_from_timestamp
        )
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_chrono_datetime_from_timestamp_seconds() {
        use chrono::prelude::*;
        let datetime_from_timestamp: DateTime<Utc> = Timestamp::Seconds(2).try_into().unwrap();
        assert_eq!(
            Utc.timestamp_nanos(
                (2 * MILLIS_PER_SECOND * NANOS_PER_MILLI)
                    .try_into()
                    .unwrap()
            ),
            datetime_from_timestamp
        )
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_chrono_datetime_from_timestamp_millis() {
        use chrono::prelude::*;
        let datetime_from_timestamp: DateTime<Utc> = Timestamp::Milliseconds(2).try_into().unwrap();
        assert_eq!(
            Utc.timestamp_nanos((2 * NANOS_PER_MILLI).try_into().unwrap()),
            datetime_from_timestamp
        )
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_chrono_datetime_from_timestamp_nanos() {
        use chrono::prelude::*;
        let datetime_from_timestamp: DateTime<Utc> = Timestamp::Nanoseconds(1).try_into().unwrap();
        assert_eq!(Utc.timestamp_nanos(1), datetime_from_timestamp)
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_chrono_datetime_from_timestamp_micros() {
        use chrono::prelude::*;
        let datetime_from_timestamp: DateTime<Utc> = Timestamp::Microseconds(2).try_into().unwrap();
        assert_eq!(
            Utc.timestamp_nanos((2 * NANOS_PER_MICRO).try_into().unwrap()),
            datetime_from_timestamp
        )
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_timestamp_from_chrono_date() {
        use chrono::prelude::*;
        let timestamp_from_datetime: Timestamp = Utc
            .with_ymd_and_hms(1970, 1, 1, 0, 0, 1)
            .single()
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            Timestamp::Nanoseconds(MILLIS_PER_SECOND * NANOS_PER_MILLI),
            timestamp_from_datetime
        )
    }
}
