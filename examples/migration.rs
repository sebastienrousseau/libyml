// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `libyml` is deprecated. This example shows the two valid paths
// in 0.0.6:
//
//  1. (Recommended for low-level callers) Switch directly to
//     `unsafe-libyaml`, the upstream this crate was forked from:
//       cargo remove libyml
//       cargo add unsafe-libyaml
//       sed -i 's/libyml/unsafe_libyaml/g' src/**/*.rs
//       # then rename PascalCase types/consts to snake_case /
//       # SCREAMING_SNAKE_CASE (see MIGRATION.md).
//
//  2. (Stop-gap) Keep `libyml = "0.0.6"`; every call below forwards
//     to `unsafe-libyaml` through the shim. The compiler emits a
//     deprecation warning for each `libyml::*` usage, marking the
//     call sites for migration.
//
// Run with: `cargo run --example migration`

#![allow(deprecated)]

use core::mem::MaybeUninit;
use libyml::success::is_success;
use libyml::{
    yaml_parser_delete, yaml_parser_initialize, yaml_parser_parse,
    yaml_parser_set_input_string, YamlEventT, YamlNoEvent, YamlParserT,
    YamlStreamEndEvent,
};

fn main() {
    let yaml = b"title: MyApp\nport: 8080\n";

    unsafe {
        let mut parser = MaybeUninit::<YamlParserT>::uninit();
        assert!(is_success(
            yaml_parser_initialize(parser.as_mut_ptr()).ok
        ));
        let mut parser = parser.assume_init();

        yaml_parser_set_input_string(
            &mut parser,
            yaml.as_ptr(),
            yaml.len() as u64,
        );

        let mut event_count = 0_u32;
        loop {
            let mut event = MaybeUninit::<YamlEventT>::uninit();
            if !is_success(
                yaml_parser_parse(&mut parser, event.as_mut_ptr()).ok,
            ) {
                eprintln!("parse failed");
                break;
            }
            let event = event.assume_init();
            event_count += 1;
            if event.type_ == YamlStreamEndEvent
                || event.type_ == YamlNoEvent
            {
                break;
            }
        }
        println!(
            "parsed {event_count} events from {} bytes",
            yaml.len()
        );

        yaml_parser_delete(&mut parser);
    }
}
