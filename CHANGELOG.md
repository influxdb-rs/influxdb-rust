# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2020-11-15

### Changed

-  Internal request connection pooling ([@Robert-Steiner](https://github.com/Robert-Steiner) in [#73](https://github.com/Empty2k12/influxdb-rust/pull/73))

    Previously visible `Client` fields are now private. If you were using them before, please reference them from the creation of the Client.

-  Support async-std ([@JEnoch](https://github.com/JEnoch) in [#72](https://github.com/Empty2k12/influxdb-rust/pull/72))

    The default backend is still Tokio, but if you're keen on switching, [other backends are exposed](https://github.com/Empty2k12/influxdb-rust/blob/master/influxdb/Cargo.toml) as Cargo features

## [0.2.0] - 2020-10-25

### Added

-  Allow `GROUP BY` queries by providing `deserialize_next_tagged` to deserialize the group fields ([@SafariMonkey](https://github.com/SafariMonkey) in [#69](https://github.com/Empty2k12/influxdb-rust/pull/69))
-  Added `Default` for `series` in InfluxDb Response ([@SafariMonkey](https://github.com/SafariMonkey) in [#67](https://github.com/Empty2k12/influxdb-rust/pull/67))
- `WriteQuery` and `ReadQuery` now derive `Debug` and `Clone` ([@jaredwolff](https://github.com/jaredwolff) in [#63](https://github.com/Empty2k12/influxdb-rust/pull/63))

### Changed

-  Replaced `failure` crate with `thiserror` crate ([@msrd0](https://github.com/msrd0) in [#70](https://github.com/Empty2k12/influxdb-rust/pull/70))
-  Deserialize series are now deserialized using field names not field order ([@SafariMonkey](https://github.com/SafariMonkey) in [#62](https://github.com/Empty2k12/influxdb-rust/pull/62))
-  Due to InfluxDb inconsistencies between versions and ambiguities, `Timestamp::Now` has been removed. Please calculate the current timestamp since the epoch yourself and use the other available `Timestamp` values like so:

    ```
    use influxdb::{Timestamp};
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start
      .duration_since(UNIX_EPOCH)
      .expect("Time went backwards")
      .as_millis();
    let query = Timestamp::Milliseconds(since_the_epoch)
        .into_query("weather")
        .add_field("temperature", 82);
    ```

### Fixed

-  Fixed quotation marks of tag values and escaping of field values ([@Robert-Steiner](https://github.com/Robert-Steiner) in [#68](https://github.com/Empty2k12/influxdb-rust/pull/68))
-  Fixed improper quoting on tag values when the value was text ([@sparky8251](https://github.com/sparky8251) in [#64](https://github.com/Empty2k12/influxdb-rust/pull/64))

## [0.1.0] - 2020-03-17

This adds `#[derive(InfluxDbWriteable)]` for Structs, fixes escaping for the line-protocol and improves timestamp handling.

### Added

-   `#[derive(InfluxDbWriteable)]` for deriving struct writing ([@msrd0](https://github.com/msrd0))

### Changed

-   Change type of timestamp-variants to `u128` ([@mario-kr](https://github.com/mario-kr))

### Fixed

-   Use `rfc3339` as default timestamp precision ([@zjhmale](https://github.com/zjhmale))

## [0.0.6] - 2020-02-07

### Changed

-   Rewrite to `async` / `await`. Rust 1.39 is now the minimum required Rust version.

## [0.0.5] - 2019-08-16

This release removes the prefix `InfluxDb` of most types in this library and reexports the types under the `influxdb::` path. In most cases, you can directly use the types now: e.g. `influxdb::Client` vs `influxdb::client::InfluxDbClient`.

### Added

-   Switch to `cargo-readme` for README generation ([@senden9](https://github.com/senden9))
-   Contributing Guidelines, Code of Conduct and Issue Templates

### Changed

-   Removed dependency `itertools` ([@mvucenovic](https://github.com/mvucenovic))
-   Replace internal representation in query of `Any` by an enum ([@pcpthm](https://github.com/pcpthm))
-   Remove `InfluxDb` in type names
-   Replace ToString with Into<String>

### Fixed

-   Fix Crates.io detecting license incorrectly ([@mimetypes](https://github.com/mimetypes))
-   Don't commit Cargo.lock ([@msrd0](https://github.com/msrd0))
-   Fix and Enforce Clippy Lints ([@msrd0](https://github.com/msrd0))

## [0.0.4] - 2019-08-16

### Added

-   Possibility to authenticate against a InfluxDb instance ([@valkum](https://github.com/valkum))

## [0.0.3] - 2019-07-14

### Added

-   Possibility to run multiple queries in one. See the Integration Tests in `tests/integration_tests.rs` for examples.
-   Ability to specify Timestamp for write queries

### Changed

-   You now have to borrow a query when passing it to the `query` method

## [0.0.2] - 2019-07-23

### Changed

-   URLEncode Query before sending it to InfluxDB, which caused some empty returns (#5)
-   Improved Test Coverage: There's now even more tests verifying correctness of the crate (#5)
-   It's no longer necessary to supply a wildcard generic when working with serde*integration: `client.json_query::<Weather>(query)` instead of `client.json_query::<Weather, *>(query)`

[unreleased]: https://github.com/Empty2k12/influxdb-rust/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/Empty2k12/influxdb-rust/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Empty2k12/influxdb-rust/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Empty2k12/influxdb-rust/compare/v0.0.6...v0.1.0
[0.0.5]: https://github.com/Empty2k12/influxdb-rust/compare/v0.0.5...v0.0.6
[0.0.5]: https://github.com/Empty2k12/influxdb-rust/compare/v0.0.4...v0.0.5
[0.0.4]: https://github.com/Empty2k12/influxdb-rust/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/Empty2k12/influxdb-rust/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/Empty2k12/influxdb-rust/releases/tag/v0.0.2
