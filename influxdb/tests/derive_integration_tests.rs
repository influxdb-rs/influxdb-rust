#[path = "./utilities.rs"]
mod utilities;

#[cfg(feature = "derive")]
use influxdb::InfluxDbWriteable;

#[cfg(feature = "derive")]
#[derive(InfluxDbWriteable)]
struct WeatherReading {
    time: Timestamp,
    humidity: i32,
    wind_strength: Option<u64>,
}

#[cfg(feature = "derive")]
#[test]
fn test_derive_simple_write() {
    run_influx_integration_test("test_derive_simple_write", |client| {
        let weather_reading = WeatherReading {
            time: Timestamp::NOW,
            humidity: 30,
            wind_strength: Some(5),
        };
        let query = weather_reading.into_query("weather_reading".to_string());
        let client = create_client(test_name);
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
    use serde::Deserialize;

    run_influx_integration_test("test_write_and_read_option", |client| {
        let weather_reading = WeatherReading {
            time: Timestamp::HOURS(11),
            humidity: 30,
            wind_strength: None,
        };
        let query = weather_reading.into_query("weather_reading".to_string());
        let write_result = get_runtime().block_on(client.query(query));
        assert_result_ok(&write_result);

        let query = Query::raw_read_query("SELECT time, temperature, wind_strength FROM weather");
        let future = client
            .json_query(query)
            .and_then(|mut db_result| db_result.deserialize_next::<WeatherReading>());
        let result = get_runtime().block_on(future);
        assert_result_ok(&result);
        assert_eq!(
            result.unwrap().series[0].values[0],
            Weather {
                time: "1970-01-01T11:00:00Z".to_string(),
                humidity: 30,
                wind_strength: None,
            }
        );
    });
}
