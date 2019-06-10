//! Write Query Builder returned by InfluxDbQuery::write_query
//!
//! Can only be instantiated by using InfluxDbQuery::write_query

use crate::error::InfluxDbError;
use crate::query::{InfluxDbQuery, QueryType, ValidQuery};
use itertools::Itertools;

/// Internal Representation of a Write query that has not yet been built
pub struct InfluxDbWriteQuery {
    fields: Vec<(String, String)>,
    tags: Vec<(String, String)>,
    measurement: String,
    // precision: Precision
}

impl InfluxDbWriteQuery {
    /// Creates a new [`InfluxDbWriteQuery`](crate::query::write_query::InfluxDbWriteQuery)
    pub fn new<S>(measurement: S) -> Self
    where
        S: ToString,
    {
        InfluxDbWriteQuery {
            fields: vec![],
            tags: vec![],
            measurement: measurement.to_string(),
            // precision: Precision
        }
    }

    /// Adds a field to the [`InfluxDbWriteQuery`](crate::query::write_query::InfluxDbWriteQuery)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::query::InfluxDbQuery;
    ///
    /// InfluxDbQuery::write_query("measurement").add_field("field1", 5).build();
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
    /// use influxdb::query::InfluxDbQuery;
    ///
    /// InfluxDbQuery::write_query("measurement")
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

// todo: fuse_with(other: ValidQuery), so multiple queries can be run at the same time
impl InfluxDbQuery for InfluxDbWriteQuery {
    // todo: time (with precision)
    fn build(self) -> Result<ValidQuery, InfluxDbError> {
        if self.fields.is_empty() {
            return Err(InfluxDbError::InvalidQueryError {
                error: "fields cannot be empty".to_string(),
            });
        }

        let mut tags = self
            .tags
            .into_iter()
            .map(|(tag, value)| format!("{tag}={value}", tag = tag, value = value))
            .join(",");
        if !tags.is_empty() {
            tags.insert_str(0, ",");
        }
        let fields = self
            .fields
            .into_iter()
            .map(|(field, value)| format!("{field}={value}", field = field, value = value))
            .join(",");

        Ok(ValidQuery(format!(
            "{measurement}{tags} {fields}{time}",
            measurement = self.measurement,
            tags = tags,
            fields = fields,
            time = ""
        )))
    }

    fn get_type(&self) -> QueryType {
        QueryType::WriteQuery
    }
}
