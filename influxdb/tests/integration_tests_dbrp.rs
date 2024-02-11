extern crate influxdb;

use influxdb::Client;

mod integration_tests_v2;

/// INTEGRATION TEST
///

/// This test case tests the Authentication without rp. It should panic
#[async_std::test]
#[should_panic]
#[cfg(not(tarpaulin))]
async fn test_authed_write_and_read() {
    integration_tests_v2::test_authed_write_and_read();
}

/// This test case adds in the set retention policy to the write and read test
#[async_std::test]
#[cfg(not(tarpaulin))]
async fn test_rp_write_and_read() {
    let client = Client::new("http://127.0.0.1:2086", "mydb")
        .with_token("admintoken")
        .with_retention_policy("autogen");
    integration_tests_v2::test_authd_write_and_read_helper(&client).await;
}
