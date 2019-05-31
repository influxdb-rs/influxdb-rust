extern crate futures;
extern crate reqwest;
extern crate tokio;

use futures::Future;
use reqwest::r#async::Client;

trait InfluxDbQuery {
    fn build<'a>(self) -> String;
}

impl InfluxDbQuery {
    pub fn write() -> InfluxDbWrite {
        InfluxDbWrite {
            _measurement: String::from("marina_3"),
            _fields: Vec::new(),
            _tags: Vec::new(),
        }
    }

    // pub fn read() {}
}

pub struct InfluxDbWrite {
    _fields: Vec<(String, String)>,
    _tags: Vec<(String, String)>,
    _measurement: String,
    // precision: Precision
}

impl InfluxDbWrite {
    fn add_field<'a, S>(&'a mut self, point: S, value: S) -> &'a mut Self
    where
        S: Into<String>,
    {
        self._fields.push((point.into(), value.into()));
        self
    }

    fn add_tag<'a, S>(&'a mut self, tag: S, value: S) -> &'a mut Self
    where
        S: Into<String>,
    {
        self._tags.push((tag.into(), value.into()));
        self
    }
}

impl InfluxDbQuery for InfluxDbWrite {
    // fixme: time (with precision) and measurement
    fn build<'a>(self) -> String {
        let tags = self
            ._tags
            .into_iter()
            .map(|(tag, value)| format!("{tag}={value}", tag = tag, value = value))
            .collect::<Vec<String>>()
            .join(",");
        let fields = self
            ._fields
            .into_iter()
            .map(|(field, value)| format!("{field}={value}", field = field, value = value))
            .collect::<Vec<String>>()
            .join(",");

        format!(
            "measurement,{tags} {fields} time",
            tags = tags,
            fields = fields
        )
    }
}

pub struct InfluxDbClient {
    _url: String,
    _database: String,
    // _auth: InfluxDbAuthentication | NoAuthentication
}

pub fn main() {}

impl InfluxDbClient {
    pub fn ping(&self) -> impl Future<Item = (String, String), Error = ()> {
        Client::new()
            .get(format!("{}/ping", self._url).as_str())
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
            _url: String::from("http://localhost:8086"),
            _database: String::from("test"),
        }
    }

    #[test]
    fn test_ping() {
        let client = create_client();
        let result = get_runtime().block_on(client.ping());
        assert!(!result.is_err(), "Should be no error");

        let (build, version) = result.unwrap();
        assert!(!build.is_empty(), "Build should not be empty");
        assert!(!version.is_empty(), "Build should not be empty");

        println!("build: {} version: {}", build, version);
    }

    #[test]
    fn test_write_builder_single_field() {
        let mut query = InfluxDbQuery::write();

        query.add_field("water_level", "2");
        assert_eq!(query.build(), "measurement, water_level=2 time");
    }

    #[test]
    fn test_write_builder_multiple_fields() {
        let mut query = InfluxDbQuery::write();

        query.add_field("water_level", "2");
        query.add_field("boat_count", "31");
        query.add_field("algae_content", "0.85");
        assert_eq!(
            query.build(),
            "measurement, water_level=2,boat_count=31,algae_content=0.85 time"
        );
    }

    // fixme: double space
    // fixme: quoting / escaping of long strings
    #[test]
    fn test_write_builder_single_tag() {
        let mut query = InfluxDbQuery::write();

        query.add_tag("marina_manager", "Smith");
        assert_eq!(query.build(), "measurement,marina_manager=Smith  time");
    }

    #[test]
    fn test_write_builder_multiple_tags() {
        let mut query = InfluxDbQuery::write();

        query.add_tag("marina_manager", "Smith");
        query.add_tag("manager_to_the_marina_manager", "Jonson");
        assert_eq!(
            query.build(),
            "measurement,marina_manager=Smith,manager_to_the_marina_manager=Jonson  time"
        );
    }

    #[test]
    fn test_write_builder_full_query() {
        let mut query = InfluxDbQuery::write();

        query.add_field("water_level", "2");
        query.add_field("boat_count", "31");
        query.add_field("algae_content", "0.85");
        query.add_tag("marina_manager", "Smith");
        query.add_tag("manager_to_the_marina_manager", "Jonson");
        assert_eq!(
            query.build(),
            "measurement,marina_manager=Smith,manager_to_the_marina_manager=Jonson water_level=2,boat_count=31,algae_content=0.85 time"
        );
    }
}