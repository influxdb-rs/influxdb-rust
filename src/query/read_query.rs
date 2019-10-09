//! Read Query Builder returned by InfluxDbQuery::raw_read_query
//!
//! Can only be instantiated by using InfluxDbQuery::raw_read_query

use crate::error::InfluxDbError;
use crate::query::{InfluxDbQuery, QueryType, ValidQuery};

pub struct InfluxDbReadQuery {
    queries: Vec<String>,
}

impl InfluxDbReadQuery {
    /// Creates a new [`InfluxDbReadQuery`]
    pub fn new<S>(query: S) -> Self
    where
        S: ToString,
    {
        InfluxDbReadQuery {
            queries: vec![query.to_string()],
        }
    }

    /// Adds a query to the [`InfluxDbReadQuery`]
    pub fn add_query<S>(mut self, query: S) -> Self
    where
        S: ToString,
    {
        self.queries.push(query.to_string());
        self
    }
}

impl InfluxDbQuery for InfluxDbReadQuery {
    fn build(&self) -> Result<ValidQuery, InfluxDbError> {
        Ok(ValidQuery(self.queries.join(";")))
    }

    fn get_type(&self) -> QueryType {
        QueryType::ReadQuery
    }
}

#[cfg(test)]
mod tests {
    use crate::query::{InfluxDbQuery, QueryType};

    #[test]
    fn test_read_builder_single_query() {
        let query = InfluxDbQuery::raw_read_query("SELECT * FROM aachen").build();

        assert_eq!(query.unwrap(), "SELECT * FROM aachen");
    }

    #[test]
    fn test_read_builder_multi_query() {
        let query = InfluxDbQuery::raw_read_query("SELECT * FROM aachen")
            .add_query("SELECT * FROM cologne")
            .build();

        assert_eq!(query.unwrap(), "SELECT * FROM aachen;SELECT * FROM cologne");
    }

    #[test]
    fn test_correct_query_type() {
        let query = InfluxDbQuery::raw_read_query("SELECT * FROM aachen");

        assert_eq!(query.get_type(), QueryType::ReadQuery);
    }
}
