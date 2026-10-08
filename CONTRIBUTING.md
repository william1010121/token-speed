# Contributing

Bug reports with a minimal synthetic example are especially useful. Explain the expected usage counts / timing and the actual result. Never attach full personal session logs or cache databases.

## Development

Requires Rust 1.92+ and a C compiler. From the repository root:

```bash
cargo run -- --help
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

Before changing parser or metric behavior, read [the measurement contract](docs/metrics.md). Add a regression fixture for consequential parsing, timing, cache or terminal behavior. Avoid tests that merely duplicate implementation details.

Keep pull requests focused. Describe the concrete before / after behavior and relevant verification. For TUI changes, check empty data, narrow windows and keyboard navigation; include a synthetic screenshot when it helps review. The [repo instructions](AGENTS.md) also guide Codex reviews.

## Screenshots

The test suite can export four native TestBackend frames as SVG:

```bash
TOKEN_SPEED_SNAPSHOT=/tmp/token-speed.svg cargo test --locked
```

This writes the overview plus `-timeline.svg`, `-models.svg` and `-requests.svg` variants. Render them to PNG with an SVG renderer for README previews. The runtime does not depend on that renderer.

## Releases

Update the package version, lockfile, changelog and versioned README commands together. After CI passes, push an annotated `vX.Y.Z` tag matching `Cargo.toml`. The release workflow runs tests, builds macOS / Linux binaries on native runners, packages them with the license and README, calculates SHA-256 checksums, and publishes a GitHub release. Release workflows also support manual retry for an existing tag.
