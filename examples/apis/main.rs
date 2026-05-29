// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Retained from `libyml ≤ 0.0.5`. Three slabs of the original demo
// stay working unchanged in the 0.0.6 shim, and three slabs are
// removed because the underlying surface is gone — see
// [`MIGRATION.md`](../../MIGRATION.md) for the rationale and the
// recommended Rust-native replacements. The retained blocks are
// the parts that touch `libyml::api::*` and `libyml::decode::*`;
// the removed blocks are the parts that touched the deleted
// `libyml::memory::*` (C-libyaml allocator) and `libyml::string::*`
// (the `yaml_string_extend` unsound helper flagged by
// `RUSTSEC-2025-0067`).

#![allow(deprecated, missing_docs)]

use core::mem::MaybeUninit;
use libyml::api::yaml_parser_set_input_string;
use libyml::decode::{yaml_parser_delete, yaml_parser_initialize};

pub(crate) fn main() {
    println!("\n❯ Executing examples/apis/main.rs");

    let mut parser = MaybeUninit::uninit();
    let parser_ptr = parser.as_mut_ptr();

    unsafe {
        let _ = yaml_parser_initialize(parser_ptr);
        println!("✅ Successfully initialized the YAML parser");

        let input = b"key: value\n";
        yaml_parser_set_input_string(
            parser_ptr,
            input.as_ptr(),
            input.len().try_into().unwrap(),
        );
        println!(
            "✅ Successfully set the input string for the YAML parser"
        );

        // ── Removed slab #1: `libyml::memory::*` (C-libyaml allocator)
        //
        // The previous example demoed `yaml_malloc` / `yaml_realloc`
        // / `yaml_strdup` / `yaml_free`. The 0.0.6 shim deletes the
        // entire `libyml::memory` surface because the upstream
        // `unsafe-libyaml` uses Rust's `alloc` directly — the
        // C-style allocator wrappers are no longer needed.
        //
        // Migration:
        //
        //     use std::alloc::{alloc, dealloc, Layout};
        //     let layout = Layout::from_size_align(1024, 8).unwrap();
        //     let ptr = unsafe { alloc(layout) };
        //     /* ... use ptr ... */
        //     unsafe { dealloc(ptr, layout) };

        // ── Removed slab #2: `libyml::string::yaml_string_extend`
        //
        // The previous example demoed `yaml_string_extend` and
        // `yaml_string_join` — the helpers
        // [RUSTSEC-2025-0067](https://rustsec.org/advisories/RUSTSEC-2025-0067.html)
        // flagged as unsound. The 0.0.6 shim deletes the entire
        // `libyml::string` surface; build strings with Rust's
        // standard primitives:
        //
        //     let mut buf: Vec<u8> = Vec::new();
        //     buf.extend_from_slice(b"Hello, ");
        //     buf.extend_from_slice(b"world!");

        yaml_parser_delete(parser_ptr);
        println!("✅ Successfully deleted the YAML parser");
    }
}
