# Rust release procedure

The Rust workspace publishes eight libraries. The 0.1.0 release was published
on 2026-11-11:

```text
pulselink-core
pulselink-transport
pulselink-pattern
pulselink-protocol
pulselink-ble
pulselink-runtime
pulselink-ws
pulselink
```

`pulselink-integration-tests` has `publish = false`.

## Preflight

Run from `pulselink-rs` on a clean commit:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --workspace --no-default-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo deny check
./scripts/check-features.sh
./scripts/check-versions.sh
cargo package --workspace --list
cargo publish --workspace --dry-run
```

Linux does not need system D-Bus development headers: `pulselink-ble` enables
the vendored `libdbus-sys` build.

## Publishing

The first release must be uploaded with a crates.io API token that has
`publish-new` and `publish-update` permissions. Use the dependency order above,
waiting for each version to appear in the crates.io index before publishing the
next package.

crates.io rate-limits new crates (currently five new crates per ten-minute
window). If a publish returns HTTP 429, wait until the reported reset time and
retry the same crate. Do not change the version to work around a rate limit.

The manual GitHub workflow is:

```text
Actions -> Release -> Run workflow
publish = true
```

It accepts `CARGO_REGISTRY_TOKEN` from the protected `crates-io` environment.
For future releases, configure crates.io Trusted Publishing for every crate and
switch the workflow to `rust-lang/crates-io-auth-action@v1` with
`id-token: write`; the first release still has to be manual.

## After publishing

1. Spot-check all crate pages and docs.rs builds.
2. Verify a clean Cargo project can depend on `pulselink = "0.1.0"`.
3. Create and push `v0.1.0`.
4. Create the GitHub release from the `[0.1.0]` changelog entry.
5. Add maintainers with `cargo owner --add github:Chiloven945 <crate>`.

Published versions cannot be deleted. If a release is broken, yank it and publish
a patch version.
