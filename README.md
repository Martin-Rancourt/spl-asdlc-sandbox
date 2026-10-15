# spl-asdlc-sandbox

The end-to-end scenario, step by step: [`SCENARIO.md`](SCENARIO.md).

The sandbox where the [SPL-ASDLC](https://github.com/Martin-Rancourt/spl-asdlc) flow is exercised end to end, before each release of the bridge, against a **staging** build of it: the latest `main` of spl-asdlc. A flow bug found here never blocks its own fix, since spl-asdlc does not run its own flow.

What it holds:

- A tiny Rust crate (`src/lib.rs`, one function and its unit tests), so that a ticket has something real to change and CI has something real to run.
- The callers of the method's reusable workflows (`.github/workflows/`), every one at `Martin-Rancourt/spl-asdlc/...@main`: the sandbox always runs the method's latest `main`, never a released tag.
- `.spl-asdlc/config.toml`: the light configuration, no PR review and a release cycle with no QA angle, so that a merged ticket waits To Deploy and a release cycle is cut, gated and published. It carries the method's shape only; the identities of the installation (repository, owner, tracker team, logins) are in the instance overlay on the bridge's host.
- `CHANGELOG.md` and the version in `Cargo.toml`: the release cut's `spl-asdlc release prepare` (the staging build) computes each version from the commits since the last release, tagged `v<X.Y.Z>`, and writes its CHANGELOG section, which is also the GitHub release's notes.
- `.spl-asdlc/business-rules/`: the business rules, one file per rule (none yet).
- `SCENARIO.md`: the end-to-end scenario, step by step, with what the bridge must do at each one.

Nothing is published from here: the publication job of a release only prints the version.

## Develop

```sh
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```
