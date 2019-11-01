#[path = "./utilities.rs"]
mod utilities;

#[cfg(feature = "derive")]
use influxdb::InfluxDbWriteable;

use chrono::{DateTime, Utc};
use futures::prelude::*;
use influxdb::{Query, Timestamp};
use utilities::{assert_result_ok, get_runtime, run_influx_integration_test};

#[cfg(feature = "use-serde")]
use serde::Deserialize;

#[derive(InfluxDbWriteable, Debug, PartialEq)]
#[cfg_attr(feature = "use-serde", derive(Deserialize))]
struct WeatherReading {
    time: DateTime<Utc>,
    humidity: i32,
    wind_strength: Option<u64>,
}

#[cfg(feature = "derive")]
#[test]
fn test_derive_simple_write() {
    run_influx_integration_test("test_derive_simple_write", |client| {
        let weather_reading = WeatherReading {
            time: Timestamp::Now.into(),
            humidity: 30,
            wind_strength: Some(5),
        };
        let query = weather_reading.into_query("weather_reading".to_string());
        let future = client.query(&query);
        let result = get_runtime().block_on(future);
        assert!(result.is_ok(), "unable to insert into db");
    });
}

#[test]
#[cfg(feature = "derive")]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This integration tests that writing data and retrieving the data again
/// is working, and if initial and retrieved data are equal
fn test_write_and_read_option() {
    run_influx_integration_test("test_write_and_read_option", |client| {
        let weather_reading = WeatherReading {
            time: Timestamp::Hours(11).into(),
            humidity: 30,
            wind_strength: None,
        };
        let query = weather_reading.into_query("weather_reading".to_string());
        let write_result = get_runtime().block_on(client.query(&query));
        assert_result_ok(&write_result);

        let query =
            Query::raw_read_query("SELECT time, humidity, wind_strength FROM weather_reading");
        let future = client.json_query(query).and_then(|mut db_result| {
            println!("{:?}", db_result);
            db_result.deserialize_next::<WeatherReading>()
        });
        let result = get_runtime().block_on(future);
        assert_result_ok(&result);
        let result = result.unwrap();
        assert_eq!(result.series[0].values[0].humidity, 30);
        assert_eq!(result.series[0].values[0].wind_strength, None);
    });
}
