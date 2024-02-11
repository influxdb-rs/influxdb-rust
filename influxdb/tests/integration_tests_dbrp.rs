extern crate influxdb;

#[path = "./utilities.rs"]
pub mod utilities;
use utilities::{assert_result_err, assert_result_ok, run_test};

use influxdb::InfluxDbWriteable;
use influxdb::{Client, Error, ReadQuery, Timestamp};

/// INTEGRATION TEST
///
/// This test case tests connection error
#[async_std::test]
#[cfg(not(tarpaulin_include))]
async fn test_no_rp() {
    let client = Client::new("http://127.0.0.1:2086", "mydb").with_token("admintoken");
    let read_query = ReadQuery::new("SELECT * FROM weather");
    let read_result = client.query(read_query).await;
    assert_result_err(&read_result);
    match read_result {
        Err(Error::ConnectionError { error: s }) => {
            println!("NO_RP_ERROR: {}", s);
        }
        _ => panic!(
            "Should cause a ConnectionError: {}",
            read_result.unwrap_err()
        ),
    }
}

/// INTEGRATION TEST
///
/// This test case tests using the retention policy with DBRP mapping
#[async_std::test]
#[cfg(not(tarpaulin))]
pub async fn test_authed_write_and_read_with_rp() {
    run_test(
        || async move {
            let client = Client::new("http://127.0.0.1:2086", "mydb")
                .with_token("admintoken")
                .with_retention_policy("testing2");
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
            let client = Client::new("http://127.0.0.1:2086", "mydb")
                .with_token("admintoken")
                .with_retention_policy("testing2");
            let read_query = ReadQuery::new("DELETE MEASUREMENT weather");
            let read_result = client.query(read_query).await;
            assert_result_ok(&read_result);
            assert!(!read_result.unwrap().contains("error"), "Teardown failed");
        },
    )
    .await;
}
