//! Read Query Builder returned by InfluxDbQuery::raw_read_query
//!
//! Can only be instantiated by using InfluxDbQuery::raw_read_query

use crate::error::InfluxDbError;
use crate::query::{InfluxDbQuery, QueryType, ValidQuery};

// todo: orm for query
pub struct InfluxDbReadQuery {
    query: String,
}

impl InfluxDbReadQuery {
    /// Creates a new [`InfluxDbReadQuery`]
    pub fn new<S>(query: S) -> Self
    where
        S: ToString,
    {
        InfluxDbReadQuery {
            query: query.to_string(),
        }
    }
}

impl InfluxDbQuery for InfluxDbReadQuery {
    fn build(self) -> Result<ValidQuery, InfluxDbError> {
        Ok(ValidQuery(self.query))
    }

    fn get_type(&self) -> QueryType {
        QueryType::ReadQuery
    }
}
