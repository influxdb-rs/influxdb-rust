//! Client which can read and write data from InfluxDB.
//!
//! # Arguments
//!
//!  * `url`: The URL where InfluxDB is running (ex. `http://localhost:8086`).
//!  * `database`: The Database against which queries and writes will be run.
//!
//! # Examples
//!
//! ```rust
//! use influxdb::client::InfluxDbClient;
//!
//! let client = InfluxDbClient::new("http://localhost:8086", "test");
//!
//! assert_eq!(client.database_name(), "test");
//! ```

use futures::{Future, Stream};
use reqwest::r#async::{Client, Decoder};

use std::mem;

use crate::error::InfluxDbError;
use crate::query::read_query::InfluxDbReadQuery;
use crate::query::write_query::InfluxDbWriteQuery;
use crate::query::InfluxDbQuery;

use url::form_urlencoded;

use std::any::Any;

/// Internal Representation of a Client
pub struct InfluxDbClient {
    url: String,
    database: String,
    // auth: Option<InfluxDbAuthentication>
}

impl InfluxDbClient {
    /// Instantiates a new [`InfluxDbClient`](crate::client::InfluxDbClient)
    ///
    /// # Arguments
    ///
    ///  * `url`: The URL where InfluxDB is running (ex. `http://localhost:8086`).
    ///  * `database`: The Database against which queries and writes will be run.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::client::InfluxDbClient;
    ///
    /// let _client = InfluxDbClient::new("http://localhost:8086", "test");
    /// ```
    pub fn new<S1, S2>(url: S1, database: S2) -> Self
    where
        S1: ToString,
        S2: ToString,
    {
        InfluxDbClient {
            url: url.to_string(),
            database: database.to_string(),
        }
    }

    /// Returns the name of the database the client is using
    pub fn database_name(&self) -> &str {
        &self.database
    }

    /// Returns the URL of the InfluxDB installation the client is using
    pub fn database_url(&self) -> &str {
        &self.url
    }

    /// Pings the InfluxDB Server
    ///
    /// Returns a tuple of build type and version number
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
            .map_err(|err| InfluxDbError::ProtocolError {
                error: format!("{}", err),
            })
    }

    /// Sends a [`InfluxDbReadQuery`](crate::query::read_query::InfluxDbReadQuery) or [`InfluxDbWriteQuery`](crate::query::write_query::InfluxDbWriteQuery) to the InfluxDB Server.InfluxDbError
    ///
    /// A version capable of parsing the returned string is available under the [serde_integration](crate::integrations::serde_integration)
    ///
    /// # Arguments
    ///
    ///  * `q`: Query of type [`InfluxDbReadQuery`](crate::query::read_query::InfluxDbReadQuery) or [`InfluxDbWriteQuery`](crate::query::write_query::InfluxDbWriteQuery)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::client::InfluxDbClient;
    /// use influxdb::query::{InfluxDbQuery, Timestamp};
    ///
    /// let client = InfluxDbClient::new("http://localhost:8086", "test");
    /// let _future = client.query(
    ///     &InfluxDbQuery::write_query(Timestamp::NOW, "weather")
    ///         .add_field("temperature", 82)
    /// );
    /// ```
    pub fn query<Q>(&self, q: &Q) -> Box<dyn Future<Item = String, Error = InfluxDbError>>
    where
        Q: Any + InfluxDbQuery,
    {
        use futures::future;

        let query = match q.build() {
            Err(err) => {
                let error = InfluxDbError::InvalidQueryError {
                    error: format!("{}", err),
                };
                return Box::new(future::err::<String, InfluxDbError>(error));
            }
            Ok(query) => query,
        };

        let any_value = q as &dyn Any;

        let client = if let Some(_) = any_value.downcast_ref::<InfluxDbReadQuery>() {
            let read_query = query.get();
            let encoded: String = form_urlencoded::Serializer::new(String::new())
                .append_pair("db", self.database_name())
                .append_pair("q", &read_query)
                .finish();
            let http_query_string = format!(
                "{url}/query?{encoded}",
                url = self.database_url(),
                encoded = encoded
            );
            if read_query.contains("SELECT") || read_query.contains("SHOW") {
                Client::new().get(http_query_string.as_str())
            } else {
                Client::new().post(http_query_string.as_str())
            }
        } else if let Some(write_query) = any_value.downcast_ref::<InfluxDbWriteQuery>() {
            Client::new()
                .post(
                    format!(
                        "{url}/write?db={db}{precision_str}",
                        url = self.database_url(),
                        db = self.database_name(),
                        precision_str = write_query.get_precision_modifier()
                    )
                    .as_str(),
                )
                .body(query.get())
        } else {
            unreachable!()
        };

        Box::new(
            client
                .send()
                .and_then(|mut res| {
                    let body = mem::replace(res.body_mut(), Decoder::empty());
                    body.concat2()
                })
                .map_err(|err| InfluxDbError::ProtocolError {
                    error: format!("{}", err),
                })
                .and_then(|body| {
                    if let Ok(utf8) = std::str::from_utf8(&body) {
                        let s = utf8.to_owned();

                        // todo: improve error parsing without serde
                        if s.contains("\"error\"") {
                            return futures::future::err(InfluxDbError::DatabaseError {
                                error: format!("influxdb error: \"{}\"", s),
                            });
                        }

                        return futures::future::ok(s);
                    }

                    futures::future::err(InfluxDbError::DeserializationError {
                        error: "response could not be converted to UTF-8".to_string(),
                    })
                }),
        )
    }
}
