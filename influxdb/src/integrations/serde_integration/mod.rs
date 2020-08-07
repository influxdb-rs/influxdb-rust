//! Serde Integration for InfluxDB. Provides deserialization of query returns.
//!
//! When querying multiple series in the same query (e.g. with a regex query), it might be desirable to flat map
//! the resulting series into a single `Vec` like so. The example assumes, that there are weather readings in multiple
//! series named `weather_<city_name>` (e.g. `weather_berlin`, or `weather_london`). Since we're using a Regex query,
//! we don't actually know which series will be returned. To assign the city name to the series, we can use the series
//! `name`, InfluxDB provides alongside query results.
//!
//! ```rust,no_run
//! use futures::prelude::*;
//! use influxdb::{Client, Query};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct WeatherWithoutCityName {
//!     temperature: i32,
//! }
//!
//! #[derive(Deserialize)]
//! struct Weather {
//!     city_name: String,
//!     weather: WeatherWithoutCityName,
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), failure::Error> {
//! let client = Client::new("http://localhost:8086", "test");
//! let query = Query::raw_read_query(
//!     "SELECT temperature FROM /weather_[a-z]*$/ WHERE time > now() - 1m ORDER BY DESC",
//! );
//! let mut db_result = client.json_query(query).await?;
//! let _result = db_result
//!     .deserialize_next::<WeatherWithoutCityName>()?
//!     .series
//!     .into_iter()
//!     .map(|mut city_series| {
//!         let city_name =
//!             city_series.name.split("_").collect::<Vec<&str>>().remove(2);
//!         Weather {
//!             weather: city_series.values.remove(0),
//!             city_name: city_name.to_string(),
//!         }
//!     })
//!     .collect::<Vec<Weather>>();
//! # Ok(())
//! # }
//! ```

mod de;

use reqwest::{Client as ReqwestClient, StatusCode, Url};

use serde::{de::DeserializeOwned, Deserialize};

use crate::{Client, Error, Query, ReadQuery};

#[derive(Deserialize)]
#[doc(hidden)]
struct _DatabaseError {
    error: String,
}

#[derive(Deserialize, Debug)]
#[doc(hidden)]
pub struct DatabaseQueryResult {
    pub results: Vec<serde_json::Value>,
}

impl DatabaseQueryResult {
    pub fn deserialize_next<T: 'static>(&mut self) -> Result<Return<T>, Error>
    where
        T: DeserializeOwned + Send,
    {
        serde_json::from_value::<Return<T>>(self.results.remove(0)).map_err(|err| {
            Error::DeserializationError {
                error: format!("could not deserialize: {}", err),
            }
        })
    }
}

#[derive(Deserialize, Debug)]
#[doc(hidden)]
pub struct Return<T> {
    #[serde(default = "Vec::new")]
    pub series: Vec<Series<T>>,
}

#[derive(Debug)]
/// Represents a returned series from InfluxDB
pub struct Series<T> {
    pub name: String,
    pub values: Vec<T>,
}

impl Client {
    pub async fn json_query(&self, q: ReadQuery) -> Result<DatabaseQueryResult, Error> {
        let query = q.build().unwrap();
        let basic_parameters: Vec<(String, String)> = self.into();
        let client = {
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
                    return Err(error);
                }
            };
            url.query_pairs_mut().append_pair("q", &read_query);

            if !read_query.contains("SELECT") && !read_query.contains("SHOW") {
                let error = Error::InvalidQueryError {
                    error: String::from(
                        "Only SELECT and SHOW queries supported with JSON deserialization",
                    ),
                };
                return Err(error);
            }

            ReqwestClient::new().get(url.as_str())
        };

        let res = client
            .send()
            .await
            .map_err(|err| Error::ConnectionError { error: err })?;

        match res.status() {
            StatusCode::UNAUTHORIZED => return Err(Error::AuthorizationError),
            StatusCode::FORBIDDEN => return Err(Error::AuthenticationError),
            _ => {}
        }

        let body = res.bytes().await.map_err(|err| Error::ProtocolError {
            error: format!("{}", err),
        })?;

        // Try parsing InfluxDBs { "error": "error message here" }
        if let Ok(error) = serde_json::from_slice::<_DatabaseError>(&body) {
            return Err(Error::DatabaseError { error: error.error });
        }

        // Json has another structure, let's try actually parsing it to the type we're deserializing
        serde_json::from_slice::<DatabaseQueryResult>(&body).map_err(|err| {
            Error::DeserializationError {
                error: format!("serde error: {}", err),
            }
        })
    }
}
