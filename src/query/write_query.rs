//! Write Query Builder returned by InfluxDbQuery::write_query
//!
//! Can only be instantiated by using InfluxDbQuery::write_query

use crate::error::InfluxDbError;
use crate::query::{InfluxDbQuery, QueryType, Timestamp, ValidQuery};
use itertools::Itertools;

/// Internal Representation of a Write query that has not yet been built
pub struct InfluxDbWriteQuery {
    fields: Vec<(String, String)>,
    tags: Vec<(String, String)>,
    measurement: String,
    timestamp: Timestamp,
}

impl InfluxDbWriteQuery {
    /// Creates a new [`InfluxDbWriteQuery`](crate::query::write_query::InfluxDbWriteQuery)
    pub fn new<S>(timestamp: Timestamp, measurement: S) -> Self
    where
        S: ToString,
    {
        InfluxDbWriteQuery {
            fields: vec![],
            tags: vec![],
            measurement: measurement.to_string(),
            timestamp,
        }
    }

    /// Adds a field to the [`InfluxDbWriteQuery`](crate::query::write_query::InfluxDbWriteQuery)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::query::{InfluxDbQuery, Timestamp};
    ///
    /// InfluxDbQuery::write_query(Timestamp::NOW, "measurement").add_field("field1", 5).build();
    /// ```
    pub fn add_field<S, I>(mut self, tag: S, value: I) -> Self
    where
        S: ToString,
        I: Into<InfluxDbType>,
    {
        let val: InfluxDbType = value.into();
        self.fields.push((tag.to_string(), val.to_string()));
        self
    }

    /// Adds a tag to the [`InfluxDbWriteQuery`](crate::query::write_query::InfluxDbWriteQuery)
    ///
    /// Please note that a [`InfluxDbWriteQuery`](crate::query::write_query::InfluxDbWriteQuery) requires at least one field. Composing a query with
    /// only tags will result in a failure building the query.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::query::{InfluxDbQuery, Timestamp};
    ///
    /// InfluxDbQuery::write_query(Timestamp::NOW, "measurement")
    ///     .add_tag("field1", 5); // calling `.build()` now would result in a `Err(InfluxDbError::InvalidQueryError)`
    /// ```
    pub fn add_tag<S, I>(mut self, tag: S, value: I) -> Self
    where
        S: ToString,
        I: Into<InfluxDbType>,
    {
        let val: InfluxDbType = value.into();
        self.tags.push((tag.to_string(), val.to_string()));
        self
    }

    pub fn get_precision_modifier(&self) -> String {
        if self.timestamp == Timestamp::NOW {
            return String::from("");
        }

        let modifier = match self.timestamp {
            Timestamp::NANOSECONDS(_) => "ns",
            Timestamp::MICROSECONDS(_) => "u",
            Timestamp::MILLISECONDS(_) => "ms",
            Timestamp::SECONDS(_) => "s",
            Timestamp::MINUTES(_) => "m",
            Timestamp::HOURS(_) => "h",
            Timestamp::NOW => unreachable!(),
        };

        format!("&precision={modifier}", modifier = modifier)
    }
}

pub enum InfluxDbType {
    Boolean(bool),
    Float(f64),
    SignedInteger(i64),
    UnsignedInteger(u64),
    Text(String),
}

impl ToString for InfluxDbType {
    fn to_string(&self) -> String {
        use InfluxDbType::*;

        match self {
            Boolean(x) => x.to_string(),
            Float(x) => x.to_string(),
            SignedInteger(x) => x.to_string(),
            UnsignedInteger(x) => x.to_string(),
            Text(text) => format!("\"{text}\"", text = text),
        }
    }
}

macro_rules! from_impl {
        ( $variant:ident => $( $typ:ident ),+ ) => (
                $(
                    impl From<$typ> for InfluxDbType {
                        fn from(b: $typ) -> Self {
                            InfluxDbType::$variant(b.into())
                        }
                    }
                )+
        )
}
from_impl! {Boolean => bool}
from_impl! {Float => f32, f64}
from_impl! {SignedInteger => i8, i16, i32, i64}
from_impl! {UnsignedInteger => u8, u16, u32, u64}
from_impl! {Text => String}
impl From<&str> for InfluxDbType {
    fn from(b: &str) -> Self {
        InfluxDbType::Text(b.into())
    }
}

impl InfluxDbQuery for InfluxDbWriteQuery {
    fn build(&self) -> Result<ValidQuery, InfluxDbError> {
        if self.fields.is_empty() {
            return Err(InfluxDbError::InvalidQueryError {
                error: "fields cannot be empty".to_string(),
            });
        }

        let mut tags = self
            .tags
            .iter()
            .map(|(tag, value)| format!("{tag}={value}", tag = tag, value = value))
            .join(",");
        if !tags.is_empty() {
            tags.insert_str(0, ",");
        }
        let fields = self
            .fields
            .iter()
            .map(|(field, value)| format!("{field}={value}", field = field, value = value))
            .join(",");

        Ok(ValidQuery(format!(
            "{measurement}{tags} {fields}{time}",
            measurement = self.measurement,
            tags = tags,
            fields = fields,
            time = match self.timestamp {
                Timestamp::NOW => String::from(""),
                Timestamp::NANOSECONDS(ts)
                | Timestamp::MICROSECONDS(ts)
                | Timestamp::MILLISECONDS(ts)
                | Timestamp::SECONDS(ts)
                | Timestamp::MINUTES(ts)
                | Timestamp::HOURS(ts) => format!(" {}", ts),
            }
        )))
    }

    fn get_type(&self) -> QueryType {
        QueryType::WriteQuery
    }
}

#[cfg(test)]
mod tests {
    use crate::query::{InfluxDbQuery, Timestamp};

    #[test]
    fn test_write_builder_empty_query() {
        let query = InfluxDbQuery::write_query(Timestamp::HOURS(5), "marina_3").build();

        assert!(query.is_err(), "Query was not empty");
    }

    #[test]
    fn test_write_builder_single_field() {
        let query = InfluxDbQuery::write_query(Timestamp::HOURS(11), "weather")
            .add_field("temperature", 82)
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(query.unwrap(), "weather temperature=82 11");
    }

    #[test]
    fn test_write_builder_multiple_fields() {
        let query = InfluxDbQuery::write_query(Timestamp::HOURS(11), "weather")
            .add_field("temperature", 82)
            .add_field("wind_strength", 3.7)
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(
            query.unwrap(),
            "weather temperature=82,wind_strength=3.7 11"
        );
    }

    #[test]
    fn test_write_builder_only_tags() {
        let query = InfluxDbQuery::write_query(Timestamp::HOURS(11), "weather")
            .add_tag("season", "summer")
            .build();

        assert!(query.is_err(), "Query missing one or more fields");
    }

    #[test]
    fn test_write_builder_full_query() {
        let query = InfluxDbQuery::write_query(Timestamp::HOURS(11), "weather")
            .add_field("temperature", 82)
            .add_tag("location", "us-midwest")
            .add_tag("season", "summer")
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(
            query.unwrap(),
            "weather,location=\"us-midwest\",season=\"summer\" temperature=82 11"
        );
    }
}
