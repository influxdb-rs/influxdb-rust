
use crate::error::InfluxDbError;
use crate::query::{InfluxDbQuery, QueryType, ValidQuery};
use itertools::Itertools;

/// Write Query Builder returned by [InfluxDbQuery::write_query]()
///
/// Can only be instantiated by using [InfluxDbQuery::write_query]()
pub struct InfluxDbWriteQuery {
    fields: Vec<(String, String)>,
    tags: Vec<(String, String)>,
    measurement: String,
    // precision: Precision
}

impl InfluxDbWriteQuery {
    pub fn new<S>(measurement: S) -> Self
    where
        S: Into<String>,
    {
        InfluxDbWriteQuery {
            fields: vec![],
            tags: vec![],
            measurement: measurement.into(),
            // precision: Precision
        }
    }

    /// Adds a field to the [`InfluxDbWriteQuery`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::query::InfluxDbQuery;
    ///
    /// InfluxDbQuery::write_query("measurement").add_field("field1", "5").build();
    /// ```
    pub fn add_field<S>(mut self, point: S, value: S) -> Self
    where
        S: Into<String>,
    {
        self.fields.push((point.into(), value.into()));
        self
    }

    /// Adds a tag to the [`InfluxDbWriteQuery`]
    ///
    /// Please note that a [`InfluxDbWriteQuery`] requires at least one field. Composing a query with
    /// only tags will result in a failure building the query.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::query::InfluxDbQuery;
    ///
    /// InfluxDbQuery::write_query("measurement")
    ///     .add_tag("field1", "5"); // calling `.build()` now would result in a `Err(InfluxDbError::InvalidQueryError)`
    /// ```
    pub fn add_tag<S>(mut self, tag: S, value: S) -> Self
    where
        S: Into<String>,
    {
        self.tags.push((tag.into(), value.into()));
        self
    }
}

// todo: fuse_with(other: ValidQuery), so multiple queries can be run at the same time
impl InfluxDbQuery for InfluxDbWriteQuery {
    // todo: time (with precision)
    fn build(self) -> Result<ValidQuery, InfluxDbError> {
        if self.fields.is_empty() {
            return Err(InfluxDbError::InvalidQueryError);
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