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

use http::StatusCode;
use isahc::prelude::*;
use url::Url;

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
    pub async fn ping(&self) -> Result<(String, String), Error> {
        let res = isahc::get_async(format!("{}/ping", self.url).as_str())
            .await
            .map_err(|err| Error::ProtocolError {
                error: format!("{}", err),
            })?;

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
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), failure::Error> {
    /// let client = Client::new("http://localhost:8086", "test");
    /// let query = Timestamp::Now
    ///     .into_query("weather")
    ///     .add_field("temperature", 82);
    /// let results = client.query(&query).await?;
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
        &'q Q: Into<QueryTypes<'q>>,
    {
        let query = q.build().map_err(|err| Error::InvalidQueryError {
            error: format!("{}", err),
        })?;

        let basic_parameters: Vec<(String, String)> = self.into();

        let res = match q.into() {
            QueryTypes::Read(_) => {
                let read_query = query.get();
                let mut url = Url::parse_with_params(
                    format!("{url}/query", url = self.database_url()).as_str(),
                    basic_parameters,
                )
                .map_err(|err| Error::UrlConstructionError {
                    error: format!("{}", err),
                })?;

                url.query_pairs_mut().append_pair("q", &read_query);

                if read_query.contains("SELECT") || read_query.contains("SHOW") {
                    isahc::get_async(url.as_str()).await
                } else {
                    isahc::post_async(url.as_str().to_owned(), "").await
                }
            }
            QueryTypes::Write(write_query) => {
                let mut url = Url::parse_with_params(
                    format!("{url}/write", url = self.database_url()).as_str(),
                    basic_parameters,
                )
                .map_err(|err| Error::InvalidQueryError {
                    error: format!("{}", err),
                })?;

                url.query_pairs_mut()
                    .append_pair("precision", &write_query.get_precision());

                isahc::post_async(url.as_str().to_owned(), query.get()).await
            }
        };

        let mut res = res.map_err(|err| Error::ConnectionError { error: err })?;

        match res.status() {
            StatusCode::UNAUTHORIZED => return Err(Error::AuthorizationError),
            StatusCode::FORBIDDEN => return Err(Error::AuthenticationError),
            _ => {}
        }

        let s = res
            .text_async()
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
