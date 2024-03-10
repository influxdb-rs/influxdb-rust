## Description

{ describe your changes here }

### Checklist
- [ ] Formatted code using `cargo fmt --all`
- [ ] Linted code using clippy
  - [ ] with reqwest feature: `cargo clippy --manifest-path influxdb/Cargo.toml --all-targets --no-default-features --features serde,derive,reqwest-client-rustls -- -D warnings`
  - [ ] with surf feature: `cargo clippy --manifest-path influxdb/Cargo.toml --all-targets --no-default-features --features serde,derive,hyper-client -- -D warnings`
- [ ] Updated README.md using `cargo doc2readme -p influxdb --expand-macros`
- [ ] Reviewed the diff. Did you leave any print statements or unnecessary comments?
- [ ] Any unfinished work that warrants a separate issue captured in an issue with a TODO code comment
