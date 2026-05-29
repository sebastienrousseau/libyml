// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! # libyml examples (deprecation shim)
//!
//! `libyml` is deprecated; the example below exercises the public
//! surface that the 0.0.6 shim continues to provide. The original
//! example suite included demonstrations of the now-removed
//! `libyml::memory::*` and `libyml::string::*` modules — those
//! sub-blocks are kept as comments in `examples/apis/main.rs` with
//! the Rust-native replacements, so you can see the before/after
//! diff. See `MIGRATION.md` for the upstream equivalents.
//!
//! Run with: `cargo run --example example`.

#![allow(deprecated, missing_docs)]

use core::mem::MaybeUninit;
use core::ptr;
use libyml::success::is_success;
use libyml::{
    yaml_emitter_delete, yaml_emitter_emit, yaml_emitter_initialize,
    yaml_emitter_set_output_string, yaml_mapping_end_event_initialize,
    yaml_mapping_start_event_initialize, yaml_scalar_event_initialize,
    yaml_stream_end_event_initialize,
    yaml_stream_start_event_initialize, YamlBlockMappingStyle,
    YamlEmitterT, YamlEventT, YamlPlainScalarStyle, YamlUtf8Encoding,
};

/// `apis::main` mirrors the historical
/// `libyml ≤ 0.0.5` `examples/apis/main.rs` aggregator — the parts
/// that still work are kept; the removed slabs are documented inline.
mod apis;

fn main() {
    apis::main::main();
    emit_simple_document();
}

fn emit_simple_document() {
    unsafe {
        let mut emitter = MaybeUninit::<YamlEmitterT>::uninit();
        assert!(is_success(
            yaml_emitter_initialize(emitter.as_mut_ptr()).ok
        ));
        let mut emitter = emitter.assume_init();

        let mut buf = [0u8; 128];
        let mut size_written: u64 = 0;
        yaml_emitter_set_output_string(
            &mut emitter,
            buf.as_mut_ptr(),
            buf.len() as u64,
            &mut size_written,
        );

        emit(&mut emitter, |ev| {
            yaml_stream_start_event_initialize(ev, YamlUtf8Encoding).ok
        });
        emit(&mut emitter, |ev| {
            unsafe_libyaml::yaml_document_start_event_initialize(
                ev,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                true,
            )
            .ok
        });
        emit(&mut emitter, |ev| {
            yaml_mapping_start_event_initialize(
                ev,
                ptr::null_mut(),
                ptr::null_mut(),
                true,
                YamlBlockMappingStyle,
            )
            .ok
        });
        let key = b"hello\0";
        emit(&mut emitter, |ev| {
            yaml_scalar_event_initialize(
                ev,
                ptr::null_mut(),
                ptr::null_mut(),
                key.as_ptr(),
                (key.len() - 1) as i32,
                true,
                true,
                YamlPlainScalarStyle,
            )
            .ok
        });
        let val = b"world\0";
        emit(&mut emitter, |ev| {
            yaml_scalar_event_initialize(
                ev,
                ptr::null_mut(),
                ptr::null_mut(),
                val.as_ptr(),
                (val.len() - 1) as i32,
                true,
                true,
                YamlPlainScalarStyle,
            )
            .ok
        });
        emit(&mut emitter, |ev| {
            yaml_mapping_end_event_initialize(ev).ok
        });
        emit(&mut emitter, |ev| {
            unsafe_libyaml::yaml_document_end_event_initialize(ev, true)
                .ok
        });
        emit(&mut emitter, |ev| {
            yaml_stream_end_event_initialize(ev).ok
        });

        let out = core::str::from_utf8(&buf[..size_written as usize])
            .unwrap();
        println!("✅ emitter produced:\n{out}");

        yaml_emitter_delete(&mut emitter);
    }
}

unsafe fn emit(
    emitter: *mut YamlEmitterT,
    init: impl FnOnce(*mut YamlEventT) -> bool,
) {
    let mut ev = MaybeUninit::<YamlEventT>::uninit();
    assert!(is_success(init(ev.as_mut_ptr())));
    assert!(is_success(yaml_emitter_emit(emitter, ev.as_mut_ptr()).ok));
}
