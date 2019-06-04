use crate::client::InfluxDbClient;

use serde::de::DeserializeOwned;

use futures::{Future, Stream};
use reqwest::r#async::{Client, Decoder};

use serde_json;
use serde::Deserialize;
use std::mem;

use crate::error::InfluxDbError;
use crate::query::{InfluxDbQuery, QueryType};

#[derive(Deserialize)]
#[doc(hidden)]
struct _DatabaseError {
    error: String,
}

pub trait InfluxDbSerdeORM {
    fn json_query<T: 'static, Q>(self, q: Q) -> Box<dyn Future<Item = T, Error = InfluxDbError>> where
        Q: InfluxDbQuery,
        T: DeserializeOwned;
}

impl InfluxDbSerdeORM for InfluxDbClient {
    fn json_query<T: 'static, Q>(self, q: Q) -> Box<dyn Future<Item = T, Error = InfluxDbError>>
    where
        Q: InfluxDbQuery,
        T: DeserializeOwned,
    {
        use futures::future;

        let query_type = q.get_type();
        let endpoint = match query_type {
            QueryType::ReadQuery => "query",
            QueryType::WriteQuery => "write",
        };

        let query = match q.build() {
            Err(err) => {
                let error = InfluxDbError::UnspecifiedError {
                    error: format!("{}", err),
                };
                return Box::new(future::err::<T, InfluxDbError>(error));
            }
            Ok(query) => query,
        };

        let query_str = query.get();
        let url_params = match query_type {
            QueryType::ReadQuery => format!("&q={}", query_str),
            QueryType::WriteQuery => String::from(""),
        };

        let client = match query_type {
            QueryType::ReadQuery => Client::new().get(
                format!(
                    "{url}/{endpoint}?db={db}{url_params}",
                    url = self.database_url(),
                    endpoint = endpoint,
                    db = self.database_name(),
                    url_params = url_params
                )
                .as_str(),
            ),
            QueryType::WriteQuery => Client::new()
                .post(
                    format!(
                        "{url}/{endpoint}?db={db}",
                        url = self.database_url(),
                        endpoint = endpoint,
                        db = self.database_name(),
                    )
                    .as_str(),
                )
                .body(query_str),
        };

        Box::new(
            client
                .send()
                .and_then(|mut res| {
                    let body = mem::replace(res.body_mut(), Decoder::empty());
                    body.concat2()
                })
                .map_err(|err| InfluxDbError::UnspecifiedError {
                    error: format!("{}", err)
                })
                .and_then(|body| {
                    // Try parsing InfluxDBs { "error": "error message here" }
                    if let Ok(error) = serde_json::from_slice::<_DatabaseError>(&body) {
                        return futures::future::err(InfluxDbError::DatabaseError {
                            error: error.error.to_string()
                        })
                    } else if let Ok(t_result) = serde_json::from_slice::<T>(&body) {
                        // Json has another structure, let's try actually parsing it to the type we're deserializing to
                        return futures::future::result(Ok(t_result));
                    } else {
                        return futures::future::err(InfluxDbError::UnspecifiedError {
                            error: "something wen't wrong during deserializsation of the database response. this might be a bug!".to_string()
                        })
                    }
                })
        )
    }
}