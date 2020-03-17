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
    <a href="https://docs.rs/crate/influxdb">
        <img src="https://docs.rs/influxdb/badge.svg" alt='Documentation Status' />
    </a>
    <a href="https://www.rust-lang.org/en-US/">
        <img src="https://img.shields.io/badge/Made%20with-Rust-orange.svg" alt='Build with Rust' />
    </a>
    <a href="https://blog.rust-lang.org/2019/11/07/Rust-1.39.0.html">
        <img src="https://img.shields.io/badge/rustc-1.39+-yellow.svg" alt='Minimum Rust Version' />
    </a>
</p>

This library is a work in progress. Although we've been using it in production at [OpenVelo](https://openvelo.org/),
we've prioritized features that fit our use cases. This means a feature you might need is not implemented
yet or could be handled better.

Pull requests are always welcome. See [Contributing](https://github.com/Empty2k12/influxdb-rust/blob/master/CONTRIBUTING.md) and [Code of Conduct](https://github.com/Empty2k12/influxdb-rust/blob/master/CODE_OF_CONDUCT.md).

### Currently Supported Features

-   Reading and Writing to InfluxDB
-   Optional Serde Support for Deserialization
-   Running multiple queries in one request (e.g. `SELECT * FROM weather_berlin; SELECT * FROM weather_london`)
-   Authenticated and Unauthenticated Connections
-   `async`/`await` support
-   `#[derive(InfluxDbWriteable)]` Derive Macro for Writing / Reading into Structs

### Planned Features

-   Read Query Builder instead of supplying raw queries

## Quickstart

Add the following to your `Cargo.toml`

```toml
influxdb = { version = "0.1.0", features = ["derive"] }
```

For an example with using Serde deserialization, please refer to [serde_integration](crate::integrations::serde_integration)

```rust
use influxdb::{Client, Query, Timestamp};
use influxdb::InfluxDbWriteable;
use chrono::{DateTime, Utc};

#[tokio::main]
async fn main() {
    // Connect to db `test` on `http://localhost:8086`
    let client = Client::new("http://localhost:8086", "test");

    #[derive(InfluxDbWriteable)]
    struct WeatherReading {
        time: DateTime<Utc>,
        humidity: i32,
        #[tag] wind_direction: String,
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

## License

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

@ 2019 Gero Gerke, All rights reserved.
