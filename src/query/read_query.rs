
use crate::error::InfluxDbError;
use crate::query::{InfluxDbQuery, QueryType, ValidQuery};
// todo: orm for query
pub struct InfluxDbReadQuery {
    query: String,
}

impl InfluxDbReadQuery {
    pub fn new<S>(query: S) -> Self
    where
        S: Into<String>,
    {
        InfluxDbReadQuery {
            query: query.into(),
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