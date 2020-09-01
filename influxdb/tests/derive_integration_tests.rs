#[path = "./utilities.rs"]
mod utilities;

#[cfg(feature = "derive")]
use influxdb::InfluxDbWriteable;

use chrono::{DateTime, Utc};
use influxdb::{Query, Timestamp};

#[cfg(feature = "use-serde")]
use serde::Deserialize;

use utilities::{assert_result_ok, create_client, create_db, delete_db, run_test};

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "derive", derive(InfluxDbWriteable))]
#[cfg_attr(feature = "use-serde", derive(Deserialize))]
struct WeatherReading {
    time: DateTime<Utc>,
    humidity: i32,
    #[tag]
    wind_strength: Option<u64>,
}

#[test]
fn test_build_query() {
    let weather_reading = WeatherReading {
        time: Timestamp::Hours(1).into(),
        humidity: 30,
        wind_strength: Some(5),
    };
    let query = weather_reading
        .into_query("weather_reading")
        .build()
        .unwrap();
    assert_eq!(
        query.get(),
        "weather_reading,wind_strength=5 humidity=30i 3600000000000"
    );
}

#[cfg(feature = "derive")]
/// INTEGRATION TEST
///
/// This integration tests that writing data and retrieving the data again is working
#[tokio::test]
async fn test_derive_simple_write() {
    const TEST_NAME: &str = "test_derive_simple_write";

    run_test(
        || async move {
            create_db(TEST_NAME).await.expect("could not setup db");
            let client = create_client(TEST_NAME);
            let weather_reading = WeatherReading {
                time: Timestamp::Nanoseconds(0).into(),
                humidity: 30,
                wind_strength: Some(5),
            };
            let query = weather_reading.into_query("weather_reading");
            let result = client.query(&query).await;
            assert!(result.is_ok(), "unable to insert into db");
        },
        || async move {
            delete_db(TEST_NAME).await.expect("could not clean up db");
        },
    )
    .await;
}

/// INTEGRATION TEST
///
/// This integration tests that writing data and retrieving the data again is working
#[cfg(feature = "derive")]
#[cfg(feature = "use-serde")]
#[tokio::test]
async fn test_write_and_read_option() {
    const TEST_NAME: &str = "test_write_and_read_option";

    run_test(
        || async move {
            create_db(TEST_NAME).await.expect("could not setup db");
            let client = create_client(TEST_NAME);
            let weather_reading = WeatherReading {
                time: Timestamp::Hours(11).into(),
                humidity: 30,
                wind_strength: None,
            };
            let write_result = client
                .query(&weather_reading.into_query("weather_reading".to_string()))
                .await;
            assert_result_ok(&write_result);
            let query =
                Query::raw_read_query("SELECT time, humidity, wind_strength FROM weather_reading");
            let result = client.json_query(query).await.and_then(|mut db_result| {
                println!("{:?}", db_result);
                db_result.deserialize_next::<WeatherReading>()
            });
            assert_result_ok(&result);
            let result = result.unwrap();
            assert_eq!(result.series[0].values[0].humidity, 30);
            assert_eq!(result.series[0].values[0].wind_strength, None);
        },
        || async move {
            delete_db(TEST_NAME).await.expect("could not clean up db");
        },
    )
    .await;
}
