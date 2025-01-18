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

LibYML is a robust Rust library for parsing, emitting and manipulating YAML data. Built upon the foundation of [unsafe-libyaml][01], it provides a safe and efficient interface whilst maintaining high performance.

## Key Features

- **Safe and Efficient Processing**
  - Memory-safe abstractions with minimal unsafe code
  - Zero-copy parsing for optimal performance
  - Protection against common pitfalls and vulnerabilities
  - Efficient memory utilisation and management

- **Comprehensive Data Handling**
  - Full YAML 1.2 specification support
  - Seamless serialisation and deserialisation of Rust structs and enums
  - Custom struct and enum type support
  - Complex data structures with aliases and anchors
  - Custom tag handling and type-specific serialisation

- **Flexible Configuration**
  - Customisable YAML output generation
  - Configurable emitter settings
  - Multiple encoding support (UTF-8, UTF-16)
  - Streaming support for large documents

- **Robust Error Management**
  - Comprehensive error handling
  - Detailed error messages
  - Recovery mechanisms
  - Type-safe failure handling

- **Developer Experience**
  - Easy-to-use serialisation APIs
  - Extensive documentation and examples
  - Clear, consistent interface design
  - Straightforward onboarding process
  - Type-safe API with helpful compiler feedback

- **Performance Optimised**
  - Minimal allocations
  - Efficient memory reuse
  - Optimised parsing algorithms
  - Zero-cost abstractions over low-level operations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
libyml = "0.0.6"
```

## Usage

Here's a quick example on how to use LibYML to parse a YAML string:

```rust
use core::mem::MaybeUninit;
use libyml::{
    success::is_success,
    yaml_parser_delete,
    yaml_parser_initialize,
    yaml_parser_parse,
    yaml_parser_set_input_string,
    YamlEventT,
    YamlParserT,
};

fn main() {
    unsafe {
        let mut parser = MaybeUninit::<YamlParserT>::uninit();
        if is_success(yaml_parser_initialize(parser.as_mut_ptr())) {
            let mut parser = parser.assume_init();
            let yaml = "{key1: value1, key2: [item1, item2]}";
            yaml_parser_set_input_string(
                &mut parser,
                yaml.as_ptr(),
                yaml.len() as u64,
            );
            let mut event = MaybeUninit::<YamlEventT>::uninit();
            let result = yaml_parser_parse(&mut parser, event.as_mut_ptr());
            if is_success(result) {
                // Process the event here
            } else {
                // Failed to parse YAML
            }
            yaml_parser_delete(&mut parser);
        } else {
            // Failed to initialize parser
        }
    }
}
```

## Documentation

For full API documentation, please visit [https://doc.libyml.com/libyml/][03] or [https://docs.rs/libyml][07].

## Rust Version Compatibility

Compiler support: requires rustc 1.56.0+

## Contributing

Contributions are welcome! If you'd like to contribute, please feel free to submit a Pull Request on [GitHub][05].

## Credits and Acknowledgements

LibYML is a fork of the work done by [David Tolnay][04] and the maintainers of [unsafe-libyaml][01]. While it has evolved into a separate library, we express our sincere gratitude to them as well as the [libyaml][02] maintainers for their contributions to the Rust and C programming communities.

## License

[MIT license](LICENSE-MIT), same as libyaml.

[00]: https://libyml.com
[01]: https://github.com/dtolnay/unsafe-libyaml
[02]: https://github.com/yaml/libyaml/tree/2c891fc7a770e8ba2fec34fc6b545c672beb37e6
[03]: https://doc.libyml.com/libyml/
[04]: https://github.com/dtolnay
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
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.6-orange.svg?style=for-the-badge "View on lib.rs"
[docs-badge]: https://img.shields.io/badge/docs.rs-libyml-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs "View Documentation"
[github-badge]: https://img.shields.io/badge/github-sebastienrousseau/libyml-8da0cb?style=for-the-badge&labelColor=555555&logo=github "View on GitHub"
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust'
