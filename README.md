<!-- markdownlint-disable MD033 MD041 -->

<img src="https://kura.pro/libyml/images/logos/libyml.svg"
alt="LibYML logo" width="66" align="right" />

<!-- markdownlint-enable MD033 MD041 -->

# LibYML (a fork of unsafe-libyaml)

[![GitHub][github-badge]][06]
[![Crates.io][crates-badge]][07]
[![lib.rs][libs-badge]][06]
[![Docs.rs][docs-badge]][08]
[![Codecov][codecov-badge]][09]
[![Build Status][build-badge]][10]

This library is a fork of [unsafe-libyaml][01], a version of [libyaml][02]
translated from C to unsafe Rust with the assistance of [c2rust][03].

This project, has been renamed to [LibYML][00] for simplicity and to avoid
confusion with the original [unsafe-libyaml][01] crate which is now
archived and no longer maintained.

## Credits and Acknowledgements

This library is a fork of the excellent work done by [David Tolnay][04] and the
maintainers of the [unsafe-libyaml][01] library.

LibYML has now evolved into a separate library with its own goals and direction
in mind and does not intend to replace the original unsafe-libyaml crate.

If you are currently using unsafe-libyaml in your projects, we recommend
carefully evaluating your requirements and considering the stability and
maturity of the original library as well as looking at the features and
improvements offered by other libraries in the Rust ecosystem.

I would like to express my sincere gratitude to [David Tolnay][04] and the
[unsafe-libyaml][01] and [libyaml][02] maintainers for their valuable
contributions to the Rust and C programming communities.

```toml
[dependencies]
libyml = "0.0.4"
```

Release notes are available under [GitHub releases][05].

### Rust Version Compatibility

This library is compatible with Rust 1.60 and above.

## Features

- **Serialization and Deserialization**: LibYML provides easy-to-use APIs for
  serializing Rust structs and enums into YAML format and deserializing YAML
  data into Rust types.
- **Custom Struct and Enum Support**: The library allows you to work with
  custom structs and enums, enabling seamless serialization and deserialization
  of your own data types.
- **Error Handling**: LibYML provides comprehensive error handling mechanisms,
  including detailed error messages and the ability to handle and recover from
  parsing and emission errors.
- **Streaming Support**: The library supports streaming of YAML data, allowing
  you to efficiently process large YAML documents incrementally.
- **Alias and Anchor Support**: LibYML handles YAML aliases and anchors,
  enabling you to work with complex YAML structures that involve references and
  duplicated data.
- **Tag Handling**: The library provides support for custom tags, allowing you
  to serialize and deserialize YAML data with specific type information.
- **Configurable Emitter**: LibYML allows you to customize the emitter
  settings, such as indentation, line width, and scalar style, to generate YAML
  output according to your preferences.
- **Extensive Documentation**: The library comes with detailed documentation
  and examples, making it easy to get started and understand how to use its
  various features effectively.
- **Safety and Efficiency**: LibYML is designed with safety and efficiency in
  mind. It minimizes the use of unsafe code and provides an interface that
  helps prevent common pitfalls and errors.

## Examples

To get started with LibYML, you can use the examples provided in the
`examples` directory of the project.

LibYML provides a set of comprehensive examples to demonstrate its usage and
capabilities. The examples cover various aspects of the library, including
initializing and deleting parsers, setting input strings, memory management,
string manipulation, and more.

To run the examples, clone the repository and navigate to the examples
directory. Each example is contained in a separate file, focusing on specific
functionalities or use cases.

For instance, to run all the example demonstrating the usage of the LibYML
APIs, you can execute the following command:

```shell
cargo run --example example
```

The command will execute all the examples code, demonstrating all the features
and use cases of the LibYML library.

Here are a few notable examples you can also run individually:

### 1) The `apis` Example

The `api.rs` file provides a set of low-level API functions for working with
YAML data in the LibYML library. The `apis` example showcases the usage of
these low-level APIs covering functions such as:

- `yaml_parser_initialize` - Initialize a YAML parser,
- `yaml_parser_delete` - Delete a YAML parser,
- `yaml_parser_set_input_string` - Set the input string for the parser,
- `yaml_malloc` - Allocate memory,
- `yaml_realloc` - Reallocate memory,
- `yaml_free` - Free memory,
- `yaml_strdup` - Duplicate a string,
- `yaml_string_extend` - Extend a string,
- `yaml_string_join` - Join strings,

You can run the `apis` example using the following command:

```rust
cargo run --example apis
```

## License

[MIT license](LICENSE-MIT), same as libyaml.

[00]: https://libyml.com
[01]: https://github.com/dtolnay/unsafe-libyaml
[02]: https://github.com/yaml/libyaml/tree/2c891fc7a770e8ba2fec34fc6b545c672beb37e6
[03]: https://github.com/immunant/c2rust
[04]: https://github.com/dtolnay
[05]: https://github.com/sebastienrousseau/libyml/releases
[06]: https://github.com/sebastienrousseau/libyml
[07]: https://crates.io/crates/libyml
[08]: https://docs.rs/libyml
[09]: https://codecov.io/gh/sebastienrousseau/libyml
[10]: https://github.com/sebastienrousseau/libyml/actions?query=branch%3Amaster
[build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/libyml/release.yml?branch=master&style=for-the-badge&logo=github
[codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/libyml?style=for-the-badge&token=yc9s578xIk&logo=codecov
[crates-badge]: https://img.shields.io/crates/v/libyml.svg?style=for-the-badge&color=fc8d62&logo=rust
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.4-orange.svg?style=for-the-badge
[docs-badge]: https://img.shields.io/badge/docs.rs-libyml-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
[github-badge]: https://img.shields.io/badge/github-sebastienrousseau/libyml-8da0cb?style=for-the-badge&labelColor=555555&logo=github
