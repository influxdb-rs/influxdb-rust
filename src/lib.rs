#![allow(dead_code)]
#![feature(toowned_clone_into)]

#[macro_use]
extern crate failure;

// use itertools::Itertools;
// use std::mem;
// use std::io::{self, Cursor};
// use futures::{Future, Stream};
// use reqwest::r#async::{Client, Decoder};

use futures::{Future, Stream};
use itertools::Itertools;
use reqwest::r#async::{Client, Decoder};
use serde::de::DeserializeOwned;

use serde::{Deserialize, Serialize};
use serde_json;
use std::mem;

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

#[derive(Deserialize)]
struct _DatabaseError {
    error: String,
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
/// assert!(write_query.is_ok());
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
    /// assert!(invalid_query.is_err());
    ///
    /// let valid_query = InfluxDbQuery::write_query("measurement").add_field("myfield1", "11").build();
    /// assert!(valid_query.is_ok());
    /// ```
    fn build<'a>(self) -> Result<ValidQuery, InfluxDbError>;

    fn get_type(&self) -> QueryType;
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

    pub fn raw_read_query<S>(read_query: S) -> InfluxDbReadQuery
    where
        S: Into<String>,
    {
        InfluxDbReadQuery {
            query: read_query.into(),
        }
    }
}

pub enum QueryType {
    ReadQuery,
    WriteQuery,
}

// todo: orm for query
pub struct InfluxDbReadQuery {
    query: String,
}

impl InfluxDbQuery for InfluxDbReadQuery {
    fn build<'a>(self) -> Result<ValidQuery, InfluxDbError> {
        Ok(ValidQuery(self.query))
    }

    fn get_type(&self) -> QueryType {
        QueryType::ReadQuery
    }
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
    // Influx Line Protocol
    // weather,location=us-midwest temperature=82 1465839830100400200
    // |    -------------------- --------------  |
    // |             |             |             |
    // |             |             |             |
    // +-----------+--------+-+---------+-+---------+
    // |measurement|,tag_set| |field_set| |timestamp|
    // +-----------+--------+-+---------+-+---------+
    fn build<'a>(self) -> Result<ValidQuery, InfluxDbError> {
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

    pub fn ping(&self) -> impl Future<Item = (String, String), Error = InfluxDbError> {
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
            .map_err(|err| InfluxDbError::UnspecifiedError {
                error: format!("{}", err),
            })
    }

    pub fn json_query<T: 'static, Q>(self, q: Q) -> Box<dyn Future<Item = T, Error = InfluxDbError>>
    where
        Q: InfluxDbQuery,
        T: DeserializeOwned,
    {
        use futures::future;

        let query_type = q.get_type();
        let endpoint = match query_type {
            QueryType::ReadQuery => "query",
            QueryType::WriteQuery => "write",
        };

        let query = match q.build() {
            Err(err) => {
                let error = InfluxDbError::UnspecifiedError {
                    error: format!("{}", err),
                };
                return Box::new(future::err::<T, InfluxDbError>(error));
            }
            Ok(query) => query,
        };

        let query_str = query.get();
        let url_params = match query_type {
            QueryType::ReadQuery => format!("&q={}", query_str),
            QueryType::WriteQuery => String::from(""),
        };

        println!(
            "{url}/{endpoint}?db={db}{url_params}",
            url = self.url,
            endpoint = endpoint,
            db = self.database,
            url_params = url_params
        );

        let client = match query_type {
            QueryType::ReadQuery => Client::new().get(
                format!(
                    "{url}/{endpoint}?db={db}{url_params}",
                    url = self.url,
                    endpoint = endpoint,
                    db = self.database,
                    url_params = url_params
                )
                .as_str(),
            ),
            QueryType::WriteQuery => Client::new()
                .post(
                    format!(
                        "{url}/{endpoint}?db={db}",
                        url = self.url,
                        endpoint = endpoint,
                        db = self.database,
                    )
                    .as_str(),
                )
                .body(query_str),
        };

        Box::new(
            client
                .send()
                .and_then(|mut res| {
                    println!("{}", res.status());

                    let body = mem::replace(res.body_mut(), Decoder::empty());
                    body.concat2()
                })
                .map_err(|err| InfluxDbError::UnspecifiedError {
                    error: format!("{}", err)
                })
                .and_then(|body| {
                    // Try parsing InfluxDBs { "error": "error message here" }
                    if let Ok(error) = serde_json::from_slice::<_DatabaseError>(&body) {
                        return futures::future::err(InfluxDbError::DatabaseError {
                            error: format!("{}", error.error)
                        })
                    } else if let Ok(t_result) = serde_json::from_slice::<T>(&body) {
                        // Json has another structure, let's try actually parsing it to the type we're deserializing to
                        return futures::future::result(Ok(t_result));
                    } else {
                        return futures::future::err(InfluxDbError::UnspecifiedError {
                            error: String::from("something wen't wrong during deserializsation of the database response. this might be a bug!")
                        })
                    }
                })
        )
    }

    pub fn query<Q>(self, q: Q) -> Box<dyn Future<Item = String, Error = InfluxDbError>>
    where
        Q: InfluxDbQuery,
    {
        use futures::future;

        let query_type = q.get_type();
        let endpoint = match query_type {
            QueryType::ReadQuery => "query",
            QueryType::WriteQuery => "write",
        };

        let query = match q.build() {
            Err(err) => {
                let error = InfluxDbError::UnspecifiedError {
                    error: format!("{}", err),
                };
                return Box::new(future::err::<String, InfluxDbError>(error));
            }
            Ok(query) => query,
        };

        let query_str = query.get();
        let url_params = match query_type {
            QueryType::ReadQuery => format!("&q={}", query_str),
            QueryType::WriteQuery => String::from(""),
        };

        println!(
            "{url}/{endpoint}?db={db}{url_params}",
            url = self.url,
            endpoint = endpoint,
            db = self.database,
            url_params = url_params
        );

        let client = match query_type {
            QueryType::ReadQuery => Client::new().get(
                format!(
                    "{url}/{endpoint}?db={db}{url_params}",
                    url = self.url,
                    endpoint = endpoint,
                    db = self.database,
                    url_params = url_params
                )
                .as_str(),
            ),
            QueryType::WriteQuery => Client::new()
                .post(
                    format!(
                        "{url}/{endpoint}?db={db}",
                        url = self.url,
                        endpoint = endpoint,
                        db = self.database,
                    )
                    .as_str(),
                )
                .body(query_str),
        };

        Box::new(
            client
                .send()
                .and_then(|mut res| {
                    println!("{}", res.status());

                    let body = mem::replace(res.body_mut(), Decoder::empty());
                    body.concat2()
                })
                .map_err(|err| InfluxDbError::UnspecifiedError {
                    error: format!("{}", err),
                })
                .and_then(|body| {
                    // Try parsing InfluxDBs { "error": "error message here" }
                    if let Ok(error) = serde_json::from_slice::<_DatabaseError>(&body) {
                        return futures::future::err(InfluxDbError::DatabaseError {
                            error: format!("{}", error.error),
                        });
                    }

                    if let Ok(utf8) = std::str::from_utf8(&body) {
                        let mut s = String::new();
                        utf8.clone_into(&mut s);
                        return futures::future::ok(s);
                    }

                    return futures::future::err(InfluxDbError::UnspecifiedError {
                        error: format!("{}", "some other error has happened here!"),
                    });
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::InfluxDbQuery;

    #[test]
    fn test_write_builder_empty_query() {
        let query = InfluxDbQuery::write_query("marina_3").build();

        assert!(query.is_err(), "Query was not empty");
    }

    #[test]
    fn test_write_builder_single_field() {
        let query = InfluxDbQuery::write_query("weather")
            .add_field("temperature", "82")
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(query.unwrap(), "weather temperature=82");
    }

    #[test]
    fn test_write_builder_multiple_fields() {
        let query = InfluxDbQuery::write_query("weather")
            .add_field("temperature", "82")
            .add_field("wind_strength", "3.7")
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(query.unwrap(), "weather temperature=82,wind_strength=3.7");
    }

    // fixme: quoting / escaping of long strings
    #[test]
    fn test_write_builder_only_tags() {
        let query = InfluxDbQuery::write_query("weather")
            .add_tag("season", "summer")
            .build();

        assert!(query.is_err(), "Query missing one or more fields");
    }

    #[test]
    fn test_write_builder_full_query() {
        let query = InfluxDbQuery::write_query("weather")
            .add_field("temperature", "82")
            .add_tag("location", "us-midwest")
            .add_tag("season", "summer")
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(
            query.unwrap(),
            "weather,location=us-midwest,season=summer temperature=82"
        );
    }
}
