// src/loader/error_handling.rs

use crate::memory::yaml_free;
use crate::{
    libc,
    success::{Success, FAIL},
    YamlErrorTypeT::YamlComposerError,
    YamlEventTypeT::{
        self, YamlAliasEvent, YamlDocumentEndEvent,
        YamlDocumentStartEvent, YamlMappingEndEvent,
        YamlMappingStartEvent, YamlScalarEvent, YamlSequenceEndEvent,
        YamlSequenceStartEvent, YamlStreamEndEvent,
        YamlStreamStartEvent,
    },
    YamlMarkT, YamlParserT,
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
