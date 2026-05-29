<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/libyml/images/logos/libyml.svg"
alt="libyml logo" width="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# libyml — Deprecated

[![Crates.io][crates-badge]][07] [![Docs.rs][docs-badge]][08] [![Migration][migration-badge]][13]

⚠️ **This crate is unmaintained.** The `0.0.6` release is a thin
compatibility shim that forwards every call to the upstream
`unsafe-libyaml` (the Rust translation of C libyaml that `libyml`
was originally forked from) so existing call sites keep working
while you migrate to a maintained alternative of your choice.

- [`README.md`][06] — overview, install, migration paths
- [`MIGRATION.md`][13] — full mapping tables per destination crate

[06]: https://github.com/sebastienrousseau/libyml
[07]: https://crates.io/crates/libyml
[08]: https://docs.rs/libyml
[13]: https://github.com/sebastienrousseau/libyml/blob/master/MIGRATION.md
[crates-badge]: https://img.shields.io/crates/v/libyml.svg?style=for-the-badge&color=red&label=deprecated&logo=rust "Crates.io (deprecated)"
[docs-badge]: https://img.shields.io/badge/docs.rs-libyml-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs "Docs.rs"
[migration-badge]: https://img.shields.io/badge/migration-guide-66c2a5?style=for-the-badge "Migration guide"
