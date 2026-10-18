# Working on spl-asdlc-sandbox

This project runs SPL-ASDLC. Before any work, read `.spl-asdlc/process.md` and follow it: it is the procedure of every ticket, pull request and release here, and `.spl-asdlc/config.toml` says which of its steps this project has. SPL-ASDLC delivers that page at the version this repository's workflows pin; it is never edited here, and CI refuses a pull request that changes it. If anything below contradicts it, the page wins.

## This project

- Build and test with `cargo test`.
- `SCENARIO.md` is the scenario the method is exercised with; keep it in step with the flow.
