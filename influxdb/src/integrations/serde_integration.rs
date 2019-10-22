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
//! let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();
//! let client = Client::new("http://localhost:8086", "test");
//! let query = Query::raw_read_query(
//!     "SELECT temperature FROM /weather_[a-z]*$/ WHERE time > now() - 1m ORDER BY DESC",
//! );
//! let _result = rt
//!     .block_on(client.json_query(query))
//!     .map(|mut db_result| db_result.deserialize_next::<WeatherWithoutCityName>())
//!     .map(|it| {
//!         it.map(|series_vec| {
//!             series_vec
//!                 .series
//!                 .into_iter()
//!                 .map(|mut city_series| {
//!                     let city_name =
//!                         city_series.name.split("_").collect::<Vec<&str>>().remove(2);
//!                     Weather {
//!                         weather: city_series.values.remove(0),
//!                         city_name: city_name.to_string(),
//!                     }
//!                 })
//!                 .collect::<Vec<Weather>>()
//!         })
//!     });
//! ```

use serde::de::DeserializeOwned;

use futures::{Future, Stream};
use reqwest::r#async::{Client as ReqwestClient, Decoder};
use reqwest::{StatusCode, Url};
use std::mem;

use serde::Deserialize;
use serde_json;

use crate::{Client, Error, Query, ReadQuery};

use futures::future::Either;

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
    pub fn deserialize_next<T: 'static>(
        &mut self,
    ) -> impl Future<Item = Return<T>, Error = Error> + Send
    where
        T: DeserializeOwned + Send,
    {
        match serde_json::from_value::<Return<T>>(self.results.remove(0)) {
            Ok(item) => futures::future::result(Ok(item)),
            Err(err) => futures::future::err(Error::DeserializationError {
                error: format!("could not deserialize: {}", err),
            }),
        }
    }
}

#[derive(Deserialize, Debug)]
#[doc(hidden)]
pub struct Return<T> {
    pub series: Vec<Series<T>>,
}

#[derive(Deserialize, Debug)]
/// Represents a returned series from InfluxDB
pub struct Series<T> {
    pub name: String,
    pub values: Vec<T>,
}

impl Client {
    pub fn json_query(
        &self,
        q: ReadQuery,
    ) -> impl Future<Item = DatabaseQueryResult, Error = Error> + Send {
        use futures::future;

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
                    return Either::B(future::err::<DatabaseQueryResult, Error>(error));
                }
            };
            url.query_pairs_mut().append_pair("q", &read_query.clone());

            if read_query.contains("SELECT") || read_query.contains("SHOW") {
                ReqwestClient::new().get(url.as_str())
            } else {
                let error = Error::InvalidQueryError {
                    error: String::from(
                        "Only SELECT and SHOW queries supported with JSON deserialization",
                    ),
                };
                return Either::B(future::err::<DatabaseQueryResult, Error>(error));
            }
        };

        Either::A(
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
                    // Try parsing InfluxDBs { "error": "error message here" }
                    if let Ok(error) = serde_json::from_slice::<_DatabaseError>(&body) {
                        futures::future::err(Error::DatabaseError {
                            error: error.error.to_string(),
                        })
                    } else {
                        // Json has another structure, let's try actually parsing it to the type we're deserializing
                        let from_slice = serde_json::from_slice::<DatabaseQueryResult>(&body);

                        let deserialized = match from_slice {
                            Ok(deserialized) => deserialized,
                            Err(err) => {
                                return futures::future::err(Error::DeserializationError {
                                    error: format!("serde error: {}", err),
                                })
                            }
                        };

                        futures::future::result(Ok(deserialized))
                    }
                }),
        )
    }
}
