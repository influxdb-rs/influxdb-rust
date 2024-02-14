//! Write Query Builder returned by Query::write_query
//!
//! Can only be instantiated by using Query::write_query

use crate::query::line_proto_term::LineProtoTerm;
use crate::query::{QueryType, ValidQuery};
use crate::{Error, Query, Timestamp};
use std::fmt::{Display, Formatter};

pub trait WriteType {
    fn add_to(self, tag: String, fields_or_tags: &mut Vec<(String, Type)>);
}

impl<T: Into<Type>> WriteType for T {
    fn add_to(self, tag: String, fields_or_tags: &mut Vec<(String, Type)>) {
        let val: Type = self.into();
        fields_or_tags.push((tag, val));
    }
}

impl<T: Into<Type>> WriteType for Option<T> {
    fn add_to(self, tag: String, fields_or_tags: &mut Vec<(String, Type)>) {
        if let Some(val) = self {
            val.add_to(tag, fields_or_tags);
        }
    }
}

/// Internal Representation of a Write query that has not yet been built
#[derive(Debug, Clone)]
pub struct WriteQuery {
    fields: Vec<(String, Type)>,
    tags: Vec<(String, Type)>,
    measurement: String,
    timestamp: Timestamp,
}

impl WriteQuery {
    /// Creates a new [`WriteQuery`](crate::query::write_query::WriteQuery)
    #[must_use = "Creating a query is pointless unless you execute it"]
    pub fn new<S>(timestamp: Timestamp, measurement: S) -> Self
    where
        S: Into<String>,
    {
        WriteQuery {
            fields: vec![],
            tags: vec![],
            measurement: measurement.into(),
            timestamp,
        }
    }

    /// Adds a field to the [`WriteQuery`](crate::WriteQuery)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::{Query, Timestamp};
    /// use influxdb::InfluxDbWriteable;
    ///
    /// Timestamp::Nanoseconds(0).into_query("measurement").add_field("field1", 5).build();
    /// ```
    #[must_use = "Creating a query is pointless unless you execute it"]
    pub fn add_field<S, F>(mut self, field: S, value: F) -> Self
    where
        S: Into<String>,
        F: WriteType,
    {
        value.add_to(field.into(), &mut self.fields);
        self
    }

    /// Adds a tag to the [`WriteQuery`](crate::WriteQuery)
    ///
    /// Please note that a [`WriteQuery`](crate::WriteQuery) requires at least one field. Composing a query with
    /// only tags will result in a failure building the query.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::{Query, Timestamp};
    /// use influxdb::InfluxDbWriteable;
    ///
    /// Timestamp::Nanoseconds(0)
    ///     .into_query("measurement")
    ///     .add_tag("field1", 5); // calling `.build()` now would result in a `Err(Error::InvalidQueryError)`
    /// ```
    #[must_use = "Creating a query is pointless unless you execute it"]
    pub fn add_tag<S, I>(mut self, tag: S, value: I) -> Self
    where
        S: Into<String>,
        I: WriteType,
    {
        value.add_to(tag.into(), &mut self.tags);
        self
    }

    pub fn get_precision(&self) -> String {
        let modifier = match self.timestamp {
            Timestamp::Nanoseconds(_) => "ns",
            Timestamp::Microseconds(_) => "u",
            Timestamp::Milliseconds(_) => "ms",
            Timestamp::Seconds(_) => "s",
            Timestamp::Minutes(_) => "m",
            Timestamp::Hours(_) => "h",
        };
        modifier.to_string()
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Boolean(bool),
    Float(f64),
    SignedInteger(i64),
    UnsignedInteger(u64),
    Text(String),
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use Type::*;

        match self {
            Boolean(x) => write!(f, "{}", x),
            Float(x) => write!(f, "{}", x),
            SignedInteger(x) => write!(f, "{}", x),
            UnsignedInteger(x) => write!(f, "{}", x),
            Text(text) => write!(f, "{text}", text = text),
        }
    }
}

macro_rules! from_impl {
        ( $variant:ident => $( $typ:ident ),+ ) => (
                $(
                    impl From<$typ> for Type {
                        fn from(b: $typ) -> Self {
                            Type::$variant(b.into())
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
impl From<&str> for Type {
    fn from(b: &str) -> Self {
        Type::Text(b.into())
    }
}
impl<T> From<&T> for Type
where
    T: Copy + Into<Type>,
{
    fn from(t: &T) -> Self {
        (*t).into()
    }
}

impl Query for WriteQuery {
    fn build(&self) -> Result<ValidQuery, Error> {
        self.build_with_opts(false)
    }

    fn build_with_opts(&self, use_v2: bool) -> Result<ValidQuery, Error> {
        if self.fields.is_empty() {
            return Err(Error::InvalidQueryError {
                error: "fields cannot be empty".to_string(),
            });
        }

        let mut tags = self
            .tags
            .iter()
            .map(|(tag, value)| {
                let escaped_tag_key = if use_v2 {
                    LineProtoTerm::TagKey(tag).escape_v2()
                } else {
                    LineProtoTerm::TagKey(tag).escape()
                };
                let escaped_tag_value = if use_v2 {
                    LineProtoTerm::TagValue(value).escape_v2()
                } else {
                    LineProtoTerm::TagValue(value).escape()
                };
                format!(
                    "{tag}={value}",
                    tag = escaped_tag_key,
                    value = escaped_tag_value,
                )
            })
            .collect::<Vec<String>>()
            .join(",");

        if !tags.is_empty() {
            tags.insert(0, ',');
        }
        let fields = self
            .fields
            .iter()
            .map(|(field, value)| {
                let escaped_field_key = if use_v2 {
                    LineProtoTerm::FieldKey(field).escape_v2()
                } else {
                    LineProtoTerm::FieldKey(field).escape()
                };
                let escaped_field_value = if use_v2 {
                    LineProtoTerm::FieldValue(value).escape_v2()
                } else {
                    LineProtoTerm::FieldValue(value).escape()
                };
                format!(
                    "{field}={value}",
                    field = escaped_field_key,
                    value = escaped_field_value,
                )
            })
            .collect::<Vec<String>>()
            .join(",");

        let escaped_measurement = if use_v2 {
            LineProtoTerm::Measurement(&self.measurement).escape_v2()
        } else {
            LineProtoTerm::Measurement(&self.measurement).escape()
        };

        Ok(ValidQuery(format!(
            "{measurement}{tags} {fields} {time}",
            measurement = escaped_measurement,
            tags = tags,
            fields = fields,
            time = self.timestamp
        )))
    }

    fn get_type(&self) -> QueryType {
        QueryType::WriteQuery(self.get_precision())
    }
}

impl Query for Vec<WriteQuery> {
    fn build(&self) -> Result<ValidQuery, Error> {
        let mut qlines = Vec::new();

        for q in self {
            let valid_query = q.build()?;
            qlines.push(valid_query.0);
        }

        Ok(ValidQuery(qlines.join("\n")))
    }

    fn build_with_opts(&self, use_v2: bool) -> Result<ValidQuery, Error> {
        let mut qlines = Vec::new();

        for q in self {
            let valid_query = q.build_with_opts(use_v2)?;
            qlines.push(valid_query.0);
        }

        Ok(ValidQuery(qlines.join("\n")))
    }

    fn get_type(&self) -> QueryType {
        QueryType::WriteQuery(
            self.first()
                .map(|q| q.get_precision())
                // use "ms" as placeholder if query is empty
                .unwrap_or_else(|| "ms".to_owned()),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::query::{InfluxDbWriteable, Query, Timestamp};

    #[test]
    fn test_write_builder_empty_query() {
        let query = Timestamp::Hours(5)
            .into_query("marina_3".to_string())
            .build();

        assert!(query.is_err(), "Query was not empty");
    }

    #[test]
    fn test_write_builder_single_field() {
        let query = Timestamp::Hours(11)
            .into_query("weather".to_string())
            .add_field("temperature", 82)
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(query.unwrap(), "weather temperature=82i 11");
    }

    #[test]
    fn test_write_builder_multiple_fields() {
        let query = Timestamp::Hours(11)
            .into_query("weather".to_string())
            .add_field("temperature", 82)
            .add_field("wind_strength", 3.7)
            .add_field("temperature_unsigned", 82u64)
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(
            query.unwrap(),
            "weather temperature=82i,wind_strength=3.7,temperature_unsigned=82i 11"
        );
    }

    #[test]
    fn test_write_builder_multiple_fields_with_v2() {
        let query = Timestamp::Hours(11)
            .into_query("weather".to_string())
            .add_field("temperature", 82)
            .add_field("wind_strength", 3.7)
            .add_field("temperature_unsigned", 82u64)
            .build_with_opts(true);

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(
            query.unwrap(),
            "weather temperature=82i,wind_strength=3.7,temperature_unsigned=82u 11"
        );
    }

    #[test]
    fn test_write_builder_optional_fields() {
        let query = Timestamp::Hours(11)
            .into_query("weather".to_string())
            .add_field("temperature", 82u64)
            .add_tag("wind_strength", <Option<u64>>::None)
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(query.unwrap(), "weather temperature=82i 11");
    }

    #[test]
    fn test_write_builder_optional_fields_with_v2() {
        let query = Timestamp::Hours(11)
            .into_query("weather".to_string())
            .add_field("temperature", 82u64)
            .add_tag("wind_strength", <Option<u64>>::None)
            .build_with_opts(true);

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(query.unwrap(), "weather temperature=82u 11");
    }

    #[test]
    fn test_write_builder_only_tags() {
        let query = Timestamp::Hours(11)
            .into_query("weather".to_string())
            .add_tag("season", "summer")
            .build();

        assert!(query.is_err(), "Query missing one or more fields");
    }

    #[test]
    fn test_write_builder_full_query() {
        let query = Timestamp::Hours(11)
            .into_query("weather".to_string())
            .add_field("temperature", 82)
            .add_tag("location", "us-midwest")
            .add_tag("season", "summer")
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(
            query.unwrap(),
            r#"weather,location=us-midwest,season=summer temperature=82i 11"#
        );
    }

    #[test]
    fn test_correct_query_type() {
        use crate::query::QueryType;

        let query = Timestamp::Hours(11)
            .into_query("weather".to_string())
            .add_field("temperature", 82)
            .add_tag("location", "us-midwest")
            .add_tag("season", "summer");

        assert_eq!(query.get_type(), QueryType::WriteQuery("h".to_owned()));
    }

    #[test]
    fn test_escaping() {
        let query = Timestamp::Hours(11)
            .into_query("wea, ther=")
            .add_field("temperature", 82)
            .add_field("\"temp=era,t ure\"", r#"too"\\hot"#)
            .add_field("float", 82.0)
            .add_tag("location", "us-midwest")
            .add_tag("loc, =\"ation", r#"us, "mid=west"#)
            .build();

        assert!(query.is_ok(), "Query was empty");
        let query_res = query.unwrap().get();
        assert_eq!(
            query_res,
            r#"wea\,\ ther=,location=us-midwest,loc\,\ \="ation=us\,\ \"mid\=west temperature=82i,"temp\=era\,t\ ure"="too\"\\\\hot",float=82 11"#
        );
    }

    #[test]
    fn test_batch() {
        let q0 = Timestamp::Hours(11)
            .into_query("weather")
            .add_field("temperature", 82)
            .add_tag("location", "us-midwest");

        let q1 = Timestamp::Hours(12)
            .into_query("weather")
            .add_field("temperature", 65)
            .add_tag("location", "us-midwest");

        let query = vec![q0, q1].build();

        assert_eq!(
            query.unwrap().get(),
            r#"weather,location=us-midwest temperature=82i 11
weather,location=us-midwest temperature=65i 12"#
        );
    }
}
