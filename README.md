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
    <a href="https://coveralls.io/github/Empty2k12/influxdb-rust?branch=master">
        <img src="https://coveralls.io/repos/github/Empty2k12/influxdb-rust/badge.svg?branch=master" alt='Coverage Status' />
    </a>
    <a href="https://docs.rs/crate/influxdb">
        <img src="https://docs.rs/influxdb/badge.svg" alt='Documentation Status' />
    </a>
    <a href="https://www.rust-lang.org/en-US/">
        <img src="https://img.shields.io/badge/Made%20with-Rust-orange.svg" alt='Build with Rust' />
    </a>
</p>

This library is a work in progress. Although we've been using it in production at [OpenVelo](https://openvelo.org/),
we're prioritized features that fit our use cases. This means a feature you might need is not implemented
yet or could be handled better.

Pull requests are always welcome. See [Contributing][contributing.md] and [Code of Conduct][code-of-conduct.md].

## Currently Supported Features

-   Reading and Writing to InfluxDB
-   Optional Serde Support for Deserialization
-   Running multiple queries in one request (e.g. `SELECT * FROM weather_berlin; SELECT * FROM weather_london`)
-   Authenticated and Unauthenticated Connections

## Planned Features

-   Read Query Builder instead of supplying raw queries
-   `#[derive(InfluxDbWritable)]`

## Quickstart

Add the following to your `Cargo.toml`

```toml
influxdb = "0.0.4"
```

For an example with using Serde deserialization, please refer to [serde_integration](crate::integrations::serde_integration)

```rust
use influxdb::query::{InfluxDbQuery, Timestamp};
use influxdb::client::InfluxDbClient;
use tokio::runtime::current_thread::Runtime;

// Create a InfluxDbClient with URL `http://localhost:8086`
// and database name `test`
let client = InfluxDbClient::new("http://localhost:8086", "test");

// Let's write something to InfluxDB. First we're creating a
// InfluxDbWriteQuery to write some data.
// This creates a query which writes a new measurement into a series called `weather`
let write_query = InfluxDbQuery::write_query(Timestamp::NOW, "weather")
    .add_field("temperature", 82);

// Since this library is async by default, we're going to need a Runtime,
// which can asynchonously run our query.
// The [tokio](https://crates.io/crates/tokio) crate lets us easily create a new Runtime.
let mut rt = Runtime::new().expect("Unable to create a runtime");

// To actually submit the data to InfluxDB, the `block_on` method can be used to
// halt execution of our program until it has been completed.
let write_result = rt.block_on(client.query(&write_query));
assert!(write_result.is_ok(), "Write result was not okay");

// Reading data is as simple as writing. First we need to create a query
let read_query = InfluxDbQuery::raw_read_query("SELECT _ FROM weather");

// Again, we're blocking until the request is done
let read_result = rt.block_on(client.query(&read_query));

assert!(read_result.is_ok(), "Read result was not ok");

// We can be sure the result was successful, so we can unwrap the result to get
// the response String from InfluxDB
println!("{}", read_result.unwrap());
```

For further examples, check out the Integration Tests in `tests/integration_tests.rs`
in the repository.

## License

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

@ 2019 Gero Gerke, All rights reserved.
