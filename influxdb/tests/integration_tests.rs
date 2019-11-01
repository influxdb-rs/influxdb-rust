#[path = "./utilities.rs"]
mod utilities;

extern crate influxdb;

use futures::prelude::*;
use influxdb::{Client, Error, InfluxDbWriteable, Query, Timestamp};
use utilities::{
    assert_result_err, assert_result_ok, get_runtime, run_influx_integration_test,
    run_influx_integration_test_authed,
};

#[test]
/// INTEGRATION TEST
///
/// This test case tests whether the InfluxDB server can be connected to and gathers info about it
fn test_ping_influx_db() {
    run_influx_integration_test("test_ping_influx_db", |client| {
        let result = get_runtime().block_on(client.ping());
        assert_result_ok(&result);

        let (build, version) = result.unwrap();
        assert!(!build.is_empty(), "Build should not be empty");
        assert!(!version.is_empty(), "Build should not be empty");

        println!("build: {} version: {}", build, version);
    });
}

#[test]
/// INTEGRATION TEST
///
/// This test case tests connection error
fn test_connection_error() {
    run_influx_integration_test_authed("test_connection_error", |_| {
        let client: Client = Client::new("http://localhost:10086", "test");
        let read_query = Query::raw_read_query("SELECT * FROM weather");
        let read_result = get_runtime().block_on(client.query(&read_query));
        assert_result_err(&read_result);
        match read_result {
            Err(Error::ConnectionError { .. }) => {}
            _ => panic!(format!(
                "Should cause a ConnectionError: {}",
                read_result.unwrap_err()
            )),
        }
    });
}

#[test]
/// INTEGRATION TEST
///
/// This test case tests the Authentication
fn test_authed_write_and_read() {
    run_influx_integration_test_authed("test_authed_write_and_read", |_| {
        let client = Client::new("http://localhost:9086", "test_authed_write_and_read")
            .with_auth("admin", "password");
        let write_query = Timestamp::Hours(11)
            .into_query("weather".to_string())
            .add_field("temperature", 82);
        let write_result = get_runtime().block_on(client.query(&write_query));
        assert_result_ok(&write_result);

        let read_query = Query::raw_read_query("SELECT * FROM weather");
        let read_result = get_runtime().block_on(client.query(&read_query));
        assert_result_ok(&read_result);
        assert!(
            !read_result.unwrap().contains("error"),
            "Data contained a database error"
        );
    });
}

#[test]
/// INTEGRATION TEST
///
/// This test case tests the Authentication with wrong credentials
fn test_wrong_authed_write_and_read() {
    run_influx_integration_test("test_wrong_authed_write_and_read", |_| {
        let client =
            Client::new("http://localhost:9086", "test").with_auth("wrong_user", "password");
        let write_query = Timestamp::Hours(11)
            .into_query("weather".to_string())
            .add_field("temperature", 82);
        let write_result = get_runtime().block_on(client.query(&write_query));
        match write_result {
            Err(Error::AuthorizationError) => {}
            _ => panic!(format!(
                "Should be an AuthorizationError: {}",
                write_result.unwrap()
            )),
        }
    });
}

#[test]
/// INTEGRATION TEST
///
/// This test case tests the Authentication with a client that does not send authentication
fn test_non_authed_write_and_read() {
    run_influx_integration_test("test_non_authed_write_and_read", |_| {
        let client = Client::new("http://localhost:9086", "test_non_authed_write_and_read");
        let write_query = Timestamp::Hours(11)
            .into_query("weather".to_string())
            .add_field("temperature", 82);
        let write_result = get_runtime().block_on(client.query(&write_query));
        assert_result_err(&write_result);
        match write_result {
            Err(Error::AuthorizationError) => {}
            _ => panic!(format!(
                "Should be an AuthorizationError: {}",
                write_result.unwrap()
            )),
        }

        let read_query = Query::raw_read_query("SELECT * FROM weather");
        let read_result = get_runtime().block_on(client.query(&read_query));
        assert_result_err(&read_result);
        match read_result {
            Err(Error::AuthorizationError) => {}
            _ => panic!(format!(
                "Should be an AuthorizationError: {}",
                read_result.unwrap()
            )),
        }
    });
}

#[test]
/// INTEGRATION TEST
///
/// This integration tests that writing data and retrieving the data again is working
fn test_write_and_read_field() {
    run_influx_integration_test("test_write_and_read_field", |client| {
        let write_query = Timestamp::Hours(11)
            .into_query("weather".to_string())
            .add_field("temperature", 82);
        let write_result = get_runtime().block_on(client.query(&write_query));
        assert_result_ok(&write_result);

        let read_query = Query::raw_read_query("SELECT * FROM weather");
        let read_result = get_runtime().block_on(client.query(&read_query));
        assert_result_ok(&read_result);
        assert!(
            !read_result.unwrap().contains("error"),
            "Data contained a database error"
        );
    });
}

#[test]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This integration tests that writing data and retrieving the data again is working
fn test_write_and_read_option() {
    run_influx_integration_test("test_write_and_read_option", |client| {
        use serde::Deserialize;
        let write_query = Timestamp::Hours(11)
            .into_query("weather".to_string())
            .add_field("temperature", 82)
            .add_field("wind_strength", <Option<u64>>::None);
        let write_result = get_runtime().block_on(client.query(&write_query));
        assert_result_ok(&write_result);

        #[derive(Deserialize, Debug, PartialEq)]
        struct Weather {
            time: String,
            temperature: i32,
            wind_strength: Option<u64>,
        }

        let query = Query::raw_read_query("SELECT time, temperature, wind_strength FROM weather");
        let future = client
            .json_query(query)
            .and_then(|mut db_result| db_result.deserialize_next::<Weather>());
        let result = get_runtime().block_on(future);
        assert_result_ok(&result);

        assert_eq!(
            result.unwrap().series[0].values[0],
            Weather {
                time: "1970-01-01T11:00:00Z".to_string(),
                temperature: 82,
                wind_strength: None,
            }
        );
    });
}

#[test]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This test case tests whether JSON can be decoded from a InfluxDB response and whether that JSON
/// is equal to the data which was written to the database
fn test_json_query() {
    run_influx_integration_test("test_json_query", |client| {
        use serde::Deserialize;
        let write_query = Timestamp::Hours(11)
            .into_query("test_json_query".to_string())
            .add_field("temperature", 82);
        let write_result = get_runtime().block_on(client.query(&write_query));
        assert_result_ok(&write_result);

        #[derive(Deserialize, Debug, PartialEq)]
        struct Weather {
            time: String,
            temperature: i32,
        }

        let query = Query::raw_read_query("SELECT * FROM test_json_query");
        let future = client
            .json_query(query)
            .and_then(|mut db_result| db_result.deserialize_next::<Weather>());
        let result = get_runtime().block_on(future);
        assert_result_ok(&result);

        assert_eq!(
            result.unwrap().series[0].values[0],
            Weather {
                time: "1970-01-01T11:00:00Z".to_string(),
                temperature: 82
            }
        );
    });
}

#[test]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This test case tests whether JSON can be decoded from a InfluxDB response and wether that JSON
/// is equal to the data which was written to the database
fn test_json_query_vec() {
    run_influx_integration_test("test_json_query_vec", |client| {
        use serde::Deserialize;
        let write_query1 = Timestamp::Hours(11)
            .into_query("test_json_query_vec".to_string())
            .add_field("temperature", 16);
        let write_query2 = Timestamp::Hours(12)
            .into_query("test_json_query_vec".to_string())
            .add_field("temperature", 17);
        let write_query3 = Timestamp::Hours(13)
            .into_query("test_json_query_vec".to_string())
            .add_field("temperature", 18);

        let _write_result = get_runtime().block_on(client.query(&write_query1));
        let _write_result2 = get_runtime().block_on(client.query(&write_query2));
        let _write_result2 = get_runtime().block_on(client.query(&write_query3));

        #[derive(Deserialize, Debug, PartialEq)]
        struct Weather {
            time: String,
            temperature: i32,
        }

        let query = Query::raw_read_query("SELECT * FROM test_json_query_vec");
        let future = client
            .json_query(query)
            .and_then(|mut db_result| db_result.deserialize_next::<Weather>());
        let result = get_runtime().block_on(future);
        assert_result_ok(&result);
        assert_eq!(result.unwrap().series[0].values.len(), 3);
    });
}

#[test]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This integration test tests whether using the wrong query method fails building the query
fn test_serde_multi_query() {
    run_influx_integration_test("test_serde_multi_query", |client| {
        use serde::Deserialize;
        use utilities::create_db;

        create_db("humidity").expect("could not setup second db");

        #[derive(Deserialize, Debug, PartialEq)]
        struct Temperature {
            time: String,
            temperature: i32,
        }
        #[derive(Deserialize, Debug, PartialEq)]
        struct Humidity {
            time: String,
            humidity: i32,
        }
        let write_query = Timestamp::Hours(11)
            .into_query("test_serde_multi_query".to_string())
            .add_field("temperature", 16);
        let write_query2 = Timestamp::Hours(11)
            .into_query("humidity".to_string())
            .add_field("humidity", 69);

        let write_result = get_runtime().block_on(client.query(&write_query));
        let write_result2 = get_runtime().block_on(client.query(&write_query2));
        assert_result_ok(&write_result);
        assert_result_ok(&write_result2);

        let future = client
            .json_query(
                Query::raw_read_query("SELECT * FROM test_serde_multi_query")
                    .add_query("SELECT * FROM humidity"),
            )
            .and_then(|mut db_result| {
                let temp = db_result.deserialize_next::<Temperature>();
                let humidity = db_result.deserialize_next::<Humidity>();

                (temp, humidity)
            });
        let result = get_runtime().block_on(future);
        assert_result_ok(&result);

        let (temp, humidity) = result.unwrap();
        assert_eq!(
            temp.series[0].values[0],
            Temperature {
                time: "1970-01-01T11:00:00Z".to_string(),
                temperature: 16
            },
        );
        assert_eq!(
            humidity.series[0].values[0],
            Humidity {
                time: "1970-01-01T11:00:00Z".to_string(),
                humidity: 69
            }
        );
    });
}

#[test]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This integration test tests whether using the wrong query method fails building the query
fn test_wrong_query_errors() {
    run_influx_integration_test_authed("test_name", |client| {
        let future = client.json_query(Query::raw_read_query("CREATE DATABASE this_should_fail"));
        let result = get_runtime().block_on(future);
        assert!(
            result.is_err(),
            "Should only build SELECT and SHOW queries."
        );
    });
}
