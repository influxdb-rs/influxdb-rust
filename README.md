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
    <a href="https://github.com/rust-lang/rust/releases/tag/1.71.0">
        <img src="https://img.shields.io/badge/rustc-1.71.0+-yellow.svg" alt='Minimum Rust Version: 1.71.0' />
    </a>
</p>

Pull requests are always welcome. See [Contributing][__link0] and [Code of Conduct][__link1]. For a list of past changes, see [CHANGELOG.md][__link2].

### Currently Supported Features

* Reading and writing to InfluxDB
* Optional Serde support for deserialization
* Running multiple queries in one request (e.g. `SELECT * FROM weather_berlin; SELECT * FROM weather_london`)
* Writing single or multiple measurements in one request (e.g. `WriteQuery` or `Vec<WriteQuery>` argument)
* Authenticated and unauthenticated connections
* `async`/`await` support
* `#[derive(InfluxDbWriteable)]` derive macro for writing / reading into structs
* `GROUP BY` support
* Tokio and async-std support (see example below) or [available backends][__link3]
* Swappable HTTP backends ([see below](#Choice-of-HTTP-backend))

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
            time: Timestamp::Hours(1).try_into().unwrap(),
            humidity: 30,
            wind_direction: String::from("north"),
        }
        .try_into_query("weather")
        .unwrap(),
        WeatherReading {
            time: Timestamp::Hours(2).try_into().unwrap(),
            humidity: 40,
            wind_direction: String::from("west"),
        }
        .try_into_query("weather")
        .unwrap(),
    ];

    client.query(weather_readings).await?;

    // Read back all records
    let read_query = ReadQuery::new("SELECT * FROM weather");

    let read_result = client.query(read_query).await?;
    println!("{}", read_result);
    Ok(())
}
```

For further examples, check out the integration tests in `tests/integration_tests.rs`
in the repository.

## Choice of HTTP backend

To communicate with InfluxDB, you can choose the HTTP backend to be used configuring the appropriate feature. We recommend sticking with the default reqwest-based client, unless you really need async-std compatibility.

* **[hyper][__link5]** (through reqwest, used by default), with [rustls][__link6]
  ```toml
  influxdb = { version = "0.7.2", features = ["derive"] }
  ```

* **[hyper][__link7]** (through reqwest), with native TLS (OpenSSL)
  ```toml
  influxdb = { version = "0.7.2", default-features = false, features = ["derive", "serde", "native-tls"] }
  ```

* **[hyper][__link8]** (through reqwest), with vendored native TLS (OpenSSL)
  ```toml
  influxdb = { version = "0.7.2", default-features = false, features = ["derive", "serde", "native-tls-vendored"] }
  ```

## License

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)][__link9]


@ 2020-2026 Gero Gerke, msrd0 and [contributors].

 [contributors]: https://github.com/influxdb-rs/influxdb-rust/graphs/contributors
 [__cargo_doc2readme_dependencies_info]: ggGkYW0CYXSEG_hV_Hhi195rG1bQ50Z796M6G0clIrwU3dD1GxD-fO9UhKvaYXKEGxuf7s5mIUnXG0aItuf7_gNCG97yq-v-QgOpG7Xm07crWXUNYWSBgmhpbmZsdXhkYmUwLjcuMg
 [__link0]: https://github.com/influxdb-rs/influxdb-rust/blob/main/CONTRIBUTING.md
 [__link1]: https://github.com/influxdb-rs/influxdb-rust/blob/main/CODE_OF_CONDUCT.md
 [__link2]: https://github.com/influxdb-rs/influxdb-rust/blob/main/CHANGELOG.md
 [__link3]: https://github.com/influxdb-rs/influxdb-rust/blob/main/influxdb/Cargo.toml
 [__link4]: https://docs.rs/influxdb/0.7.2/influxdb/?search=integrations::serde_integration
 [__link5]: https://github.com/hyperium/hyper
 [__link6]: https://github.com/ctz/rustls
 [__link7]: https://github.com/hyperium/hyper
 [__link8]: https://github.com/hyperium/hyper
 [__link9]: https://opensource.org/licenses/MIT

