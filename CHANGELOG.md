# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[unreleased]: https://github.com/Empty2k12/influxdb-rust/compare/v0.0.3...HEAD
[0.0.3]: https://github.com/Empty2k12/influxdb-rust/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/Empty2k12/influxdb-rust/releases/tag/v0.0.2
