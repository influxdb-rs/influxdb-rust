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
//! use influxdb::Client;
//!
//! let client = Client::new("http://localhost:8086", "test");
//!
//! assert_eq!(client.database_name(), "test");
//! ```

use futures::{Future, Stream};
use reqwest::r#async::{Client as ReqwestClient, Decoder};
use reqwest::{StatusCode, Url};

use std::mem;

use crate::query::QueryTypes;
use crate::Error;
use crate::Query;

#[derive(Clone, Debug)]
/// Internal Authentication representation
pub(crate) struct Authentication {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug)]
/// Internal Representation of a Client
pub struct Client {
    url: String,
    database: String,
    auth: Option<Authentication>,
}

impl Into<Vec<(String, String)>> for Client {
    fn into(self) -> Vec<(String, String)> {
        let mut vec: Vec<(String, String)> = Vec::new();
        vec.push(("db".to_string(), self.database));
        if let Some(auth) = self.auth {
            vec.push(("u".to_string(), auth.username));
            vec.push(("p".to_string(), auth.password));
        }
        vec
    }
}

impl<'a> Into<Vec<(String, String)>> for &'a Client {
    fn into(self) -> Vec<(String, String)> {
        let mut vec: Vec<(String, String)> = Vec::new();
        vec.push(("db".to_string(), self.database.to_owned()));
        if let Some(auth) = &self.auth {
            vec.push(("u".to_string(), auth.username.to_owned()));
            vec.push(("p".to_string(), auth.password.to_owned()));
        }
        vec
    }
}

impl Client {
    /// Instantiates a new [`Client`](crate::Client)
    ///
    /// # Arguments
    ///
    ///  * `url`: The URL where InfluxDB is running (ex. `http://localhost:8086`).
    ///  * `database`: The Database against which queries and writes will be run.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::Client;
    ///
    /// let _client = Client::new("http://localhost:8086", "test");
    /// ```
    pub fn new<S1, S2>(url: S1, database: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Client {
            url: url.into(),
            database: database.into(),
            auth: None,
        }
    }

    /// Add authentication/authorization information to [`Client`](crate::Client)
    ///
    /// # Arguments
    ///
    /// * username: The Username for InfluxDB.
    /// * password: THe Password for the user.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::Client;
    ///
    /// let _client = Client::new("http://localhost:9086", "test").with_auth("admin", "password");
    /// ```
    pub fn with_auth<S1, S2>(mut self, username: S1, password: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.auth = Some(Authentication {
            username: username.into(),
            password: password.into(),
        });
        self
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
    pub fn ping(&self) -> impl Future<Item = (String, String), Error = Error> {
        ReqwestClient::new()
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
            .map_err(|err| Error::ProtocolError {
                error: format!("{}", err),
            })
    }

    /// Sends a [`ReadQuery`](crate::ReadQuery) or [`WriteQuery`](crate::WriteQuery) to the InfluxDB Server.
    ///
    /// A version capable of parsing the returned string is available under the [serde_integration](crate::integrations::serde_integration)
    ///
    /// # Arguments
    ///
    ///  * `q`: Query of type [`ReadQuery`](crate::ReadQuery) or [`WriteQuery`](crate::WriteQuery)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use influxdb::{Client, Query, Timestamp};
    ///
    /// let client = Client::new("http://localhost:8086", "test");
    /// let _future = client.query(
    ///     &Query::write_query(Timestamp::NOW, "weather")
    ///         .add_field("temperature", 82)
    /// );
    /// ```
    /// # Errors
    ///
    /// If the function can not finish the query,
    /// a [`Error`] variant will be returned.
    ///
    /// [`Error`]: enum.Error.html
    pub fn query<'q, Q>(&self, q: &'q Q) -> Box<dyn Future<Item = String, Error = Error>>
    where
        Q: Query,
        &'q Q: Into<QueryTypes<'q>>,
    {
        use futures::future;

        let query = match q.build() {
            Err(err) => {
                let error = Error::InvalidQueryError {
                    error: format!("{}", err),
                };
                return Box::new(future::err::<String, Error>(error));
            }
            Ok(query) => query,
        };

        let basic_parameters: Vec<(String, String)> = self.into();

        let client = match q.into() {
            QueryTypes::Read(_) => {
                let read_query = query.get();
                let mut url = match Url::parse_with_params(
                    format!("{url}/query", url = self.database_url()).as_str(),
                    basic_parameters,
                ) {
                    Ok(url) => url,
                    Err(err) => {
                        let error = Error::UrlConstructionError {
                            error: format!("{}", err),
                        };
                        return Box::new(future::err::<String, Error>(error));
                    }
                };
                url.query_pairs_mut().append_pair("q", &read_query.clone());

                if read_query.contains("SELECT") || read_query.contains("SHOW") {
                    ReqwestClient::new().get(url)
                } else {
                    ReqwestClient::new().post(url)
                }
            }
            QueryTypes::Write(write_query) => {
                let mut url = match Url::parse_with_params(
                    format!("{url}/write", url = self.database_url()).as_str(),
                    basic_parameters,
                ) {
                    Ok(url) => url,
                    Err(err) => {
                        let error = Error::InvalidQueryError {
                            error: format!("{}", err),
                        };
                        return Box::new(future::err::<String, Error>(error));
                    }
                };
                url.query_pairs_mut()
                    .append_pair("precision", &write_query.get_precision());
                ReqwestClient::new().post(url).body(query.get())
            }
        };
        Box::new(
            client
                .send()
                .map_err(|err| Error::ConnectionError { error: err })
                .and_then(
                    |res| -> future::FutureResult<reqwest::r#async::Response, Error> {
                        match res.status() {
                            StatusCode::UNAUTHORIZED => {
                                futures::future::err(Error::AuthorizationError)
                            }
                            StatusCode::FORBIDDEN => {
                                futures::future::err(Error::AuthenticationError)
                            }
                            _ => futures::future::ok(res),
                        }
                    },
                )
                .and_then(|mut res| {
                    let body = mem::replace(res.body_mut(), Decoder::empty());
                    body.concat2().map_err(|err| Error::ProtocolError {
                        error: format!("{}", err),
                    })
                })
                .and_then(|body| {
                    if let Ok(utf8) = std::str::from_utf8(&body) {
                        let s = utf8.to_owned();

                        // todo: improve error parsing without serde
                        if s.contains("\"error\"") {
                            return futures::future::err(Error::DatabaseError {
                                error: format!("influxdb error: \"{}\"", s),
                            });
                        }

                        return futures::future::ok(s);
                    }

                    futures::future::err(Error::DeserializationError {
                        error: "response could not be converted to UTF-8".to_string(),
                    })
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::Client;

    #[test]
    fn test_fn_database() {
        let client = Client::new("http://localhost:8068", "database");
        assert_eq!("database", client.database_name());
    }

    #[test]
    fn test_with_auth() {
        let client = Client::new("http://localhost:8068", "database");
        assert_eq!(client.url, "http://localhost:8068");
        assert_eq!(client.database, "database");
        assert!(client.auth.is_none());
        let with_auth = client.with_auth("username", "password");
        assert!(with_auth.auth.is_some());
        let auth = with_auth.auth.unwrap();
        assert_eq!(&auth.username, "username");
        assert_eq!(&auth.password, "password");
    }

    #[test]
    fn test_into_impl() {
        let client = Client::new("http://localhost:8068", "database");
        assert!(client.auth.is_none());
        let basic_parameters: Vec<(String, String)> = client.into();
        assert_eq!(
            vec![("db".to_string(), "database".to_string())],
            basic_parameters
        );

        let with_auth =
            Client::new("http://localhost:8068", "database").with_auth("username", "password");
        let basic_parameters_with_auth: Vec<(String, String)> = with_auth.into();
        assert_eq!(
            vec![
                ("db".to_string(), "database".to_string()),
                ("u".to_string(), "username".to_string()),
                ("p".to_string(), "password".to_string())
            ],
            basic_parameters_with_auth
        );

        let client = Client::new("http://localhost:8068", "database");
        assert!(client.auth.is_none());
        let basic_parameters: Vec<(String, String)> = (&client).into();
        assert_eq!(
            vec![("db".to_string(), "database".to_string())],
            basic_parameters
        );

        let with_auth =
            Client::new("http://localhost:8068", "database").with_auth("username", "password");
        let basic_parameters_with_auth: Vec<(String, String)> = (&with_auth).into();
        assert_eq!(
            vec![
                ("db".to_string(), "database".to_string()),
                ("u".to_string(), "username".to_string()),
                ("p".to_string(), "password".to_string())
            ],
            basic_parameters_with_auth
        );
    }
}
