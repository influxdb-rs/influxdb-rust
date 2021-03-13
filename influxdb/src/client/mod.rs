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

use futures::prelude::*;
use surf::{self, Client as SurfClient, RequestBuilder, StatusCode};

use crate::query::QueryType;
use crate::Error;
use crate::Query;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
/// Internal Representation of a Client
pub struct Client {
    pub(crate) url: Arc<String>,
    pub(crate) parameters: Arc<HashMap<&'static str, String>>,
    pub(crate) client: SurfClient,
    pub(crate) token: Option<String>,
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
        let mut parameters = HashMap::<&str, String>::new();
        parameters.insert("db", database.into());
        Client {
            url: Arc::new(url.into()),
            parameters: Arc::new(parameters),
            client: SurfClient::new(),
            token: None,
        }
    }

    /// Add authentication/authorization information to [`Client`](crate::Client)
    ///
    /// # Arguments
    ///
    /// * username: The Username for InfluxDB.
    /// * password: The Password for the user.
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
        let mut with_auth = self.parameters.as_ref().clone();
        with_auth.insert("u", username.into());
        with_auth.insert("p", password.into());
        self.parameters = Arc::new(with_auth);
        self
    }

    /// Add authorization token to [`Client`](crate::Client)
    ///
    /// This is designed for influxdb 2.0's backward-compatible API which
    /// requires authrozation by default. You can create such token from
    /// console of influxdb 2.0 .
    pub fn with_token<S>(mut self, token: S) -> Self
    where
        S: Into<String>,
    {
        self.token = Some(token.into());
        self
    }

    /// Returns the name of the database the client is using
    pub fn database_name(&self) -> &str {
        // safe to unwrap: we always set the database name in `Self::new`
        self.parameters.get("db").unwrap()
    }

    /// Returns the URL of the InfluxDB installation the client is using
    pub fn database_url(&self) -> &str {
        &self.url
    }

    /// Pings the InfluxDB Server
    ///
    /// Returns a tuple of build type and version number
    pub async fn ping(&self) -> Result<(String, String), Error> {
        let url = &format!("{}/ping", self.url);
        let res = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|err| Error::ProtocolError {
                error: format!("{}", err),
            })?;

        let build = res.header("X-Influxdb-Build").unwrap().as_str();
        let version = res.header("X-Influxdb-Version").unwrap().as_str();

        Ok((build.to_owned(), version.to_owned()))
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
    /// ```rust,no_run
    /// use influxdb::{Client, Query, Timestamp};
    /// use influxdb::InfluxDbWriteable;
    /// use std::time::{SystemTime, UNIX_EPOCH};
    ///
    /// # #[async_std::main]
    /// # async fn main() -> Result<(), influxdb::Error> {
    /// let start = SystemTime::now();
    /// let since_the_epoch = start
    ///   .duration_since(UNIX_EPOCH)
    ///   .expect("Time went backwards")
    ///   .as_millis();
    ///
    /// let client = Client::new("http://localhost:8086", "test");
    /// let query = Timestamp::Milliseconds(since_the_epoch)
    ///     .into_query("weather")
    ///     .add_field("temperature", 82);
    /// let results = client.query(&query).await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// If the function can not finish the query,
    /// a [`Error`] variant will be returned.
    ///
    /// [`Error`]: enum.Error.html
    pub async fn query<'q, Q>(&self, q: &'q Q) -> Result<String, Error>
    where
        Q: Query,
    {
        let query = q.build().map_err(|err| Error::InvalidQueryError {
            error: err.to_string(),
        })?;

        let mut parameters = self.parameters.as_ref().clone();
        let request_builder = match q.get_type() {
            QueryType::ReadQuery => {
                let read_query = query.get();
                let url = &format!("{}/query", &self.url);
                parameters.insert("q", read_query.clone());

                if read_query.contains("SELECT") || read_query.contains("SHOW") {
                    self.client.get(url).query(&parameters)
                } else {
                    self.client.post(url).query(&parameters)
                }
            }
            QueryType::WriteQuery(precision) => {
                let url = &format!("{}/write", &self.url);
                let mut parameters = self.parameters.as_ref().clone();
                parameters.insert("precision", precision);

                self.client.post(url).body(query.get()).query(&parameters)
            }
        }
        .map_err(|err| Error::UrlConstructionError {
            error: err.to_string(),
        })?;

        let request = self.auth_if_needed(request_builder).build();
        let mut res = self
            .client
            .send(request)
            .map_err(|err| Error::ConnectionError {
                error: err.to_string(),
            })
            .await?;

        match res.status() {
            StatusCode::Unauthorized => return Err(Error::AuthorizationError),
            StatusCode::Forbidden => return Err(Error::AuthenticationError),
            _ => {}
        }

        let s = res
            .body_string()
            .await
            .map_err(|_| Error::DeserializationError {
                error: "response could not be converted to UTF-8".to_string(),
            })?;

        // todo: improve error parsing without serde
        if s.contains("\"error\"") {
            return Err(Error::DatabaseError {
                error: format!("influxdb error: \"{}\"", s),
            });
        }

        Ok(s)
    }

    fn auth_if_needed(&self, rb: RequestBuilder) -> RequestBuilder {
        if let Some(ref token) = self.token {
            rb.header("Authorization", format!("Token {}", token))
        } else {
            rb
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Client;

    #[test]
    fn test_fn_database() {
        let client = Client::new("http://localhost:8068", "database");
        assert_eq!(client.database_name(), "database");
        assert_eq!(client.database_url(), "http://localhost:8068");
    }

    #[test]
    fn test_with_auth() {
        let client = Client::new("http://localhost:8068", "database");
        assert_eq!(client.parameters.len(), 1);
        assert_eq!(client.parameters.get("db").unwrap(), "database");

        let with_auth = client.with_auth("username", "password");
        assert_eq!(with_auth.parameters.len(), 3);
        assert_eq!(with_auth.parameters.get("db").unwrap(), "database");
        assert_eq!(with_auth.parameters.get("u").unwrap(), "username");
        assert_eq!(with_auth.parameters.get("p").unwrap(), "password");

        let client = Client::new("http://localhost:8068", "database");
        let with_auth = client.with_token("token");
        assert_eq!(with_auth.parameters.len(), 1);
        assert_eq!(with_auth.parameters.get("db").unwrap(), "database");
        assert_eq!(with_auth.token.unwrap(), "token");
    }
}
