//! This library is a work in progress. This means a feature you might need is not implemented
//! yet or could be handled better.
//!
//! Pull requests are always welcome. See [Contributing](https://github.com/Empty2k12/influxdb-rust/blob/master/CONTRIBUTING.md) and [Code of Conduct](https://github.com/Empty2k12/influxdb-rust/blob/master/CODE_OF_CONDUCT.md). For a list of past changes, see [CHANGELOG.md](https://github.com/Empty2k12/influxdb-rust/blob/master/CHANGELOG.md).
//!
//! ## Currently Supported Features
//!
//! -   Reading and Writing to InfluxDB
//! -   Optional Serde Support for Deserialization
//! -   Running multiple queries in one request (e.g. `SELECT * FROM weather_berlin; SELECT * FROM weather_london`)
//! -   Authenticated and Unauthenticated Connections
//! -   `async`/`await` support
//! -   `#[derive(InfluxDbWriteable)]` Derive Macro for Writing / Reading into Structs
//! -   `GROUP BY` support
//! -   Tokio and async-std support (see example below) or [available backends](https://github.com/Empty2k12/influxdb-rust/blob/master/influxdb/Cargo.toml)
//! -   Swappable HTTP backends ([see below](#Choice-of-HTTP-backend))
//!
//! # Quickstart
//!
//! Add the following to your `Cargo.toml`
//!
//! ```toml
//! influxdb = { version = "0.4.0", features = ["derive"] }
//! ```
//!
//! For an example with using Serde deserialization, please refer to [serde_integration](crate::integrations::serde_integration)
//!
//! ```rust,no_run
//! use influxdb::{Client, Query, Timestamp};
//! use influxdb::InfluxDbWriteable;
//! use chrono::{DateTime, Utc};
//!
//! #[tokio::main]
//! // or #[async_std::main] if you prefer
//! async fn main() {
//!     // Connect to db `test` on `http://localhost:8086`
//!     let client = Client::new("http://localhost:8086", "test");
//!
//!     #[derive(InfluxDbWriteable)]
//!     struct WeatherReading {
//!         time: DateTime<Utc>,
//!         humidity: i32,
//!         #[influxdb(tag)] wind_direction: String,
//!     }
//!
//!     // Let's write some data into a measurement called `weather`
//!     let weather_reading = WeatherReading {
//!         time: Timestamp::Hours(1).into(),
//!         humidity: 30,
//!         wind_direction: String::from("north"),
//!     };
//!
//!     let write_result = client
//!         .query(&weather_reading.into_query("weather"))
//!         .await;
//!     assert!(write_result.is_ok(), "Write result was not okay");
//!
//!     // Let's see if the data we wrote is there
//!     let read_query = Query::raw_read_query("SELECT * FROM weather");
//!
//!     let read_result = client.query(&read_query).await;
//!     assert!(read_result.is_ok(), "Read result was not ok");
//!     println!("{}", read_result.unwrap());
//! }
//! ```
//!
//! For further examples, check out the Integration Tests in `tests/integration_tests.rs`
//! in the repository.
//!
//! # Choice of HTTP backend
//!
//! To communicate with InfluxDB, you can choose the HTTP backend to be used configuring the appropriate feature. We recommend sticking with the default reqwest-based client, unless you really need async-std compatibility.
//!
//! - **[hyper](https://github.com/hyperium/hyper)** (through reqwest, used by default), with [rustls](https://github.com/ctz/rustls)
//!   ```toml
//!   influxdb = { version = "0.4.0", features = ["derive"] }
//!   ```
//!
//! - **[hyper](https://github.com/hyperium/hyper)** (through reqwest), with native TLS (OpenSSL)
//!   ```toml
//!   influxdb = { version = "0.4.0", default-features = false, features = ["derive", "use-serde", "reqwest-client"] }
//!   ```
//!
//! - **[hyper](https://github.com/hyperium/hyper)** (through surf), use this if you need tokio 0.2 compatibility
//!    ```toml
//!    influxdb = { version = "0.4.0", default-features = false, features = ["derive", "use-serde", "curl-client"] }
//!    ```
//! - **[curl](https://github.com/alexcrichton/curl-rust)**, using [libcurl](https://curl.se/libcurl/)
//!    ```toml
//!    influxdb = { version = "0.4.0", default-features = false, features = ["derive", "use-serde", "curl-client"] }
//!    ```
//! - **[async-h1](https://github.com/http-rs/async-h1)** with native TLS (OpenSSL)
//!    ```toml
//!    influxdb = { version = "0.4.0", default-features = false, features = ["derive", "use-serde", "h1-client"] }
//!    ```
//! - **[async-h1](https://github.com/http-rs/async-h1)** with [rustls](https://github.com/ctz/rustls)
//!    ```toml
//!    influxdb = { version = "0.4.0", default-features = false, features = ["derive", "use-serde", "h1-client-rustls"] }
//!    ```
//! - WebAssembly's `window.fetch`, via `web-sys` and **[wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)**
//!    ```toml
//!    influxdb = { version = "0.4.0", default-features = false, features = ["derive", "use-serde", "wasm-client"] }
//!    ```
//!
//! # License
//!
//! [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

#![allow(clippy::needless_doctest_main)]
#![allow(clippy::needless_lifetimes)] // False positive in client/mod.rs query fn

#[cfg(all(feature = "reqwest", feature = "surf"))]
compile_error!("You need to choose between reqwest and surf; enabling both is not supported");

#[cfg(not(any(feature = "reqwest", feature = "surf")))]
compile_error!("You need to choose an http client; consider not disabling default features");

mod client;
mod error;
mod query;

pub use client::Client;
pub use error::Error;
pub use query::{
    read_query::ReadQuery,
    write_query::{Type, WriteQuery},
    InfluxDbWriteable, Query, QueryType, Timestamp, ValidQuery,
};

#[cfg(feature = "use-serde")]
pub mod integrations {
    #[cfg(feature = "use-serde")]
    pub mod serde_integration;
}
