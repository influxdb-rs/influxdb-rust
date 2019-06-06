use futures::{Future, Stream};
use reqwest::r#async::{Client, Decoder};

use std::mem;

use crate::error::InfluxDbError;
use crate::query::{InfluxDbQuery, QueryType};

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
/// use influxdb::client::InfluxDbClient;
///
/// let client = InfluxDbClient::new("http://localhost:8086", "test");
///
/// assert_eq!(client.database_name(), "test");
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
    /// use influxdb::client::InfluxDbClient;
    ///
    /// let _client = InfluxDbClient::new("http://localhost:8086", "test");
    /// ```
    pub fn new<S1, S2>(url: S1, database: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        InfluxDbClient {
            url: url.into(),
            database: database.into(),
        }
    }

    pub fn database_name<'a>(&'a self) -> &'a str {
        &self.database
    }

    pub fn database_url<'a>(&'a self) -> &'a str {
        &self.url
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

    pub fn query<Q>(&self, q: Q) -> Box<dyn Future<Item = String, Error = InfluxDbError>>
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
                    let body = mem::replace(res.body_mut(), Decoder::empty());
                    body.concat2()
                })
                .map_err(|err| InfluxDbError::UnspecifiedError {
                    error: format!("{}", err),
                })
                .and_then(|body| {
                    if let Ok(utf8) = std::str::from_utf8(&body) {
                        let mut s = String::new();
                        utf8.clone_into(&mut s);

                        // todo: improve error parsing without serde
                        if s.contains("\"error\"") {
                            return futures::future::err(InfluxDbError::UnspecifiedError {
                                error: format!("influxdb error: \"{}\"", s),
                            });
                        }

                        return futures::future::ok(s);
                    }

                    futures::future::err(InfluxDbError::UnspecifiedError {
                        error: "some other error has happened here!".to_string(),
                    })
                }),
        )
    }
}
