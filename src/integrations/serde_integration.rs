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
//! let _result = rt.block_on(client.json_query::<WeatherWithoutCityName>(query))
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
//! ```

use crate::client::InfluxDbClient;

use serde::de::DeserializeOwned;

use futures::{Future, Stream};
use reqwest::r#async::{Client, Decoder};
use std::mem;

use serde::Deserialize;
use serde_json;

use crate::error::InfluxDbError;

use crate::query::read_query::InfluxDbReadQuery;
use crate::query::InfluxDbQuery;

use url::form_urlencoded;

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
    pub fn json_query<T: 'static>(
        &self,
        q: InfluxDbReadQuery,
    ) -> Box<dyn Future<Item = Option<Vec<InfluxDbSeries<T>>>, Error = InfluxDbError>>
    where
        T: DeserializeOwned,
    {
        use futures::future;

        let query = q.build().unwrap();

        let client = {
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
                let error = InfluxDbError::InvalidQueryError {
                    error: String::from(
                        "Only SELECT and SHOW queries supported with JSON deserialization",
                    ),
                };
                return Box::new(
                    future::err::<Option<Vec<InfluxDbSeries<T>>>, InfluxDbError>(error),
                );
            }
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
