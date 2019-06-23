extern crate influxdb;

use influxdb::client::InfluxDbClient;
use influxdb::query::InfluxDbQuery;
use tokio::runtime::current_thread::Runtime;

fn get_runtime() -> Runtime {
    Runtime::new().expect("Unable to create a runtime")
}

fn create_client() -> InfluxDbClient {
    InfluxDbClient::new("http://localhost:8086", "test")
}

#[test]
/// INTEGRATION TEST
///
/// This test case tests whether the InfluxDB server can be connected to and gathers info about it
fn test_ping_influx_db() {
    let client = create_client();
    let result = get_runtime().block_on(client.ping());
    assert!(result.is_ok(), "Should be no error");

    let (build, version) = result.unwrap();
    assert!(!build.is_empty(), "Build should not be empty");
    assert!(!version.is_empty(), "Build should not be empty");

    println!("build: {} version: {}", build, version);
}

#[test]
/// INTEGRATION TEST
///
/// Tests if a database can be created
fn test_create_database() {
    let client = create_client();
    let query = InfluxDbQuery::raw_read_query("CREATE DATABASE test");
    let result = get_runtime().block_on(client.query(query));
    assert!(
        result.is_ok(),
        format!("Should be no error: {}", result.unwrap_err())
    );
}

#[test]
/// INTEGRATION TEST
///
/// This test case tests whether the InfluxDB server can be connected to and gathers info about it
fn test_write_field() {
    let client = create_client();
    let query = InfluxDbQuery::write_query("weather").add_field("temperature", 82);
    let result = get_runtime().block_on(client.query(query));
    assert!(
        result.is_ok(),
        format!("Should be no error: {}", result.unwrap_err())
    );
}

#[test]
/// INTEGRATION TEST
///
/// This test case tests whether the raw string can be returned from the InfluxDB
fn test_read() {
    let client = create_client();
    let query = InfluxDbQuery::raw_read_query("SELECT * FROM weather");
    let result = get_runtime().block_on(client.query(query));
    assert!(
        result.is_ok(),
        format!("Should be no error: {}", result.unwrap_err())
    );
    assert!(
        !result.unwrap().contains("error"),
        "Data contained a database error"
    );
}

#[test]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This test case tests whether JSON can be decoded from a InfluxDB response
fn test_json_query() {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct Weather {
        time: String,
        temperature: i32,
    }

    let client = create_client();
    let query = InfluxDbQuery::raw_read_query("SELECT * FROM weather");
    let result = get_runtime().block_on(client.json_query::<Weather>(query));

    assert!(
        result.is_ok(),
        format!("We couldn't read from the DB: {}", result.unwrap_err())
    );
}

#[test]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This integration test tests whether using the wrong query method fails building the query
fn test_serde_query_build_error() {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct Weather {
        time: String,
        temperature: i32,
    }

    let client = create_client();
    let query = InfluxDbQuery::raw_read_query("CREATE database should_fail");
    let result = get_runtime().block_on(client.json_query::<Weather>(query));

    assert!(
        result.is_err(),
        format!(
            "Should not be able to build JSON query that is not SELECT or SELECT .. INTO: {}",
            result.unwrap_err()
        )
    );
}

#[test]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This test case tests whether JSON can be decoded from a InfluxDB response
fn test_raw_query_build_error() {
    let client = create_client();
    let query = InfluxDbQuery::write_query("weather").add_tag("season", "summer");
    let result = get_runtime().block_on(client.query(query));

    assert!(
        result.is_err(),
        format!(
            "Should not be able to build JSON query that is not SELECT or SELECT .. INTO: {}",
            result.unwrap_err()
        )
    );
}
