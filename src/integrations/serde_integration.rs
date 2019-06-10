//! Serde Integration for InfluxDB. Provides deserialization of query returns.
//!
//! When querying multiple series in the same query (e.g. with a regex query), it might be desirable to flat map
//! the resulting series into a single `Vec` like so. The example assumes, that there are weather readings in multiple
//! series named `weather_<city_name>` (e.g. `weather_berlin`, or `weather_london`). Since we're using a Regex query,
//! we don't actually know which series will be returned. To assign the city name to the series, we can use the series
//! `name`, InfluxDB provides alongside query results.
//!
//! ```rust,no_run
//! use influxdb::query::InfluxDbQuery;
//! use influxdb::client::InfluxDbClient;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct WeatherWithoutCityName {
//!     temperature: i32
//! }
//!
//! #[derive(Deserialize)]
//! struct Weather {
//!     city_name: String,
//!     weather: WeatherWithoutCityName,
//! }
//!
//! let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();
//! let client = InfluxDbClient::new("http://localhost:8086", "test");
//! let query = InfluxDbQuery::raw_read_query("SELECT temperature FROM /weather_[a-z]*$/ WHERE time > now() - 1m ORDER BY DESC");
//! let _result = rt.block_on(client.json_query::<WeatherWithoutCityName, _>(query))
//!     .map(|it| {
//!         it.map(|series_vec| {
//!             series_vec
//!                 .into_iter()
//!                 .map(|mut city_series| {
//!                     let city_name = city_series.name.split("_").collect::<Vec<&str>>().remove(2);
//!                     Weather { weather: city_series.values.remove(0), city_name: city_name.to_string() }
//!                 }).collect::<Vec<Weather>>()
//!         })
//!     });

use crate::client::InfluxDbClient;

use serde::de::DeserializeOwned;

use futures::{Future, Stream};
use reqwest::r#async::{Client, Decoder};
use std::mem;

use serde::Deserialize;
use serde_json;

use crate::error::InfluxDbError;
use crate::query::{InfluxDbQuery, QueryType};

#[derive(Deserialize)]
#[doc(hidden)]
struct _DatabaseError {
    error: String,
}

#[derive(Deserialize, Debug)]
#[doc(hidden)]
pub struct DatabaseQueryResult<T> {
    pub results: Vec<InfluxDbReturn<T>>,
}

#[derive(Deserialize, Debug)]
#[doc(hidden)]
pub struct InfluxDbReturn<T> {
    pub series: Option<Vec<InfluxDbSeries<T>>>,
}

#[derive(Deserialize, Debug)]
/// Represents a returned series from InfluxDB
pub struct InfluxDbSeries<T> {
    pub name: String,
    pub values: Vec<T>,
}

impl InfluxDbClient {
    pub fn json_query<T: 'static, Q>(
        &self,
        q: Q,
    ) -> Box<dyn Future<Item = Option<Vec<InfluxDbSeries<T>>>, Error = InfluxDbError>>
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
                let error = InfluxDbError::InvalidQueryError {
                    error: format!("{}", err),
                };
                return Box::new(
                    future::err::<Option<Vec<InfluxDbSeries<T>>>, InfluxDbError>(error),
                );
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
                    url = self.database_url(),
                    endpoint = endpoint,
                    db = self.database_name(),
                    url_params = url_params
                )
                .as_str(),
            ),
            QueryType::WriteQuery => Client::new()
                .post(
                    format!(
                        "{url}/{endpoint}?db={db}",
                        url = self.database_url(),
                        endpoint = endpoint,
                        db = self.database_name(),
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
                .map_err(|err| InfluxDbError::ProtocolError {
                    error: format!("{}", err),
                })
                .and_then(|body| {
                    // Try parsing InfluxDBs { "error": "error message here" }
                    if let Ok(error) = serde_json::from_slice::<_DatabaseError>(&body) {
                        return futures::future::err(InfluxDbError::DatabaseError {
                            error: error.error.to_string(),
                        });
                    } else {
                        // Json has another structure, let's try actually parsing it to the type we're deserializing
                        let from_slice = serde_json::from_slice::<DatabaseQueryResult<T>>(&body);

                        let mut deserialized = match from_slice {
                            Ok(deserialized) => deserialized,
                            Err(err) => {
                                return futures::future::err(InfluxDbError::DeserializationError {
                                    error: format!("serde error: {}", err),
                                })
                            }
                        };

                        return futures::future::result(Ok(deserialized.results.remove(0).series));
                    }
                }),
        )
    }
}