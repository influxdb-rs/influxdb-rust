#![allow(dead_code)]

#[macro_use]
extern crate failure;

pub mod client;
pub mod error;
pub mod query;

#[cfg(feature = "use-serde")]
pub mod integrations {
    #[cfg(feature = "use-serde")]
    pub mod serde_integration;
}

#[cfg(test)]
mod tests {
    use crate::query::InfluxDbQuery;

    #[test]
    fn test_write_builder_empty_query() {
        let query = InfluxDbQuery::write_query("marina_3").build();

        assert!(query.is_err(), "Query was not empty");
    }

    #[test]
    fn test_write_builder_single_field() {
        let query = InfluxDbQuery::write_query("weather")
            .add_field("temperature", "82")
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(query.unwrap(), "weather temperature=82");
    }

    #[test]
    fn test_write_builder_multiple_fields() {
        let query = InfluxDbQuery::write_query("weather")
            .add_field("temperature", "82")
            .add_field("wind_strength", "3.7")
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(query.unwrap(), "weather temperature=82,wind_strength=3.7");
    }

    // todo: (fixme) quoting / escaping of long strings
    #[test]
    fn test_write_builder_only_tags() {
        let query = InfluxDbQuery::write_query("weather")
            .add_tag("season", "summer")
            .build();

        assert!(query.is_err(), "Query missing one or more fields");
    }

    #[test]
    fn test_write_builder_full_query() {
        let query = InfluxDbQuery::write_query("weather")
            .add_field("temperature", "82")
            .add_tag("location", "us-midwest")
            .add_tag("season", "summer")
            .build();

        assert!(query.is_ok(), "Query was empty");
        assert_eq!(
            query.unwrap(),
            "weather,location=us-midwest,season=summer temperature=82"
        );
    }
}
