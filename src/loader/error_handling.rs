// src/loader/error_handling.rs

use crate::memory::yaml_free;
use crate::YamlEventTypeT;
use crate::YamlMarkT;
use crate::{
    libc,
    success::{Success, FAIL},
    YamlErrorTypeT::YamlComposerError,
    YamlEventTypeT::{
        YamlAliasEvent, YamlDocumentEndEvent, YamlDocumentStartEvent,
        YamlMappingEndEvent, YamlMappingStartEvent, YamlScalarEvent,
        YamlSequenceEndEvent, YamlSequenceStartEvent,
        YamlStreamEndEvent, YamlStreamStartEvent,
    },
    YamlParserT,
};
use core::{error, fmt, ptr};

/// Enum representing possible YAML parsing errors.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum YamlError {
    /// Received a null pointer.
    NullPointer,
    /// Memory allocation failed.
    MemoryAllocationFailed,
    /// Found a duplicate anchor.
    DuplicateAnchor,
    /// Found an undefined alias.
    UndefinedAlias,
    /// Encountered an invalid event type.
    InvalidEventType,
    /// Invalid YAML document structure.
    InvalidDocumentStructure,
    /// An unknown error occurred.
    UnknownError,
}

impl fmt::Display for YamlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YamlError::NullPointer => {
                write!(f, "Received a null pointer.")
            }
            YamlError::MemoryAllocationFailed => {
                write!(f, "Memory allocation failed.")
            }
            YamlError::DuplicateAnchor => {
                write!(f, "Found a duplicate anchor.")
            }
            YamlError::UndefinedAlias => {
                write!(f, "Found an undefined alias.")
            }
            YamlError::InvalidEventType => {
                write!(f, "Encountered an invalid event type.")
            }
            YamlError::InvalidDocumentStructure => {
                write!(f, "Invalid YAML document structure.")
            }
            YamlError::UnknownError => {
                write!(f, "An unknown error occurred.")
            }
        }
    }
}

impl error::Error for YamlError {}

/// Sets a composer error in the parser.
///
/// This function is deprecated. Use `yaml_parser_set_error` instead.
///
/// # Arguments
/// * `parser` - A mutable pointer to the `YamlParserT` struct.
/// * `context` - A pointer to a constant C string providing error context.
/// * `context_mark` - A `YamlMarkT` struct representing the mark for the context.
///
/// # Returns
/// * `Result<Success, YamlError>` indicating the outcome of the operation.
///
/// # Safety
/// * `parser` must be a valid, non-null pointer to a properly initialized `YamlParserT` struct.
/// * If `context` is not null, it must point to a valid null-terminated C string.
#[deprecated(
    since = "0.0.6",
    note = "please use `yaml_parser_set_error` instead"
)]
pub unsafe fn yaml_parser_set_composer_error(
    parser: *mut YamlParserT,
    context: *const libc::c_char,
    context_mark: YamlMarkT,
) -> Result<Success, YamlError> {
    yaml_parser_set_error(
        parser,
        if context.is_null() {
            None
        } else {
            Some((context, context_mark))
        },
        b"Composer error occurred\0" as *const u8
            as *const libc::c_char,
        context_mark,
    )
}

/// Sets an error in the parser with optional context.
///
/// # Arguments
/// * `parser` - A mutable pointer to the `YamlParserT` struct.
/// * `context` - An optional tuple containing a context string and its associated mark.
/// * `problem` - A pointer to a constant C string representing the problem.
/// * `problem_mark` - A `YamlMarkT` struct representing the mark where the problem occurred.
///
/// # Returns
/// * `Result<Success, YamlError>` indicating the outcome of the operation.
///
/// # Safety
/// * All pointers must be valid and properly initialized.
pub unsafe fn yaml_parser_set_error(
    parser: *mut YamlParserT,
    context: Option<(*const libc::c_char, YamlMarkT)>,
    problem: *const libc::c_char,
    problem_mark: YamlMarkT,
) -> Result<Success, YamlError> {
    if parser.is_null() {
        return Err(YamlError::NullPointer);
    }

    (*parser).error = YamlComposerError;

    if let Some((ctx, ctx_mark)) = context {
        (*parser).context = ctx;
        (*parser).context_mark = ctx_mark;
    }

    (*parser).problem = problem;
    (*parser).problem_mark = problem_mark;

    Ok(FAIL)
}

/// Deletes all aliases associated with the parser.
///
/// # Arguments
/// * `parser` - A mutable pointer to the `YamlParserT` struct.
///
/// # Safety
/// * `parser` must be a valid, non-null pointer to a properly initialized `YamlParserT` struct.
pub unsafe fn yaml_parser_delete_aliases(parser: *mut YamlParserT) {
    if parser.is_null() {
        return;
    }

    while !STACK_EMPTY!((*parser).aliases) {
        yaml_free(POP!((*parser).aliases).anchor as *mut libc::c_void);
    }
    STACK_DEL!((*parser).aliases);
}

/// Enum representing the state of the parser.
///
/// This is used to indicate the current state of the parser.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ParserState {
    /// Parser is in a stream start state.
    StreamStart,
    /// Parser is in a stream end state.
    StreamEnd,
    /// Parser is in a document start state.
    DocumentStart,
    /// Parser is in a document end state.
    DocumentEnd,
    /// Parser is in a sequence start state.
    SequenceStart,
    /// Parser is in a sequence end state.
    SequenceEnd,
    /// Parser is in a mapping start state.
    MappingStart,
    /// Parser is in a mapping end state.
    MappingEnd,
    /// Parser is in a scalar state.
    Scalar,
    /// Parser is in an alias state.
    Alias,
    /// Parser is in an unknown state.
    Unknown,
}

impl From<YamlEventTypeT> for ParserState {
    fn from(event_type: YamlEventTypeT) -> Self {
        match event_type {
            YamlStreamStartEvent => ParserState::StreamStart,
            YamlStreamEndEvent => ParserState::StreamEnd,
            YamlDocumentStartEvent => ParserState::DocumentStart,
            YamlDocumentEndEvent => ParserState::DocumentEnd,
            YamlSequenceStartEvent => ParserState::SequenceStart,
            YamlSequenceEndEvent => ParserState::SequenceEnd,
            YamlMappingStartEvent => ParserState::MappingStart,
            YamlMappingEndEvent => ParserState::MappingEnd,
            YamlScalarEvent => ParserState::Scalar,
            YamlAliasEvent => ParserState::Alias,
            _ => ParserState::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::internal::yaml_stack_extend;
    use crate::memory::yaml_malloc;
    use crate::memory::yaml_strdup;
    use crate::yaml::yaml_char_t;
    use crate::YamlAliasDataT;
    use crate::YamlEventTypeT::YamlStreamEndEvent;
    use crate::{
        libc,
        loader::{
            yaml_parser_delete_aliases, yaml_parser_set_error,
            ParserState, YamlError,
        },
        success::FAIL,
        YamlErrorTypeT::YamlComposerError,
        YamlEventTypeT::{
            YamlAliasEvent, YamlDocumentEndEvent,
            YamlDocumentStartEvent, YamlMappingEndEvent,
            YamlMappingStartEvent, YamlNoEvent, YamlScalarEvent,
            YamlSequenceEndEvent, YamlSequenceStartEvent,
            YamlStreamStartEvent,
        },
        YamlMarkT, YamlParserT,
    };
    use alloc::format;
    use core::ptr;
    use core::ptr::addr_of_mut;

    #[test]
    fn test_yaml_error_display() {
        assert_eq!(
            format!("{}", YamlError::NullPointer),
            "Received a null pointer."
        );
        assert_eq!(
            format!("{}", YamlError::MemoryAllocationFailed),
            "Memory allocation failed."
        );
        assert_eq!(
            format!("{}", YamlError::DuplicateAnchor),
            "Found a duplicate anchor."
        );
        assert_eq!(
            format!("{}", YamlError::UndefinedAlias),
            "Found an undefined alias."
        );
        assert_eq!(
            format!("{}", YamlError::InvalidEventType),
            "Encountered an invalid event type."
        );
        assert_eq!(
            format!("{}", YamlError::InvalidDocumentStructure),
            "Invalid YAML document structure."
        );
        assert_eq!(
            format!("{}", YamlError::UnknownError),
            "An unknown error occurred."
        );
    }

    #[test]
    fn test_yaml_parser_set_error_valid_input() {
        let mut parser = YamlParserT::default();
        let problem =
            b"Test problem\0" as *const u8 as *const libc::c_char;

        let result = unsafe {
            yaml_parser_set_error(
                &mut parser,
                Some((
                    b"Context message\0" as *const u8
                        as *const libc::c_char,
                    YamlMarkT {
                        index: 1,
                        line: 2,
                        column: 3,
                    },
                )),
                problem,
                YamlMarkT {
                    index: 4,
                    line: 5,
                    column: 6,
                },
            )
        };

        assert_eq!(result, Ok(FAIL));
        assert_eq!(parser.error, YamlComposerError);
        assert_eq!(parser.problem, problem);
        assert_eq!(parser.problem_mark.index, 4);
        assert_eq!(parser.problem_mark.line, 5);
        assert_eq!(parser.problem_mark.column, 6);
        assert_eq!(
            unsafe { core::ffi::CStr::from_ptr(parser.context) }
                .to_str(),
            Ok("Context message")
        );
    }

    #[test]
    fn test_yaml_parser_set_error_no_context() {
        let mut parser = YamlParserT::default();
        let problem =
            b"No context test\0" as *const u8 as *const libc::c_char;

        let result = unsafe {
            yaml_parser_set_error(
                &mut parser,
                None,
                problem,
                YamlMarkT {
                    index: 7,
                    line: 8,
                    column: 9,
                },
            )
        };

        assert_eq!(result, Ok(FAIL));
        assert_eq!(parser.error, YamlComposerError);
        assert_eq!(parser.problem, problem);
        assert_eq!(parser.problem_mark.index, 7);
        assert_eq!(parser.problem_mark.line, 8);
        assert_eq!(parser.problem_mark.column, 9);
        assert!(parser.context.is_null());
    }

    #[test]
    fn test_yaml_parser_delete_aliases_empty() {
        let mut parser = YamlParserT::default();
        unsafe {
            yaml_parser_delete_aliases(&mut parser);
        }
        // Expect no errors; stack was empty.
    }

    #[test]
    fn test_yaml_parser_delete_aliases_mocked() {
        let mut parser = YamlParserT::default();

        unsafe {
            // Mock alias management using some external setup or mocks
            // If aliases aren't relevant, replace this logic
            yaml_parser_delete_aliases(&mut parser);

            // Assert that no operations occurred, or check specific behavior
        }
    }

    #[test]
    fn test_parser_state_from_yaml_event_type() {
        assert_eq!(
            ParserState::from(YamlStreamStartEvent),
            ParserState::StreamStart
        );
        assert_eq!(
            ParserState::from(YamlDocumentStartEvent),
            ParserState::DocumentStart
        );
        assert_eq!(
            ParserState::from(YamlDocumentEndEvent),
            ParserState::DocumentEnd
        );
        assert_eq!(
            ParserState::from(YamlSequenceStartEvent),
            ParserState::SequenceStart
        );
        assert_eq!(
            ParserState::from(YamlSequenceEndEvent),
            ParserState::SequenceEnd
        );
        assert_eq!(
            ParserState::from(YamlMappingStartEvent),
            ParserState::MappingStart
        );
        assert_eq!(
            ParserState::from(YamlMappingEndEvent),
            ParserState::MappingEnd
        );
        assert_eq!(
            ParserState::from(YamlScalarEvent),
            ParserState::Scalar
        );
        assert_eq!(
            ParserState::from(YamlAliasEvent),
            ParserState::Alias
        );
        assert_eq!(
            ParserState::from(YamlNoEvent),
            ParserState::Unknown
        ); // Unknown event type
    }

    #[test]
    fn test_yaml_parser_set_composer_error_behavior() {
        let mut parser = YamlParserT::default();

        let result = unsafe {
            yaml_parser_set_error(
                &mut parser,
                Some((
                    b"Composer context\0" as *const u8
                        as *const libc::c_char,
                    YamlMarkT {
                        index: 10,
                        line: 20,
                        column: 30,
                    },
                )),
                b"Composer error occurred\0" as *const u8
                    as *const libc::c_char,
                YamlMarkT {
                    index: 10,
                    line: 20,
                    column: 30,
                },
            )
        };

        assert_eq!(result, Ok(FAIL));
        assert_eq!(parser.error, YamlComposerError);
        assert_eq!(
            unsafe { core::ffi::CStr::from_ptr(parser.context) }
                .to_str(),
            Ok("Composer context")
        );
        assert_eq!(parser.context_mark.index, 10);
        assert_eq!(parser.context_mark.line, 20);
        assert_eq!(parser.context_mark.column, 30);
    }

    #[test]
    fn test_yaml_parser_set_composer_error() {
        let mut parser = YamlParserT::default();
        let context =
            b"Test context\0" as *const u8 as *const libc::c_char;
        let mark = YamlMarkT {
            index: 1,
            line: 1,
            column: 1,
        };

        let result = unsafe {
            yaml_parser_set_error(
                &mut parser,
                Some((context, mark)),
                b"Composer error occurred\0" as *const u8
                    as *const libc::c_char,
                mark,
            )
        };

        assert_eq!(result, Ok(FAIL));
        assert_eq!(parser.error, YamlComposerError);
        unsafe {
            assert_eq!(
                core::ffi::CStr::from_ptr(parser.context).to_str(),
                Ok("Test context")
            );
        }
        assert_eq!(parser.context_mark.index, 1);
        assert_eq!(parser.context_mark.line, 1);
        assert_eq!(parser.context_mark.column, 1);
    }

    #[test]
    fn test_yaml_parser_set_composer_error_null_context() {
        let mut parser = YamlParserT::default();
        let mark = YamlMarkT {
            index: 1,
            line: 1,
            column: 1,
        };

        let result = unsafe {
            yaml_parser_set_error(
                &mut parser,
                None,
                b"Composer error occurred\0" as *const u8
                    as *const libc::c_char,
                mark,
            )
        };

        assert_eq!(result, Ok(FAIL));
        assert_eq!(parser.error, YamlComposerError);
        assert!(parser.context.is_null());
    }

    #[test]
    fn test_yaml_parser_delete_aliases_with_content() {
        let mut parser = YamlParserT::default();

        // Set up test data
        unsafe {
            // Initialize the aliases stack
            STACK_INIT!(parser.aliases, YamlAliasDataT);

            // Create a test alias
            let alias_data = YamlAliasDataT {
                anchor: yaml_strdup(
                    b"test_anchor\0" as *const u8 as *const yaml_char_t,
                ),
                index: 1,
                mark: YamlMarkT {
                    index: 0,
                    line: 0,
                    column: 0,
                },
            };

            // Push the test alias onto the stack
            PUSH!(parser.aliases, alias_data);

            // Delete aliases
            yaml_parser_delete_aliases(&mut parser);

            // Verify cleanup
            assert!(parser.aliases.start.is_null());
            assert!(parser.aliases.top.is_null());
            assert!(parser.aliases.end.is_null());
        }
    }

    #[test]
    fn test_yaml_parser_set_error_with_long_messages() {
        let mut parser = YamlParserT::default();
        let long_problem = b"This is a very long error message that tests handling of longer strings in the error system\0" as *const u8 as *const libc::c_char;
        let long_context = b"This is a very long context message that tests handling of longer strings in the context system\0" as *const u8 as *const libc::c_char;

        let mark = YamlMarkT {
            index: 100,
            line: 50,
            column: 25,
        };

        let result = unsafe {
            yaml_parser_set_error(
                &mut parser,
                Some((long_context, mark)),
                long_problem,
                mark,
            )
        };

        assert_eq!(result, Ok(FAIL));
        assert_eq!(parser.error, YamlComposerError);
        unsafe {
            assert!(core::ffi::CStr::from_ptr(parser.problem)
                .to_str()
                .unwrap()
                .contains("very long error message"));
            assert!(core::ffi::CStr::from_ptr(parser.context)
                .to_str()
                .unwrap()
                .contains("very long context message"));
        }
    }

    #[test]
    fn test_parser_state_transitions() {
        // First test the initial case
        assert_eq!(
            ParserState::from(YamlStreamStartEvent),
            ParserState::StreamStart
        );

        // Test transitions for all event types with correct mappings
        let transitions = [
            (YamlStreamStartEvent, ParserState::StreamStart),
            (YamlStreamEndEvent, ParserState::StreamEnd), // Fixed: Changed from YamlStreamStartEvent
            (YamlDocumentStartEvent, ParserState::DocumentStart),
            (YamlDocumentEndEvent, ParserState::DocumentEnd),
            (YamlSequenceStartEvent, ParserState::SequenceStart),
            (YamlSequenceEndEvent, ParserState::SequenceEnd),
            (YamlMappingStartEvent, ParserState::MappingStart),
            (YamlMappingEndEvent, ParserState::MappingEnd),
            (YamlScalarEvent, ParserState::Scalar),
            (YamlAliasEvent, ParserState::Alias),
            (YamlNoEvent, ParserState::Unknown),
        ];

        for (event, expected_state) in transitions.iter() {
            assert_eq!(
                ParserState::from(*event),
                *expected_state,
                "Failed transition test: {:?} should map to {:?}",
                event,
                expected_state
            );
        }
    }

    #[test]
    fn test_yaml_error_debug_impl() {
        let error = YamlError::NullPointer;
        assert_eq!(format!("{:?}", error), "NullPointer");

        let error = YamlError::MemoryAllocationFailed;
        assert_eq!(format!("{:?}", error), "MemoryAllocationFailed");

        // Test all variants
        let errors = [
            YamlError::NullPointer,
            YamlError::MemoryAllocationFailed,
            YamlError::DuplicateAnchor,
            YamlError::UndefinedAlias,
            YamlError::InvalidEventType,
            YamlError::InvalidDocumentStructure,
            YamlError::UnknownError,
        ];

        for error in errors.iter() {
            // Verify Debug output is not empty
            assert!(!format!("{:?}", error).is_empty());
        }
    }

    #[test]
    fn test_yaml_parser_set_error_boundary_conditions() {
        let mut parser = YamlParserT::default();

        // Test with empty strings
        let result = unsafe {
            yaml_parser_set_error(
                &mut parser,
                Some((
                    b"\0" as *const u8 as *const libc::c_char,
                    YamlMarkT::default(),
                )),
                b"\0" as *const u8 as *const libc::c_char,
                YamlMarkT::default(),
            )
        };
        assert_eq!(result, Ok(FAIL));

        // Test with maximum mark values
        let max_mark = YamlMarkT {
            index: u64::MAX,
            line: u64::MAX,
            column: u64::MAX,
        };

        let result = unsafe {
            yaml_parser_set_error(
                &mut parser,
                Some((
                    b"test\0" as *const u8 as *const libc::c_char,
                    max_mark,
                )),
                b"test\0" as *const u8 as *const libc::c_char,
                max_mark,
            )
        };
        assert_eq!(result, Ok(FAIL));
    }

    #[test]
    fn test_error_conversion() {
        use crate::YamlErrorTypeT::{
            YamlComposerError, YamlEmitterError, YamlMemoryError,
            YamlNoError, YamlParserError, YamlReaderError,
            YamlScannerError, YamlWriterError,
        };

        // Test standard error conversion
        let std_errors = [
            (YamlNoError, YamlError::UnknownError),
            (YamlMemoryError, YamlError::MemoryAllocationFailed),
            (YamlParserError, YamlError::InvalidDocumentStructure),
            (YamlScannerError, YamlError::InvalidEventType),
            (YamlComposerError, YamlError::UnknownError),
            (YamlWriterError, YamlError::UnknownError),
            (YamlEmitterError, YamlError::UnknownError),
            (YamlReaderError, YamlError::UnknownError),
        ];

        for (yaml_error, expected_error) in std_errors {
            let converted_error = match yaml_error {
                YamlMemoryError => YamlError::MemoryAllocationFailed,
                YamlParserError => YamlError::InvalidDocumentStructure,
                YamlScannerError => YamlError::InvalidEventType,
                _ => YamlError::UnknownError,
            };
            assert_eq!(
                converted_error, expected_error,
                "Failed to convert {:?} to {:?}",
                yaml_error, expected_error
            );
        }

        // Test error with context
        let mut parser = YamlParserT::default();
        let context =
            b"Error context\0" as *const u8 as *const libc::c_char;
        let mark = YamlMarkT {
            index: 1,
            line: 1,
            column: 1,
        };

        unsafe {
            let result = yaml_parser_set_error(
                &mut parser,
                Some((context, mark)),
                b"Error with context\0" as *const u8
                    as *const libc::c_char,
                mark,
            );
            assert_eq!(result, Ok(FAIL));
            assert_eq!(parser.error, YamlComposerError);
            assert_eq!(parser.problem_mark, mark);
        }

        // Test error without context
        let mut parser = YamlParserT::default();
        unsafe {
            let result = yaml_parser_set_error(
                &mut parser,
                None,
                b"Error without context\0" as *const u8
                    as *const libc::c_char,
                mark,
            );
            assert_eq!(result, Ok(FAIL));
            assert_eq!(parser.error, YamlComposerError);
            assert!(parser.context.is_null());
        }

        // Test null pointer handling
        unsafe {
            let result = yaml_parser_set_error(
                ptr::null_mut(),
                None,
                b"Test error\0" as *const u8 as *const libc::c_char,
                YamlMarkT::default(),
            );
            assert_eq!(result, Err(YamlError::NullPointer));
        }

        // Test error mark conversion
        let test_marks = [
            YamlMarkT::default(),
            YamlMarkT {
                index: u64::MAX,
                line: u64::MAX,
                column: u64::MAX,
            },
            YamlMarkT {
                index: 42,
                line: 24,
                column: 80,
            },
        ];

        for mark in test_marks {
            let mut parser = YamlParserT::default();
            unsafe {
                let result = yaml_parser_set_error(
                    &mut parser,
                    None,
                    b"Test error\0" as *const u8 as *const libc::c_char,
                    mark,
                );
                assert_eq!(result, Ok(FAIL));
                assert_eq!(parser.problem_mark, mark);
            }
        }
    }

    #[test]
    fn test_yaml_parser_delete_aliases_null_parser() {
        unsafe {
            // Should return gracefully without crashing
            yaml_parser_delete_aliases(ptr::null_mut());
        }
    }

    #[test]
    fn test_yaml_parser_delete_aliases_with_multiple_aliases() {
        let mut parser = YamlParserT::default();

        unsafe {
            // Initialize the aliases stack
            STACK_INIT!(parser.aliases, YamlAliasDataT);

            // Create multiple test aliases with same size byte arrays
            let aliases = [
                (b"anchor_one\0\0\0", 1),
                (b"anchor_two\0\0\0", 2),
                (b"anchor_thr\0\0\0", 3),
            ];

            for (anchor_name, index) in aliases {
                let alias_data = YamlAliasDataT {
                    anchor: yaml_strdup(
                        anchor_name.as_ptr() as *const yaml_char_t
                    ),
                    index,
                    mark: YamlMarkT::default(),
                };
                PUSH!(parser.aliases, alias_data);
            }

            // Delete aliases
            yaml_parser_delete_aliases(&mut parser);

            // Verify cleanup
            assert!(parser.aliases.start.is_null());
            assert!(parser.aliases.top.is_null());
            assert!(parser.aliases.end.is_null());
        }
    }

    #[test]
    fn test_parser_state_unknown_values() {
        // Test known mappings
        assert_eq!(
            ParserState::from(YamlStreamStartEvent),
            ParserState::StreamStart
        );
        assert_eq!(
            ParserState::from(YamlStreamEndEvent),
            ParserState::StreamEnd
        );
        assert_eq!(
            ParserState::from(YamlDocumentStartEvent),
            ParserState::DocumentStart
        );
        assert_eq!(
            ParserState::from(YamlDocumentEndEvent),
            ParserState::DocumentEnd
        );
        assert_eq!(
            ParserState::from(YamlSequenceStartEvent),
            ParserState::SequenceStart
        );
        assert_eq!(
            ParserState::from(YamlSequenceEndEvent),
            ParserState::SequenceEnd
        );
        assert_eq!(
            ParserState::from(YamlMappingStartEvent),
            ParserState::MappingStart
        );
        assert_eq!(
            ParserState::from(YamlMappingEndEvent),
            ParserState::MappingEnd
        );
        assert_eq!(
            ParserState::from(YamlScalarEvent),
            ParserState::Scalar
        );
        assert_eq!(
            ParserState::from(YamlAliasEvent),
            ParserState::Alias
        );

        // Test the fallback case
        assert_eq!(
            ParserState::from(YamlNoEvent),
            ParserState::Unknown
        );
    }
}
