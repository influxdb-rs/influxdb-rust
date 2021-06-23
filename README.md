<div align="center">
    <br/>
    <img
        alt="rust-influxdb"
        src="https://i.imgur.com/4k7l8XJ.png"
        width=250px />
    <br/>
    <br/>
    <strong>Unofficial InfluxDB Driver for Rust</strong>
</div>
<br/>
<p align="center">
    <a href="https://crates.io/crates/influxdb">
        <img src="https://img.shields.io/crates/v/influxdb.svg"/>
    </a>
    <a href="https://travis-ci.org/Empty2k12/influxdb-rust">
        <img src="https://travis-ci.org/Empty2k12/influxdb-rust.svg?branch=master" alt='Build Status' />
    </a>
    <a href="https://Empty2k12.github.io/influxdb-rust/tarpaulin-report.html">
		<img src="https://Empty2k12.github.io/influxdb-rust/coverage.svg" alt="Coverage Report" />
	</a>
    <a href="https://docs.rs/crate/influxdb">
        <img src="https://docs.rs/influxdb/badge.svg" alt='Documentation Status' />
    </a>
    <a href="https://www.rust-lang.org/en-US/">
        <img src="https://img.shields.io/badge/Made%20with-Rust-orange.svg" alt='Build with Rust' />
    </a>
    <a href="https://blog.rust-lang.org/2020/08/27/Rust-1.46.0.html">
        <img src="https://img.shields.io/badge/rustc-1.46+-yellow.svg" alt='Minimum Rust Version' />
    </a>
</p>

This library is a work in progress. This means a feature you might need is not implemented
yet or could be handled better.

Pull requests are always welcome. See [Contributing](https://github.com/Empty2k12/influxdb-rust/blob/master/CONTRIBUTING.md) and [Code of Conduct](https://github.com/Empty2k12/influxdb-rust/blob/master/CODE_OF_CONDUCT.md). For a list of past changes, see [CHANGELOG.md](https://github.com/Empty2k12/influxdb-rust/blob/master/CHANGELOG.md).

### Currently Supported Features

-   Reading and Writing to InfluxDB
-   Optional Serde Support for Deserialization
-   Running multiple queries in one request (e.g. `SELECT * FROM weather_berlin; SELECT * FROM weather_london`)
-   Authenticated and Unauthenticated Connections
-   `async`/`await` support
-   `#[derive(InfluxDbWriteable)]` Derive Macro for Writing / Reading into Structs
-   `GROUP BY` support
-   Tokio and async-std support (see example below) or [available backends](https://github.com/Empty2k12/influxdb-rust/blob/master/influxdb/Cargo.toml)
-   Swappable HTTP backends ([see below](#Choice-of-HTTP-backend))

## Quickstart

Add the following to your `Cargo.toml`

```toml
influxdb = { version = "0.4.0", features = ["derive"] }
```

For an example with using Serde deserialization, please refer to [serde_integration](crate::integrations::serde_integration)

```rust
use influxdb::{Client, Query, Timestamp};
use influxdb::InfluxDbWriteable;
use chrono::{DateTime, Utc};

#[tokio::main]
// or #[async_std::main] if you prefer
async fn main() {
    // Connect to db `test` on `http://localhost:8086`
    let client = Client::new("http://localhost:8086", "test");

    #[derive(InfluxDbWriteable)]
    struct WeatherReading {
        time: DateTime<Utc>,
        humidity: i32,
        #[influxdb(tag)] wind_direction: String,
    }

    // Let's write some data into a measurement called `weather`
    let weather_reading = WeatherReading {
        time: Timestamp::Hours(1).into(),
        humidity: 30,
        wind_direction: String::from("north"),
    };

    let write_result = client
        .query(&weather_reading.into_query("weather"))
        .await;
    assert!(write_result.is_ok(), "Write result was not okay");

    // Let's see if the data we wrote is there
    let read_query = Query::raw_read_query("SELECT * FROM weather");

    let read_result = client.query(&read_query).await;
    assert!(read_result.is_ok(), "Read result was not ok");
    println!("{}", read_result.unwrap());
}
```

For further examples, check out the Integration Tests in `tests/integration_tests.rs`
in the repository.

## Choice of HTTP backend

To communicate with InfluxDB, you can choose the HTTP backend to be used configuring the appropriate feature. We recommend sticking with the default reqwest-based client, unless you really need async-std compatibility.

- **[hyper](https://github.com/hyperium/hyper)** (through reqwest, used by default), with [rustls](https://github.com/ctz/rustls)
  ```toml
  influxdb = { version = "0.4.0", features = ["derive"] }
  ```

- **[hyper](https://github.com/hyperium/hyper)** (through reqwest), with native TLS (OpenSSL)
  ```toml
  influxdb = { version = "0.4.0", default-features = false, features = ["derive", "use-serde", "reqwest-client"] }
  ```

- **[hyper](https://github.com/hyperium/hyper)** (through surf), use this if you need tokio 0.2 compatibility
   ```toml
   influxdb = { version = "0.4.0", default-features = false, features = ["derive", "use-serde", "curl-client"] }
   ```
- **[curl](https://github.com/alexcrichton/curl-rust)**, using [libcurl](https://curl.se/libcurl/)
   ```toml
   influxdb = { version = "0.4.0", default-features = false, features = ["derive", "use-serde", "curl-client"] }
   ```
- **[async-h1](https://github.com/http-rs/async-h1)** with native TLS (OpenSSL)
   ```toml
   influxdb = { version = "0.4.0", default-features = false, features = ["derive", "use-serde", "h1-client"] }
   ```
- **[async-h1](https://github.com/http-rs/async-h1)** with [rustls](https://github.com/ctz/rustls)
   ```toml
   influxdb = { version = "0.4.0", default-features = false, features = ["derive", "use-serde", "h1-client-rustls"] }
   ```
- WebAssembly's `window.fetch`, via `web-sys` and **[wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)**
   ```toml
   influxdb = { version = "0.4.0", default-features = false, features = ["derive", "use-serde", "wasm-client"] }
   ```

## License

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

@ 2020 Gero Gerke and [contributors](https://github.com/Empty2k12/influxdb-rust/graphs/contributors).
