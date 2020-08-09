//! Read Query Builder returned by Query::raw_read_query
//!
//! Can only be instantiated by using Query::raw_read_query

use crate::query::{QueryType, ValidQuery};
use crate::{Error, Query};

#[derive(Debug, Clone)]
pub struct ReadQuery {
    queries: Vec<String>,
}

impl ReadQuery {
    /// Creates a new [`ReadQuery`]
    pub fn new<S>(query: S) -> Self
    where
        S: Into<String>,
    {
        ReadQuery {
            queries: vec![query.into()],
        }
    }

    /// Adds a query to the [`ReadQuery`]
    pub fn add_query<S>(mut self, query: S) -> Self
    where
        S: Into<String>,
    {
        self.queries.push(query.into());
        self
    }
}

impl Query for ReadQuery {
    fn build(&self) -> Result<ValidQuery, Error> {
        Ok(ValidQuery(self.queries.join(";")))
    }

    fn get_type(&self) -> QueryType {
        QueryType::ReadQuery
    }
}

#[cfg(test)]
mod tests {
    use crate::query::{Query, QueryType};

    #[test]
    fn test_read_builder_single_query() {
        let query = Query::raw_read_query("SELECT * FROM aachen").build();

        assert_eq!(query.unwrap(), "SELECT * FROM aachen");
    }

    #[test]
    fn test_read_builder_multi_query() {
        let query = Query::raw_read_query("SELECT * FROM aachen")
            .add_query("SELECT * FROM cologne")
            .build();

        assert_eq!(query.unwrap(), "SELECT * FROM aachen;SELECT * FROM cologne");
    }

    #[test]
    fn test_correct_query_type() {
        let query = Query::raw_read_query("SELECT * FROM aachen");

        assert_eq!(query.get_type(), QueryType::ReadQuery);
    }
}
