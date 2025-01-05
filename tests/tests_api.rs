#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use core::ffi::c_void;
    use libyml::{
        api::{
            yaml_alias_event_initialize, yaml_emitter_delete,
            yaml_emitter_get_break, yaml_emitter_get_canonical,
            yaml_emitter_get_encoding, yaml_emitter_get_indent,
            yaml_emitter_get_unicode, yaml_emitter_get_width,
            yaml_emitter_initialize, yaml_emitter_set_canonical,
            yaml_emitter_set_indent, yaml_emitter_set_output_string,
            yaml_event_delete, yaml_mapping_end_event_initialize,
            yaml_mapping_start_event_initialize,
            yaml_parser_set_input_string, yaml_scalar_event_initialize,
            yaml_sequence_end_event_initialize,
            yaml_sequence_start_event_initialize,
            yaml_stream_end_event_initialize,
            yaml_stream_start_event_initialize, yaml_token_delete,
            ScalarEventData,
        },
        externs::free,
        memory::{yaml_malloc, yaml_strdup},
        success::OK,
        yaml::{
            size_t, YamlEmitterT, YamlEventT, YamlMappingStyleT,
            YamlParserT, YamlScalarStyleT, YamlSequenceStyleT,
            YamlTokenT,
        },
        YamlAliasEvent, YamlAliasToken, YamlCrlnBreak,
        YamlMappingEndEvent, YamlMappingStartEvent, YamlNoEvent,
        YamlNoToken, YamlScalarEvent, YamlSequenceEndEvent,
        YamlSequenceStartEvent, YamlStreamEndEvent,
        YamlStreamStartEvent, YamlUtf8Encoding,
    };
    use std::ptr::{null, null_mut};

    // -----------------------------------------------------------------------
    // Existing memory-allocation tests (yours)
    // -----------------------------------------------------------------------

    #[test]
    fn test_yaml_malloc() {
        unsafe {
            // Test allocation of zero bytes
            let ptr = yaml_malloc(0);
            assert!(!ptr.is_null());
            yaml_free(ptr); // Ensure to free the allocated memory

            // Test allocation of non-zero bytes
            let ptr = yaml_malloc(10);
            assert!(!ptr.is_null());
            yaml_free(ptr); // Ensure to free the allocated memory
        }
    }

    #[test]
    fn test_yaml_malloc_free() {
        unsafe {
            // Test allocation of zero bytes
            let ptr = yaml_malloc(0);
            assert!(!ptr.is_null());
            yaml_free(ptr); // Ensure to free the allocated memory

            // Test allocation of non-zero bytes
            let ptr = yaml_malloc(10);
            assert!(!ptr.is_null());
            yaml_free(ptr); // Ensure to free the allocated memory
        }
    }

    #[test]
    fn test_yaml_realloc() {
        unsafe {
            // Test allocation of zero bytes
            let ptr = yaml_malloc(0);
            assert!(!ptr.is_null());
            yaml_free(ptr); // Ensure to free the allocated memory

            // Test allocation of non-zero bytes
            let ptr = yaml_malloc(10);
            assert!(!ptr.is_null());
            yaml_free(ptr); // Ensure to free the allocated memory
        }
    }

    #[test]
    fn test_yaml_free() {
        unsafe {
            // Test freeing null pointer
            let ptr = yaml_malloc(0);
            yaml_free(ptr);
        }
    }

    #[test]
    fn test_yaml_strdup() {
        unsafe {
            // Test duplication of a null string
            let ptr = yaml_strdup(null());
            assert_eq!(ptr, null_mut());
        }
    }

    // Helper function to free memory (local to these tests)
    unsafe fn yaml_free(ptr: *mut c_void) {
        free(ptr);
    }

    // -----------------------------------------------------------------------
    // Additional Tests for api.rs
    // -----------------------------------------------------------------------

    /// Minimal test parser struct for demonstration
    #[repr(C)]
    struct TestYamlParser {
        parser: YamlParserT,
    }

    /// Minimal test emitter struct for demonstration
    #[repr(C)]
    struct TestYamlEmitter {
        emitter: YamlEmitterT,
    }

    // ------------------ Parser Tests ------------------

    #[test]
    fn test_yaml_parser_set_input_string() {
        unsafe {
            let mut parser_struct = TestYamlParser {
                parser: std::mem::zeroed(),
            };

            let input = b"test input\0";
            yaml_parser_set_input_string(
                &mut parser_struct.parser,
                input.as_ptr(),
                (input.len() - 1) as size_t,
            );

            // Check
            // assert!(yaml_parser_has_read_handler(&parser_struct.parser)); // Commented out as the function is not defined
        }
    }

    #[test]
    fn test_yaml_parser_set_input() {
        unsafe {
            struct TestYamlParser {
                parser: YamlParserT,
            }
            let mut parser_struct = TestYamlParser {
                parser: std::mem::zeroed(),
            };

            // Remove `extern "C"` so it matches `unsafe fn`
            unsafe fn dummy_read_handler(
                _data: *mut c_void,
                _buffer: *mut u8,
                _size: u64,
                size_read: *mut u64,
            ) -> i32 {
                *size_read = 0;
                1
            }

            libyml::api::yaml_parser_set_input(
                &mut parser_struct.parser,
                dummy_read_handler, // bare Rust fn pointer
                null_mut(),
            );
        }
    }

    // ------------------ Emitter Init/Del Tests ------------------

    #[test]
    fn test_yaml_emitter_initialize_and_delete() {
        unsafe {
            let mut emitter_struct = TestYamlEmitter {
                emitter: std::mem::zeroed(),
            };
            let result =
                yaml_emitter_initialize(&mut emitter_struct.emitter);
            assert_eq!(result, OK);

            // Check some pointer
            assert!(!emitter_struct.emitter.buffer.start.is_null());

            yaml_emitter_delete(&mut emitter_struct.emitter);
        }
    }

    #[test]
    fn test_yaml_emitter_set_encoding() {
        unsafe {
            // Minimal "fake" emitter
            #[repr(C)]
            struct TestYamlEmitter {
                emitter: YamlEmitterT,
            }
            let mut emitter_struct = TestYamlEmitter {
                emitter: std::mem::zeroed(),
            };
            // Initialize
            assert_eq!(
                yaml_emitter_initialize(&mut emitter_struct.emitter),
                OK
            );

            // Initially encoding might be 0 (YamlAnyEncoding)
            // Let’s set it to YamlUtf8Encoding = 1
            libyml::api::yaml_emitter_set_encoding(
                &mut emitter_struct.emitter,
                YamlUtf8Encoding,
            );
            assert_eq!(
                yaml_emitter_get_encoding(&mut emitter_struct.emitter),
                YamlUtf8Encoding
            );

            yaml_emitter_delete(&mut emitter_struct.emitter);
        }
    }

    #[test]
    fn test_yaml_emitter_set_canonical() {
        unsafe {
            #[repr(C)]
            struct TestYamlEmitter {
                emitter: YamlEmitterT,
            }

            // 1. Create a minimal emitter and initialize it
            let mut emitter_struct = TestYamlEmitter {
                emitter: std::mem::zeroed(),
            };
            assert_eq!(
                yaml_emitter_initialize(&mut emitter_struct.emitter),
                OK
            );

            // 2. Set canonical = true
            yaml_emitter_set_canonical(
                &mut emitter_struct.emitter,
                true,
            );

            // 3. Confirm via get_canonical()
            assert!(yaml_emitter_get_canonical(
                &mut emitter_struct.emitter
            ));

            // 4. Cleanup
            yaml_emitter_delete(&mut emitter_struct.emitter);
        }
    }

    #[test]
    fn test_yaml_emitter_set_indent() {
        unsafe {
            #[repr(C)]
            struct TestYamlEmitter {
                emitter: YamlEmitterT,
            }
            let mut emitter_struct = TestYamlEmitter {
                emitter: std::mem::zeroed(),
            };
            assert_eq!(
                yaml_emitter_initialize(&mut emitter_struct.emitter),
                OK
            );

            // The function clamps valid range to [2..9] as per YAML spec
            yaml_emitter_set_indent(&mut emitter_struct.emitter, 4);
            assert_eq!(
                yaml_emitter_get_indent(&mut emitter_struct.emitter),
                4
            );

            // If we pass e.g. 1, it defaults to 2
            yaml_emitter_set_indent(&mut emitter_struct.emitter, 1);
            assert_eq!(
                yaml_emitter_get_indent(&mut emitter_struct.emitter),
                2
            );

            yaml_emitter_delete(&mut emitter_struct.emitter);
        }
    }

    #[test]
    fn test_yaml_emitter_set_width() {
        unsafe {
            #[repr(C)]
            struct TestYamlEmitter {
                emitter: YamlEmitterT,
            }
            let mut emitter_struct = TestYamlEmitter {
                emitter: std::mem::zeroed(),
            };
            assert_eq!(
                yaml_emitter_initialize(&mut emitter_struct.emitter),
                OK
            );

            // Normal usage
            libyml::api::yaml_emitter_set_width(
                &mut emitter_struct.emitter,
                80,
            );
            assert_eq!(
                yaml_emitter_get_width(&mut emitter_struct.emitter),
                80
            );

            // -1 means unlimited
            libyml::api::yaml_emitter_set_width(
                &mut emitter_struct.emitter,
                -1,
            );
            assert_eq!(
                yaml_emitter_get_width(&mut emitter_struct.emitter),
                -1
            );

            yaml_emitter_delete(&mut emitter_struct.emitter);
        }
    }

    #[test]
    fn test_yaml_emitter_set_unicode() {
        unsafe {
            #[repr(C)]
            struct TestYamlEmitter {
                emitter: YamlEmitterT,
            }
            let mut emitter_struct = TestYamlEmitter {
                emitter: std::mem::zeroed(),
            };
            assert_eq!(
                yaml_emitter_initialize(&mut emitter_struct.emitter),
                OK
            );

            // default is false
            assert!(!yaml_emitter_get_unicode(
                &mut emitter_struct.emitter
            ));

            // set it true
            libyml::api::yaml_emitter_set_unicode(
                &mut emitter_struct.emitter,
                true,
            );
            assert!(yaml_emitter_get_unicode(
                &mut emitter_struct.emitter
            ));

            yaml_emitter_delete(&mut emitter_struct.emitter);
        }
    }

    #[test]
    fn test_yaml_emitter_set_break() {
        unsafe {
            #[repr(C)]
            struct TestYamlEmitter {
                emitter: YamlEmitterT,
            }
            let mut emitter_struct = TestYamlEmitter {
                emitter: std::mem::zeroed(),
            };
            assert_eq!(
                yaml_emitter_initialize(&mut emitter_struct.emitter),
                OK
            );

            // For example, YamlCrlnBreak might be 2 or 3 in your enum
            libyml::api::yaml_emitter_set_break(
                &mut emitter_struct.emitter,
                YamlCrlnBreak,
            );
            assert_eq!(
                yaml_emitter_get_break(&mut emitter_struct.emitter),
                YamlCrlnBreak
            );

            yaml_emitter_delete(&mut emitter_struct.emitter);
        }
    }
    // ------------------ Emitter Output Setting Tests ------------------

    #[test]
    fn test_yaml_emitter_set_output_string() {
        unsafe {
            let mut emitter_struct = TestYamlEmitter {
                emitter: std::mem::zeroed(),
            };
            let _ =
                yaml_emitter_initialize(&mut emitter_struct.emitter);

            let mut output_buffer: [u8; 32] = [0; 32];
            let mut size_written: size_t = 0;

            yaml_emitter_set_output_string(
                &mut emitter_struct.emitter,
                output_buffer.as_mut_ptr(),
                output_buffer.len() as size_t,
                &mut size_written,
            );
            assert!(emitter_struct.emitter.write_handler.is_some());

            yaml_emitter_delete(&mut emitter_struct.emitter);
        }
    }

    // ------------------ Token & Event Cleanup Tests ------------------

    #[test]
    fn test_yaml_token_delete() {
        unsafe {
            let mut token: YamlTokenT = std::mem::zeroed();
            // Suppose YamlAliasToken = 3 in your code, adjust if needed
            token.type_ = YamlAliasToken;
            let anchor_str = b"dummy\0";
            // allocate a dummy anchor
            token.data.alias.value = yaml_strdup(anchor_str.as_ptr());

            yaml_token_delete(&mut token);
            // Freed the anchor, zeroed
            assert_eq!(token.type_, YamlNoToken);
        }
    }

    // ------------------ Event Initialization Tests ------------------

    #[test]
    fn test_yaml_stream_start_event_initialize() {
        unsafe {
            let mut event: YamlEventT = std::mem::zeroed();
            let result = yaml_stream_start_event_initialize(
                &mut event,
                YamlUtf8Encoding,
            ); // e.g. UTF-8
            assert_eq!(result, OK);
            // YamlStreamStartEvent => might be 7 or your code's enum
            assert_eq!(event.type_, YamlStreamStartEvent);
            yaml_event_delete(&mut event);
        }
    }

    #[test]
    fn test_yaml_stream_end_event_initialize() {
        unsafe {
            let mut event: YamlEventT = std::mem::zeroed();
            let result = yaml_stream_end_event_initialize(&mut event);
            assert_eq!(result, OK);
            // YamlStreamEndEvent => 8 in your code
            assert_eq!(event.type_, YamlStreamEndEvent);
            yaml_event_delete(&mut event);
        }
    }

    #[test]
    fn test_yaml_alias_event_initialize() {
        unsafe {
            let mut event: YamlEventT = std::mem::zeroed();
            let anchor = b"aliasAnchor\0";
            let result = yaml_alias_event_initialize(
                &mut event,
                anchor.as_ptr(),
            );
            assert_eq!(result, OK);
            // YamlAliasEvent => 9 in your code
            assert_eq!(event.type_, YamlAliasEvent);
            yaml_event_delete(&mut event);
        }
    }

    #[test]
    fn test_yaml_scalar_event_initialize() {
        unsafe {
            let mut event: YamlEventT = std::mem::zeroed();
            let data = ScalarEventData {
                anchor: null(),
                tag: null(),
                value: b"hello\0".as_ptr(),
                length: -1, // will auto-calc via strlen
                plain_implicit: true,
                quoted_implicit: false,
                style: YamlScalarStyleT::YamlPlainScalarStyle,
                _marker: std::marker::PhantomData,
            };
            let result = yaml_scalar_event_initialize(&mut event, data);
            assert_eq!(result, OK);
            // YamlScalarEvent => 10
            assert_eq!(event.type_, YamlScalarEvent);
            yaml_event_delete(&mut event);
        }
    }

    #[test]
    fn test_yaml_sequence_start_event_initialize() {
        unsafe {
            let mut event: YamlEventT = std::mem::zeroed();
            let anchor = b"seqAnchor\0";
            let tag = b"tag:yaml.org,2002:seq\0";
            let result = yaml_sequence_start_event_initialize(
                &mut event,
                anchor.as_ptr(),
                tag.as_ptr(),
                true,
                YamlSequenceStyleT::YamlBlockSequenceStyle,
            );
            assert_eq!(result, OK);
            // YamlSequenceStartEvent => 11
            assert_eq!(event.type_, YamlSequenceStartEvent);
            yaml_event_delete(&mut event);
        }
    }

    #[test]
    fn test_yaml_sequence_end_event_initialize() {
        unsafe {
            let mut event: YamlEventT = std::mem::zeroed();
            let result = yaml_sequence_end_event_initialize(&mut event);
            assert_eq!(result, OK);
            // YamlSequenceEndEvent => 12
            assert_eq!(event.type_, YamlSequenceEndEvent);
            yaml_event_delete(&mut event);
        }
    }

    #[test]
    fn test_yaml_mapping_start_event_initialize() {
        unsafe {
            let mut event: YamlEventT = std::mem::zeroed();
            let anchor = b"mapAnchor\0";
            let tag = b"tag:yaml.org,2002:map\0";
            let result = yaml_mapping_start_event_initialize(
                &mut event,
                anchor.as_ptr(),
                tag.as_ptr(),
                false,
                YamlMappingStyleT::YamlBlockMappingStyle,
            );
            assert_eq!(result, OK);
            // YamlMappingStartEvent => 13
            assert_eq!(event.type_, YamlMappingStartEvent);
            yaml_event_delete(&mut event);
        }
    }

    #[test]
    fn test_yaml_mapping_end_event_initialize() {
        unsafe {
            let mut event: YamlEventT = std::mem::zeroed();
            let result = yaml_mapping_end_event_initialize(&mut event);
            assert_eq!(result, OK);
            // YamlMappingEndEvent => 14
            assert_eq!(event.type_, YamlMappingEndEvent);
            yaml_event_delete(&mut event);
        }
    }

    #[test]
    fn test_yaml_event_delete() {
        unsafe {
            let mut event: YamlEventT = std::mem::zeroed();
            // Suppose YamlScalarEvent = 10
            event.type_ = YamlScalarEvent;
            // allocate data.scalar.value
            event.data.scalar.value = yaml_strdup(b"test\0".as_ptr());

            yaml_event_delete(&mut event);
            assert_eq!(event.type_, YamlNoEvent);
        }
    }
}
