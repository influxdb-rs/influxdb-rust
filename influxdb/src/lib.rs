//! Pull requests are always welcome. See [Contributing](https://github.com/influxdb-rs/influxdb-rust/blob/main/CONTRIBUTING.md) and [Code of Conduct](https://github.com/influxdb-rs/influxdb-rust/blob/main/CODE_OF_CONDUCT.md). For a list of past changes, see [CHANGELOG.md](https://github.com/influxdb-rs/influxdb-rust/blob/main/CHANGELOG.md).
//!
//! ## Currently Supported Features
//!
//! -   Reading and writing to InfluxDB
//! -   Optional Serde support for deserialization
//! -   Running multiple queries in one request (e.g. `SELECT * FROM weather_berlin; SELECT * FROM weather_london`)
//! -   Writing single or multiple measurements in one request (e.g. `WriteQuery` or `Vec<WriteQuery>` argument)
//! -   Authenticated and unauthenticated connections
//! -   `async`/`await` support
//! -   `#[derive(InfluxDbWriteable)]` derive macro for writing / reading into structs
//! -   `GROUP BY` support
//! -   Tokio and async-std support (see example below) or [available backends](https://github.com/influxdb-rs/influxdb-rust/blob/main/influxdb/Cargo.toml)
//! -   Swappable HTTP backends ([see below](#Choice-of-HTTP-backend))
//!
//! # Quickstart
//!
//! Add the following to your `Cargo.toml`
//!
#![doc = cargo_toml!(indent="", "derive")]
//!
//! For an example with using Serde deserialization, please refer to [serde_integration](crate::integrations::serde_integration)
//!
//! ```rust,no_run
//! use chrono::{DateTime, Utc};
//! use influxdb::{Client, Error, InfluxDbWriteable, ReadQuery, Timestamp};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     // Connect to db `test` on `http://localhost:8086`
//!     let client = Client::new("http://localhost:8086", "test");
//!
//!     #[derive(InfluxDbWriteable)]
//!     struct WeatherReading {
//!         time: DateTime<Utc>,
//!         humidity: i32,
//!         #[influxdb(tag)]
//!         wind_direction: String,
//!     }
//!
//!     // Let's write some data into a measurement called `weather`
//!     let weather_readings = vec![
//!         WeatherReading {
//!             time: Timestamp::Hours(1).into(),
//!             humidity: 30,
//!             wind_direction: String::from("north"),
//!         }
//!         .into_query("weather"),
//!         WeatherReading {
//!             time: Timestamp::Hours(2).into(),
//!             humidity: 40,
//!             wind_direction: String::from("west"),
//!         }
//!         .into_query("weather"),
//!     ];
//!
//!     client.query(weather_readings).await?;
//!
//!     // Read back all records
//!     let read_query = ReadQuery::new("SELECT * FROM weather");
//!
//!     let read_result = client.query(read_query).await?;
//!     println!("{}", read_result);
//!     Ok(())
//! }
//! ```
//!
//! For further examples, check out the integration tests in `tests/integration_tests.rs`
//! in the repository.
//!
//! # Choice of HTTP backend
//!
//! To communicate with InfluxDB, you can choose the HTTP backend to be used configuring the appropriate feature. We recommend sticking with the default reqwest-based client, unless you really need async-std compatibility.
//!
//! - **[hyper](https://github.com/hyperium/hyper)** (through reqwest, used by default), with [rustls](https://github.com/ctz/rustls)
#![doc = cargo_toml!(indent="\t", "derive")]
//! - **[hyper](https://github.com/hyperium/hyper)** (through reqwest), with native TLS (OpenSSL)
#![doc = cargo_toml!(indent="\t", default-features = false, "derive", "serde", "reqwest-client-native-tls")]
//! - **[hyper](https://github.com/hyperium/hyper)** (through reqwest), with vendored native TLS (OpenSSL)
#![doc = cargo_toml!(indent="\t", default-features = false, "derive", "serde", "reqwest-client-native-tls-vendored")]
//!
//! # License
//!
//! [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

macro_rules! cargo_toml {
    (indent=$indent:literal, $firstfeat:literal $(, $feature:literal)*) => {
        cargo_toml_private!($indent, "", $firstfeat $(, $feature)*)
    };

    (indent=$indent:literal, default-features = false, $firstfeat:literal $(, $feature:literal)*) => {
        cargo_toml_private!($indent, "default-features = false, ", $firstfeat $(, $feature)*)
    };
}
use cargo_toml;

macro_rules! cargo_toml_private {
    ($indent:literal, $deffeats:literal, $firstfeat:literal $(, $feature:literal)*) => {
        concat!(
            $indent,
            "```toml\n",

            $indent,
            "influxdb = { version = \"",
            env!("CARGO_PKG_VERSION"),
            "\", ",
            $deffeats,
            "features = [",
            "\"", $firstfeat, "\"",
            $(", \"", $feature, "\"",)*
            "] }\n",

            $indent,
            "```"
        )
    };
}
use cargo_toml_private;

mod client;
mod error;
mod query;

pub use client::{Client, InfluxVersion1, InfluxVersion2, InfluxVersion3};
pub use error::Error;
pub use query::{
    read_query::ReadQuery,
    write_query::{Type, WriteQuery},
    InfluxDbWriteable, Query, QueryType, Timestamp, ValidQuery,
};

#[cfg(feature = "serde")]
pub mod integrations {
    #[cfg(feature = "serde")]
    pub mod serde_integration;
}
