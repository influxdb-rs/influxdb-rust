extern crate influxdb;

use futures::prelude::*;
use influxdb::client::InfluxDbClient;
use influxdb::error::InfluxDbError;
use influxdb::query::{InfluxDbQuery, Timestamp};
use tokio::runtime::current_thread::Runtime;

fn get_runtime() -> Runtime {
    Runtime::new().expect("Unable to create a runtime")
}

fn create_client<T>(db_name: T) -> InfluxDbClient
where
    T: ToString,
{
    InfluxDbClient::new("http://localhost:8086", db_name)
}

///
/// HELPER METHODS
///

fn create_db<T>(test_name: T) -> Result<String, InfluxDbError>
where
    T: ToString,
{
    let query = format!("CREATE DATABASE {}", test_name.to_string());
    get_runtime().block_on(create_client(test_name).query(&InfluxDbQuery::raw_read_query(query)))
}

fn delete_db<T>(test_name: T) -> Result<String, InfluxDbError>
where
    T: ToString,
{
    let query = format!("DROP DATABASE {}", test_name.to_string());
    get_runtime().block_on(create_client(test_name).query(&InfluxDbQuery::raw_read_query(query)))
}

#[test]
/// INTEGRATION TEST
///
/// This test case tests whether the InfluxDB server can be connected to and gathers info about it
fn test_ping_influx_db() {
    let client = create_client("notusedhere");
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
/// //todo: describe what this test is doing!
fn test_write_and_read_field() {
    let test_name = "test_write_field";
    create_db(test_name).expect("could not setup db");

    let client = create_client(test_name);
    let write_query =
        InfluxDbQuery::write_query(Timestamp::HOURS(11), "weather").add_field("temperature", 82);
    let write_result = get_runtime().block_on(client.query(&write_query));
    assert!(
        write_result.is_ok(),
        format!("Should be no error: {}", write_result.unwrap_err())
    );

    let read_query = InfluxDbQuery::raw_read_query("SELECT * FROM weather");
    let read_result = get_runtime().block_on(client.query(&read_query));
    assert!(
        read_result.is_ok(),
        format!("Should be no error: {}", read_result.unwrap_err())
    );
    assert!(
        !read_result.unwrap().contains("error"),
        "Data contained a database error"
    );

    delete_db(test_name).expect("could not clean up db");
}

#[test]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This test case tests whether JSON can be decoded from a InfluxDB response
fn test_json_query() {
    use serde::Deserialize;

    let test_name = "test_json_query";
    create_db(test_name).expect("could not setup db");

    let client = create_client(test_name);

    // todo: implement deriving so objects can easily be placed in InfluxDB
    let write_query =
        InfluxDbQuery::write_query(Timestamp::HOURS(11), "weather").add_field("temperature", 82);
    let write_result = get_runtime().block_on(client.query(&write_query));
    assert!(
        write_result.is_ok(),
        format!("Should be no error: {}", write_result.unwrap_err())
    );

    #[derive(Deserialize, Debug)]
    struct Weather {
        time: String,
        temperature: i32,
    }

    let query = InfluxDbQuery::raw_read_query("SELECT * FROM weather");
    let future = client
        .json_query(query)
        .map(|mut db_result| db_result.deserialize_next::<Weather>());
    let result = get_runtime().block_on(future);

    assert!(
        result.is_ok(),
        format!("We couldn't read from the DB: "
        //, result.unwrap_err() // todo
        )
    );

    // todo check this out!
    // assert_eq!(
    //     result.unwrap(),
    //     Weather {
    //         time: 11,
    //         temperature: 82
    //     }
    // )
}

#[test]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This integration test tests whether using the wrong query method fails building the query
fn test_serde_multi_query() {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    struct Weather {
        time: String,
        temperature: i32,
    }

    let client = create_client("todo");
    let query = InfluxDbQuery::raw_read_query("CREATE database should_fail");
    let future = client
        .json_query(query)
        .map(|mut db_result| db_result.deserialize_next::<Weather>());
    let result = get_runtime().block_on(future);

    assert!(
        result.is_err(),
        format!(
            "Should not be able to build JSON query that is not SELECT or SELECT .. INTO: " //result.unwrap_err()
        )
    );
}

#[test]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This test case tests whether JSON can be decoded from a InfluxDB response
fn test_raw_query_build_error() {
    let client = create_client("todo");
    let query =
        InfluxDbQuery::write_query(Timestamp::HOURS(11), "weather").add_tag("season", "summer");
    let result = get_runtime().block_on(client.query(&query));

    assert!(
        result.is_err(),
        format!(
            "Should not be able to build JSON query that is not SELECT or SELECT .. INTO: {}",
            result.unwrap_err()
        )
    );
}

#[test]
/// INTEGRATION TEST
fn test_delete_database() {
    let client = create_client("todo");
    let query = InfluxDbQuery::raw_read_query("DELETE DATABASE test");
    let result = get_runtime().block_on(client.query(&query));

    assert!(
        result.is_err(),
        format!(
            "Should not be able to build JSON query that is not SELECT or SELECT .. INTO: {}",
            result.unwrap_err()
        )
    );
}
