// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Smoke tests for the `libyml 0.0.6` deprecation shim. The public
// surface here is a thin re-export of `unsafe-libyaml` with
// PascalCase type aliases restored; these tests just verify the
// re-exports resolve and round-trip on representative shapes so a
// downstream `cargo update -p libyml` does not break compilation
// or runtime behaviour for typical call sites.

#![allow(deprecated)]

use core::mem::MaybeUninit;
use core::ptr;
use libyml::success::{is_failure, is_success};
use libyml::{
    yaml_emitter_delete, yaml_emitter_emit, yaml_emitter_initialize,
    yaml_emitter_set_output_string, yaml_mapping_end_event_initialize,
    yaml_mapping_start_event_initialize, yaml_parser_delete,
    yaml_parser_initialize, yaml_parser_parse,
    yaml_parser_set_input_string, yaml_scalar_event_initialize,
    yaml_stream_end_event_initialize,
    yaml_stream_start_event_initialize, YamlBlockMappingStyle,
    YamlEmitterT, YamlEventT, YamlMappingEndEvent, YamlParserT,
    YamlPlainScalarStyle, YamlStreamEndEvent, YamlStreamStartEvent,
    YamlUtf8Encoding,
};

#[test]
fn parser_initialize_and_delete() {
    unsafe {
        let mut parser = MaybeUninit::<YamlParserT>::uninit();
        assert!(is_success(
            yaml_parser_initialize(parser.as_mut_ptr()).ok
        ));
        let mut parser = parser.assume_init();
        yaml_parser_delete(&mut parser);
    }
}

#[test]
fn parser_set_input_string_and_parse_event() {
    unsafe {
        let mut parser = MaybeUninit::<YamlParserT>::uninit();
        assert!(is_success(
            yaml_parser_initialize(parser.as_mut_ptr()).ok
        ));
        let mut parser = parser.assume_init();

        let input = b"key: value\n";
        yaml_parser_set_input_string(
            &mut parser,
            input.as_ptr(),
            input.len() as u64,
        );

        let mut event = MaybeUninit::<YamlEventT>::uninit();
        assert!(is_success(
            yaml_parser_parse(&mut parser, event.as_mut_ptr()).ok
        ));
        let event = event.assume_init();
        // First event should always be a stream-start under the
        // libyml-flavoured PascalCase const surface.
        assert!(matches!(event.type_, t if t == YamlStreamStartEvent));

        yaml_parser_delete(&mut parser);
    }
}

#[test]
fn emit_simple_mapping_round_trips() {
    // Build: --- {greeting: hello} → emit it → assert the bytes the
    // upstream wrote contain `greeting` and `hello`.
    let emitted: String;
    unsafe {
        let mut emitter = MaybeUninit::<YamlEmitterT>::uninit();
        assert!(is_success(
            yaml_emitter_initialize(emitter.as_mut_ptr()).ok
        ));
        let mut emitter = emitter.assume_init();

        let mut size_written: u64 = 0;
        let mut buf = [0u8; 256];
        yaml_emitter_set_output_string(
            &mut emitter,
            buf.as_mut_ptr(),
            buf.len() as u64,
            &mut size_written,
        );

        let mut ev = MaybeUninit::<YamlEventT>::uninit();
        assert!(is_success(
            yaml_stream_start_event_initialize(
                ev.as_mut_ptr(),
                YamlUtf8Encoding,
            )
            .ok
        ));
        assert!(is_success(
            yaml_emitter_emit(&mut emitter, ev.as_mut_ptr()).ok
        ));

        // Implicit document-start.
        let mut ev = MaybeUninit::<YamlEventT>::uninit();
        assert!(is_success(
            unsafe_libyaml::yaml_document_start_event_initialize(
                ev.as_mut_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                true,
            )
            .ok
        ));
        assert!(is_success(
            yaml_emitter_emit(&mut emitter, ev.as_mut_ptr()).ok
        ));

        let mut ev = MaybeUninit::<YamlEventT>::uninit();
        assert!(is_success(
            yaml_mapping_start_event_initialize(
                ev.as_mut_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                true,
                YamlBlockMappingStyle,
            )
            .ok
        ));
        assert!(is_success(
            yaml_emitter_emit(&mut emitter, ev.as_mut_ptr()).ok
        ));

        let key = b"greeting\0";
        let mut ev = MaybeUninit::<YamlEventT>::uninit();
        assert!(is_success(
            yaml_scalar_event_initialize(
                ev.as_mut_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                key.as_ptr(),
                (key.len() - 1) as i32,
                true,
                true,
                YamlPlainScalarStyle,
            )
            .ok
        ));
        assert!(is_success(
            yaml_emitter_emit(&mut emitter, ev.as_mut_ptr()).ok
        ));

        let val = b"hello\0";
        let mut ev = MaybeUninit::<YamlEventT>::uninit();
        assert!(is_success(
            yaml_scalar_event_initialize(
                ev.as_mut_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
                val.as_ptr(),
                (val.len() - 1) as i32,
                true,
                true,
                YamlPlainScalarStyle,
            )
            .ok
        ));
        assert!(is_success(
            yaml_emitter_emit(&mut emitter, ev.as_mut_ptr()).ok
        ));

        let mut ev = MaybeUninit::<YamlEventT>::uninit();
        assert!(is_success(
            yaml_mapping_end_event_initialize(ev.as_mut_ptr()).ok
        ));
        assert!(is_success(
            yaml_emitter_emit(&mut emitter, ev.as_mut_ptr()).ok
        ));
        // Mapping-end event constant resolves through the shim.
        let _: libyml::YamlEventTypeT = YamlMappingEndEvent;

        let mut ev = MaybeUninit::<YamlEventT>::uninit();
        assert!(is_success(
            unsafe_libyaml::yaml_document_end_event_initialize(
                ev.as_mut_ptr(),
                true,
            )
            .ok
        ));
        assert!(is_success(
            yaml_emitter_emit(&mut emitter, ev.as_mut_ptr()).ok
        ));

        let mut ev = MaybeUninit::<YamlEventT>::uninit();
        assert!(is_success(
            yaml_stream_end_event_initialize(ev.as_mut_ptr()).ok
        ));
        assert!(is_success(
            yaml_emitter_emit(&mut emitter, ev.as_mut_ptr()).ok
        ));
        // Stream-end event constant resolves through the shim.
        let _: libyml::YamlEventTypeT = YamlStreamEndEvent;

        emitted = core::str::from_utf8(&buf[..size_written as usize])
            .unwrap()
            .to_owned();
        yaml_emitter_delete(&mut emitter);
    }

    assert!(emitted.contains("greeting"));
    assert!(emitted.contains("hello"));
}

#[test]
fn type_aliases_resolve_at_libyml_paths() {
    // Compile-time check: the historical libyml PascalCase type and
    // const names still resolve at the crate root.
    #[allow(unused_imports)]
    use libyml::{
        YamlAnyScalarStyle, YamlBlockMappingStyle,
        YamlBlockSequenceStyle, YamlDoubleQuotedScalarStyle,
        YamlEmitterError, YamlEmitterT, YamlEventT,
        YamlFoldedScalarStyle, YamlLiteralScalarStyle, YamlMappingNode,
        YamlMemoryError, YamlPlainScalarStyle, YamlScalarNode,
        YamlScalarStyleT, YamlSequenceNode,
        YamlSingleQuotedScalarStyle, YamlTagDirectiveT,
        YamlUtf8Encoding, YamlVersionDirectiveT, YamlWriterError,
    };
}

#[test]
fn success_helpers_resolve() {
    assert!(is_success(true));
    assert!(is_failure(false));
}
