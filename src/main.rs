#![allow(dead_code)]

extern crate itertools;

extern crate futures;
extern crate reqwest;
extern crate tokio;

use futures::Future;

use itertools::Itertools;
use reqwest::r#async::Client;
trait InfluxDbQuery {
    fn build<'a>(self) -> String;
}

impl InfluxDbQuery {
    pub fn write() -> InfluxDbWrite {
        InfluxDbWrite {
            measurement: String::from("marina_3"),
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

impl InfluxDbQuery for InfluxDbWrite {
    // fixme: time (with precision) and measurement
    fn build<'a>(self) -> String {
        let tags = self
            .tags
            .into_iter()
            .map(|(tag, value)| format!("{tag}={value}", tag = tag, value = value))
            .join(",");
        let fields = self
            .fields
            .into_iter()
            .map(|(field, value)| format!("{field}={value}", field = field, value = value))
            .join(",");

        format!(
            "measurement,{tags} {fields} time",
            tags = tags,
            fields = fields
        )
    }
}

pub struct InfluxDbClient {
    url: String,
    database: String,
    // _auth: InfluxDbAuthentication | NoAuthentication
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
    fn test_write_builder_single_field() {
        let query = InfluxDbQuery::write().add_field("water_level", "2");

        assert_eq!(query.build(), "measurement, water_level=2 time");
    }

    #[test]
    fn test_write_builder_multiple_fields() {
        let query = InfluxDbQuery::write()
            .add_field("water_level", "2")
            .add_field("boat_count", "31")
            .add_field("algae_content", "0.85");

        assert_eq!(
            query.build(),
            "measurement, water_level=2,boat_count=31,algae_content=0.85 time"
        );
    }

    // fixme: double space
    // fixme: quoting / escaping of long strings
    #[test]
    fn test_write_builder_single_tag() {
        let query = InfluxDbQuery::write().add_tag("marina_manager", "Smith");

        assert_eq!(query.build(), "measurement,marina_manager=Smith  time");
    }

    #[test]
    fn test_write_builder_multiple_tags() {
        let query = InfluxDbQuery::write()
            .add_tag("marina_manager", "Smith")
            .add_tag("manager_to_the_marina_manager", "Jonson");

        assert_eq!(
            query.build(),
            "measurement,marina_manager=Smith,manager_to_the_marina_manager=Jonson  time"
        );
    }

    #[test]
    fn test_write_builder_full_query() {
        let query = InfluxDbQuery::write()
            .add_field("water_level", "2")
            .add_field("boat_count", "31")
            .add_field("algae_content", "0.85")
            .add_tag("marina_manager", "Smith")
            .add_tag("manager_to_the_marina_manager", "Jonson");

        assert_eq!(
            query.build(),
            "measurement,marina_manager=Smith,manager_to_the_marina_manager=Jonson water_level=2,boat_count=31,algae_content=0.85 time"
        );
    }

    #[test]
    fn test_test() {
        InfluxDbQuery::write()
            .add_field("test", "1")
            .add_tag("my_tag", "0.85")
            .build();
    }
}
