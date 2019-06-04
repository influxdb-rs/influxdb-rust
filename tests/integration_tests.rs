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
/// This test case tests whether the InfluxDB server can be connected to and gathers info about it
fn test_write_field() {
    let client = create_client();
    let query = InfluxDbQuery::write_query("weather").add_field("temperature", "82");
    let result = get_runtime().block_on(client.query(query));
    assert!(result.is_ok(), "Should be no error");
}

#[test]
/// INTEGRATION TEST
///
/// This test case tests whether the InfluxDB server can be connected to and gathers info about it
fn test_read() {
    let client = create_client();
    let query = InfluxDbQuery::raw_read_query("SELECT * FROM weather");
    let result = get_runtime().block_on(client.query(query));
    println!("{:?}", result);
    assert!(result.is_ok(), "Should be no error");
    println!("{}", result.unwrap());
}