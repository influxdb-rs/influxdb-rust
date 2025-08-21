use futures_util::FutureExt;
#[cfg(not(tarpaulin_include))]
use influxdb::{InfluxVersion1, InfluxVersion2};
use influxdb::{Client, Error, ReadQuery};
use std::future::Future;
use std::panic::{AssertUnwindSafe, UnwindSafe};

#[allow(dead_code)]
#[cfg(not(tarpaulin_include))]
pub fn assert_result_err<A: std::fmt::Debug, B: std::fmt::Debug>(result: &Result<A, B>) {
    result.as_ref().expect_err("assert_result_err failed");
}

#[cfg(not(tarpaulin_include))]
pub fn assert_result_ok<A: std::fmt::Debug, B: std::fmt::Debug>(result: &Result<A, B>) {
    result.as_ref().expect("assert_result_ok failed");
}

#[allow(dead_code)]
#[cfg(not(tarpaulin_include))]
pub fn create_client_v1<T>(db_name: T) -> Client<InfluxVersion1, reqwest::Client>
where
    T: Into<String>,
{
    Client::<InfluxVersion1>::new("http://127.0.0.1:8086", db_name)
}

#[allow(dead_code)]
#[cfg(not(tarpaulin_include))]
pub fn create_client_v2<T>(bucket: T) -> Client<InfluxVersion2, reqwest::Client>
where
    T: Into<String>,
{
    Client::<InfluxVersion2>::new("http://127.0.0.1:8086", bucket)
}

#[allow(dead_code)]
#[cfg(not(tarpaulin_include))]
pub async fn create_db_v1<T>(name: T) -> Result<String, Error>
where
    T: Into<String>,
{
    let test_name = name.into();
    let query = format!("CREATE DATABASE {test_name}");
    create_client_v1(test_name).query(ReadQuery::new(query)).await
}

#[allow(dead_code)]
#[cfg(not(tarpaulin_include))]
pub async fn create_db_v2<T>(name: T) -> Result<String, Error>
where
    T: Into<String>,
{
    let test_name = name.into();
    let query = format!("CREATE DATABASE {test_name}");
    create_client_v2(test_name).query(ReadQuery::new(query)).await
}

#[allow(dead_code)]
#[cfg(not(tarpaulin_include))]
pub async fn delete_db_v1<T>(name: T) -> Result<String, Error>
where
    T: Into<String>,
{
    let test_name = name.into();
    let query = format!("DROP DATABASE {test_name}");
    create_client_v1(test_name).query(ReadQuery::new(query)).await
}

#[allow(dead_code)]
#[cfg(not(tarpaulin_include))]
pub async fn delete_db_v2<T>(name: T) -> Result<String, Error>
where
    T: Into<String>,
{
    let test_name = name.into();
    let query = format!("DROP DATABASE {test_name}");
    create_client_v2(test_name).query(ReadQuery::new(query)).await
}

#[cfg(not(tarpaulin_include))]
pub async fn run_test<F, T, Fut1, Fut2>(test_fn: F, teardown: T)
where
    F: FnOnce() -> Fut1 + UnwindSafe,
    T: FnOnce() -> Fut2,
    Fut1: Future,
    Fut2: Future,
{
    let test_result = AssertUnwindSafe(test_fn()).catch_unwind().await;
    AssertUnwindSafe(teardown())
        .catch_unwind()
        .await
        .expect("failed teardown");
    test_result.expect("failed test");
}
