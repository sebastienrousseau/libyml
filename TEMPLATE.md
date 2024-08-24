<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/libyml/images/logos/libyml.svg"
alt="LibYML logo" width="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# LibYML (a fork of unsafe-libyaml)

[![Made With Love][made-with-rust]][10]
[![Crates.io][crates-badge]][06]
[![lib.rs][libs-badge]][11]
[![Docs.rs][docs-badge]][07]
[![Codecov][codecov-badge]][08]
[![Build Status][build-badge]][09]
[![GitHub][github-badge]][05]

LibYML is a Rust library for working with YAML data, forked from [unsafe-libyaml][01]. It offers a safe and efficient interface for parsing, emitting, and manipulating YAML data.

## Features

- **Serialization and Deserialization**: Easy-to-use APIs for serializing Rust structs and enums to YAML and vice versa.
- **Custom Struct and Enum Support**: Seamless serialization and deserialization of custom data types.
- **Comprehensive Error Handling**: Detailed error messages and recovery mechanisms.
- **Streaming Support**: Efficient processing of large YAML documents.
- **Alias and Anchor Support**: Handling of complex YAML structures with references.
- **Tag Handling**: Support for custom tags and type-specific serialization.
- **Configurable Emitter**: Customizable YAML output generation.
- **Extensive Documentation**: Detailed docs and examples for easy onboarding.
- **Safety and Efficiency**: Minimized unsafe code with an interface designed to prevent common pitfalls.

[01]: https://github.com/dtolnay/unsafe-libyaml
[05]: https://github.com/sebastienrousseau/libyml
[06]: https://crates.io/crates/libyml
[07]: https://docs.rs/libyml
[08]: https://codecov.io/gh/sebastienrousseau/libyml
[09]: https://github.com/sebastienrousseau/libyml/actions?query=branch%3Amaster
[10]: https://www.rust-lang.org/
[11]: https://lib.rs/crates/libyml

[build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/libyml/release.yml?branch=master&style=for-the-badge&logo=github "Build Status"
[codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/libyml?style=for-the-badge&logo=codecov&token=yc9s578xIk "Code Coverage"
[crates-badge]: https://img.shields.io/crates/v/libyml.svg?style=for-the-badge&color=fc8d62&logo=rust "View on Crates.io"
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.5-orange.svg?style=for-the-badge "View on lib.rs"
[docs-badge]: https://img.shields.io/badge/docs.rs-libyml-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs "View Documentation"
[github-badge]: https://img.shields.io/badge/github-sebastienrousseau/libyml-8da0cb?style=for-the-badge&labelColor=555555&logo=github "View on GitHub"
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust'

## Changelog 📚
