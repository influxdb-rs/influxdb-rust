# How to release this crate

This project consists of two crates which need to be published to crates.io in the correct order. Additonally, there's some steps one's likely to miss when releasing.

## Pre-Release-Checklist

- [ ] `influxdb/Cargo.toml` and `influxdb-derive/Cargo.toml`: Bumped `influxdb` and `influxdb-derive` versions to new version number?
- [ ] `influxdb/Cargo.toml`: Changed `influxdb` dependecy on `influxdb-derive` to new version number?
- [ ] `CHANGELOG.md`: Remove all entries from the unreleased section
- [ ] `CHANGELOG.md`: Add new Section for the new version number with subsections `Added`, `Changed` and `Fixed`.
- [ ] `CHANGELOG.md`: For each commit since the last release commit `Release vX.Y.Z`, added a changelog entry in the correct section linking to the author and PR in this format?
    ```
    ([@GithubUserName](https://github.com/GithubUserName) in [#PRNumber](https://github.com/Empty2k12/influxdb-rust/pull/PRNumber))
    ```
- [ ] `CHANGELOG.md`: At the bottom, changed the unreleased section link to `NEWVERSIONNUMBER...HEAD` and created a new link for the current release?
- [ ] `influxdb/lib.rs`: Changed the version number mentioned in the doc-comment to the new version number?
- [ ] `influxdb/lib.rs`: If the release contains any new features that should be mentioned in the Github Readme, are they listed in the doc-comment?
- [ ] `terminal`: Updated README with `cargo readme -r influxdb -t ../README.tpl > README.md`?
- [ ] `terminal`: Verified there are no errors with clippy `cargo clippy --all-targets --all-features -- -D warnings`?

## Releasing

1) `git add .` and `git commit -m "Release vX.Y.Z"`
2) `git tag vX.Y.Z`
3) `git push origin master` and `git push --tags`
4) `(cd influxdb-derive/ && cargo publish)`
5) `(cd influxdb/ && cargo publish)`
6) Create a Release in the Github Web UI
