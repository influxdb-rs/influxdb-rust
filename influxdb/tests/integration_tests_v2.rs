extern crate influxdb;

#[path = "./utilities.rs"]
mod utilities;
use utilities::{assert_result_err, assert_result_ok, create_client, run_test};

use influxdb::InfluxDbWriteable;
use influxdb::{Client, Error, Query, Timestamp};

/// INTEGRATION TEST
///
/// This test case tests whether the InfluxDB server can be connected to and gathers info about it - tested with async_std
#[async_std::test]
#[cfg(not(tarpaulin_include))]
async fn test_ping_influx_db_async_std() {
    let client = create_client("notusedhere");
    let result = client.ping().await;
    assert_result_ok(&result);

    let (build, version) = result.unwrap();
    assert!(!build.is_empty(), "Build should not be empty");
    assert!(!version.is_empty(), "Build should not be empty");

    println!("build: {} version: {}", build, version);
}

/// INTEGRATION TEST
///
/// This test case tests whether the InfluxDB server can be connected to and gathers info about it * tested with tokio
#[tokio::test]
#[cfg(not(tarpaulin_include))]
async fn test_ping_influx_db_tokio() {
    let client = create_client("notusedhere");
    let result = client.ping().await;
    assert_result_ok(&result);

    let (build, version) = result.unwrap();
    assert!(!build.is_empty(), "Build should not be empty");
    assert!(!version.is_empty(), "Build should not be empty");

    println!("build: {} version: {}", build, version);
}

/// INTEGRATION TEST
///
/// This test case tests connection error
#[async_std::test]
#[cfg(not(tarpaulin_include))]
async fn test_connection_error() {
    let test_name = "test_connection_error";
    let client = Client::new("http://127.0.0.1:10086", test_name).with_token("admintoken");
    let read_query = <dyn Query>::raw_read_query("SELECT * FROM weather");
    let read_result = client.query(&read_query).await;
    assert_result_err(&read_result);
    match read_result {
        Err(Error::ConnectionError { .. }) => {}
        _ => panic!(
            "Should cause a ConnectionError: {}",
            read_result.unwrap_err()
        ),
    }
}

/// INTEGRATION TEST
///
/// This test case tests the Authentication
#[async_std::test]
#[cfg(not(tarpaulin_include))]
async fn test_authed_write_and_read() {
    const TEST_NAME: &str = "test_authed_write_and_read";

    run_test(
        || async move {
            let client = Client::new("http://127.0.0.1:9086", TEST_NAME).with_token("admintoken");
            let query = format!("CREATE DATABASE {}", TEST_NAME);
            client
                .query(&<dyn Query>::raw_read_query(query))
                .await
                .expect("could not setup db");

            let client = Client::new("http://127.0.0.1:9086", TEST_NAME).with_token("admintoken");
            let write_query = Timestamp::Hours(11)
                .into_query("weather")
                .add_field("temperature", 82);
            let write_result = client.query(&write_query).await;
            assert_result_ok(&write_result);

            let read_query = <dyn Query>::raw_read_query("SELECT * FROM weather");
            let read_result = client.query(&read_query).await;
            assert_result_ok(&read_result);
            assert!(
                !read_result.unwrap().contains("error"),
                "Data contained a database error"
            );
        },
        || async move {
            let client = Client::new("http://127.0.0.1:9086", TEST_NAME).with_token("admintoken");
            let query = format!("DROP DATABASE {}", TEST_NAME);

            client
                .query(&<dyn Query>::raw_read_query(query))
                .await
                .expect("could not clean up db");
        },
    )
    .await;
}

/// INTEGRATION TEST
///
/// This test case tests the Authentication
#[async_std::test]
#[cfg(not(tarpaulin_include))]
async fn test_wrong_authed_write_and_read() {
    const TEST_NAME: &str = "test_wrong_authed_write_and_read";

    run_test(
        || async move {
            let client = Client::new("http://127.0.0.1:9086", TEST_NAME).with_token("admintoken");
            let query = format!("CREATE DATABASE {}", TEST_NAME);
            client
                .query(&<dyn Query>::raw_read_query(query))
                .await
                .expect("could not setup db");

            let client = Client::new("http://127.0.0.1:9086", TEST_NAME).with_token("falsetoken");
            let write_query = Timestamp::Hours(11)
                .into_query("weather")
                .add_field("temperature", 82);
            let write_result = client.query(&write_query).await;
            assert_result_err(&write_result);
            match write_result {
                Err(Error::AuthorizationError) => {}
                _ => panic!(
                    "Should be an AuthorizationError: {}",
                    write_result.unwrap_err()
                ),
            }

            let read_query = <dyn Query>::raw_read_query("SELECT * FROM weather");
            let read_result = client.query(&read_query).await;
            assert_result_err(&read_result);
            match read_result {
                Err(Error::AuthorizationError) => {}
                _ => panic!(
                    "Should be an AuthorizationError: {}",
                    read_result.unwrap_err()
                ),
            }

            let client = Client::new("http://127.0.0.1:9086", TEST_NAME)
                .with_auth("nopriv_user", "password");
            let read_query = <dyn Query>::raw_read_query("SELECT * FROM weather");
            let read_result = client.query(&read_query).await;
            assert_result_err(&read_result);
            match read_result {
                Err(Error::AuthenticationError) => {}
                _ => panic!(
                    "Should be an AuthenticationError: {}",
                    read_result.unwrap_err()
                ),
            }
        },
        || async move {
            let client = Client::new("http://127.0.0.1:9086", TEST_NAME).with_token("admintoken");
            let query = format!("DROP DATABASE {}", TEST_NAME);
            client
                .query(&<dyn Query>::raw_read_query(query))
                .await
                .expect("could not clean up db");
        },
    )
    .await;
}

/// INTEGRATION TEST
///
/// This test case tests the Authentication
#[async_std::test]
#[cfg(not(tarpaulin_include))]
async fn test_non_authed_write_and_read() {
    const TEST_NAME: &str = "test_non_authed_write_and_read";

    run_test(
        || async move {
            let client = Client::new("http://127.0.0.1:9086", TEST_NAME).with_token("admintoken");
            let query = format!("CREATE DATABASE {}", TEST_NAME);
            client
                .query(&<dyn Query>::raw_read_query(query))
                .await
                .expect("could not setup db");
            let non_authed_client = Client::new("http://127.0.0.1:9086", TEST_NAME);
            let write_query = Timestamp::Hours(11)
                .into_query("weather")
                .add_field("temperature", 82);
            let write_result = non_authed_client.query(&write_query).await;
            assert_result_err(&write_result);
            match write_result {
                Err(Error::AuthorizationError) => {}
                _ => panic!(
                    "Should be an AuthorizationError: {}",
                    write_result.unwrap_err()
                ),
            }

            let read_query = <dyn Query>::raw_read_query("SELECT * FROM weather");
            let read_result = non_authed_client.query(&read_query).await;
            assert_result_err(&read_result);
            match read_result {
                Err(Error::AuthorizationError) => {}
                _ => panic!(
                    "Should be an AuthorizationError: {}",
                    read_result.unwrap_err()
                ),
            }
        },
        || async move {
            let client = Client::new("http://127.0.0.1:9086", TEST_NAME).with_token("admintoken");
            let query = format!("DROP DATABASE {}", TEST_NAME);
            client
                .query(&<dyn Query>::raw_read_query(query))
                .await
                .expect("could not clean up db");
        },
    )
    .await;
}
