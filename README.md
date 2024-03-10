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
    <a href="https://blog.rust-lang.org/2022/11/03/Rust-1.65.0.html">
        <img src="https://img.shields.io/badge/rustc-1.65+-yellow.svg" alt='Minimum Rust Version: 1.65' />
    </a>
</p>

Pull requests are always welcome. See [Contributing][__link0] and [Code of Conduct][__link1]. For a list of past changes, see [CHANGELOG.md][__link2].


### Currently Supported Features

 - Reading and writing to InfluxDB
 - Optional Serde support for deserialization
 - Running multiple queries in one request (e.g. `SELECT * FROM weather_berlin; SELECT * FROM weather_london`)
 - Writing single or multiple measurements in one request (e.g. `WriteQuery` or `Vec<WriteQuery>` argument)
 - Authenticated and unauthenticated connections
 - `async`/`await` support
 - `#[derive(InfluxDbWriteable)]` derive macro for writing / reading into structs
 - `GROUP BY` support
 - Tokio and async-std support (see example below) or [available backends][__link3]
 - Swappable HTTP backends ([see below](#Choice-of-HTTP-backend))


## Quickstart

Add the following to your `Cargo.toml`


```toml
influxdb = { version = "0.7.2", features = ["derive"] }
```

For an example with using Serde deserialization, please refer to [serde_integration][__link4]


```rust
use chrono::{DateTime, Utc};
use influxdb::{Client, Error, InfluxDbWriteable, ReadQuery, Timestamp};

#[tokio::main]
// or #[async_std::main] if you prefer
async fn main() -> Result<(), Error> {
    // Connect to db `test` on `http://localhost:8086`
    let client = Client::new("http://localhost:8086", "test");

    #[derive(InfluxDbWriteable)]
    struct WeatherReading {
        time: DateTime<Utc>,
        humidity: i32,
        #[influxdb(tag)]
        wind_direction: String,
    }

    // Let's write some data into a measurement called `weather`
    let weather_readings = vec![
        WeatherReading {
            time: Timestamp::Hours(1).into(),
            humidity: 30,
            wind_direction: String::from("north"),
        }
        .into_query("weather"),
        WeatherReading {
            time: Timestamp::Hours(2).into(),
            humidity: 40,
            wind_direction: String::from("west"),
        }
        .into_query("weather"),
    ];

    client.query(weather_readings).await?;

    // Read back all records
    let read_query = ReadQuery::new("SELECT * FROM weather");

    let read_result = client.query(read_query).await?;
    println!("{}", read_result);
    Ok(())
}
```

For further examples, check out the integration tests in `tests/integration_tests.rs` in the repository.


## Choice of HTTP backend

To communicate with InfluxDB, you can choose the HTTP backend to be used configuring the appropriate feature. We recommend sticking with the default reqwest-based client, unless you really need async-std compatibility.

 - **[hyper][__link5]** (through reqwest, used by default), with [rustls][__link6]
	```toml
	influxdb = { version = "0.7.2", features = ["derive"] }
	```
	
	
 - **[hyper][__link7]** (through reqwest), with native TLS (OpenSSL)
	```toml
	influxdb = { version = "0.7.2", default-features = false, features = ["derive", "serde", "reqwest-client-native-tls"] }
	```
	
	
 - **[hyper][__link8]** (through reqwest), with vendored native TLS (OpenSSL)
	```toml
	influxdb = { version = "0.7.2", default-features = false, features = ["derive", "serde", "reqwest-client-native-tls-vendored"] }
	```
	
	
 - **[hyper][__link9]** (through surf), use this if you need tokio 0.2 compatibility
	```toml
	influxdb = { version = "0.7.2", default-features = false, features = ["derive", "serde", "hyper-client"] }
	```
	
	
 - **[curl][__link10]**, using [libcurl][__link11]
	```toml
	influxdb = { version = "0.7.2", default-features = false, features = ["derive", "serde", "curl-client"] }
	```
	
	
 - **[async-h1][__link12]** with native TLS (OpenSSL)
	```toml
	influxdb = { version = "0.7.2", default-features = false, features = ["derive", "serde", "h1-client"] }
	```
	
	
 - **[async-h1][__link13]** with [rustls][__link14]
	```toml
	influxdb = { version = "0.7.2", default-features = false, features = ["derive", "serde", "h1-client-rustls"] }
	```
	
	
 - WebAssembly’s `window.fetch`, via `web-sys` and **[wasm-bindgen][__link15]**
	```toml
	influxdb = { version = "0.7.2", default-features = false, features = ["derive", "serde", "wasm-client"] }
	```
	
	


## License

[![License: MIT][__link16]][__link17]



@ 2020-2024 Gero Gerke, msrd0 and [contributors].

 [contributors]: https://github.com/influxdb-rs/influxdb-rust/graphs/contributors
 [__cargo_doc2readme_dependencies_info]: ggGkYW0BYXSEG_RDmlyxxvyrG0rwcLBKoYdvG5It9hbWNgjUGzjD8iBYfsFFYXKEG1LaAVLASZMqG5J2qfpyCvbMG_Rohh5BobOmG0DqLv5454SZYWSBgmhpbmZsdXhkYmUwLjcuMg
 [__link0]: https://github.com/influxdb-rs/influxdb-rust/blob/main/CONTRIBUTING.md
 [__link1]: https://github.com/influxdb-rs/influxdb-rust/blob/main/CODE_OF_CONDUCT.md
 [__link10]: https://github.com/alexcrichton/curl-rust
 [__link11]: https://curl.se/libcurl/
 [__link12]: https://github.com/http-rs/async-h1
 [__link13]: https://github.com/http-rs/async-h1
 [__link14]: https://github.com/ctz/rustls
 [__link15]: https://github.com/rustwasm/wasm-bindgen
 [__link16]: https://img.shields.io/badge/License-MIT-yellow.svg
 [__link17]: https://opensource.org/licenses/MIT
 [__link2]: https://github.com/influxdb-rs/influxdb-rust/blob/main/CHANGELOG.md
 [__link3]: https://github.com/influxdb-rs/influxdb-rust/blob/main/influxdb/Cargo.toml
 [__link4]: https://docs.rs/influxdb/0.7.2/influxdb/?search=integrations::serde_integration
 [__link5]: https://github.com/hyperium/hyper
 [__link6]: https://github.com/ctz/rustls
 [__link7]: https://github.com/hyperium/hyper
 [__link8]: https://github.com/hyperium/hyper
 [__link9]: https://github.com/hyperium/hyper

