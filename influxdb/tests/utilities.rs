use influxdb::{Client, Error, Query};
use tokio::runtime::current_thread::Runtime;

/* Integration Test Harness for unauthed connections to InfluxDb */
#[allow(dead_code)]
pub fn run_influx_integration_test<T>(test_name: &str, test: T) -> ()
where
    T: FnOnce(Client) -> (),
{
    create_db(test_name).expect("could not setup db");
    let client = create_client(test_name.to_string());

    test(client);

    delete_db(test_name).expect("could not clean up db");
}

/* Integration Test Harness for authed connections to InfluxDb */
#[allow(dead_code)]
pub fn run_influx_integration_test_authed<T>(test_name: &str, test: T) -> ()
where
    T: FnOnce(Client) -> (),
{
    create_db(test_name).expect("could not setup db");
    let client = Client::new("http://localhost:9086", test_name).with_auth("admin", "password");

    test(client);

    delete_db(test_name).expect("could not clean up db");
}

/* Returns a runtime from Tokio */
#[allow(dead_code)]
pub fn get_runtime() -> Runtime {
    Runtime::new().expect("Unable to create a runtime")
}

/* Creates a Database in InfluxDb */
#[allow(dead_code)]
pub fn create_db<T>(name: T) -> Result<String, Error>
where
    T: Into<String>,
{
    let test_name = name.into();
    let query = format!("CREATE DATABASE {}", test_name);
    get_runtime().block_on(create_client(test_name).query(&Query::raw_read_query(query)))
}

/* Deletes a Database in InfluxDb */
#[allow(dead_code)]
fn delete_db<T>(name: T) -> Result<String, Error>
where
    T: Into<String>,
{
    let test_name = name.into();
    let query = format!("DROP DATABASE {}", test_name);
    get_runtime().block_on(create_client(test_name).query(&Query::raw_read_query(query)))
}

/* Creates a new InfluxDb Client */
#[allow(dead_code)]
fn create_client(test_name: String) -> Client {
    Client::new("http://localhost:8086", test_name)
}

/* Helper for asserting that a Result is `Ok` */
#[allow(dead_code)]
pub fn assert_result_ok<A: std::fmt::Debug, B: std::fmt::Debug>(result: &Result<A, B>) {
    result.as_ref().expect("assert_result_ok failed");
}

/* Helper for asserting that a Result is `Error` */
#[allow(dead_code)]
pub fn assert_result_err<A: std::fmt::Debug, B: std::fmt::Debug>(result: &Result<A, B>) {
    result.as_ref().expect_err("assert_result_err failed");
}
