# Qualification

Criticality uses [zcheck](https://github.com/zsumz/zcheck) as its canonical
local gate.

```sh
cargo +1.96.1 install zcheck --version 0.0.1 --locked
cargo fetch --locked
zcheck
```

The checked-in `zcheck.toml` is the qualification graph. It covers architecture
and repository shape, Rust formatting and static analysis, host and
`thumbv7em-none-eabi` `no_std` builds, behavioral tests, rustdoc, and the
published-crate boundary. Hosted CI runs the same graph.

## Package proof

The package gate compares the crate archive with `package-contents.txt`, then
compiles a fresh `no_std` consumer whose manifest names only Criticality. The
consumer imports `ByteCount` and `Retained` from the Criticality root; it cannot
rely on the internal `ByteBudget` owner.

The two guides under `docs/` are repository documentation rather than crate
archive contents. README links use absolute GitHub URLs so they also work when
the README is rendered on a package registry.

## Repository shape

The repository remains one crate. `scripts/structure-check` permits the root
README and exactly the two purpose-named guides under `docs/`; additional
Markdown must be added deliberately to that contract.

## Toolchains

The library's minimum supported Rust version is 1.88. The qualification graph
currently runs with Rust 1.96.1 or newer.
