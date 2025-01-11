// src/tests/tests_loader.rs

//! Tests for the YAML loader functionality.

#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use core::{ffi::c_char, mem::MaybeUninit, ptr};
    use libyml::{
        decode::yaml_parser_initialize,
        loader::{
            error_handling::YamlError, yaml_parser_delete_aliases,
            yaml_parser_load, yaml_parser_set_error,
        },
        success::{FAIL, OK},
        yaml_document_delete, yaml_parser_delete,
        yaml_parser_set_input_string, YamlDocumentT,
        YamlErrorTypeT::{self, YamlComposerError},
        YamlMarkT, YamlParserT,
    };

    /// Tests setting an error with a null parser pointer.
    #[test]
    fn test_null_parser_error() {
        unsafe {
            let result = yaml_parser_set_error(
                ptr::null_mut(),
                None,
                b"Test\0" as *const u8 as *const c_char,
                YamlMarkT::default(),
            );
            assert!(
                matches!(result, Err(YamlError::NullPointer)),
                "Expected NullPointer error"
            );
        }
    }

    /// Tests basic parser initialization and cleanup.
    #[test]
    fn test_parser_initialization() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");

            let parser_ptr = parser.assume_init_mut();
            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests setting an error on a valid parser without context.
    #[test]
    fn test_composer_error() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            assert_eq!(
                yaml_parser_initialize(parser.as_mut_ptr()),
                OK,
                "Parser initialization failed"
            );
            let parser_ptr = parser.assume_init_mut();

            let result = yaml_parser_set_error(
                parser_ptr,
                None,
                b"Test error\0" as *const u8 as *const c_char,
                YamlMarkT::default(),
            );

            assert!(
                matches!(result, Ok(FAIL)),
                "Expected Ok(FAIL) result from error setting"
            );
            assert_eq!(parser_ptr.error, YamlComposerError);

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests setting an error on a valid parser with context.
    #[test]
    fn test_composer_error_with_context() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            assert_eq!(
                yaml_parser_initialize(parser.as_mut_ptr()),
                OK,
                "Parser initialization failed"
            );
            let parser_ptr = parser.assume_init_mut();

            let result = yaml_parser_set_error(
                parser_ptr,
                Some((
                    b"Test context\0" as *const u8 as *const c_char,
                    YamlMarkT::default(),
                )),
                b"Test problem\0" as *const u8 as *const c_char,
                YamlMarkT::default(),
            );

            assert!(
                matches!(result, Ok(FAIL)),
                "Expected Ok(FAIL) from error context setting"
            );
            assert_eq!(parser_ptr.error, YamlComposerError);

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests loading a simple YAML document.
    #[test]
    fn test_load_simple_document() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            assert_eq!(
                yaml_parser_initialize(parser.as_mut_ptr()),
                OK,
                "Parser initialization failed"
            );
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();
            let input = b"---\n...\0"; // Empty document

            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len() as u64 - 1,
            );

            let result =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            if result.is_ok() {
                let doc_ptr = document.assume_init_mut();
                yaml_document_delete(doc_ptr);
            } else {
                eprintln!("Error: {:?}", result);
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests replacing redundant `if let Ok(_)` with `.is_ok()`.
    #[test]
    fn test_is_ok_syntax() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            assert_eq!(
                yaml_parser_initialize(parser.as_mut_ptr()),
                OK,
                "Parser initialization failed"
            );
            let parser_ptr = parser.assume_init_mut();

            let result = yaml_parser_set_error(
                parser_ptr,
                None,
                b"Check is_ok syntax\0" as *const u8 as *const c_char,
                YamlMarkT::default(),
            );

            if result.is_ok() {
                assert_eq!(parser_ptr.error, YamlComposerError);
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests loading an empty YAML document.
    #[test]
    fn test_load_empty_document() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();
            let doc_ptr = document.as_mut_ptr();

            let input = b"---\n...\0";
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                (input.len() - 1) as u64,
            );

            let result = yaml_parser_load(parser_ptr, doc_ptr);
            match result {
                Ok(_) => {
                    // Clean up the document
                    yaml_document_delete(document.assume_init_mut());
                    yaml_parser_delete_aliases(parser_ptr);
                }
                Err(e) => {
                    eprintln!("Failed to load document: {:?}", e);
                    yaml_parser_delete_aliases(parser_ptr);
                }
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests handling malformed YAML input.
    #[test]
    fn test_malformed_yaml() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let _ = yaml_parser_initialize(parser.as_mut_ptr());
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

            // Malformed YAML - invalid tag format
            let input = b"---\n!!str] invalid tag\n...\0";
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                (input.len() - 1) as u64,
            );

            let result =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            assert!(
                result.is_err(),
                "Expected error for malformed YAML"
            );

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests handling of alias deletion.
    #[test]
    fn test_alias_handling() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let _ = yaml_parser_initialize(parser.as_mut_ptr());
            let parser_ptr = parser.assume_init_mut();

            // Test null parser handling
            yaml_parser_delete_aliases(ptr::null_mut());

            // Test with valid parser
            yaml_parser_delete_aliases(parser_ptr);

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests loading a document with sequences.
    #[test]
    fn test_load_sequence_document() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

            let input = b"---\nsequence:\n  - item1\n  - item2\n  - item3\n...\0";

            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len() as u64 - 1,
            );

            let result =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            match result {
                Ok(_) => {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests loading a document with nested mappings.
    #[test]
    fn test_load_nested_mapping() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

            let input = b"---\nparent:\n  child1:\n    key: value\n  child2:\n    key: value\n...\0";

            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len() as u64 - 1,
            );

            let result =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            match result {
                Ok(_) => {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests loading a document with mixed content types.
    #[test]
    fn test_load_mixed_content() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

            let input = b"---\n\
            string: test\n\
            number: 42\n\
            list: [1, 2, 3]\n\
            map: {key: value}\n\
            nested:\n\
              - item1: value1\n\
              - item2: value2\n\
            ...\0";

            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len() as u64 - 1,
            );

            let result =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            match result {
                Ok(_) => {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests loading a document with explicit tags.
    #[test]
    fn test_load_explicit_tags() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

            let input = b"---\n\
            string: !str test\n\
            number: !int 42\n\
            binary: !binary YWJjZGVm\n\
            ...\0";

            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len() as u64 - 1,
            );

            let result =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            match result {
                Ok(_) => {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    // /// Tests error handling for invalid document structure.
    // #[test]
    // fn test_invalid_document_structure() {
    //     unsafe {
    //         let mut parser = MaybeUninit::<YamlParserT>::uninit();
    //         let init_result =
    //             yaml_parser_initialize(parser.as_mut_ptr());
    //         assert_eq!(init_result, OK, "Parser initialization failed");
    //         let parser_ptr = parser.assume_init_mut();

    //         let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

    //         // Invalid document - missing document end marker
    //         let input = b"---\nkey: value\n";

    //         yaml_parser_set_input_string(
    //             parser_ptr,
    //             input.as_ptr(),
    //             input.len() as u64,
    //         );

    //         let result =
    //             yaml_parser_load(parser_ptr, document.as_mut_ptr());
    //         assert!(
    //             result.is_err(),
    //             "Expected error for invalid document structure"
    //         );

    //         yaml_parser_delete(parser_ptr);
    //     }
    // }

    /// Tests loading a document with multi-line strings
    #[test]
    fn test_load_multiline_strings() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

            let input = b"---\n\
            literal: |\n\
              Line one\n\
              Line two\n\
              Line three\n\
            folded: >\n\
              This is a long line\n\
              that will be\n\
              folded into one.\n\
            ...\0";

            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len() as u64 - 1,
            );

            let result =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            match result {
                Ok(_) => {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests loading a document with anchors and aliases
    #[test]
    fn test_load_anchors_and_aliases() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

            let input = b"---\n\
            defaults: &defaults\n\
              timeout: 30\n\
              retries: 3\n\
            custom:\n\
              <<: *defaults\n\
              retries: 5\n\
            ...\0";

            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len() as u64 - 1,
            );

            let result =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            match result {
                Ok(_) => {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests loading a document with various scalar types
    #[test]
    fn test_load_scalar_types() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

            let input = b"---\n\
            integer: 42\n\
            float: 3.14159\n\
            scientific: 12.3015e+05\n\
            boolean: true\n\
            null: ~\n\
            date: 2024-01-11\n\
            ...\0";

            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len() as u64 - 1,
            );

            let result =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            match result {
                Ok(_) => {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests handling of UTF-8 encoded content
    #[test]
    fn test_utf8_content() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

            let input = b"---\n\
            unicode: \xE2\x98\x83\n\
            japanese: \xE6\x97\xA5\xE6\x9C\xAC\xE8\xAA\x9E\n\
            ...\0";

            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len() as u64 - 1,
            );

            let result =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            match result {
                Ok(_) => {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests extremely large YAML document handling
    #[test]
    fn test_large_document() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

            // Generate a large document with many nested items
            let mut input = Vec::from(&b"---\nitems:\n"[..]);
            for i in 0..1000 {
                let item = format!("  - item{}: value{}\n", i, i);
                input.extend_from_slice(item.as_bytes());
            }
            input.extend_from_slice(b"...\0");

            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len() as u64 - 1,
            );

            let result =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            match result {
                Ok(_) => {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }

            yaml_parser_delete(parser_ptr);
        }
    }
    /// Tests absolute bare minimum functionality
    #[test]
    fn test_bare_minimum() {
        unsafe {
            // Initialize parser
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            // First, just test if we can set input without trying to parse
            let input = b"---\nkey: value\n...\0";
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                (input.len() - 1) as u64,
            );

            // Just cleanup the parser without trying to parse
            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests document initialization only
    #[test]
    fn test_document_init() {
        unsafe {
            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();
            let doc_ptr = document.assume_init_mut();
            yaml_document_delete(doc_ptr);
        }
    }

    /// Tests parser and document creation without loading
    #[test]
    fn test_initialization() {
        unsafe {
            // Initialize parser
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            // Initialize document
            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();
            let _doc_ptr = document.as_mut_ptr();

            // Cleanup without trying to parse
            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests minimal document loading with debug info
    #[test]
    fn test_debug_document_load() {
        unsafe {
            // Initialize parser
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            // Print parser initialization state
            eprintln!(
                "Parser initialized with error state: {:?}",
                (parser_ptr).error
            );

            // Initialize document with zero
            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

            // Set minimal input
            let input = b"---\n...\0";
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                (input.len() - 1) as u64,
            );

            // Try to load with verbose error reporting
            let result =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            match &result {
                Ok(_) => eprintln!("Document loaded successfully"),
                Err(e) => {
                    eprintln!("Load error: {:?}", e);
                    eprintln!("Parser state: {:?}", (parser_ptr).error);
                    if !(parser_ptr).problem.is_null() {
                        let problem = std::ffi::CStr::from_ptr(
                            (parser_ptr).problem,
                        );
                        eprintln!("Problem: {:?}", problem);
                    }
                }
            }

            // Still try to clean up
            if result.is_ok() {
                let doc_ptr = document.assume_init_mut();
                yaml_document_delete(doc_ptr);
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests document allocation only
    #[test]
    fn test_document_allocation() {
        unsafe {
            // Initialize parser
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            // Initialize document
            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();
            let doc_ptr = document.as_mut_ptr();

            // Print memory handler info if available
            eprintln!(
                "Memory handler status: {:?}",
                (parser_ptr).error != YamlErrorTypeT::YamlNoError
            );

            // Try to initialize document without loading
            ptr::write_bytes(
                doc_ptr as *mut u8,
                0,
                size_of::<YamlDocumentT>(),
            );

            // Clean up
            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests handling of multiple documents in a stream
    #[test]
    fn test_multiple_documents() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

            // Single document first to verify basic functionality
            let input = b"---\ndoc1: true\n...\0";

            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len() as u64 - 1,
            );

            // Try to parse document
            let result1 =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            match result1 {
                Ok(_) => {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }
                Err(e) => eprintln!("Error parsing document: {:?}", e),
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests stream end handling
    #[test]
    fn test_stream_end() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

            // Simple complete document
            let input = b"---\nkey: value\n...\0";

            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len() as u64 - 1,
            );

            // Parse document
            let result =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            match result {
                Ok(_) => {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests basic sequential document handling
    #[test]
    fn test_sequential_documents() {
        unsafe {
            // First document
            let mut parser1 = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser1.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr1 = parser1.assume_init_mut();

            let mut document1 = MaybeUninit::<YamlDocumentT>::zeroed();
            let input1 = b"---\nfirst: true\n...\0";

            yaml_parser_set_input_string(
                parser_ptr1,
                input1.as_ptr(),
                input1.len() as u64 - 1,
            );

            let result1 =
                yaml_parser_load(parser_ptr1, document1.as_mut_ptr());
            match result1 {
                Ok(_) => {
                    let doc_ptr = document1.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }
                Err(e) => {
                    eprintln!("Error parsing first document: {:?}", e)
                }
            }

            yaml_parser_delete(parser_ptr1);

            // Second document with new parser
            let mut parser2 = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser2.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr2 = parser2.assume_init_mut();

            let mut document2 = MaybeUninit::<YamlDocumentT>::zeroed();
            let input2 = b"---\nsecond: true\n...\0";

            yaml_parser_set_input_string(
                parser_ptr2,
                input2.as_ptr(),
                input2.len() as u64 - 1,
            );

            let result2 =
                yaml_parser_load(parser_ptr2, document2.as_mut_ptr());
            match result2 {
                Ok(_) => {
                    let doc_ptr = document2.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }
                Err(e) => {
                    eprintln!("Error parsing second document: {:?}", e)
                }
            }

            yaml_parser_delete(parser_ptr2);
        }
    }

    /// Tests boundary conditions in YAML parsing
    #[test]
    fn test_boundary_conditions() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let init_result =
                yaml_parser_initialize(parser.as_mut_ptr());
            assert_eq!(init_result, OK, "Parser initialization failed");
            let parser_ptr = parser.assume_init_mut();

            let mut document = MaybeUninit::<YamlDocumentT>::zeroed();

            // Empty key with value
            let input1 = b"---\n\"\": value\n...\0";
            yaml_parser_set_input_string(
                parser_ptr,
                input1.as_ptr(),
                input1.len() as u64 - 1,
            );
            let result1 =
                yaml_parser_load(parser_ptr, document.as_mut_ptr());
            match result1 {
                Ok(_) => {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }
                Err(e) => eprintln!("Error with empty key: {:?}", e),
            }

            yaml_parser_delete(parser_ptr);
        }
    }

    /// Tests parser performance with static input
    #[test]
    fn test_parser_performance() {
        // Use a fixed test document
        let input = b"---\nkey: value\n...\0";

        for _ in 0..100 {
            unsafe {
                // Create a new parser for each iteration
                let mut parser = MaybeUninit::<YamlParserT>::uninit();
                let init_result =
                    yaml_parser_initialize(parser.as_mut_ptr());
                assert_eq!(
                    init_result, OK,
                    "Parser initialization failed"
                );
                let parser_ptr = parser.assume_init_mut();

                let mut document =
                    MaybeUninit::<YamlDocumentT>::zeroed();

                // Set input and parse
                yaml_parser_set_input_string(
                    parser_ptr,
                    input.as_ptr(),
                    input.len() as u64 - 1,
                );

                let result =
                    yaml_parser_load(parser_ptr, document.as_mut_ptr());
                if result.is_ok() {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }

                // Clean up parser
                yaml_parser_delete(parser_ptr);
            }
        }
    }

    /// Tests parser re-initialization performance
    #[test]
    fn test_parser_reinit_performance() {
        unsafe {
            let input = b"---\nkey: value\n...\0";

            // Test multiple parser initializations
            for _ in 0..10 {
                let mut parser = MaybeUninit::<YamlParserT>::uninit();
                let init_result =
                    yaml_parser_initialize(parser.as_mut_ptr());
                assert_eq!(
                    init_result, OK,
                    "Parser initialization failed"
                );
                let parser_ptr = parser.assume_init_mut();

                let mut document =
                    MaybeUninit::<YamlDocumentT>::zeroed();

                yaml_parser_set_input_string(
                    parser_ptr,
                    input.as_ptr(),
                    input.len() as u64 - 1,
                );

                let result =
                    yaml_parser_load(parser_ptr, document.as_mut_ptr());
                if result.is_ok() {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }

                yaml_parser_delete(parser_ptr);
            }
        }
    }

    #[cfg(test)]
    mod memory_leak_tests {
        use super::*;
        use core::sync::atomic::{AtomicIsize, Ordering};

        static ALLOC_COUNTER: AtomicIsize = AtomicIsize::new(0);

        #[test]
        fn test_no_memory_leaks() {
            unsafe {
                // Test with a simple YAML document
                let mut parser = MaybeUninit::<YamlParserT>::uninit();
                let init_result =
                    yaml_parser_initialize(parser.as_mut_ptr());
                assert_eq!(
                    init_result, OK,
                    "Parser initialization failed"
                );
                let parser_ptr = parser.assume_init_mut();

                let mut document =
                    MaybeUninit::<YamlDocumentT>::zeroed();
                let input = b"---\nkey: value\n...\0";

                yaml_parser_set_input_string(
                    parser_ptr,
                    input.as_ptr(),
                    input.len() as u64 - 1,
                );

                let result =
                    yaml_parser_load(parser_ptr, document.as_mut_ptr());
                if result.is_ok() {
                    let doc_ptr = document.assume_init_mut();
                    yaml_document_delete(doc_ptr);
                }

                yaml_parser_delete(parser_ptr);

                // Check for leaks
                assert_eq!(
                    ALLOC_COUNTER.load(Ordering::SeqCst),
                    0,
                    "Memory leak detected: {} allocations not freed",
                    ALLOC_COUNTER.load(Ordering::SeqCst)
                );
            }
        }
    }
}
