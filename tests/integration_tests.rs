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

struct RunOnDrop {
    closure: Box<dyn Fn() -> ()>,
}

impl Drop for RunOnDrop {
    fn drop(&mut self) {
        (self.closure)();
    }
}

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
    assert!(
        result.is_ok(),
        "Should be no error: {}",
        result.unwrap_err()
    );

    let (build, version) = result.unwrap();
    assert!(!build.is_empty(), "Build should not be empty");
    assert!(!version.is_empty(), "Build should not be empty");

    println!("build: {} version: {}", build, version);
}

#[test]
/// INTEGRATION TEST
///
/// This test case tests connection error
fn test_connection_error() {
    let test_name = "test_connection_error";
    let client = InfluxDbClient::new("http://localhost:10086", test_name)
        .with_auth("nopriv_user", "password");
    let read_query = InfluxDbQuery::raw_read_query("SELECT * FROM weather");
    let read_result = get_runtime().block_on(client.query(&read_query));
    assert!(
        read_result.is_err(),
        format!("Should be an error: {}", read_result.unwrap_err())
    );
    match read_result {
        Err(InfluxDbError::ConnectionError { .. }) => assert!(true),
        _ => assert!(
            false,
            format!(
                "Should cause a ConnectionError: {}",
                read_result.unwrap_err()
            )
        ),
    }
}

#[test]
/// INTEGRATION TEST
///
/// This test case tests the Authentication
fn test_authed_write_and_read() {
    let test_name = "test_authed_write_and_read";
    let client =
        InfluxDbClient::new("http://localhost:9086", test_name).with_auth("admin", "password");
    let query = format!("CREATE DATABASE {}", test_name);
    get_runtime()
        .block_on(client.query(&InfluxDbQuery::raw_read_query(query)))
        .expect("could not setup db");

    let _run_on_drop = RunOnDrop {
        closure: Box::new(|| {
            let test_name = "test_authed_write_and_read";
            let client = InfluxDbClient::new("http://localhost:9086", test_name)
                .with_auth("admin", "password");
            let query = format!("DROP DATABASE {}", test_name);
            get_runtime()
                .block_on(client.query(&InfluxDbQuery::raw_read_query(query)))
                .expect("could not clean up db");
        }),
    };

    let client =
        InfluxDbClient::new("http://localhost:9086", test_name).with_auth("admin", "password");
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
}

#[test]
/// INTEGRATION TEST
///
/// This test case tests the Authentication
fn test_wrong_authed_write_and_read() {
    let test_name = "test_wrong_authed_write_and_read";
    let client =
        InfluxDbClient::new("http://localhost:9086", test_name).with_auth("admin", "password");
    let query = format!("CREATE DATABASE {}", test_name);
    get_runtime()
        .block_on(client.query(&InfluxDbQuery::raw_read_query(query)))
        .expect("could not setup db");

    let _run_on_drop = RunOnDrop {
        closure: Box::new(|| {
            let test_name = "test_wrong_authed_write_and_read";
            let client = InfluxDbClient::new("http://localhost:9086", test_name)
                .with_auth("admin", "password");
            let query = format!("DROP DATABASE {}", test_name);
            get_runtime()
                .block_on(client.query(&InfluxDbQuery::raw_read_query(query)))
                .expect("could not clean up db");
        }),
    };

    let client =
        InfluxDbClient::new("http://localhost:9086", test_name).with_auth("wrong_user", "password");
    let write_query =
        InfluxDbQuery::write_query(Timestamp::HOURS(11), "weather").add_field("temperature", 82);
    let write_result = get_runtime().block_on(client.query(&write_query));
    assert!(
        write_result.is_err(),
        format!("Should be an error: {}", write_result.unwrap_err())
    );
    match write_result {
        Err(InfluxDbError::AuthorizationError) => assert!(true),
        _ => assert!(
            false,
            format!(
                "Should be an AuthorizationError: {}",
                write_result.unwrap_err()
            )
        ),
    }

    let read_query = InfluxDbQuery::raw_read_query("SELECT * FROM weather");
    let read_result = get_runtime().block_on(client.query(&read_query));
    assert!(
        read_result.is_err(),
        format!("Should be an error: {}", read_result.unwrap_err())
    );
    match read_result {
        Err(InfluxDbError::AuthorizationError) => assert!(true),
        _ => assert!(
            false,
            format!(
                "Should be an AuthorizationError: {}",
                read_result.unwrap_err()
            )
        ),
    }

    let client = InfluxDbClient::new("http://localhost:9086", test_name)
        .with_auth("nopriv_user", "password");
    let read_query = InfluxDbQuery::raw_read_query("SELECT * FROM weather");
    let read_result = get_runtime().block_on(client.query(&read_query));
    assert!(
        read_result.is_err(),
        format!("Should be an error: {}", read_result.unwrap_err())
    );
    match read_result {
        Err(InfluxDbError::AuthenticationError) => assert!(true),
        _ => assert!(
            false,
            format!(
                "Should be an AuthenticationError: {}",
                read_result.unwrap_err()
            )
        ),
    }
}

#[test]
/// INTEGRATION TEST
///
/// This test case tests the Authentication
fn test_non_authed_write_and_read() {
    let test_name = "test_non_authed_write_and_read";
    let client =
        InfluxDbClient::new("http://localhost:9086", test_name).with_auth("admin", "password");
    let query = format!("CREATE DATABASE {}", test_name);
    get_runtime()
        .block_on(client.query(&InfluxDbQuery::raw_read_query(query)))
        .expect("could not setup db");

    let _run_on_drop = RunOnDrop {
        closure: Box::new(|| {
            let test_name = "test_non_authed_write_and_read";
            let client = InfluxDbClient::new("http://localhost:9086", test_name)
                .with_auth("admin", "password");
            let query = format!("DROP DATABASE {}", test_name);
            get_runtime()
                .block_on(client.query(&InfluxDbQuery::raw_read_query(query)))
                .expect("could not clean up db");
        }),
    };
    let non_authed_client = InfluxDbClient::new("http://localhost:9086", test_name);
    let write_query =
        InfluxDbQuery::write_query(Timestamp::HOURS(11), "weather").add_field("temperature", 82);
    let write_result = get_runtime().block_on(non_authed_client.query(&write_query));
    assert!(
        write_result.is_err(),
        format!("Should be an error: {}", write_result.unwrap_err())
    );
    match write_result {
        Err(InfluxDbError::AuthorizationError) => assert!(true),
        _ => assert!(
            false,
            format!(
                "Should be an AuthorizationError: {}",
                write_result.unwrap_err()
            )
        ),
    }

    let read_query = InfluxDbQuery::raw_read_query("SELECT * FROM weather");
    let read_result = get_runtime().block_on(non_authed_client.query(&read_query));
    assert!(
        read_result.is_err(),
        format!("Should be an error: {}", read_result.unwrap())
    );
    match read_result {
        Err(InfluxDbError::AuthorizationError) => assert!(true),
        _ => assert!(
            false,
            format!(
                "Should be an AuthorizationError: {}",
                read_result.unwrap_err()
            )
        ),
    }
}

#[test]
/// INTEGRATION TEST
///
/// This integration tests that writing data and retrieving the data again is working
fn test_write_and_read_field() {
    let test_name = "test_write_field";
    create_db(test_name).expect("could not setup db");
    let _run_on_drop = RunOnDrop {
        closure: Box::new(|| {
            delete_db("test_write_field").expect("could not clean up db");
        }),
    };

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
/// This test case tests whether JSON can be decoded from a InfluxDB response and wether that JSON
/// is equal to the data which was written to the database
fn test_json_query() {
    use serde::Deserialize;

    let test_name = "test_json_query";
    create_db(test_name).expect("could not setup db");
    let _run_on_drop = RunOnDrop {
        closure: Box::new(|| {
            delete_db("test_json_query").expect("could not clean up db");
        }),
    };

    let client = create_client(test_name);

    // todo: implement deriving so objects can easily be placed in InfluxDB
    let write_query =
        InfluxDbQuery::write_query(Timestamp::HOURS(11), "weather").add_field("temperature", 82);
    let write_result = get_runtime().block_on(client.query(&write_query));
    assert!(
        write_result.is_ok(),
        format!("Should be no error: {}", write_result.unwrap_err())
    );

    #[derive(Deserialize, Debug, PartialEq)]
    struct Weather {
        time: String,
        temperature: i32,
    }

    let query = InfluxDbQuery::raw_read_query("SELECT * FROM weather");
    let future = client
        .json_query(query)
        .and_then(|mut db_result| db_result.deserialize_next::<Weather>());
    let result = get_runtime().block_on(future);

    assert!(
        result.is_ok(),
        format!("We couldn't read from the DB: {}", result.unwrap_err())
    );

    assert_eq!(
        result.unwrap().series[0].values[0],
        Weather {
            time: "1970-01-01T11:00:00Z".to_string(),
            temperature: 82
        }
    );

    delete_db(test_name).expect("could not clean up db");
}

#[test]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This test case tests whether JSON can be decoded from a InfluxDB response and wether that JSON
/// is equal to the data which was written to the database
fn test_json_query_vec() {
    use serde::Deserialize;

    let test_name = "test_json_query_vec";
    create_db(test_name).expect("could not setup db");
    let _run_on_drop = RunOnDrop {
        closure: Box::new(|| {
            delete_db("test_json_query_vec").expect("could not clean up db");
        }),
    };

    let client = create_client(test_name);
    let write_query1 = InfluxDbQuery::write_query(Timestamp::HOURS(11), "temperature_vec")
        .add_field("temperature", 16);
    let write_query2 = InfluxDbQuery::write_query(Timestamp::HOURS(12), "temperature_vec")
        .add_field("temperature", 17);
    let write_query3 = InfluxDbQuery::write_query(Timestamp::HOURS(13), "temperature_vec")
        .add_field("temperature", 18);

    let _write_result = get_runtime().block_on(client.query(&write_query1));
    let _write_result2 = get_runtime().block_on(client.query(&write_query2));
    let _write_result2 = get_runtime().block_on(client.query(&write_query3));

    #[derive(Deserialize, Debug, PartialEq)]
    struct Weather {
        time: String,
        temperature: i32,
    }

    let query = InfluxDbQuery::raw_read_query("SELECT * FROM temperature_vec");
    let future = client
        .json_query(query)
        .and_then(|mut db_result| db_result.deserialize_next::<Weather>());
    let result = get_runtime().block_on(future);

    assert!(
        result.is_ok(),
        format!("We couldn't read from the DB: {}", result.unwrap_err())
    );

    assert_eq!(result.unwrap().series[0].values.len(), 3);

    delete_db(test_name).expect("could not clean up db");
}

#[test]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This integration test tests whether using the wrong query method fails building the query
fn test_serde_multi_query() {
    use serde::Deserialize;

    let test_name = "test_serde_multi_query";
    create_db(test_name).expect("could not setup db");
    let _run_on_drop = RunOnDrop {
        closure: Box::new(|| {
            delete_db("test_serde_multi_query").expect("could not clean up db");
        }),
    };

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

    let client = create_client(test_name);
    let write_query = InfluxDbQuery::write_query(Timestamp::HOURS(11), "temperature")
        .add_field("temperature", 16);
    let write_query2 =
        InfluxDbQuery::write_query(Timestamp::HOURS(11), "humidity").add_field("humidity", 69);

    let write_result = get_runtime().block_on(client.query(&write_query));
    let write_result2 = get_runtime().block_on(client.query(&write_query2));

    assert!(
        write_result.is_ok(),
        format!("Write Query 1 failed: {}", write_result.unwrap_err())
    );

    assert!(
        write_result2.is_ok(),
        format!("Write Query 2 failed: {}", write_result2.unwrap_err())
    );

    let future = client
        .json_query(
            InfluxDbQuery::raw_read_query("SELECT * FROM temperature")
                .add_query("SELECT * FROM humidity"),
        )
        .and_then(|mut db_result| {
            let temp = db_result.deserialize_next::<Temperature>();
            let humidity = db_result.deserialize_next::<Humidity>();

            (temp, humidity)
        });
    let result = get_runtime().block_on(future);

    assert!(
        result.is_ok(),
        format!("No problems should be had: {}", result.unwrap_err())
    );

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

    delete_db(test_name).expect("could not clean up db");
}

#[test]
#[cfg(feature = "use-serde")]
/// INTEGRATION TEST
///
/// This integration test tests whether using the wrong query method fails building the query
fn test_wrong_query_errors() {
    let client = create_client("test_name");
    let future = client.json_query(InfluxDbQuery::raw_read_query(
        "CREATE DATABASE this_should_fail",
    ));
    let result = get_runtime().block_on(future);
    assert!(
        result.is_err(),
        "Should only build SELECT and SHOW queries."
    );
}
