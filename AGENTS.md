# Codex Agent Instructions for `barter-rs`

Barter is an ecosystem of Rust libraries for building high-performance live,
paper and back testing trading systems.

**Fast**: Native Rust with minimal allocations and direct index lookups.
**Robust**: Strongly typed, thread safe, and extensively tested.
**Customisable**: Plug-and-play `Strategy` and `RiskManager` components.
**Scalable**: Multithreaded modular architecture using Tokio and memory
efficient data structures.

See the [`Barter`](https://crates.io/crates/barter) family on crates.io for
comprehensive documentation.

---

These guidelines apply to the entire repository. They may be overridden by
nested `AGENTS.md` files in subdirectories.

## Contributor Guide

### Dev Environment
* Install Rust with [rustup](https://rustup.rs/). The project targets the
  2024 edition and works on nightly or the latest stable release.
* Clone and build:
  ```sh
  git clone <repo_url>
  cd barter-rs
  cargo build --workspace
  ```
* Format & lint:
  ```sh
  cargo fmt --all
  cargo clippy --all-targets -- -D warnings
  cargo check
  ```
* Run examples:
  ```sh
  cargo run --example <example_name>
  ```
* Generate docs locally with `cargo doc --open`.

### Testing
* Run the full test suite:
  ```sh
  cargo test --workspace
  ```
* Coverage via `cargo tarpaulin` if desired.
* GitHub Actions CI checks builds, formatting, lint and tests.
* Tests are required for all new features and bug fixes.

### Pull Requests
* **Title:** `[crate] <Short Description>` (e.g. `[barter-data] Add Bybit stream support`)
* **Description:** Summarise the problem, solution and link relevant issues.
  Include examples for new features.
* **Format:** ensure `cargo fmt` and `cargo clippy` pass with no warnings and
  public APIs are documented with `///` comments.
* **Tests:** add or extend unit/integration tests. All tests must pass.
* **Acceptance:** PRs are merged once CI passes and the code follows project
  style.

## Additional Notes
* The Rust edition is defined as `2024` in `rustfmt.toml`.
* For details on AGENTS.md behaviour see
  <https://platform.openai.com/docs/codex/overview#using-agents-md>.

