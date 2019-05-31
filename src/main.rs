#![allow(dead_code)]

#[macro_use]
extern crate failure;

use futures::Future;
use itertools::Itertools;
use reqwest::r#async::Client;

#[derive(Debug, Fail)]
enum InfluxDbError {
    #[fail(display = "query must contain at least one field")]
    InvalidQueryError,
}

#[derive(Debug)]
struct ValidQuery(String);
impl From<String> for ValidQuery {
    fn from(s: String) -> ValidQuery {
        ValidQuery(s)
    }
}
impl PartialEq<String> for ValidQuery {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}
impl PartialEq<&str> for ValidQuery {
    fn eq(&self, other: &&str) -> bool {
        &self.0 == other
    }
}

trait InfluxDbQuery {
    fn build<'a>(self) -> Result<ValidQuery, InfluxDbError>;
}

impl InfluxDbQuery {
    pub fn write<S>(measurement: S) -> InfluxDbWrite
    where
        S: Into<String>,
    {
        InfluxDbWrite {
            measurement: measurement.into(),
            fields: Vec::new(),
            tags: Vec::new(),
        }
    }

    // pub fn read() {}
}

pub struct InfluxDbWrite {
    fields: Vec<(String, String)>,
    tags: Vec<(String, String)>,
    measurement: String,
    // precision: Precision
}

impl InfluxDbWrite {
    fn add_field<'a, S>(mut self, point: S, value: S) -> Self
    where
        S: Into<String>,
    {
        self.fields.push((point.into(), value.into()));
        self
    }

    fn add_tag<'a, S>(mut self, tag: S, value: S) -> Self
    where
        S: Into<String>,
    {
        self.tags.push((tag.into(), value.into()));
        self
    }
}

// todo: fuse_with(other: ValidQuery), so multiple queries can be run at the same time
impl InfluxDbQuery for InfluxDbWrite {
    // fixme: time (with precision) and measurement
    fn build<'a>(self) -> Result<ValidQuery, InfluxDbError> {
        if self.fields.is_empty() {
            return Err(InfluxDbError::InvalidQueryError);
        }

        let tags = self
            .tags
            .into_iter()
            .map(|(tag, value)| format!("{tag}={value}", tag = tag, value = value))
            .join(",")
            + " ";
        let fields = self
            .fields
            .into_iter()
            .map(|(field, value)| format!("{field}={value}", field = field, value = value))
            .join(",")
            + " ";

        Ok(ValidQuery::from(format!(
            "{measurement},{tags}{fields}time",
            measurement = self.measurement,
            tags = tags,
            fields = fields
        )))
    }
}

pub struct InfluxDbClient {
    url: String,
    database: String,
    // auth: Option<InfluxDbAuthentication>
}

pub fn main() {}

impl InfluxDbClient {
    pub fn ping(&self) -> impl Future<Item = (String, String), Error = ()> {
        Client::new()
            .get(format!("{}/ping", self.url).as_str())
            .send()
            .map(|res| {
                let build = res
                    .headers()
                    .get("X-Influxdb-Build")
                    .unwrap()
                    .to_str()
                    .unwrap();
                let version = res
                    .headers()
                    .get("X-Influxdb-Version")
                    .unwrap()
                    .to_str()
                    .unwrap();

                (String::from(build), String::from(version))
            })
            .map_err(|err| println!("request error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::{InfluxDbClient, InfluxDbQuery};
    use tokio::runtime::current_thread::Runtime;

    fn get_runtime() -> Runtime {
        Runtime::new().expect("Unable to create a runtime")
    }

    fn create_client() -> InfluxDbClient {
        InfluxDbClient {
            url: String::from("http://localhost:8086"),
            database: String::from("test"),
        }
    }

    #[test]
    fn test_ping() {
        let client = create_client();
        let result = get_runtime().block_on(client.ping());
        assert!(result.is_ok(), "Should be no error");

        let (build, version) = result.unwrap();
        assert!(!build.is_empty(), "Build should not be empty");
        assert!(!version.is_empty(), "Build should not be empty");

        println!("build: {} version: {}", build, version);
    }

    #[test]
    fn test_write_builder_empty_query() {
        let query = InfluxDbQuery::write("marina_3").build();

        assert!(query.is_err(), "Query was not empty");
    }

    #[test]
    fn test_write_builder_single_field() {
        let query = InfluxDbQuery::write("marina_3")
            .add_field("water_level", "2")
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(query.unwrap(), "marina_3, water_level=2 time");
    }

    #[test]
    fn test_write_builder_multiple_fields() {
        let query = InfluxDbQuery::write("marina_3")
            .add_field("water_level", "2")
            .add_field("boat_count", "31")
            .add_field("algae_content", "0.85")
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(
            query.unwrap(),
            "marina_3, water_level=2,boat_count=31,algae_content=0.85 time"
        );
    }

    // fixme: quoting / escaping of long strings
    #[test]
    fn test_write_builder_only_tags() {
        let query = InfluxDbQuery::write("marina_3")
            .add_tag("marina_manager", "Smith")
            .build();

        assert!(query.is_err(), "Query missing one or more fields");
    }

    #[test]
    fn test_write_builder_full_query() {
        let query = InfluxDbQuery::write("marina_3")
            .add_field("water_level", "2")
            .add_field("boat_count", "31")
            .add_field("algae_content", "0.85")
            .add_tag("marina_manager", "Smith")
            .add_tag("manager_to_the_marina_manager", "Jonson")
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(
            query.unwrap(),
            "marina_3,marina_manager=Smith,manager_to_the_marina_manager=Jonson water_level=2,boat_count=31,algae_content=0.85 time"
        );
    }
}
