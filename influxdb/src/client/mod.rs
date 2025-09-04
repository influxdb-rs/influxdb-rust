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

use futures_util::TryFutureExt;
use reqwest::{Client as HttpClient, RequestBuilder, Response as HttpResponse};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::sync::Arc;

use crate::query::QueryType;
use crate::Error;
use crate::Query;

/// Marker type for InfluxDB Version 1
#[derive(Clone)]
pub struct InfluxVersion1;
/// Marker type for InfluxDB Version 2
#[derive(Clone)]
pub struct InfluxVersion2;
/// Marker type for InfluxDB Version 3
#[derive(Clone)]
pub struct InfluxVersion3;

#[derive(Clone)]
/// Internal Representation of a Client
pub struct Client<V, H = reqwest::Client> {
    pub(crate) url: Arc<String>,
    pub(crate) parameters: Arc<HashMap<&'static str, String>>,
    pub(crate) token: Option<String>,
    pub(crate) client: H,
    _version: PhantomData<V>,
}

struct RedactPassword<'a>(&'a HashMap<&'static str, String>);

impl<'a> Debug for RedactPassword<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let entries = self
            .0
            .iter()
            .map(|(k, v)| match *k {
                "p" => (*k, "<redacted>"),
                _ => (*k, v.as_str()),
            })
            .collect::<BTreeMap<&'static str, &str>>();
        f.debug_map().entries(entries).finish()
    }
}

impl<V, H> Debug for Client<V, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("url", &self.url)
            .field("parameters", &RedactPassword(&self.parameters))
            .finish_non_exhaustive()
    }
}

impl<V> Client<V, reqwest::Client> {
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
    #[must_use = "Creating a client is pointless unless you use it"]
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
            client: HttpClient::new(),
            token: None,
            _version: PhantomData,
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
    #[must_use = "Creating a client is pointless unless you use it"]
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

    /// Replaces the HTTP Client
    #[must_use = "Creating a client is pointless unless you use it"]
    pub fn with_http_client(mut self, http_client: HttpClient) -> Self {
        self.client = http_client;
        self
    }

    /// Add authorization token to [`Client`](crate::Client)
    ///
    /// This is designed for influxdb 2.0's backward-compatible API which
    /// requires authorization by default. You can create such token from
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
                error: err.to_string(),
            })?;

        const BUILD_HEADER: &str = "X-Influxdb-Build";
        const VERSION_HEADER: &str = "X-Influxdb-Version";

        let (build, version) = {
            let hdrs = res.headers();
            (
                hdrs.get(BUILD_HEADER).and_then(|value| value.to_str().ok()),
                hdrs.get(VERSION_HEADER)
                    .and_then(|value| value.to_str().ok()),
            )
        };

        Ok((build.unwrap().to_owned(), version.unwrap().to_owned()))
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
    /// # #[tokio::main]
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
    /// let results = client.query(query).await?;
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
    pub async fn query<Q>(&self, q: Q) -> Result<String, Error>
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
        };

        let res = self
            .auth_if_needed(request_builder)
            .send()
            .map_err(|err| Error::ConnectionError {
                error: err.to_string(),
            })
            .await?;
        check_status(&res)?;

        let body = res.text();

        let s = body.await.map_err(|_| Error::DeserializationError {
            error: "response could not be converted to UTF-8".into(),
        })?;

        // todo: improve error parsing without serde
        if s.contains("\"error\"") || s.contains("\"Error\"") {
            return Err(Error::DatabaseError {
                error: format!("influxdb error: {s:?}"),
            });
        }

        Ok(s)
    }

    fn auth_if_needed(&self, rb: RequestBuilder) -> RequestBuilder {
        if let Some(ref token) = self.token {
            rb.header("Authorization", format!("Token {token}"))
        } else {
            rb
        }
    }
}

pub(crate) fn check_status(res: &HttpResponse) -> Result<(), Error> {
    let status = res.status();
    if !status.is_success() {
        return Err(Error::ApiError(status.into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::client::InfluxVersion1;

    use super::Client;
    use indoc::indoc;

    #[test]
    fn test_client_debug_redacted_password() {
        let client: Client<InfluxVersion1> =
            Client::new("https://localhost:8086", "db").with_auth("user", "pass");
        let actual = format!("{client:#?}");
        let expected = indoc! { r#"
            Client {
                url: "https://localhost:8086",
                parameters: {
                    "db": "db",
                    "p": "<redacted>",
                    "u": "user",
                },
                ..
            }
        "# };
        assert_eq!(actual.trim(), expected.trim());
    }

    #[test]
    fn test_fn_database() {
        let client: Client<InfluxVersion1> = Client::new("http://localhost:8068", "database");
        assert_eq!(client.database_name(), "database");
        assert_eq!(client.database_url(), "http://localhost:8068");
    }

    #[test]
    fn test_with_auth() {
        let client: Client<InfluxVersion1> = Client::new("http://localhost:8068", "database");
        assert_eq!(client.parameters.len(), 1);
        assert_eq!(client.parameters.get("db").unwrap(), "database");

        let with_auth = client.with_auth("username", "password");
        assert_eq!(with_auth.parameters.len(), 3);
        assert_eq!(with_auth.parameters.get("db").unwrap(), "database");
        assert_eq!(with_auth.parameters.get("u").unwrap(), "username");
        assert_eq!(with_auth.parameters.get("p").unwrap(), "password");

        let client: Client<InfluxVersion1> = Client::new("http://localhost:8068", "database");
        let with_auth = client.with_token("token");
        assert_eq!(with_auth.parameters.len(), 1);
        assert_eq!(with_auth.parameters.get("db").unwrap(), "database");
        assert_eq!(with_auth.token.unwrap(), "token");
    }
}
