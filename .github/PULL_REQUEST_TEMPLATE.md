## Description

{ describe your changes here }

### Checklist
- [ ] Formatted code using `cargo fmt --all`
- [ ] Linted code using clippy
  - [ ] surf based features `cd influxdb;cargo clippy --all-targets --no-default-features --features derive,use-serde,curl-client,h1-client,h1-client-rustls,hyper-client,wasm-client -- -D warnings;cd ..`
  - [ ] reqwest based features `cargo clippy --all-targets --features derive,use-serde,reqwest-client,reqwest-client-rustls -- -D warnings`
- [ ] Updated README.md using `cargo readme -r influxdb -t ../README.tpl > README.md`
- [ ] Reviewed the diff. Did you leave any print statements or unnecessary comments?
- [ ] Any unfinished work that warrants a separate issue captured in an issue with a TODO code comment
