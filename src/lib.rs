#![allow(dead_code)]

#[macro_use]
extern crate failure;

use futures::Future;
use itertools::Itertools;
use reqwest::r#async::Client;

#[derive(Debug, Fail)]
/// Errors that might happen in the crate
pub enum InfluxDbError {
    #[fail(display = "query must contain at least one field")]
    /// Error happens when query has zero fields
    InvalidQueryError,
}

#[derive(Debug)]
#[doc(hidden)]
pub struct ValidQuery(String);
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

/// Used to create read or [`InfluxDbWriteQuery`] queries to InfluxDB
///
/// # Examples
///
/// ```rust
/// use influxdb::InfluxDbQuery;
///
/// let write_query = InfluxDbQuery::write_query("measurement")
///     .add_field("field1", "5")
///     .add_tag("tag1", "Gero")
///     .build();
///
/// assert!(query.is_ok());
///
/// //todo: document read query once it's implemented.
/// ```
pub trait InfluxDbQuery {
    /// Builds valid InfluxSQL which can be run against the Database.
    /// In case no fields have been specified, it will return an error,
    /// as that is invalid InfluxSQL syntax.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::InfluxDbQuery;
    ///
    /// let invalid_query = InfluxDbQuery::write_query("measurement").build();
    /// assert!(query.is_err());
    ///
    /// let valid_query = InfluxDbQuery::write_query("measurement").add_field("myfield1", "11").build();
    /// assert!(query.is_ok());
    /// ```
    fn build<'a>(self) -> Result<ValidQuery, InfluxDbError>;
}

impl InfluxDbQuery {
    /// Returns a [`InfluxDbWriteQuery`] builder.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::InfluxDbQuery;
    ///
    /// InfluxDbQuery::write_query("measurement"); // Is of type [`InfluxDbWriteQuery`]
    /// ```
    pub fn write_query<S>(measurement: S) -> InfluxDbWriteQuery
    where
        S: Into<String>,
    {
        InfluxDbWriteQuery {
            measurement: measurement.into(),
            fields: Vec::new(),
            tags: Vec::new(),
        }
    }

    // pub fn read() {}
}

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
    /// Adds a field to the [`InfluxDbWriteQuery`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::InfluxDbQuery;
    ///
    /// InfluxDbQuery::write_query("measurement").add_field("field1", "5").build();
    /// ```
    pub fn add_field<'a, S>(mut self, point: S, value: S) -> Self
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
    /// use influxdb::InfluxDbQuery;
    ///
    /// InfluxDbQuery::write_query("measurement")
    ///     .add_tag("field1", "5"); // calling `.build()` now would result in a `Err(InfluxDbError::InvalidQueryError)`
    /// ```
    pub fn add_tag<'a, S>(mut self, tag: S, value: S) -> Self
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
    fn build<'a>(self) -> Result<ValidQuery, InfluxDbError> {
        if self.fields.is_empty() {
            return Err(InfluxDbError::InvalidQueryError);
        }

        let tags = self
            .tags
            .into_iter()
            .map(|(tag, value)| format!("{tag}={value}", tag = tag, value = value))
            .join(",")
            + " ";
        let fields = self
            .fields
            .into_iter()
            .map(|(field, value)| format!("{field}={value}", field = field, value = value))
            .join(",")
            + " ";

        Ok(ValidQuery(format!(
            "{measurement},{tags}{fields}time",
            measurement = self.measurement,
            tags = tags,
            fields = fields
        )))
    }
}

/// Client which can read and write data from InfluxDB.
///
/// # Arguments
///
///  * `url`: The URL where InfluxDB is running (ex. `http://localhost:8086`).
///  * `database`: The Database against which queries and writes will be run.
///
/// # Examples
///
/// ```rust
/// use influxdb::InfluxDbClient;
///
/// let client = InfluxDbClient::new("http://localhost:8086", "test");
///
/// assert_eq!(client.get_database_name(), "test");
/// ```
pub struct InfluxDbClient {
    url: String,
    database: String,
    // auth: Option<InfluxDbAuthentication>
}

impl InfluxDbClient {
    /// Instantiates a new [`InfluxDbClient`]
    ///
    /// # Arguments
    ///
    ///  * `url`: The URL where InfluxDB is running (ex. `http://localhost:8086`).
    ///  * `database`: The Database against which queries and writes will be run.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::InfluxDbClient;
    ///
    /// let _client = InfluxDbClient::new("http://localhost:8086", "test");
    /// ```
    pub fn new<S>(url: S, database: S) -> Self
    where
        S: Into<String>,
    {
        InfluxDbClient {
            url: url.into(),
            database: database.into(),
        }
    }

    pub fn get_database_name(self) -> String {
        self.database
    }

    pub fn get_database_url(self) -> String {
        self.url
    }

    pub fn ping(&self) -> impl Future<Item = (String, String), Error = ()> {
        Client::new()
            .get(format!("{}/ping", self.url).as_str())
            .send()
            .map(|res| {
                let build = res
                    .headers()
                    .get("X-Influxdb-Build")
                    .unwrap()
                    .to_str()
                    .unwrap();
                let version = res
                    .headers()
                    .get("X-Influxdb-Version")
                    .unwrap()
                    .to_str()
                    .unwrap();

                (String::from(build), String::from(version))
            })
            .map_err(|err| println!("request error: {}", err))
    }
}

pub fn main() {}

#[cfg(test)]
mod tests {
    use super::{InfluxDbClient, InfluxDbQuery};
    use tokio::runtime::current_thread::Runtime;

    fn get_runtime() -> Runtime {
        Runtime::new().expect("Unable to create a runtime")
    }

    fn create_client() -> InfluxDbClient {
        InfluxDbClient::new("http://localhost:8086", "test")
    }

    #[test]
    fn test_ping() {
        let client = create_client();
        let result = get_runtime().block_on(client.ping());
        assert!(result.is_ok(), "Should be no error");

        let (build, version) = result.unwrap();
        assert!(!build.is_empty(), "Build should not be empty");
        assert!(!version.is_empty(), "Build should not be empty");

        println!("build: {} version: {}", build, version);
    }

    #[test]
    fn test_write_builder_empty_query() {
        let query = InfluxDbQuery::write_query("marina_3").build();

        assert!(query.is_err(), "Query was not empty");
    }

    #[test]
    fn test_write_builder_single_field() {
        let query = InfluxDbQuery::write_query("marina_3")
            .add_field("water_level", "2")
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(query.unwrap(), "marina_3, water_level=2 time");
    }

    #[test]
    fn test_write_builder_multiple_fields() {
        let query = InfluxDbQuery::write_query("marina_3")
            .add_field("water_level", "2")
            .add_field("boat_count", "31")
            .add_field("algae_content", "0.85")
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(
            query.unwrap(),
            "marina_3, water_level=2,boat_count=31,algae_content=0.85 time"
        );
    }

    // fixme: quoting / escaping of long strings
    #[test]
    fn test_write_builder_only_tags() {
        let query = InfluxDbQuery::write_query("marina_3")
            .add_tag("marina_manager", "Smith")
            .build();

        assert!(query.is_err(), "Query missing one or more fields");
    }

    #[test]
    fn test_write_builder_full_query() {
        let query = InfluxDbQuery::write_query("marina_3")
            .add_field("water_level", "2")
            .add_field("boat_count", "31")
            .add_field("algae_content", "0.85")
            .add_tag("marina_manager", "Smith")
            .add_tag("manager_to_the_marina_manager", "Jonson")
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(
            query.unwrap(),
            "marina_3,marina_manager=Smith,manager_to_the_marina_manager=Jonson water_level=2,boat_count=31,algae_content=0.85 time"
        );
    }
}
