extern crate influxdb;

#[path = "./utilities.rs"]
mod utilities;
use utilities::{assert_result_err, assert_result_ok, run_test};

use influxdb::InfluxDbWriteable;
use influxdb::{Client, Error, ReadQuery, Timestamp};

/// INTEGRATION TEST
///

/// This test case tests the Authentication
#[async_std::test]
#[cfg(not(tarpaulin))]
async fn test_authed_write_and_read() {
    run_test(
        || async move {
            let client = Client::new("http://127.0.0.1:9086", "mydb").with_token("admintoken");
            let write_query = Timestamp::Hours(11)
                .into_query("weather")
                .add_field("temperature", 82);
            let write_result = client.query(&write_query).await;
            assert_result_ok(&write_result);

            let read_query = ReadQuery::new("SELECT * FROM weather");
            let read_result = client.query(read_query).await;
            assert_result_ok(&read_result);
            assert!(
                !read_result.unwrap().contains("error"),
                "Data contained a database error"
            );
        },
        || async move {
            let client = Client::new("http://127.0.0.1:9086", "mydb").with_token("admintoken");
            let read_query = ReadQuery::new("DELETE MEASUREMENT weather");
            let read_result = client.query(read_query).await;
            assert_result_ok(&read_result);
            assert!(!read_result.unwrap().contains("error"), "Teardown failed");
        },
    )
    .await;
}

/// INTEGRATION TEST
///
/// This test case tests the Authentication
#[async_std::test]
#[cfg(not(tarpaulin))]
async fn test_wrong_authed_write_and_read() {
    run_test(
        || async move {
            let client = Client::new("http://127.0.0.1:9086", "mydb").with_token("falsetoken");
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

            let read_query = ReadQuery::new("SELECT * FROM weather");
            let read_result = client.query(&read_query).await;
            assert_result_err(&read_result);
            match read_result {
                Err(Error::AuthorizationError) => {}
                _ => panic!(
                    "Should be an AuthorizationError: {}",
                    read_result.unwrap_err()
                ),
            }
        },
        || async move {},
    )
    .await;
}

/// INTEGRATION TEST
///
/// This test case tests the Authentication
#[async_std::test]
#[cfg(not(tarpaulin))]
async fn test_non_authed_write_and_read() {
    run_test(
        || async move {
            let non_authed_client = Client::new("http://127.0.0.1:9086", "mydb");
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

            let read_query = ReadQuery::new("SELECT * FROM weather");
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
        || async move {},
    )
    .await;
}
