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
    <a href="https://github.com/influxdb-rs/influxdb-rust/actions/workflows/rust.yml">
        <img src="https://github.com/influxdb-rs/influxdb-rust/actions/workflows/rust.yml/badge.svg" alt='Build Status' />
    </a>
    <a href="https://influxdb-rs.github.io/influxdb-rust/tarpaulin-report.html">
        <img src="https://influxdb-rs.github.io/influxdb-rust/coverage.svg" alt="Coverage Report" />
    </a>
    <a href="https://docs.rs/influxdb">
        <img src="https://docs.rs/influxdb/badge.svg" alt='Documentation Status' />
    </a>
    <a href="https://www.rust-lang.org/en-US/">
        <img src="https://img.shields.io/badge/Made%20with-Rust-orange.svg" alt='Build with Rust' />
    </a>
    <a href="https://blog.rust-lang.org/2022/08/11/Rust-1.63.0.html">
        <img src="https://img.shields.io/badge/rustc-1.63+-yellow.svg" alt='Minimum Rust Version: 1.63' />
    </a>
</p>

This library is a work in progress. This means a feature you might need is not implemented yet or could be handled better.

Pull requests are always welcome. See [Contributing][__link0] and [Code of Conduct][__link1]. For a list of past changes, see [CHANGELOG.md][__link2].


### Currently Supported Features

 - Reading and Writing to InfluxDB
 - Optional Serde Support for Deserialization
 - Running multiple queries in one request (e.g. `SELECT * FROM weather_berlin; SELECT * FROM weather_london`)
 - Writing single or multiple measurements in one request (e.g. `WriteQuery` or `Vec<WriteQuery>` argument)
 - Authenticated and Unauthenticated Connections
 - `async`/`await` support
 - `#[derive(InfluxDbWriteable)]` Derive Macro for Writing / Reading into Structs
 - `GROUP BY` support
 - Tokio and async-std support (see example below) or [available backends][__link3]
 - Swappable HTTP backends ([see below](#Choice-of-HTTP-backend))


## Quickstart

Add the following to your `Cargo.toml`


```toml
influxdb = { version = "0.7.1", features = ["derive"] }
```

For an example with using Serde deserialization, please refer to [serde_integration][__link4]


```rust
use influxdb::{Client, Query, Timestamp, ReadQuery};
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
    let weather_readings = vec!(
        WeatherReading {
            time: Timestamp::Hours(1).into(),
            humidity: 30,
            wind_direction: String::from("north"),
        }.into_query("weather"),
        WeatherReading {
            time: Timestamp::Hours(2).into(),
            humidity: 40,
            wind_direction: String::from("west"),
        }.into_query("weather"),
    );

    let write_result = client
        .query(weather_readings)
        .await;
    assert!(write_result.is_ok(), "Write result was not okay");

    // Let's see if the data we wrote is there
    let read_query = ReadQuery::new("SELECT * FROM weather");

    let read_result = client.query(read_query).await;
    assert!(read_result.is_ok(), "Read result was not ok");
    println!("{}", read_result.unwrap());
}
```

For further examples, check out the Integration Tests in `tests/integration_tests.rs` in the repository.


## Choice of HTTP backend

To communicate with InfluxDB, you can choose the HTTP backend to be used configuring the appropriate feature. We recommend sticking with the default reqwest-based client, unless you really need async-std compatibility.

 - **[hyper][__link5]** (through reqwest, used by default), with [rustls][__link6]
	```toml
	influxdb = { version = "0.7.1", features = ["derive"] }
	```
	
	
 - **[hyper][__link7]** (through reqwest), with native TLS (OpenSSL)
	```toml
	influxdb = { version = "0.7.1", default-features = false,features = ["derive", "use-serde", "reqwest-client"] }
	```
	
	
 - **[hyper][__link8]** (through surf), use this if you need tokio 0.2 compatibility
	```toml
	influxdb = { version = "0.7.1", default-features = false,features = ["derive", "use-serde", "hyper-client"] }
	```
	
	
 - **[curl][__link9]**, using [libcurl][__link10]
	```toml
	influxdb = { version = "0.7.1", default-features = false,features = ["derive", "use-serde", "curl-client"] }
	```
	
	
 - **[async-h1][__link11]** with native TLS (OpenSSL)
	```toml
	influxdb = { version = "0.7.1", default-features = false,features = ["derive", "use-serde", "h1-client"] }
	```
	
	
 - **[async-h1][__link12]** with [rustls][__link13]
	```toml
	influxdb = { version = "0.7.1", default-features = false,features = ["derive", "use-serde", "h1-client-rustls"] }
	```
	
	
 - WebAssembly’s `window.fetch`, via `web-sys` and **[wasm-bindgen][__link14]**
	```toml
	influxdb = { version = "0.7.1", default-features = false,features = ["derive", "use-serde", "wasm-client"] }
	```
	
	


## License

[![License: MIT][__link15]][__link16]



@ 2020 Gero Gerke and [contributors].

 [contributors]: https://github.com/influxdb-rs/influxdb-rust/graphs/contributors
 [__cargo_doc2readme_dependencies_info]: ggGkYW0BYXSEG64av5CnNoNoGw8lPMr2b0MoG44uU0T70vGSG7osgcbjN7SoYXKEG8qCijK3OhAgG9r5dMb74ZyFGy-UgqMKZw5_G6wZmUfHdMJDYWSBgmhpbmZsdXhkYmUwLjcuMQ
 [__link0]: https://github.com/influxdb-rs/influxdb-rust/blob/main/CONTRIBUTING.md
 [__link1]: https://github.com/influxdb-rs/influxdb-rust/blob/main/CODE_OF_CONDUCT.md
 [__link10]: https://curl.se/libcurl/
 [__link11]: https://github.com/http-rs/async-h1
 [__link12]: https://github.com/http-rs/async-h1
 [__link13]: https://github.com/ctz/rustls
 [__link14]: https://github.com/rustwasm/wasm-bindgen
 [__link15]: https://img.shields.io/badge/License-MIT-yellow.svg
 [__link16]: https://opensource.org/licenses/MIT
 [__link2]: https://github.com/influxdb-rs/influxdb-rust/blob/main/CHANGELOG.md
 [__link3]: https://github.com/influxdb-rs/influxdb-rust/blob/main/influxdb/Cargo.toml
 [__link4]: https://docs.rs/influxdb/0.7.1/influxdb/?search=integrations::serde_integration
 [__link5]: https://github.com/hyperium/hyper
 [__link6]: https://github.com/ctz/rustls
 [__link7]: https://github.com/hyperium/hyper
 [__link8]: https://github.com/hyperium/hyper
 [__link9]: https://github.com/alexcrichton/curl-rust

