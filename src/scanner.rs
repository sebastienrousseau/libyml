// scanner.rs

use crate::externs::{memcpy, memmove, memset, strcmp, strlen};
use crate::internal::{yaml_queue_extend, yaml_stack_extend};
use crate::memory::yaml_free;
use crate::memory::yaml_malloc;
use crate::ops::{ForceAdd as _, ForceMul as _};
use crate::reader::yaml_parser_update_buffer;
use crate::string::{yaml_string_extend, yaml_string_join};
use crate::success::{Success, FAIL, OK};
use crate::yaml::{
    ptrdiff_t, size_t, yaml_char_t, YamlStringT, NULL_STRING,
};
use crate::{
    libc, PointerExt, YamlAliasToken, YamlAnchorToken,
    YamlBlockEndToken, YamlBlockEntryToken, YamlBlockMappingStartToken,
    YamlBlockSequenceStartToken, YamlDocumentEndToken,
    YamlDocumentStartToken, YamlDoubleQuotedScalarStyle,
    YamlFlowEntryToken, YamlFlowMappingEndToken,
    YamlFlowMappingStartToken, YamlFlowSequenceEndToken,
    YamlFlowSequenceStartToken, YamlFoldedScalarStyle, YamlKeyToken,
    YamlLiteralScalarStyle, YamlMarkT, YamlMemoryError, YamlNoError,
    YamlParserT, YamlPlainScalarStyle, YamlScalarToken,
    YamlScannerError, YamlSimpleKeyT, YamlSingleQuotedScalarStyle,
    YamlStreamEndToken, YamlStreamStartToken, YamlTagDirectiveToken,
    YamlTagToken, YamlTokenT, YamlTokenTypeT, YamlValueToken,
    YamlVersionDirectiveToken,
};
use core::mem::{size_of, MaybeUninit};
use core::ptr;
use core::ptr::addr_of_mut;

const MAX_SCALAR_SIZE: usize = 65536;
const SCANNING_TOKEN_ERROR: &[u8] =
    b"while scanning for the next token\0";
const INVALID_CHARACTER_ERROR: &[u8] =
    b"found character that cannot start any token\0";

unsafe fn cache(parser: *mut YamlParserT, length: size_t) -> Success {
    if (*parser).unread >= length {
        OK
    } else {
        yaml_parser_update_buffer(parser, length)
    }
}

unsafe fn skip(parser: *mut YamlParserT) {
    let width = WIDTH!((*parser).buffer);
    (*parser).mark.index = (*parser).mark.index.force_add(width as u64);
    (*parser).mark.column = (*parser).mark.column.force_add(1);
    (*parser).unread = (*parser).unread.wrapping_sub(1);
    (*parser).buffer.pointer =
        (*parser).buffer.pointer.wrapping_offset(width as isize);
}

unsafe fn skip_line(parser: *mut YamlParserT) {
    if IS_CRLF!((*parser).buffer) {
        (*parser).mark.index = (*parser).mark.index.force_add(2);
        (*parser).mark.column = 0;
        (*parser).mark.line = (*parser).mark.line.force_add(1);
        (*parser).unread = (*parser).unread.wrapping_sub(2);
        (*parser).buffer.pointer =
            (*parser).buffer.pointer.wrapping_offset(2);
    } else if IS_BREAK!((*parser).buffer) {
        let width = WIDTH!((*parser).buffer);
        (*parser).mark.index =
            (*parser).mark.index.force_add(width as u64);
        (*parser).mark.column = 0;
        (*parser).mark.line = (*parser).mark.line.force_add(1);
        (*parser).unread = (*parser).unread.wrapping_sub(1);
        (*parser).buffer.pointer =
            (*parser).buffer.pointer.wrapping_offset(width as isize);
    };
}

/// Reads a single character from the parser’s buffer into `string`,
/// checking that we do not exceed `MAX_SCALAR_SIZE`.
unsafe fn read(parser: *mut YamlParserT, string: *mut YamlStringT) {
    let current_length =
        (*string).pointer.offset_from((*string).start) as usize;
    // Check if we can add 1 more character safely.
    if current_length + 1 > MAX_SCALAR_SIZE {
        yaml_parser_set_scanner_error(
            parser,
            b"while scanning a scalar\0" as *const u8
                as *const libc::c_char,
            (*parser).mark,
            b"scalar has grown beyond the maximum allowed size\0"
                as *const u8 as *const libc::c_char,
        );
        return;
    }

    STRING_EXTEND!(*string);
    let width = WIDTH!((*parser).buffer);
    copy!(*string, (*parser).buffer);
    (*parser).mark.index = (*parser).mark.index.force_add(width as u64);
    (*parser).mark.column = (*parser).mark.column.force_add(1);
    (*parser).unread = (*parser).unread.wrapping_sub(1);
}

/// Reads a line from the parser buffer and handles various line ending formats.
///
/// This function handles the following line endings:
/// - CRLF (\r\n)
/// - CR (\r) or LF (\n)
/// - NEL (U+0085)
/// - LS (U+2028) or PS (U+2029)
///
/// # Safety
///
/// The caller must ensure that:
/// * `parser` points to a valid `YamlParserT` structure
/// * `string` points to a valid `YamlStringT` structure
/// * The parser buffer contains enough unread bytes for the checks performed
/// * The string buffer has enough capacity for the new line
///
/// # Arguments
///
/// * `parser` - Mutable pointer to the YAML parser state
/// * `string` - Mutable pointer to the string buffer where the line will be stored
///
/// # Implementation Notes
///
/// This function normalizes different line ending sequences to '\n' while preserving
/// Unicode line endings when necessary (LS and PS characters).
unsafe fn read_line(
    parser: *mut YamlParserT,
    string: *mut YamlStringT,
) {
    // Pre-condition checks in debug builds
    debug_assert!(!parser.is_null(), "Parser pointer must not be null");
    debug_assert!(!string.is_null(), "String pointer must not be null");
    debug_assert!(
        !(*string).pointer.is_null(),
        "String buffer pointer must not be null"
    );
    debug_assert!(
        !(*parser).buffer.pointer.is_null(),
        "Parser buffer pointer must not be null"
    );

    // Constants for special Unicode line endings
    const NEL_FIRST: u8 = 0xC2;
    const NEL_SECOND: u8 = 0x85;
    const LS_PS_FIRST: u8 = 0xE2;
    const LS_PS_SECOND: u8 = 0x80;
    const LS_THIRD: u8 = 0xA8;
    const PS_THIRD: u8 = 0xA9;

    // Ensure string has enough capacity
    STRING_EXTEND!(*string);

    // Helper macro to update parser state after reading a line
    macro_rules! update_parser_state {
        ($offset:expr) => {
            (*parser).mark.index =
                (*parser).mark.index.force_add($offset);
            (*parser).mark.column = 0;
            (*parser).mark.line = (*parser).mark.line.force_add(1);
        };
    }

    // Helper function to write normalized newline
    let write_normalized_newline = |s: *mut YamlStringT| {
        *(*s).pointer = b'\n';
        (*s).pointer = (*s).pointer.wrapping_offset(1);
    };

    if CHECK_AT!((*parser).buffer, b'\r', 0)
        && CHECK_AT!((*parser).buffer, b'\n', 1)
    {
        // CRLF sequence
        write_normalized_newline(string);
        (*parser).buffer.pointer =
            (*parser).buffer.pointer.wrapping_offset(2);
        update_parser_state!(2);
        (*parser).unread = (*parser).unread.wrapping_sub(2);
    } else if CHECK_AT!((*parser).buffer, b'\r', 0)
        || CHECK_AT!((*parser).buffer, b'\n', 0)
    {
        // Single CR or LF
        write_normalized_newline(string);
        (*parser).buffer.pointer =
            (*parser).buffer.pointer.wrapping_offset(1);
        update_parser_state!(1);
        (*parser).unread = (*parser).unread.wrapping_sub(1);
    } else if CHECK_AT!((*parser).buffer, NEL_FIRST, 0)
        && CHECK_AT!((*parser).buffer, NEL_SECOND, 1)
    {
        // NEL (Next Line) sequence
        write_normalized_newline(string);
        (*parser).buffer.pointer =
            (*parser).buffer.pointer.wrapping_offset(2);
        update_parser_state!(2);
        (*parser).unread = (*parser).unread.wrapping_sub(1);
    } else if CHECK_AT!((*parser).buffer, LS_PS_FIRST, 0)
        && CHECK_AT!((*parser).buffer, LS_PS_SECOND, 1)
        && (CHECK_AT!((*parser).buffer, LS_THIRD, 2)
            || CHECK_AT!((*parser).buffer, PS_THIRD, 2))
    {
        // LS (Line Separator) or PS (Paragraph Separator)
        // Preserve the original 3-byte sequence
        for _ in 0..3 {
            *(*string).pointer = *(*parser).buffer.pointer;
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer =
                (*parser).buffer.pointer.wrapping_offset(1);
        }
        update_parser_state!(3);
        (*parser).unread = (*parser).unread.wrapping_sub(1);
    }
}

macro_rules! read {
    ($parser:expr, $string:expr) => {
        read($parser, addr_of_mut!($string))
    };
}

macro_rules! read_line {
    ($parser:expr, $string:expr) => {
        read_line($parser, addr_of_mut!($string))
    };
}

/// Scans a YAML document and produces the next token in the token stream.
///
/// This function is the primary interface for tokenizing YAML content. It processes
/// the input stream and produces tokens sequentially, handling all YAML syntax elements
/// including scalars, sequences, mappings, and directives.
///
/// # Safety Requirements
///
/// This function requires that:
/// * `parser` points to a valid, initialized `YamlParserT` structure.
/// * `token` points to a valid, mutable `YamlTokenT` structure.
/// * The parser must not be in an error state.
/// * The caller must not alternate between calling this function and `yaml_parser_parse()`
///   or `yaml_parser_load()` as it may corrupt the parser state.
///
/// # Arguments
///
/// * `parser` - Mutable pointer to the YAML parser state. Must be properly initialized
///             and not in an error state.
/// * `token` - Mutable pointer to store the next token. The previous token's resources
///           will be overwritten.
///
/// # Returns
///
/// * `Success::OK` - Successfully produced a token.
/// * `Success::FAIL` - Failed to produce a token (check parser error state).
///
/// # Token Lifecycle
///
/// The function follows this sequence:
/// 1. Validates input parameters.
/// 2. Clears the target token memory.
/// 3. Checks for stream end or error conditions.
/// 4. Fetches more tokens if needed.
/// 5. Dequeues and returns the next available token.
///
/// # Memory Management
///
/// The caller is responsible for:
/// * Freeing any buffers associated with the previous token using `yaml_token_delete`.
/// * Ensuring the parser remains valid throughout the scanning process.
///
/// # Error Handling
///
/// The function may fail if:
/// * The parser is in an invalid state.
/// * Memory allocation fails.
/// * The input stream is malformed.
/// * Token processing encounters an error.
///
/// # Safety
///
/// The caller must ensure that:
/// * `parser` and `token` are non-null and properly initialized.
/// * No other operations that modify the parser state are performed simultaneously.
/// * The parser remains valid throughout the token scanning process.
///
/// Misuse of this function can lead to undefined behavior, including memory corruption.
pub unsafe fn yaml_parser_scan(
    parser: *mut YamlParserT,
    token: *mut YamlTokenT,
) -> Success {
    // Validate input parameters
    debug_assert!(!parser.is_null(), "Parser pointer must not be null");
    debug_assert!(!token.is_null(), "Token pointer must not be null");
    debug_assert!(
        (*parser).error == YamlNoError,
        "Parser must not be in error state"
    );

    // Runtime safety checks
    if parser.is_null() || token.is_null() {
        return FAIL;
    }

    // Clear the token memory to ensure clean state
    ptr::write_bytes(
        token.cast::<libc::c_void>(),
        0,
        size_of::<YamlTokenT>(),
    );

    // Check for terminal conditions
    if (*parser).stream_end_produced {
        return OK;
    }

    if (*parser).error != YamlNoError {
        return OK;
    }

    // Ensure tokens are available
    if !(*parser).token_available
        && yaml_parser_fetch_more_tokens(parser).fail
    {
        return FAIL;
    }

    // Extract next token
    *token = DEQUEUE!((*parser).tokens);
    (*parser).token_available = false;

    // Update parser state
    let tokens_parsed = addr_of_mut!((*parser).tokens_parsed);
    *tokens_parsed = (*tokens_parsed).force_add(1);

    // Check for stream end
    if (*token).type_ == YamlStreamEndToken {
        (*parser).stream_end_produced = true;
    }

    OK
}

/// Sets a scanner error in the YAML parser with detailed context information.
///
/// This function records error details when the scanner encounters an issue during
/// YAML parsing. It captures the error context, location, and specific problem
/// description for error reporting and debugging purposes.
///
/// # Safety Requirements
///
/// Caller must ensure:
/// * `parser` points to a valid, initialized `YamlParserT` structure
/// * `context` is either null or points to a valid null-terminated C string
/// * `problem` is either null or points to a valid null-terminated C string
/// * The provided context_mark contains valid position information
/// * All string pointers remain valid for the lifetime of the parser
///
/// # Arguments
///
/// * `parser` - Mutable pointer to the parser state structure
/// * `context` - Optional pointer to a null-terminated string describing the error context
/// * `context_mark` - Position information for where the context applies
/// * `problem` - Pointer to a null-terminated string describing the specific error
///
/// # Error State
///
/// The function sets the following error information in the parser:
/// * Sets error type to YamlScannerError
/// * Records the context string (if provided)
/// * Stores the context position marker
/// * Records the specific problem description
/// * Captures the current parser position as the problem location
///
#[inline]
unsafe fn yaml_parser_set_scanner_error(
    parser: *mut YamlParserT,
    context: *const libc::c_char,
    context_mark: YamlMarkT,
    problem: *const libc::c_char,
) {
    // Debug assertions for parameter validation
    debug_assert!(!parser.is_null(), "Parser pointer must not be null");
    debug_assert!(
        context.is_null() || !context.is_null(),
        "Context must be either null or a valid pointer"
    );
    debug_assert!(
        !problem.is_null(),
        "Problem description must not be null"
    );

    // Set error type
    (*parser).error = YamlScannerError;

    // Record context information
    ptr::write(addr_of_mut!((*parser).context), context);

    // Store position markers
    (*parser).context_mark = context_mark;
    (*parser).problem_mark = (*parser).mark;

    // Record problem description
    ptr::write(addr_of_mut!((*parser).problem), problem);

    // Ensure error state is properly set for error handling
    debug_assert!(
        (*parser).error == YamlScannerError,
        "Error state not properly set"
    );
}

/// Fetches more tokens from the YAML parser's input stream as needed.
///
/// This function manages the token buffer, ensuring that tokens are available
/// for processing while handling simple keys and maintaining parser state.
/// It operates in a loop until either sufficient tokens are available or
/// an error occurs.
///
/// # Safety Requirements
///
/// Caller must ensure:
/// * `parser` points to a valid, initialized `YamlParserT` structure
/// * The parser's token queue is properly initialized
/// * The parser's simple keys stack is properly initialized
/// * All memory referenced by the parser structure is valid
///
/// # Algorithm
///
/// 1. Checks if more tokens are needed by:
///    - Verifying if the token queue is empty
///    - Checking for pending simple keys that need resolution
/// 2. Fetches new tokens if needed
/// 3. Updates parser state accordingly
///
/// # Error Handling
///
/// Returns `FAIL` if:
/// * Token fetching fails
/// * Simple key processing fails
/// * Memory allocation fails
///
/// # Returns
///
/// * `Success::OK` - Successfully ensured tokens are available
/// * `Success::FAIL` - An error occurred during token fetching
///
#[inline]
pub(crate) unsafe fn yaml_parser_fetch_more_tokens(
    parser: *mut YamlParserT,
) -> Success {
    // Debug assertions for parameter validation
    debug_assert!(!parser.is_null(), "Parser pointer must not be null");
    debug_assert!(
        !(*parser).tokens.head.is_null(),
        "Token queue head must not be null"
    );
    debug_assert!(
        !(*parser).tokens.tail.is_null(),
        "Token queue tail must not be null"
    );
    debug_assert!(
        !(*parser).simple_keys.start.is_null(),
        "Simple keys start must not be null"
    );
    debug_assert!(
        !(*parser).simple_keys.top.is_null(),
        "Simple keys top must not be null"
    );

    loop {
        // Determine if more tokens are needed
        let mut need_more_tokens = false;

        // Check if token queue is empty
        if (*parser).tokens.head == (*parser).tokens.tail {
            need_more_tokens = true;
        } else {
            // Process simple keys to determine if we need more tokens
            if yaml_parser_stale_simple_keys(parser).fail {
                return FAIL;
            }

            // Get the first non-processed simple key
            let simple_key_start = (*parser)
                .simple_keys
                .start
                .add((*parser).not_simple_keys as usize);

            // Check all simple keys
            let mut simple_key = simple_key_start;
            while simple_key != (*parser).simple_keys.top {
                // Check if this key needs immediate token
                let key_needs_token = (*simple_key).possible
                    && (*simple_key).token_number
                        == (*parser).tokens_parsed;

                if key_needs_token {
                    need_more_tokens = true;
                    break;
                }

                // Move to next simple key
                simple_key = simple_key.wrapping_offset(1);
            }
        }

        // Break if we have enough tokens
        if !need_more_tokens {
            break;
        }

        // Fetch next token if needed
        if yaml_parser_fetch_next_token(parser).fail {
            return FAIL;
        }
    }

    // Mark tokens as available
    (*parser).token_available = true;

    // Post-condition check
    debug_assert!(
        (*parser).token_available,
        "Tokens should be available after successful fetch"
    );

    OK
}

/// Fetches the next token from the YAML parser's input stream.
///
/// This function implements the core YAML token scanning logic, recognizing and
/// processing all YAML syntax elements according to the specification.
///
/// # Safety Requirements
///
/// Caller must ensure:
/// * `parser` points to a valid, initialized `YamlParserT` structure
/// * The parser's buffer contains valid data
/// * All parser state (marks, levels, etc.) is valid
///
/// # Token Types Handled
///
/// - Stream tokens (start/end)
/// - Document indicators (---, ...)
/// - Directives (%)
/// - Flow collections ([, ], {, })
/// - Block elements (-, ?, :)
/// - Scalars (plain, quoted, block)
/// - Anchors and aliases (*, &)
/// - Tags (!)
///
/// # Returns
///
/// * `Success::OK` - Successfully fetched next token
/// * `Success::FAIL` - Failed to fetch token (error details in parser)
///
#[allow(clippy::cognitive_complexity)]
unsafe fn yaml_parser_fetch_next_token(
    parser: *mut YamlParserT,
) -> Success {
    debug_assert!(!parser.is_null(), "Parser pointer must not be null");

    // Initial buffer check
    if cache(parser, 1_u64).fail {
        return FAIL;
    }

    // Handle stream start
    if !(*parser).stream_start_produced {
        yaml_parser_fetch_stream_start(parser);
        return OK;
    }

    // Prepare for token scanning
    if yaml_parser_scan_to_next_token(parser).fail
        || yaml_parser_stale_simple_keys(parser).fail
    {
        return FAIL;
    }

    // Handle indentation
    yaml_parser_unroll_indent(
        parser,
        (*parser).mark.column as ptrdiff_t,
    );

    // Ensure sufficient buffer for lookahead
    if cache(parser, 4_u64).fail {
        return FAIL;
    }

    // Stream end
    if IS_Z!((*parser).buffer) {
        return yaml_parser_fetch_stream_end(parser);
    }

    // Document indicators and directives at column 0
    if (*parser).mark.column == 0 {
        if CHECK!((*parser).buffer, b'%') {
            return yaml_parser_fetch_directive(parser);
        }

        // Document start (---)
        if is_document_start(parser) {
            return yaml_parser_fetch_document_indicator(
                parser,
                YamlDocumentStartToken,
            );
        }

        // Document end (...)
        if is_document_end(parser) {
            return yaml_parser_fetch_document_indicator(
                parser,
                YamlDocumentEndToken,
            );
        }
    }

    // Flow collection tokens
    if CHECK!((*parser).buffer, b'[') {
        return yaml_parser_fetch_flow_collection_start(
            parser,
            YamlFlowSequenceStartToken,
        );
    }
    if CHECK!((*parser).buffer, b'{') {
        return yaml_parser_fetch_flow_collection_start(
            parser,
            YamlFlowMappingStartToken,
        );
    }
    if CHECK!((*parser).buffer, b']') {
        return yaml_parser_fetch_flow_collection_end(
            parser,
            YamlFlowSequenceEndToken,
        );
    }
    if CHECK!((*parser).buffer, b'}') {
        return yaml_parser_fetch_flow_collection_end(
            parser,
            YamlFlowMappingEndToken,
        );
    }

    // Flow entries and block elements
    if CHECK!((*parser).buffer, b',') {
        return yaml_parser_fetch_flow_entry(parser);
    }
    if CHECK!((*parser).buffer, b'-')
        && IS_BLANKZ_AT!((*parser).buffer, 1)
    {
        return yaml_parser_fetch_block_entry(parser);
    }

    // Keys and values
    if is_key(parser) {
        return yaml_parser_fetch_key(parser);
    }
    if is_value(parser) {
        return yaml_parser_fetch_value(parser);
    }

    // Anchors, aliases, and tags
    if CHECK!((*parser).buffer, b'*') {
        return yaml_parser_fetch_anchor(parser, YamlAliasToken);
    }
    if CHECK!((*parser).buffer, b'&') {
        return yaml_parser_fetch_anchor(parser, YamlAnchorToken);
    }
    if CHECK!((*parser).buffer, b'!') {
        return yaml_parser_fetch_tag(parser);
    }

    // Block scalars
    if CHECK!((*parser).buffer, b'|') && (*parser).flow_level == 0 {
        return yaml_parser_fetch_block_scalar(parser, true);
    }
    if CHECK!((*parser).buffer, b'>') && (*parser).flow_level == 0 {
        return yaml_parser_fetch_block_scalar(parser, false);
    }

    // Flow scalars
    if CHECK!((*parser).buffer, b'\'') {
        return yaml_parser_fetch_flow_scalar(parser, true);
    }
    if CHECK!((*parser).buffer, b'"') {
        return yaml_parser_fetch_flow_scalar(parser, false);
    }

    // Plain scalar (if no other token matches)
    if is_plain_scalar(parser) {
        return yaml_parser_fetch_plain_scalar(parser);
    }

    // If no token matched, set error
    yaml_parser_set_scanner_error(
        parser,
        SCANNING_TOKEN_ERROR.as_ptr() as *const i8,
        (*parser).mark,
        INVALID_CHARACTER_ERROR.as_ptr() as *const i8,
    );
    FAIL
}

#[inline]
unsafe fn is_document_start(parser: *const YamlParserT) -> bool {
    CHECK_AT!((*parser).buffer, b'-', 0)
        && CHECK_AT!((*parser).buffer, b'-', 1)
        && CHECK_AT!((*parser).buffer, b'-', 2)
        && IS_BLANKZ_AT!((*parser).buffer, 3)
}

#[inline]
unsafe fn is_document_end(parser: *const YamlParserT) -> bool {
    CHECK_AT!((*parser).buffer, b'.', 0)
        && CHECK_AT!((*parser).buffer, b'.', 1)
        && CHECK_AT!((*parser).buffer, b'.', 2)
        && IS_BLANKZ_AT!((*parser).buffer, 3)
}

#[inline]
unsafe fn is_key(parser: *const YamlParserT) -> bool {
    CHECK!((*parser).buffer, b'?')
        && ((*parser).flow_level != 0
            || IS_BLANKZ_AT!((*parser).buffer, 1))
}

#[inline]
unsafe fn is_value(parser: *const YamlParserT) -> bool {
    CHECK!((*parser).buffer, b':')
        && ((*parser).flow_level != 0
            || IS_BLANKZ_AT!((*parser).buffer, 1))
}

#[inline]
unsafe fn is_plain_scalar(parser: *const YamlParserT) -> bool {
    !(IS_BLANKZ!((*parser).buffer)
        || CHECK!((*parser).buffer, b'-')
        || CHECK!((*parser).buffer, b'?')
        || CHECK!((*parser).buffer, b':')
        || CHECK!((*parser).buffer, b',')
        || CHECK!((*parser).buffer, b'[')
        || CHECK!((*parser).buffer, b']')
        || CHECK!((*parser).buffer, b'{')
        || CHECK!((*parser).buffer, b'}')
        || CHECK!((*parser).buffer, b'#')
        || CHECK!((*parser).buffer, b'&')
        || CHECK!((*parser).buffer, b'*')
        || CHECK!((*parser).buffer, b'!')
        || CHECK!((*parser).buffer, b'|')
        || CHECK!((*parser).buffer, b'>')
        || CHECK!((*parser).buffer, b'\'')
        || CHECK!((*parser).buffer, b'"')
        || CHECK!((*parser).buffer, b'%')
        || CHECK!((*parser).buffer, b'@')
        || CHECK!((*parser).buffer, b'`'))
        || CHECK!((*parser).buffer, b'-')
            && !IS_BLANK_AT!((*parser).buffer, 1)
        || (*parser).flow_level == 0
            && (CHECK!((*parser).buffer, b'?')
                || CHECK!((*parser).buffer, b':'))
            && !IS_BLANKZ_AT!((*parser).buffer, 1)
}

unsafe fn yaml_parser_stale_simple_keys(
    parser: *mut YamlParserT,
) -> Success {
    let mut simple_key: *mut YamlSimpleKeyT;
    simple_key = (*parser)
        .simple_keys
        .start
        .add((*parser).not_simple_keys as usize);
    while simple_key != (*parser).simple_keys.top {
        if (*simple_key).possible
            && ((*simple_key).mark.line < (*parser).mark.line
                || (*simple_key).mark.index.force_add(1024_u64)
                    < (*parser).mark.index)
        {
            if (*simple_key).required {
                yaml_parser_set_scanner_error(
                    parser,
                    SCANNING_TOKEN_ERROR.as_ptr() as *const i8,
                    (*parser).mark,
                    INVALID_CHARACTER_ERROR.as_ptr() as *const i8,
                );
                return FAIL;
            }
            (*simple_key).possible = false;
            if (*parser)
                .simple_keys
                .start
                .add((*parser).not_simple_keys as usize)
                == simple_key
            {
                (*parser).not_simple_keys += 1;
            }
        }
        simple_key = simple_key.wrapping_offset(1);
    }
    OK
}

unsafe fn yaml_parser_save_simple_key(
    parser: *mut YamlParserT,
) -> Success {
    let required = (*parser).flow_level == 0
        && (*parser).indent as libc::c_long
            == (*parser).mark.column as ptrdiff_t;
    if (*parser).simple_key_allowed {
        let simple_key = YamlSimpleKeyT {
            possible: true,
            required,
            token_number: (*parser).tokens_parsed.force_add(
                (*parser)
                    .tokens
                    .tail
                    .c_offset_from((*parser).tokens.head)
                    as libc::c_ulong,
            ),
            mark: (*parser).mark,
        };
        if yaml_parser_remove_simple_key(parser).fail {
            return FAIL;
        }
        *(*parser).simple_keys.top.wrapping_offset(-1_isize) =
            simple_key;
        if (*parser)
            .simple_keys
            .start
            .add((*parser).not_simple_keys as usize)
            == (*parser).simple_keys.top
        {
            (*parser).not_simple_keys -= 1;
        }
    }
    OK
}

unsafe fn yaml_parser_remove_simple_key(
    parser: *mut YamlParserT,
) -> Success {
    let simple_key: *mut YamlSimpleKeyT =
        (*parser).simple_keys.top.wrapping_offset(-1_isize);
    if (*simple_key).possible && (*simple_key).required {
        yaml_parser_set_scanner_error(
            parser,
            b"while scanning a simple key\0" as *const u8
                as *const libc::c_char,
            (*simple_key).mark,
            b"could not find expected ':'\0" as *const u8
                as *const libc::c_char,
        );
        return FAIL;
    }
    (*simple_key).possible = false;
    OK
}

unsafe fn yaml_parser_increase_flow_level(
    parser: *mut YamlParserT,
) -> Success {
    let empty_simple_key = YamlSimpleKeyT {
        possible: false,
        required: false,
        token_number: 0_u64,
        mark: YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        },
    };
    PUSH!((*parser).simple_keys, empty_simple_key);
    if (*parser).flow_level == libc::c_int::MAX {
        (*parser).error = YamlMemoryError;
        return FAIL;
    }
    let fresh7 = addr_of_mut!((*parser).flow_level);
    *fresh7 += 1;
    OK
}

unsafe fn yaml_parser_decrease_flow_level(parser: *mut YamlParserT) {
    if (*parser).flow_level != 0 {
        let fresh8 = addr_of_mut!((*parser).flow_level);
        *fresh8 -= 1;
        if (*parser)
            .simple_keys
            .start
            .add((*parser).not_simple_keys as usize)
            == (*parser).simple_keys.top
        {
            (*parser).not_simple_keys -= 1;
        }
        let _ = POP!((*parser).simple_keys);
    }
}

unsafe fn yaml_parser_roll_indent(
    parser: *mut YamlParserT,
    column: ptrdiff_t,
    number: ptrdiff_t,
    type_: YamlTokenTypeT,
    mark: YamlMarkT,
) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    if (*parser).flow_level != 0 {
        return OK;
    }
    if ((*parser).indent as libc::c_long) < column {
        PUSH!((*parser).indents, (*parser).indent);
        if column > ptrdiff_t::from(libc::c_int::MAX) {
            (*parser).error = YamlMemoryError;
            return FAIL;
        }
        (*parser).indent = column as libc::c_int;
        let _ = memset(
            token.cast::<libc::c_void>(),
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
        (*token).type_ = type_;
        (*token).start_mark = mark;
        (*token).end_mark = mark;
        if number == -1_i64 {
            ENQUEUE!((*parser).tokens, *token);
        } else {
            QUEUE_INSERT!(
                (*parser).tokens,
                (number as libc::c_ulong)
                    .wrapping_sub((*parser).tokens_parsed),
                *token
            );
        }
    }
    OK
}

unsafe fn yaml_parser_unroll_indent(
    parser: *mut YamlParserT,
    column: ptrdiff_t,
) {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    if (*parser).flow_level != 0 {
        return;
    }
    while (*parser).indent as libc::c_long > column {
        let _ = memset(
            token.cast::<libc::c_void>(),
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
        (*token).type_ = YamlBlockEndToken;
        (*token).start_mark = (*parser).mark;
        (*token).end_mark = (*parser).mark;
        ENQUEUE!((*parser).tokens, *token);
        (*parser).indent = POP!((*parser).indents);
    }
}

unsafe fn yaml_parser_fetch_stream_start(parser: *mut YamlParserT) {
    let simple_key = YamlSimpleKeyT {
        possible: false,
        required: false,
        token_number: 0_u64,
        mark: YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        },
    };
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    (*parser).indent = -1;
    PUSH!((*parser).simple_keys, simple_key);
    (*parser).not_simple_keys = 1;
    (*parser).simple_key_allowed = true;
    (*parser).stream_start_produced = true;
    let _ = memset(
        token.cast::<libc::c_void>(),
        0,
        size_of::<YamlTokenT>() as libc::c_ulong,
    );
    (*token).type_ = YamlStreamStartToken;
    (*token).start_mark = (*parser).mark;
    (*token).end_mark = (*parser).mark;
    (*token).data.stream_start.encoding = (*parser).encoding;
    ENQUEUE!((*parser).tokens, *token);
}

unsafe fn yaml_parser_fetch_stream_end(
    parser: *mut YamlParserT,
) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    if (*parser).mark.column != 0_u64 {
        (*parser).mark.column = 0_u64;
        let fresh22 = addr_of_mut!((*parser).mark.line);
        *fresh22 = (*fresh22).force_add(1);
    }
    yaml_parser_unroll_indent(parser, -1_i64);
    if yaml_parser_remove_simple_key(parser).fail {
        return FAIL;
    }
    (*parser).simple_key_allowed = false;
    let _ = memset(
        token.cast::<libc::c_void>(),
        0,
        size_of::<YamlTokenT>() as libc::c_ulong,
    );
    (*token).type_ = YamlStreamEndToken;
    (*token).start_mark = (*parser).mark;
    (*token).end_mark = (*parser).mark;
    ENQUEUE!((*parser).tokens, *token);
    OK
}

unsafe fn yaml_parser_fetch_directive(
    parser: *mut YamlParserT,
) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    yaml_parser_unroll_indent(parser, -1_i64);
    if yaml_parser_remove_simple_key(parser).fail {
        return FAIL;
    }
    (*parser).simple_key_allowed = false;
    if yaml_parser_scan_directive(parser, token).fail {
        return FAIL;
    }
    ENQUEUE!((*parser).tokens, *token);
    OK
}

unsafe fn yaml_parser_fetch_document_indicator(
    parser: *mut YamlParserT,
    type_: YamlTokenTypeT,
) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    yaml_parser_unroll_indent(parser, -1_i64);
    if yaml_parser_remove_simple_key(parser).fail {
        return FAIL;
    }
    (*parser).simple_key_allowed = false;
    let start_mark: YamlMarkT = (*parser).mark;
    skip(parser);
    skip(parser);
    skip(parser);
    let end_mark: YamlMarkT = (*parser).mark;
    let _ = memset(
        token.cast::<libc::c_void>(),
        0,
        size_of::<YamlTokenT>() as libc::c_ulong,
    );
    (*token).type_ = type_;
    (*token).start_mark = start_mark;
    (*token).end_mark = end_mark;
    ENQUEUE!((*parser).tokens, *token);
    OK
}

unsafe fn yaml_parser_fetch_flow_collection_start(
    parser: *mut YamlParserT,
    type_: YamlTokenTypeT,
) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    if yaml_parser_save_simple_key(parser).fail {
        return FAIL;
    }
    if yaml_parser_increase_flow_level(parser).fail {
        return FAIL;
    }
    (*parser).simple_key_allowed = true;
    let start_mark: YamlMarkT = (*parser).mark;
    skip(parser);
    let end_mark: YamlMarkT = (*parser).mark;
    let _ = memset(
        token.cast::<libc::c_void>(),
        0,
        size_of::<YamlTokenT>() as libc::c_ulong,
    );
    (*token).type_ = type_;
    (*token).start_mark = start_mark;
    (*token).end_mark = end_mark;
    ENQUEUE!((*parser).tokens, *token);
    OK
}

unsafe fn yaml_parser_fetch_flow_collection_end(
    parser: *mut YamlParserT,
    type_: YamlTokenTypeT,
) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    if yaml_parser_remove_simple_key(parser).fail {
        return FAIL;
    }
    yaml_parser_decrease_flow_level(parser);
    (*parser).simple_key_allowed = false;
    let start_mark: YamlMarkT = (*parser).mark;
    skip(parser);
    let end_mark: YamlMarkT = (*parser).mark;
    let _ = memset(
        token.cast::<libc::c_void>(),
        0,
        size_of::<YamlTokenT>() as libc::c_ulong,
    );
    (*token).type_ = type_;
    (*token).start_mark = start_mark;
    (*token).end_mark = end_mark;
    ENQUEUE!((*parser).tokens, *token);
    OK
}

unsafe fn yaml_parser_fetch_flow_entry(
    parser: *mut YamlParserT,
) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    if yaml_parser_remove_simple_key(parser).fail {
        return FAIL;
    }
    (*parser).simple_key_allowed = true;
    let start_mark: YamlMarkT = (*parser).mark;
    skip(parser);
    let end_mark: YamlMarkT = (*parser).mark;
    let _ = memset(
        token.cast::<libc::c_void>(),
        0,
        size_of::<YamlTokenT>() as libc::c_ulong,
    );
    (*token).type_ = YamlFlowEntryToken;
    (*token).start_mark = start_mark;
    (*token).end_mark = end_mark;
    ENQUEUE!((*parser).tokens, *token);
    OK
}

unsafe fn yaml_parser_fetch_block_entry(
    parser: *mut YamlParserT,
) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    if (*parser).flow_level == 0 {
        if !(*parser).simple_key_allowed {
            yaml_parser_set_scanner_error(
                parser,
                ptr::null::<libc::c_char>(),
                (*parser).mark,
                b"block sequence entries are not allowed in this context\0" as *const u8
                    as *const libc::c_char,
            );
            return FAIL;
        }
        if yaml_parser_roll_indent(
            parser,
            (*parser).mark.column as ptrdiff_t,
            -1_i64,
            YamlBlockSequenceStartToken,
            (*parser).mark,
        )
        .fail
        {
            return FAIL;
        }
    }
    if yaml_parser_remove_simple_key(parser).fail {
        return FAIL;
    }
    (*parser).simple_key_allowed = true;
    let start_mark: YamlMarkT = (*parser).mark;
    skip(parser);
    let end_mark: YamlMarkT = (*parser).mark;
    let _ = memset(
        token.cast::<libc::c_void>(),
        0,
        size_of::<YamlTokenT>() as libc::c_ulong,
    );
    (*token).type_ = YamlBlockEntryToken;
    (*token).start_mark = start_mark;
    (*token).end_mark = end_mark;
    ENQUEUE!((*parser).tokens, *token);
    OK
}

unsafe fn yaml_parser_fetch_key(parser: *mut YamlParserT) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    if (*parser).flow_level == 0 {
        if !(*parser).simple_key_allowed {
            yaml_parser_set_scanner_error(
                parser,
                ptr::null::<libc::c_char>(),
                (*parser).mark,
                b"mapping keys are not allowed in this context\0"
                    as *const u8 as *const libc::c_char,
            );
            return FAIL;
        }
        if yaml_parser_roll_indent(
            parser,
            (*parser).mark.column as ptrdiff_t,
            -1_i64,
            YamlBlockMappingStartToken,
            (*parser).mark,
        )
        .fail
        {
            return FAIL;
        }
    }
    if yaml_parser_remove_simple_key(parser).fail {
        return FAIL;
    }
    (*parser).simple_key_allowed = (*parser).flow_level == 0;
    let start_mark: YamlMarkT = (*parser).mark;
    skip(parser);
    let end_mark: YamlMarkT = (*parser).mark;
    let _ = memset(
        token.cast::<libc::c_void>(),
        0,
        size_of::<YamlTokenT>() as libc::c_ulong,
    );
    (*token).type_ = YamlKeyToken;
    (*token).start_mark = start_mark;
    (*token).end_mark = end_mark;
    ENQUEUE!((*parser).tokens, *token);
    OK
}

unsafe fn yaml_parser_fetch_value(parser: *mut YamlParserT) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    let simple_key: *mut YamlSimpleKeyT =
        (*parser).simple_keys.top.wrapping_offset(-1_isize);
    if (*simple_key).possible {
        let _ = memset(
            token.cast::<libc::c_void>(),
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
        (*token).type_ = YamlKeyToken;
        (*token).start_mark = (*simple_key).mark;
        (*token).end_mark = (*simple_key).mark;
        QUEUE_INSERT!(
            (*parser).tokens,
            ((*simple_key).token_number)
                .wrapping_sub((*parser).tokens_parsed),
            *token
        );
        if yaml_parser_roll_indent(
            parser,
            (*simple_key).mark.column as ptrdiff_t,
            (*simple_key).token_number as ptrdiff_t,
            YamlBlockMappingStartToken,
            (*simple_key).mark,
        )
        .fail
        {
            return FAIL;
        }
        (*simple_key).possible = false;
        (*parser).simple_key_allowed = false;
    } else {
        if (*parser).flow_level == 0 {
            if !(*parser).simple_key_allowed {
                yaml_parser_set_scanner_error(
                    parser,
                    ptr::null::<libc::c_char>(),
                    (*parser).mark,
                    b"mapping values are not allowed in this context\0"
                        as *const u8
                        as *const libc::c_char,
                );
                return FAIL;
            }
            if yaml_parser_roll_indent(
                parser,
                (*parser).mark.column as ptrdiff_t,
                -1_i64,
                YamlBlockMappingStartToken,
                (*parser).mark,
            )
            .fail
            {
                return FAIL;
            }
        }
        (*parser).simple_key_allowed = (*parser).flow_level == 0;
    }
    let start_mark: YamlMarkT = (*parser).mark;
    skip(parser);
    let end_mark: YamlMarkT = (*parser).mark;
    let _ = memset(
        token.cast::<libc::c_void>(),
        0,
        size_of::<YamlTokenT>() as libc::c_ulong,
    );
    (*token).type_ = YamlValueToken;
    (*token).start_mark = start_mark;
    (*token).end_mark = end_mark;
    ENQUEUE!((*parser).tokens, *token);
    OK
}

unsafe fn yaml_parser_fetch_anchor(
    parser: *mut YamlParserT,
    type_: YamlTokenTypeT,
) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    if yaml_parser_save_simple_key(parser).fail {
        return FAIL;
    }
    (*parser).simple_key_allowed = false;
    if yaml_parser_scan_anchor(parser, token, type_).fail {
        return FAIL;
    }
    ENQUEUE!((*parser).tokens, *token);
    OK
}

unsafe fn yaml_parser_fetch_tag(parser: *mut YamlParserT) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    if yaml_parser_save_simple_key(parser).fail {
        return FAIL;
    }
    (*parser).simple_key_allowed = false;
    if yaml_parser_scan_tag(parser, token).fail {
        return FAIL;
    }
    ENQUEUE!((*parser).tokens, *token);
    OK
}

unsafe fn yaml_parser_fetch_block_scalar(
    parser: *mut YamlParserT,
    literal: bool,
) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    if yaml_parser_remove_simple_key(parser).fail {
        return FAIL;
    }
    (*parser).simple_key_allowed = true;
    if yaml_parser_scan_block_scalar(parser, token, literal).fail {
        return FAIL;
    }
    ENQUEUE!((*parser).tokens, *token);
    OK
}

unsafe fn yaml_parser_fetch_flow_scalar(
    parser: *mut YamlParserT,
    single: bool,
) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    if yaml_parser_save_simple_key(parser).fail {
        return FAIL;
    }
    (*parser).simple_key_allowed = false;
    if yaml_parser_scan_flow_scalar(parser, token, single).fail {
        return FAIL;
    }
    ENQUEUE!((*parser).tokens, *token);
    OK
}

unsafe fn yaml_parser_fetch_plain_scalar(
    parser: *mut YamlParserT,
) -> Success {
    let mut token = MaybeUninit::<YamlTokenT>::uninit();
    let token = token.as_mut_ptr();
    if yaml_parser_save_simple_key(parser).fail {
        return FAIL;
    }
    (*parser).simple_key_allowed = false;
    if yaml_parser_scan_plain_scalar(parser, token).fail {
        return FAIL;
    }
    ENQUEUE!((*parser).tokens, *token);
    OK
}

unsafe fn yaml_parser_scan_to_next_token(
    parser: *mut YamlParserT,
) -> Success {
    loop {
        if cache(parser, 1_u64).fail {
            return FAIL;
        }
        if (*parser).mark.column == 0_u64 && IS_BOM!((*parser).buffer) {
            skip(parser);
        }
        if cache(parser, 1_u64).fail {
            return FAIL;
        }
        let mut should_continue = true;
        while should_continue {
            if CHECK!((*parser).buffer, b' ')
                || ((*parser).flow_level != 0
                    || !(*parser).simple_key_allowed)
                    && CHECK!((*parser).buffer, b'\t')
            {
                skip(parser);
                if cache(parser, 1_u64).fail {
                    return FAIL;
                }
            } else {
                should_continue = false;
            }
        }
        if CHECK!((*parser).buffer, b'#') {
            while !IS_BREAKZ!((*parser).buffer) {
                skip(parser);
                if cache(parser, 1_u64).fail {
                    return FAIL;
                }
            }
        }
        if !IS_BREAK!((*parser).buffer) {
            break;
        }
        if cache(parser, 2_u64).fail {
            return FAIL;
        }
        skip_line(parser);
        if (*parser).flow_level == 0 {
            (*parser).simple_key_allowed = true;
        }
    }
    OK
}

unsafe fn yaml_parser_scan_directive(
    parser: *mut YamlParserT,
    token: *mut YamlTokenT,
) -> Success {
    let mut current_block: u64;
    let end_mark: YamlMarkT;
    let mut name: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
    let mut major: libc::c_int = 0;
    let mut minor: libc::c_int = 0;
    let mut handle: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
    let mut prefix: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
    let start_mark: YamlMarkT = (*parser).mark;
    skip(parser);
    if yaml_parser_scan_directive_name(
        parser,
        start_mark,
        addr_of_mut!(name),
    )
    .ok
    {
        if strcmp(
            name as *mut libc::c_char,
            b"YAML\0" as *const u8 as *const libc::c_char,
        ) == 0
        {
            if yaml_parser_scan_version_directive_value(
                parser,
                start_mark,
                addr_of_mut!(major),
                addr_of_mut!(minor),
            )
            .fail
            {
                current_block = 11397968426844348457;
            } else {
                end_mark = (*parser).mark;
                let _ = memset(
                    token.cast::<libc::c_void>(),
                    0,
                    size_of::<YamlTokenT>() as libc::c_ulong,
                );
                (*token).type_ = YamlVersionDirectiveToken;
                (*token).start_mark = start_mark;
                (*token).end_mark = end_mark;
                (*token).data.version_directive.major = major;
                (*token).data.version_directive.minor = minor;
                current_block = 17407779659766490442;
            }
        } else if strcmp(
            name as *mut libc::c_char,
            b"TAG\0" as *const u8 as *const libc::c_char,
        ) == 0
        {
            if yaml_parser_scan_tag_directive_value(
                parser,
                start_mark,
                addr_of_mut!(handle),
                addr_of_mut!(prefix),
            )
            .fail
            {
                current_block = 11397968426844348457;
            } else {
                end_mark = (*parser).mark;
                let _ = memset(
                    token.cast::<libc::c_void>(),
                    0,
                    size_of::<YamlTokenT>() as libc::c_ulong,
                );
                (*token).type_ = YamlTagDirectiveToken;
                (*token).start_mark = start_mark;
                (*token).end_mark = end_mark;
                let fresh112 =
                    addr_of_mut!((*token).data.tag_directive.handle);
                *fresh112 = handle;
                let fresh113 =
                    addr_of_mut!((*token).data.tag_directive.prefix);
                *fresh113 = prefix;
                current_block = 17407779659766490442;
            }
        } else {
            yaml_parser_set_scanner_error(
                parser,
                b"while scanning a directive\0" as *const u8
                    as *const libc::c_char,
                start_mark,
                b"found unknown directive name\0" as *const u8
                    as *const libc::c_char,
            );
            current_block = 11397968426844348457;
        }
        if current_block != 11397968426844348457
            && cache(parser, 1_u64).ok
        {
            loop {
                if !IS_BLANK!((*parser).buffer) {
                    current_block = 11584701595673473500;
                    break;
                }
                skip(parser);
                if cache(parser, 1_u64).fail {
                    current_block = 11397968426844348457;
                    break;
                }
            }
            if current_block != 11397968426844348457 {
                if CHECK!((*parser).buffer, b'#') {
                    loop {
                        if IS_BREAKZ!((*parser).buffer) {
                            current_block = 6669252993407410313;
                            break;
                        }
                        skip(parser);
                        if cache(parser, 1_u64).fail {
                            current_block = 11397968426844348457;
                            break;
                        }
                    }
                } else {
                    current_block = 6669252993407410313;
                }
                if current_block != 11397968426844348457 {
                    if !IS_BREAKZ!((*parser).buffer) {
                        yaml_parser_set_scanner_error(
                            parser,
                            b"while scanning a directive\0" as *const u8 as *const libc::c_char,
                            start_mark,
                            b"did not find expected comment or line break\0" as *const u8
                                as *const libc::c_char,
                        );
                    } else {
                        if IS_BREAK!((*parser).buffer) {
                            if cache(parser, 2_u64).fail {
                                current_block = 11397968426844348457;
                            } else {
                                skip_line(parser);
                                current_block = 652864300344834934;
                            }
                        } else {
                            current_block = 652864300344834934;
                        }
                        if current_block != 11397968426844348457 {
                            yaml_free(name as *mut libc::c_void);
                            return OK;
                        }
                    }
                }
            }
        }
    }
    yaml_free(prefix as *mut libc::c_void);
    yaml_free(handle as *mut libc::c_void);
    yaml_free(name as *mut libc::c_void);
    FAIL
}

unsafe fn yaml_parser_scan_directive_name(
    parser: *mut YamlParserT,
    start_mark: YamlMarkT,
    name: *mut *mut yaml_char_t,
) -> Success {
    let current_block: u64;
    let mut string = NULL_STRING;
    STRING_INIT!(string);
    if cache(parser, 1_u64).ok {
        loop {
            if !IS_ALPHA!((*parser).buffer) {
                current_block = 10879442775620481940;
                break;
            }
            read!(parser, string);
            if cache(parser, 1_u64).fail {
                current_block = 8318012024179131575;
                break;
            }
        }
        if current_block != 8318012024179131575 {
            if string.start == string.pointer {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a directive\0" as *const u8
                        as *const libc::c_char,
                    start_mark,
                    b"could not find expected directive name\0"
                        as *const u8
                        as *const libc::c_char,
                );
            } else if !IS_BLANKZ!((*parser).buffer) {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a directive\0" as *const u8
                        as *const libc::c_char,
                    start_mark,
                    b"found unexpected non-alphabetical character\0"
                        as *const u8
                        as *const libc::c_char,
                );
            } else {
                *name = string.start;
                return OK;
            }
        }
    }
    STRING_DEL!(string);
    FAIL
}

unsafe fn yaml_parser_scan_version_directive_value(
    parser: *mut YamlParserT,
    start_mark: YamlMarkT,
    major: *mut libc::c_int,
    minor: *mut libc::c_int,
) -> Success {
    if cache(parser, 1_u64).fail {
        return FAIL;
    }
    while IS_BLANK!((*parser).buffer) {
        skip(parser);
        if cache(parser, 1_u64).fail {
            return FAIL;
        }
    }
    if yaml_parser_scan_version_directive_number(
        parser, start_mark, major,
    )
    .fail
    {
        return FAIL;
    }
    if !CHECK!((*parser).buffer, b'.') {
        yaml_parser_set_scanner_error(
            parser,
            b"while scanning a %YAML directive\0" as *const u8
                as *const libc::c_char,
            start_mark,
            b"did not find expected digit or '.' character\0"
                as *const u8 as *const libc::c_char,
        );
        return FAIL;
    }
    skip(parser);
    yaml_parser_scan_version_directive_number(parser, start_mark, minor)
}

const MAX_NUMBER_LENGTH: u64 = 9_u64;

unsafe fn yaml_parser_scan_version_directive_number(
    parser: *mut YamlParserT,
    start_mark: YamlMarkT,
    number: *mut libc::c_int,
) -> Success {
    let mut value: libc::c_int = 0;
    let mut length: size_t = 0_u64;
    if cache(parser, 1_u64).fail {
        return FAIL;
    }
    while !(*parser).buffer.is_empty() && IS_DIGIT!((*parser).buffer) {
        length = length.force_add(1);
        if length > MAX_NUMBER_LENGTH {
            yaml_parser_set_scanner_error(
                parser,
                b"while scanning a %YAML directive\0" as *const u8
                    as *const libc::c_char,
                start_mark,
                b"found extremely long version number\0" as *const u8
                    as *const libc::c_char,
            );
            return FAIL;
        }
        value =
            value.force_mul(10).force_add(AS_DIGIT!((*parser).buffer));
        (*parser).buffer.next();
        if cache(parser, 1_u64).fail {
            return FAIL;
        }
    }
    if length == 0 {
        yaml_parser_set_scanner_error(
            parser,
            b"while scanning a %YAML directive\0" as *const u8
                as *const libc::c_char,
            start_mark,
            b"did not find expected version number\0" as *const u8
                as *const libc::c_char,
        );
        return FAIL;
    }
    *number = value;
    OK
}

unsafe fn yaml_parser_scan_tag_directive_value(
    parser: *mut YamlParserT,
    start_mark: YamlMarkT,
    handle: *mut *mut yaml_char_t,
    prefix: *mut *mut yaml_char_t,
) -> Success {
    let mut current_block: u64;
    let mut handle_value: *mut yaml_char_t =
        ptr::null_mut::<yaml_char_t>();
    let mut prefix_value: *mut yaml_char_t =
        ptr::null_mut::<yaml_char_t>();
    if cache(parser, 1_u64).fail {
        current_block = 5231181710497607163;
    } else {
        current_block = 14916268686031723178;
    }
    'c_34337: loop {
        match current_block {
            5231181710497607163 => {
                yaml_free(handle_value as *mut libc::c_void);
                yaml_free(prefix_value as *mut libc::c_void);
                return FAIL;
            }
            _ => {
                if IS_BLANK!((*parser).buffer) {
                    skip(parser);
                    if cache(parser, 1_u64).fail {
                        current_block = 5231181710497607163;
                    } else {
                        current_block = 14916268686031723178;
                    }
                } else {
                    if yaml_parser_scan_tag_handle(
                        parser,
                        true,
                        start_mark,
                        addr_of_mut!(handle_value),
                    )
                    .fail
                    {
                        current_block = 5231181710497607163;
                        continue;
                    }
                    if cache(parser, 1_u64).fail {
                        current_block = 5231181710497607163;
                        continue;
                    }
                    if !IS_BLANK!((*parser).buffer) {
                        yaml_parser_set_scanner_error(
                            parser,
                            b"while scanning a %TAG directive\0"
                                as *const u8
                                as *const libc::c_char,
                            start_mark,
                            b"did not find expected whitespace\0"
                                as *const u8
                                as *const libc::c_char,
                        );
                        current_block = 5231181710497607163;
                    } else {
                        while IS_BLANK!((*parser).buffer) {
                            skip(parser);
                            if cache(parser, 1_u64).fail {
                                current_block = 5231181710497607163;
                                continue 'c_34337;
                            }
                        }
                        if yaml_parser_scan_tag_uri(
                            parser,
                            true,
                            true,
                            ptr::null_mut::<yaml_char_t>(),
                            start_mark,
                            addr_of_mut!(prefix_value),
                        )
                        .fail
                        {
                            current_block = 5231181710497607163;
                            continue;
                        }
                        if cache(parser, 1_u64).fail {
                            current_block = 5231181710497607163;
                            continue;
                        }
                        if !IS_BLANKZ!((*parser).buffer) {
                            yaml_parser_set_scanner_error(
                                parser,
                                b"while scanning a %TAG directive\0" as *const u8
                                    as *const libc::c_char,
                                start_mark,
                                b"did not find expected whitespace or line break\0" as *const u8
                                    as *const libc::c_char,
                            );
                            current_block = 5231181710497607163;
                        } else {
                            *handle = handle_value;
                            *prefix = prefix_value;
                            return OK;
                        }
                    }
                }
            }
        }
    }
}

unsafe fn yaml_parser_scan_anchor(
    parser: *mut YamlParserT,
    token: *mut YamlTokenT,
    type_: YamlTokenTypeT,
) -> Success {
    let current_block: u64;
    let mut length: libc::c_int = 0;
    let end_mark: YamlMarkT;
    let mut string = NULL_STRING;
    STRING_INIT!(string);
    let start_mark: YamlMarkT = (*parser).mark;
    skip(parser);
    if cache(parser, 1_u64).ok {
        loop {
            if !IS_ALPHA!((*parser).buffer) {
                current_block = 2868539653012386629;
                break;
            }
            read!(parser, string);
            if cache(parser, 1_u64).fail {
                current_block = 5883759901342942623;
                break;
            }
            length += 1;
        }
        if current_block != 5883759901342942623 {
            end_mark = (*parser).mark;
            if length == 0
                || !(IS_BLANKZ!((*parser).buffer)
                    || CHECK!((*parser).buffer, b'?')
                    || CHECK!((*parser).buffer, b':')
                    || CHECK!((*parser).buffer, b',')
                    || CHECK!((*parser).buffer, b']')
                    || CHECK!((*parser).buffer, b'}')
                    || CHECK!((*parser).buffer, b'%')
                    || CHECK!((*parser).buffer, b'@')
                    || CHECK!((*parser).buffer, b'`'))
            {
                yaml_parser_set_scanner_error(
                    parser,
                    if type_ == YamlAnchorToken {
                        b"while scanning an anchor\0" as *const u8 as *const libc::c_char
                    } else {
                        b"while scanning an alias\0" as *const u8 as *const libc::c_char
                    },
                    start_mark,
                    b"did not find expected alphabetic or numeric character\0" as *const u8
                        as *const libc::c_char,
                );
            } else {
                if type_ == YamlAnchorToken {
                    let _ = memset(
                        token.cast::<libc::c_void>(),
                        0,
                        size_of::<YamlTokenT>() as libc::c_ulong,
                    );
                    (*token).type_ = YamlAnchorToken;
                    (*token).start_mark = start_mark;
                    (*token).end_mark = end_mark;
                    let fresh220 =
                        addr_of_mut!((*token).data.anchor.value);
                    *fresh220 = string.start;
                } else {
                    let _ = memset(
                        token.cast::<libc::c_void>(),
                        0,
                        size_of::<YamlTokenT>() as libc::c_ulong,
                    );
                    (*token).type_ = YamlAliasToken;
                    (*token).start_mark = start_mark;
                    (*token).end_mark = end_mark;
                    let fresh221 =
                        addr_of_mut!((*token).data.alias.value);
                    *fresh221 = string.start;
                }
                return OK;
            }
        }
    }
    STRING_DEL!(string);
    FAIL
}

unsafe fn yaml_parser_scan_tag(
    parser: *mut YamlParserT,
    token: *mut YamlTokenT,
) -> Success {
    let mut current_block: u64;
    let mut handle: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
    let mut suffix: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
    let end_mark: YamlMarkT;
    let start_mark: YamlMarkT = (*parser).mark;
    if cache(parser, 2_u64).ok {
        if CHECK_AT!((*parser).buffer, b'<', 1) {
            handle = yaml_malloc(1_u64) as *mut yaml_char_t;
            *handle = b'\0';
            skip(parser);
            skip(parser);
            if yaml_parser_scan_tag_uri(
                parser,
                true,
                false,
                ptr::null_mut::<yaml_char_t>(),
                start_mark,
                addr_of_mut!(suffix),
            )
            .fail
            {
                current_block = 17708497480799081542;
            } else if !CHECK!((*parser).buffer, b'>') {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a tag\0" as *const u8
                        as *const libc::c_char,
                    start_mark,
                    b"did not find the expected '>'\0" as *const u8
                        as *const libc::c_char,
                );
                current_block = 17708497480799081542;
            } else {
                skip(parser);
                current_block = 4488286894823169796;
            }
        } else if yaml_parser_scan_tag_handle(
            parser,
            false,
            start_mark,
            addr_of_mut!(handle),
        )
        .fail
        {
            current_block = 17708497480799081542;
        } else if *handle == b'!'
            && *handle.wrapping_offset(1_isize) != b'\0'
            && *handle.wrapping_offset(
                strlen(handle as *mut libc::c_char).wrapping_sub(1_u64)
                    as isize,
            ) == b'!'
        {
            if yaml_parser_scan_tag_uri(
                parser,
                false,
                false,
                ptr::null_mut::<yaml_char_t>(),
                start_mark,
                addr_of_mut!(suffix),
            )
            .fail
            {
                current_block = 17708497480799081542;
            } else {
                current_block = 4488286894823169796;
            }
        } else if yaml_parser_scan_tag_uri(
            parser,
            false,
            false,
            handle,
            start_mark,
            addr_of_mut!(suffix),
        )
        .fail
        {
            current_block = 17708497480799081542;
        } else {
            yaml_free(handle as *mut libc::c_void);
            handle = yaml_malloc(2_u64) as *mut yaml_char_t;
            *handle = b'!';
            *handle.wrapping_offset(1_isize) = b'\0';

            if *suffix == b'\0' {
                core::mem::swap(&mut handle, &mut suffix);
            }

            current_block = 4488286894823169796;
        }
        if current_block != 17708497480799081542
            && cache(parser, 1_u64).ok
        {
            if !IS_BLANKZ!((*parser).buffer) {
                if (*parser).flow_level == 0
                    || !CHECK!((*parser).buffer, b',')
                {
                    yaml_parser_set_scanner_error(
                        parser,
                        b"while scanning a tag\0" as *const u8 as *const libc::c_char,
                        start_mark,
                        b"did not find expected whitespace or line break\0" as *const u8
                            as *const libc::c_char,
                    );
                    current_block = 17708497480799081542;
                } else {
                    current_block = 7333393191927787629;
                }
            } else {
                current_block = 7333393191927787629;
            }
            if current_block != 17708497480799081542 {
                end_mark = (*parser).mark;
                let _ = memset(
                    token.cast::<libc::c_void>(),
                    0,
                    size_of::<YamlTokenT>() as libc::c_ulong,
                );
                (*token).type_ = YamlTagToken;
                (*token).start_mark = start_mark;
                (*token).end_mark = end_mark;
                let fresh234 = addr_of_mut!((*token).data.tag.handle);
                *fresh234 = handle;
                let fresh235 = addr_of_mut!((*token).data.tag.suffix);
                *fresh235 = suffix;
                return OK;
            }
        }
    }
    yaml_free(handle as *mut libc::c_void);
    yaml_free(suffix as *mut libc::c_void);
    FAIL
}

unsafe fn yaml_parser_scan_tag_handle(
    parser: *mut YamlParserT,
    directive: bool,
    start_mark: YamlMarkT,
    handle: *mut *mut yaml_char_t,
) -> Success {
    let mut current_block: u64;
    let mut string = NULL_STRING;
    STRING_INIT!(string);
    if cache(parser, 1_u64).ok {
        if !CHECK!((*parser).buffer, b'!') {
            yaml_parser_set_scanner_error(
                parser,
                if directive {
                    b"while scanning a tag directive\0" as *const u8
                        as *const libc::c_char
                } else {
                    b"while scanning a tag\0" as *const u8
                        as *const libc::c_char
                },
                start_mark,
                b"did not find expected '!'\0" as *const u8
                    as *const libc::c_char,
            );
        } else {
            read!(parser, string);
            if cache(parser, 1_u64).ok {
                loop {
                    if !IS_ALPHA!((*parser).buffer) {
                        current_block = 7651349459974463963;
                        break;
                    }
                    read!(parser, string);
                    if cache(parser, 1_u64).fail {
                        current_block = 1771849829115608806;
                        break;
                    }
                }
                if current_block != 1771849829115608806 {
                    if CHECK!((*parser).buffer, b'!') {
                        read!(parser, string);
                        current_block = 5689001924483802034;
                    } else if directive
                        && !(*string.start == b'!'
                            && *string.start.wrapping_offset(1_isize)
                                == b'\0')
                    {
                        yaml_parser_set_scanner_error(
                            parser,
                            b"while parsing a tag directive\0"
                                as *const u8
                                as *const libc::c_char,
                            start_mark,
                            b"did not find expected '!'\0" as *const u8
                                as *const libc::c_char,
                        );
                        current_block = 1771849829115608806;
                    } else {
                        current_block = 5689001924483802034;
                    }
                    if current_block != 1771849829115608806 {
                        *handle = string.start;
                        return OK;
                    }
                }
            }
        }
    }
    STRING_DEL!(string);
    FAIL
}

unsafe fn yaml_parser_scan_tag_uri(
    parser: *mut YamlParserT,
    uri_char: bool,
    directive: bool,
    head: *mut yaml_char_t,
    start_mark: YamlMarkT,
    uri: *mut *mut yaml_char_t,
) -> Success {
    let mut current_block: u64;
    let mut length: size_t = if !head.is_null() {
        strlen(head as *mut libc::c_char)
    } else {
        0_u64
    };
    let mut string = NULL_STRING;
    STRING_INIT!(string);
    current_block = 14916268686031723178;
    'c_21953: loop {
        match current_block {
            15265153392498847348 => {
                STRING_DEL!(string);
                return FAIL;
            }
            _ => {
                if string.end.c_offset_from(string.start) as size_t
                    <= length
                {
                    yaml_string_extend(
                        addr_of_mut!(string.start),
                        addr_of_mut!(string.pointer),
                        addr_of_mut!(string.end),
                    );
                    current_block = 14916268686031723178;
                    continue;
                } else {
                    if length > 1_u64 {
                        let _ = memcpy(
                            string.start as *mut libc::c_void,
                            head.wrapping_offset(1_isize)
                                as *const libc::c_void,
                            length.wrapping_sub(1_u64),
                        );
                        string.pointer =
                            string.pointer.wrapping_offset(
                                length.wrapping_sub(1_u64) as isize,
                            );
                    }
                    if cache(parser, 1_u64).fail {
                        current_block = 15265153392498847348;
                        continue;
                    }
                    while !(*parser).buffer.is_empty()
                        && (IS_ALPHA!((*parser).buffer)
                            || CHECK!((*parser).buffer, b';')
                            || CHECK!((*parser).buffer, b'/')
                            || CHECK!((*parser).buffer, b'?')
                            || CHECK!((*parser).buffer, b':')
                            || CHECK!((*parser).buffer, b'@')
                            || CHECK!((*parser).buffer, b'&')
                            || CHECK!((*parser).buffer, b'=')
                            || CHECK!((*parser).buffer, b'+')
                            || CHECK!((*parser).buffer, b'$')
                            || CHECK!((*parser).buffer, b'.')
                            || CHECK!((*parser).buffer, b'%')
                            || CHECK!((*parser).buffer, b'!')
                            || CHECK!((*parser).buffer, b'~')
                            || CHECK!((*parser).buffer, b'*')
                            || CHECK!((*parser).buffer, b'\'')
                            || CHECK!((*parser).buffer, b'(')
                            || CHECK!((*parser).buffer, b')')
                            || uri_char
                                && (CHECK!((*parser).buffer, b',')
                                    || CHECK!((*parser).buffer, b'[')
                                    || CHECK!((*parser).buffer, b']')))
                    {
                        if CHECK!((*parser).buffer, b'%') {
                            STRING_EXTEND!(string);
                            if yaml_parser_scan_uri_escapes(
                                parser,
                                directive,
                                start_mark,
                                addr_of_mut!(string),
                            )
                            .fail
                            {
                                current_block = 15265153392498847348;
                                continue 'c_21953;
                            }
                        } else {
                            read!(parser, string);
                        }
                        length = length.force_add(1);
                        if cache(parser, 1_u64).fail {
                            current_block = 15265153392498847348;
                            continue 'c_21953;
                        }
                    }
                    if length == 0 {
                        STRING_EXTEND!(string);
                        yaml_parser_set_scanner_error(
                            parser,
                            if directive {
                                b"while parsing a %TAG directive\0"
                                    as *const u8
                                    as *const libc::c_char
                            } else {
                                b"while parsing a tag\0" as *const u8
                                    as *const libc::c_char
                            },
                            start_mark,
                            b"did not find expected tag URI\0"
                                as *const u8
                                as *const libc::c_char,
                        );
                        current_block = 15265153392498847348;
                    } else {
                        *uri = string.start;
                        return OK;
                    }
                }
            }
        }
    }
}

unsafe fn yaml_parser_scan_uri_escapes(
    parser: *mut YamlParserT,
    directive: bool,
    start_mark: YamlMarkT,
    string: *mut YamlStringT,
) -> Success {
    let mut width: libc::c_int = 0;
    loop {
        if cache(parser, 3_u64).fail {
            return FAIL;
        }
        if !(CHECK!((*parser).buffer, b'%')
            && IS_HEX_AT!((*parser).buffer, 1)
            && IS_HEX_AT!((*parser).buffer, 2))
        {
            yaml_parser_set_scanner_error(
                parser,
                if directive {
                    b"while parsing a %TAG directive\0" as *const u8
                        as *const libc::c_char
                } else {
                    b"while parsing a tag\0" as *const u8
                        as *const libc::c_char
                },
                start_mark,
                b"did not find URI escaped octet\0" as *const u8
                    as *const libc::c_char,
            );
            return FAIL;
        }
        let octet: libc::c_uchar = libc::c_uchar::try_from(
            (AS_HEX_AT!((*parser).buffer, 1) << 4)
                + AS_HEX_AT!((*parser).buffer, 2),
        )
        .unwrap();
        if width == 0 {
            width = if octet & 0x80 == 0 {
                1
            } else if octet & 0xE0 == 0xC0 {
                2
            } else if octet & 0xF0 == 0xE0 {
                3
            } else if octet & 0xF8 == 0xF0 {
                4
            } else {
                0
            };
            if width == 0 {
                yaml_parser_set_scanner_error(
                    parser,
                    if directive {
                        (b"while parsing a %TAG directive\0"
                            as *const u8)
                            .cast::<libc::c_char>()
                    } else {
                        (b"while parsing a tag\0" as *const u8)
                            .cast::<libc::c_char>()
                    },
                    start_mark,
                    (b"found an incorrect leading UTF-8 octet\0"
                        as *const u8)
                        .cast::<libc::c_char>(),
                );
                return FAIL;
            }
        } else if octet & 0xC0 != 0x80 {
            yaml_parser_set_scanner_error(
                parser,
                if directive {
                    (b"while parsing a %TAG directive\0" as *const u8)
                        .cast::<libc::c_char>()
                } else {
                    (b"while parsing a tag\0" as *const u8)
                        .cast::<libc::c_char>()
                },
                start_mark,
                (b"found an incorrect trailing UTF-8 octet\0"
                    as *const u8)
                    .cast::<libc::c_char>(),
            );
            return FAIL;
        }
        let fresh368 = addr_of_mut!((*string).pointer);
        let fresh369 = *fresh368;
        *fresh368 = (*fresh368).wrapping_offset(1);
        *fresh369 = octet;
        skip(parser);
        skip(parser);
        skip(parser);
        width -= 1;
        if width == 0 {
            break;
        }
    }
    OK
}

unsafe fn scan_block_scalar_header(
    parser: *mut YamlParserT,
    chomping: &mut libc::c_int,
    increment: &mut libc::c_int,
    start_mark: YamlMarkT,
) -> u64 {
    if CHECK!((*parser).buffer, b'+') || CHECK!((*parser).buffer, b'-')
    {
        *chomping = if CHECK!((*parser).buffer, b'+') {
            1
        } else {
            -1
        };
        skip(parser);

        if cache(parser, 1_u64).fail {
            return 14984465786483313892;
        }

        if IS_DIGIT!((*parser).buffer) {
            if CHECK!((*parser).buffer, b'0') {
                yaml_parser_set_scanner_error(
                    parser,
                    (b"while scanning a block scalar\0" as *const u8)
                        .cast::<libc::c_char>(),
                    start_mark,
                    (b"found an indentation indicator equal to 0\0"
                        as *const u8)
                        .cast::<libc::c_char>(),
                );
                return 14984465786483313892;
            }
            *increment = AS_DIGIT!((*parser).buffer);
            skip(parser);
        }
        return 11913429853522160501;
    } else if IS_DIGIT!((*parser).buffer) {
        if CHECK!((*parser).buffer, b'0') {
            yaml_parser_set_scanner_error(
                parser,
                (b"while scanning a block scalar\0" as *const u8)
                    .cast::<libc::c_char>(),
                start_mark,
                (b"found an indentation indicator equal to 0\0"
                    as *const u8)
                    .cast::<libc::c_char>(),
            );
            return 14984465786483313892;
        }
        *increment = AS_DIGIT!((*parser).buffer);
        skip(parser);

        if cache(parser, 1_u64).fail {
            return 14984465786483313892;
        }

        if CHECK!((*parser).buffer, b'+')
            || CHECK!((*parser).buffer, b'-')
        {
            *chomping = if CHECK!((*parser).buffer, b'+') {
                1
            } else {
                -1
            };
            skip(parser);
        }
        return 11913429853522160501;
    }
    11913429853522160501
}

unsafe fn yaml_parser_scan_block_scalar(
    parser: *mut YamlParserT,
    token: *mut YamlTokenT,
    literal: bool,
) -> Success {
    let mut current_block: u64;
    let mut end_mark: YamlMarkT;
    let mut string = NULL_STRING;
    let mut leading_break = NULL_STRING;
    let mut trailing_breaks = NULL_STRING;
    let mut chomping: libc::c_int = 0;
    let mut increment: libc::c_int = 0;
    let mut indent: libc::c_int = 0;
    let mut leading_blank: libc::c_int = 0;
    let mut trailing_blank: libc::c_int;
    STRING_INIT!(string);
    STRING_INIT!(leading_break);
    STRING_INIT!(trailing_breaks);
    let start_mark: YamlMarkT = (*parser).mark;
    skip(parser);
    if cache(parser, 1_u64).ok {
        current_block = scan_block_scalar_header(
            parser,
            &mut chomping,
            &mut increment,
            start_mark,
        );
        if current_block != 14984465786483313892
            && cache(parser, 1_u64).ok
        {
            loop {
                if !IS_BLANK!((*parser).buffer) {
                    current_block = 4090602189656566074;
                    break;
                }
                skip(parser);
                if cache(parser, 1_u64).fail {
                    current_block = 14984465786483313892;
                    break;
                }
            }
            if current_block != 14984465786483313892 {
                if CHECK!((*parser).buffer, b'#') {
                    loop {
                        if IS_BREAKZ!((*parser).buffer) {
                            current_block = 12997042908615822766;
                            break;
                        }
                        skip(parser);
                        if cache(parser, 1_u64).fail {
                            current_block = 14984465786483313892;
                            break;
                        }
                    }
                } else {
                    current_block = 12997042908615822766;
                }
                if current_block != 14984465786483313892 {
                    if !IS_BREAKZ!((*parser).buffer) {
                        yaml_parser_set_scanner_error(
                            parser,
                            b"while scanning a block scalar\0" as *const u8 as *const libc::c_char,
                            start_mark,
                            b"did not find expected comment or line break\0" as *const u8
                                as *const libc::c_char,
                        );
                    } else {
                        if IS_BREAK!((*parser).buffer) {
                            if cache(parser, 2_u64).fail {
                                current_block = 14984465786483313892;
                            } else {
                                skip_line(parser);
                                current_block = 13619784596304402172;
                            }
                        } else {
                            current_block = 13619784596304402172;
                        }
                        if current_block != 14984465786483313892 {
                            end_mark = (*parser).mark;
                            if increment != 0 {
                                indent = if (*parser).indent >= 0 {
                                    (*parser).indent + increment
                                } else {
                                    increment
                                };
                            }
                            if yaml_parser_scan_block_scalar_breaks(
                                parser,
                                addr_of_mut!(indent),
                                addr_of_mut!(trailing_breaks),
                                start_mark,
                                addr_of_mut!(end_mark),
                            )
                            .ok && cache(parser, 1_u64).ok
                            {
                                's_281: loop {
                                    if (*parser).mark.column
                                        as libc::c_int
                                        != indent
                                        || IS_Z!((*parser).buffer)
                                    {
                                        current_block =
                                            5793491756164225964;
                                        break;
                                    }
                                    trailing_blank =
                                        IS_BLANK!((*parser).buffer)
                                            as libc::c_int;
                                    if !literal
                                        && *leading_break.start == b'\n'
                                        && leading_blank == 0
                                        && trailing_blank == 0
                                    {
                                        if *trailing_breaks.start
                                            == b'\0'
                                        {
                                            STRING_EXTEND!(string);
                                            let fresh418 =
                                                string.pointer;
                                            string.pointer = string
                                                .pointer
                                                .wrapping_offset(1);
                                            *fresh418 = b' ';
                                        }
                                        CLEAR!(leading_break);
                                    } else {
                                        JOIN!(string, leading_break);
                                        CLEAR!(leading_break);
                                    }
                                    JOIN!(string, trailing_breaks);
                                    CLEAR!(trailing_breaks);
                                    leading_blank =
                                        IS_BLANK!((*parser).buffer)
                                            as libc::c_int;
                                    while !IS_BREAKZ!((*parser).buffer)
                                    {
                                        read!(parser, string);
                                        if cache(parser, 1_u64).fail {
                                            current_block =
                                                14984465786483313892;
                                            break 's_281;
                                        }
                                    }
                                    if cache(parser, 2_u64).fail {
                                        current_block =
                                            14984465786483313892;
                                        break;
                                    }
                                    read_line!(parser, leading_break);
                                    if yaml_parser_scan_block_scalar_breaks(
                                        parser,
                                        addr_of_mut!(indent),
                                        addr_of_mut!(trailing_breaks),
                                        start_mark,
                                        addr_of_mut!(end_mark),
                                    )
                                    .fail
                                    {
                                        current_block = 14984465786483313892;
                                        break;
                                    }
                                }
                                if current_block != 14984465786483313892
                                {
                                    if chomping != -1 {
                                        JOIN!(string, leading_break);
                                        current_block =
                                            17787701279558130514;
                                    } else {
                                        current_block =
                                            17787701279558130514;
                                    }
                                    if current_block
                                        != 14984465786483313892
                                    {
                                        if chomping == 1 {
                                            JOIN!(
                                                string,
                                                trailing_breaks
                                            );
                                        }
                                        let _ = memset(
                                            token
                                                .cast::<libc::c_void>(),
                                            0,
                                            size_of::<YamlTokenT>()
                                                as libc::c_ulong,
                                        );
                                        (*token).type_ =
                                            YamlScalarToken;
                                        (*token).start_mark =
                                            start_mark;
                                        (*token).end_mark = end_mark;
                                        let fresh479 = addr_of_mut!(
                                            (*token).data.scalar.value
                                        );
                                        *fresh479 = string.start;
                                        (*token).data.scalar.length =
                                            string
                                                .pointer
                                                .c_offset_from(
                                                    string.start,
                                                )
                                                as size_t;
                                        (*token).data.scalar.style =
                                            if literal {
                                                YamlLiteralScalarStyle
                                            } else {
                                                YamlFoldedScalarStyle
                                            };
                                        STRING_DEL!(leading_break);
                                        STRING_DEL!(trailing_breaks);
                                        return OK;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    STRING_DEL!(string);
    STRING_DEL!(leading_break);
    STRING_DEL!(trailing_breaks);
    FAIL
}

unsafe fn yaml_parser_scan_block_scalar_breaks(
    parser: *mut YamlParserT,
    indent: *mut libc::c_int,
    breaks: *mut YamlStringT,
    start_mark: YamlMarkT,
    end_mark: *mut YamlMarkT,
) -> Success {
    let mut max_indent: libc::c_int = 0;
    *end_mark = (*parser).mark;
    loop {
        if cache(parser, 1_u64).fail {
            return FAIL;
        }
        while (*indent == 0
            || libc::c_int::try_from((*parser).mark.column)
                .unwrap_or(0)
                < *indent)
            && IS_SPACE!((*parser).buffer)
        {
            skip(parser);
            if cache(parser, 1_u64).fail {
                return FAIL;
            }
        }
        if libc::c_int::try_from((*parser).mark.column).unwrap_or(0)
            > max_indent
        {
            max_indent =
                match libc::c_int::try_from((*parser).mark.column) {
                    Ok(val) => val,
                    Err(_) => return FAIL,
                };
        }
        if (*indent == 0
            || libc::c_int::try_from((*parser).mark.column)
                .unwrap_or(0)
                < *indent)
            && IS_TAB!((*parser).buffer)
        {
            yaml_parser_set_scanner_error(
                parser,
                (b"while scanning a block scalar\0" as *const u8).cast::<libc::c_char>(),
                start_mark,
                (b"found a tab character where an indentation space is expected\0" as *const u8).cast::<libc::c_char>(),
            );
            return FAIL;
        }
        if !IS_BREAK!((*parser).buffer) {
            break;
        }
        if cache(parser, 2_u64).fail {
            return FAIL;
        }
        read_line!(parser, *breaks);
        *end_mark = (*parser).mark;
    }
    if *indent == 0 {
        *indent = max_indent;
        if *indent < (*parser).indent + 1 {
            *indent = (*parser).indent + 1;
        }
        if *indent < 1 {
            *indent = 1;
        }
    }
    OK
}

unsafe fn yaml_parser_scan_flow_scalar(
    parser: *mut YamlParserT,
    token: *mut YamlTokenT,
    single: bool,
) -> Success {
    let current_block: u64;
    let end_mark: YamlMarkT;
    let mut string = NULL_STRING;
    let mut leading_break = NULL_STRING;
    let mut trailing_breaks = NULL_STRING;
    let mut whitespaces = NULL_STRING;
    let mut leading_blanks;
    STRING_INIT!(string);
    STRING_INIT!(leading_break);
    STRING_INIT!(trailing_breaks);
    STRING_INIT!(whitespaces);
    let start_mark: YamlMarkT = (*parser).mark;
    skip(parser);
    's_58: loop {
        if cache(parser, 4_u64).fail {
            current_block = 8114179180390253173;
            break;
        }
        if (*parser).mark.column == 0_u64
            && (CHECK_AT!((*parser).buffer, b'-', 0)
                && CHECK_AT!((*parser).buffer, b'-', 1)
                && CHECK_AT!((*parser).buffer, b'-', 2)
                || CHECK_AT!((*parser).buffer, b'.', 0)
                    && CHECK_AT!((*parser).buffer, b'.', 1)
                    && CHECK_AT!((*parser).buffer, b'.', 2))
            && IS_BLANKZ_AT!((*parser).buffer, 3)
        {
            yaml_parser_set_scanner_error(
                parser,
                (b"while scanning a quoted scalar\0" as *const u8)
                    .cast::<libc::c_char>(),
                start_mark,
                (b"found unexpected document indicator\0" as *const u8)
                    .cast::<libc::c_char>(),
            );
            current_block = 8114179180390253173;
            break;
        } else if IS_Z!((*parser).buffer) {
            yaml_parser_set_scanner_error(
                parser,
                b"while scanning a quoted scalar\0" as *const u8
                    as *const libc::c_char,
                start_mark,
                (b"found unexpected end of stream\0" as *const u8)
                    .cast::<libc::c_char>(),
            );
            current_block = 8114179180390253173;
            break;
        } else {
            if cache(parser, 2_u64).fail {
                current_block = 8114179180390253173;
                break;
            }
            leading_blanks = false;
            while !IS_BLANKZ!((*parser).buffer) {
                if single
                    && CHECK_AT!((*parser).buffer, b'\'', 0)
                    && CHECK_AT!((*parser).buffer, b'\'', 1)
                {
                    STRING_EXTEND!(string);
                    let fresh521 = string.pointer;
                    string.pointer = string.pointer.wrapping_offset(1);
                    *fresh521 = b'\'';
                    skip(parser);
                    skip(parser);
                } else {
                    if CHECK!(
                        (*parser).buffer,
                        if single { b'\'' } else { b'"' }
                    ) {
                        break;
                    }
                    if !single
                        && CHECK!((*parser).buffer, b'\\')
                        && IS_BREAK_AT!((*parser).buffer, 1)
                    {
                        if cache(parser, 3_u64).fail {
                            current_block = 8114179180390253173;
                            break 's_58;
                        }
                        skip(parser);
                        skip_line(parser);
                        leading_blanks = true;
                        break;
                    } else if !single && CHECK!((*parser).buffer, b'\\')
                    {
                        let mut code_length: size_t = 0_u64;
                        STRING_EXTEND!(string);
                        match *(*parser)
                            .buffer
                            .pointer
                            .wrapping_offset(1_isize)
                        {
                            b'0' => {
                                let fresh542 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh542 = b'\0';
                            }
                            b'a' => {
                                let fresh543 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh543 = b'\x07';
                            }
                            b'b' => {
                                let fresh544 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh544 = b'\x08';
                            }
                            b't' | b'\t' => {
                                let fresh545 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh545 = b'\t';
                            }
                            b'n' => {
                                let fresh546 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh546 = b'\n';
                            }
                            b'v' => {
                                let fresh547 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh547 = b'\x0B';
                            }
                            b'f' => {
                                let fresh548 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh548 = b'\x0C';
                            }
                            b'r' => {
                                let fresh549 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh549 = b'\r';
                            }
                            b'e' => {
                                let fresh550 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh550 = b'\x1B';
                            }
                            b' ' => {
                                let fresh551 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh551 = b' ';
                            }
                            b'"' => {
                                let fresh552 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh552 = b'"';
                            }
                            b'/' => {
                                let fresh553 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh553 = b'/';
                            }
                            b'\\' => {
                                let fresh554 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh554 = b'\\';
                            }
                            // NEL (#x85)
                            b'N' => {
                                let fresh555 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh555 = b'\xC2';
                                let fresh556 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh556 = b'\x85';
                            }
                            // #xA0
                            b'_' => {
                                let fresh557 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh557 = b'\xC2';
                                let fresh558 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh558 = b'\xA0';
                            }
                            // LS (#x2028)
                            b'L' => {
                                let fresh559 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh559 = b'\xE2';
                                let fresh560 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh560 = b'\x80';
                                let fresh561 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh561 = b'\xA8';
                            }
                            // PS (#x2029)
                            b'P' => {
                                let fresh562 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh562 = b'\xE2';
                                let fresh563 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh563 = b'\x80';
                                let fresh564 = string.pointer;
                                string.pointer =
                                    string.pointer.wrapping_offset(1);
                                *fresh564 = b'\xA9';
                            }
                            b'x' => {
                                code_length = 2_u64;
                            }
                            b'u' => {
                                code_length = 4_u64;
                            }
                            b'U' => {
                                code_length = 8_u64;
                            }
                            _ => {
                                yaml_parser_set_scanner_error(
                                    parser,
                                    (b"while parsing a quoted scalar\0" as *const u8).cast::<libc::c_char>(),
                                    start_mark,
                                    (b"found unknown escape character\0" as *const u8).cast::<libc::c_char>(),
                                );
                                current_block = 8114179180390253173;
                                break 's_58;
                            }
                        }
                        skip(parser);
                        skip(parser);
                        if code_length != 0 {
                            let mut value: libc::c_uint = 0;
                            let mut k: size_t;
                            if cache(parser, code_length).fail {
                                current_block = 8114179180390253173;
                                break 's_58;
                            }
                            k = 0_u64;
                            while k < code_length {
                                if !IS_HEX_AT!(
                                    (*parser).buffer,
                                    k as isize
                                ) {
                                    yaml_parser_set_scanner_error(
                                        parser,
                                        (b"while parsing a quoted scalar\0" as *const u8).cast::<libc::c_char>(),
                                        start_mark,
                                        (b"did not find expected hexadecimal number\0" as *const u8).cast::<libc::c_char>(),
                                    );
                                    current_block = 8114179180390253173;
                                    break 's_58;
                                } else {
                                    value = (value << 4).force_add(
                                        AS_HEX_AT!(
                                            (*parser).buffer,
                                            k as isize
                                        )
                                            as libc::c_uint,
                                    );
                                    k = k.force_add(1);
                                }
                            }
                            if (0xD800..=0xDFFF).contains(&value)
                                || value > 0x10FFFF
                            {
                                yaml_parser_set_scanner_error(
                                    parser,
                                    (b"while parsing a quoted scalar\0" as *const u8).cast::<libc::c_char>(),
                                    start_mark,
                                    (b"found invalid Unicode character escape code\0" as *const u8).cast::<libc::c_char>(),
                                );
                                current_block = 8114179180390253173;
                                break 's_58;
                            } else {
                                if value <= 0x7F {
                                    let fresh573 = string.pointer;
                                    string.pointer = string
                                        .pointer
                                        .wrapping_offset(1);
                                    *fresh573 =
                                        yaml_char_t::try_from(value)
                                            .unwrap();
                                } else if value <= 0x7FF {
                                    let fresh574 = string.pointer;
                                    string.pointer = string
                                        .pointer
                                        .wrapping_offset(1);
                                    *fresh574 = yaml_char_t::try_from(
                                        0xC0_u32.force_add(value >> 6),
                                    )
                                    .unwrap();
                                    let fresh575 = string.pointer;
                                    string.pointer = string
                                        .pointer
                                        .wrapping_offset(1);
                                    *fresh575 = yaml_char_t::try_from(
                                        0x80_u32
                                            .force_add(value & 0x3F),
                                    )
                                    .unwrap();
                                } else if value <= 0xFFFF {
                                    let fresh576 = string.pointer;
                                    string.pointer = string
                                        .pointer
                                        .wrapping_offset(1);
                                    *fresh576 = yaml_char_t::try_from(
                                        0xE0_u32.force_add(value >> 12),
                                    )
                                    .unwrap();
                                    let fresh577 = string.pointer;
                                    string.pointer = string
                                        .pointer
                                        .wrapping_offset(1);
                                    *fresh577 = yaml_char_t::try_from(
                                        0x80_u32.force_add(
                                            (value >> 6) & 0x3F,
                                        ),
                                    )
                                    .unwrap();
                                    let fresh578 = string.pointer;
                                    string.pointer = string
                                        .pointer
                                        .wrapping_offset(1);
                                    *fresh578 = yaml_char_t::try_from(
                                        0x80_u32
                                            .force_add(value & 0x3F),
                                    )
                                    .unwrap();
                                } else {
                                    let fresh579 = string.pointer;
                                    string.pointer = string
                                        .pointer
                                        .wrapping_offset(1);
                                    *fresh579 = yaml_char_t::try_from(
                                        0xF0_u32.force_add(value >> 18),
                                    )
                                    .unwrap();
                                    let fresh580 = string.pointer;
                                    string.pointer = string
                                        .pointer
                                        .wrapping_offset(1);
                                    *fresh580 = yaml_char_t::try_from(
                                        0x80_u32.force_add(
                                            (value >> 12) & 0x3F,
                                        ),
                                    )
                                    .unwrap();
                                    let fresh581 = string.pointer;
                                    string.pointer = string
                                        .pointer
                                        .wrapping_offset(1);
                                    *fresh581 = yaml_char_t::try_from(
                                        0x80_u32.force_add(
                                            (value >> 6) & 0x3F,
                                        ),
                                    )
                                    .unwrap();
                                    let fresh582 = string.pointer;
                                    string.pointer = string
                                        .pointer
                                        .wrapping_offset(1);
                                    *fresh582 = yaml_char_t::try_from(
                                        0x80_u32
                                            .force_add(value & 0x3F),
                                    )
                                    .unwrap();
                                }
                                k = 0_u64;
                                while k < code_length {
                                    skip(parser);
                                    k = k.force_add(1);
                                }
                            }
                        }
                    } else {
                        read!(parser, string);
                    }
                }
                if cache(parser, 2_u64).fail {
                    current_block = 8114179180390253173;
                    break 's_58;
                }
            }
            if cache(parser, 1_u64).fail {
                current_block = 8114179180390253173;
                break;
            }
            if CHECK!(
                (*parser).buffer,
                if single { b'\'' } else { b'"' }
            ) {
                current_block = 7468767852762055642;
                break;
            }
            if cache(parser, 1_u64).fail {
                current_block = 8114179180390253173;
                break;
            }
            while IS_BLANK!((*parser).buffer)
                || IS_BREAK!((*parser).buffer)
            {
                if IS_BLANK!((*parser).buffer) {
                    if leading_blanks {
                        skip(parser);
                    } else {
                        read!(parser, whitespaces);
                    }
                } else {
                    if cache(parser, 2_u64).fail {
                        current_block = 8114179180390253173;
                        break 's_58;
                    }
                    if leading_blanks {
                        read_line!(parser, trailing_breaks);
                    } else {
                        CLEAR!(whitespaces);
                        read_line!(parser, leading_break);
                        leading_blanks = true;
                    }
                }
                if cache(parser, 1_u64).fail {
                    current_block = 8114179180390253173;
                    break 's_58;
                }
            }
            if leading_blanks {
                if *leading_break.start == b'\n' {
                    if *trailing_breaks.start == b'\0' {
                        STRING_EXTEND!(string);
                        let fresh711 = string.pointer;
                        string.pointer =
                            string.pointer.wrapping_offset(1);
                        *fresh711 = b' ';
                    } else {
                        JOIN!(string, trailing_breaks);
                        CLEAR!(trailing_breaks);
                    }
                    CLEAR!(leading_break);
                } else {
                    JOIN!(string, leading_break);
                    JOIN!(string, trailing_breaks);
                    CLEAR!(leading_break);
                    CLEAR!(trailing_breaks);
                }
            } else {
                JOIN!(string, whitespaces);
                CLEAR!(whitespaces);
            }
        }
    }
    if current_block != 8114179180390253173 {
        skip(parser);
        end_mark = (*parser).mark;
        let _ = memset(
            token.cast::<libc::c_void>(),
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
        (*token).type_ = YamlScalarToken;
        (*token).start_mark = start_mark;
        (*token).end_mark = end_mark;
        let fresh716 = addr_of_mut!((*token).data.scalar.value);
        *fresh716 = string.start;
        (*token).data.scalar.length =
            string.pointer.c_offset_from(string.start) as size_t;
        (*token).data.scalar.style = if single {
            YamlSingleQuotedScalarStyle
        } else {
            YamlDoubleQuotedScalarStyle
        };
        STRING_DEL!(leading_break);
        STRING_DEL!(trailing_breaks);
        STRING_DEL!(whitespaces);
        return OK;
    }
    STRING_DEL!(string);
    STRING_DEL!(leading_break);
    STRING_DEL!(trailing_breaks);
    STRING_DEL!(whitespaces);
    FAIL
}

// 1. First, let's add a safe string buffer size check function
#[inline]
unsafe fn check_string_buffer_size(
    current_size: usize,
    additional_size: usize,
) -> Result<usize, &'static str> {
    current_size
        .checked_add(additional_size)
        .filter(|&total| total <= MAX_SCALAR_SIZE)
        .ok_or("String buffer would exceed maximum allowed size")
}

macro_rules! JOIN_SAFE {
    ($target:expr, $source:expr) => {{
        let target_offset =
            ($target).pointer.offset_from(($target).start);
        let source_offset =
            ($source).pointer.offset_from(($source).start);

        // Ensure offsets are non-negative before casting to usize
        if target_offset < 0 || source_offset < 0 {
            Err(YamlScannerError)
        } else {
            let target_len = target_offset as usize;
            let source_len = source_offset as usize;

            // Check if joining would exceed buffer size
            if let Ok(_) =
                check_string_buffer_size(target_len, source_len)
            {
                if source_len > 0 {
                    let _ = memcpy(
                        ($target).pointer.cast::<libc::c_void>(),
                        ($source).start as *const libc::c_void,
                        source_len.try_into().unwrap(),
                    );
                    ($target).pointer =
                        ($target).pointer.add(source_len);
                }
                Ok(())
            } else {
                Err(YamlScannerError)
            }
        }
    }};
}

// Helper macro for cleanup and error return
macro_rules! break_with_cleanup {
    ($string:expr, $leading_break:expr, $trailing_breaks:expr, $whitespaces:expr) => {{
        STRING_DEL!($string);
        STRING_DEL!($leading_break);
        STRING_DEL!($trailing_breaks);
        STRING_DEL!($whitespaces);
        return FAIL;
    }};
}

unsafe fn scan_plain_scalar_init(
    parser: *mut YamlParserT,
) -> (
    YamlStringT,
    YamlStringT,
    YamlStringT,
    YamlStringT,
    YamlMarkT,
    i32,
) {
    let mut string = NULL_STRING;
    let mut leading_break = NULL_STRING;
    let mut trailing_breaks = NULL_STRING;
    let mut whitespaces = NULL_STRING;

    STRING_INIT!(string);
    STRING_INIT!(leading_break);
    STRING_INIT!(trailing_breaks);
    STRING_INIT!(whitespaces);

    let end_mark = (*parser).mark;
    let indent: libc::c_int = (*parser).indent + 1;

    (
        string,
        leading_break,
        trailing_breaks,
        whitespaces,
        end_mark,
        indent,
    )
}

unsafe fn check_document_indicators(parser: *mut YamlParserT) -> bool {
    if (*parser).mark.column == 0
        && ((CHECK_AT!((*parser).buffer, b'-', 0)
            && CHECK_AT!((*parser).buffer, b'-', 1)
            && CHECK_AT!((*parser).buffer, b'-', 2))
            || (CHECK_AT!((*parser).buffer, b'.', 0)
                && CHECK_AT!((*parser).buffer, b'.', 1)
                && CHECK_AT!((*parser).buffer, b'.', 2)))
        && IS_BLANKZ_AT!((*parser).buffer, 3)
    {
        return true;
    }
    false
}

unsafe fn handle_flow_indicators(
    parser: *mut YamlParserT,
    start_mark: YamlMarkT,
) -> Result<bool, ()> {
    if (*parser).flow_level != 0
        && CHECK!((*parser).buffer, b':')
        && (CHECK_AT!((*parser).buffer, b',', 1)
            || CHECK_AT!((*parser).buffer, b'?', 1)
            || CHECK_AT!((*parser).buffer, b'[', 1)
            || CHECK_AT!((*parser).buffer, b']', 1)
            || CHECK_AT!((*parser).buffer, b'{', 1)
            || CHECK_AT!((*parser).buffer, b'}', 1))
    {
        yaml_parser_set_scanner_error(
            parser,
            b"while scanning a plain scalar\0".as_ptr().cast::<i8>(),
            start_mark,
            b"found unexpected ':'\0".as_ptr().cast::<i8>(),
        );
        return Err(());
    }

    if CHECK!((*parser).buffer, b':')
        && IS_BLANKZ_AT!((*parser).buffer, 1)
        || (*parser).flow_level != 0
            && (CHECK!((*parser).buffer, b',')
                || CHECK!((*parser).buffer, b'[')
                || CHECK!((*parser).buffer, b']')
                || CHECK!((*parser).buffer, b'{')
                || CHECK!((*parser).buffer, b'}'))
    {
        return Ok(true);
    }

    Ok(false)
}

unsafe fn handle_whitespace_and_breaks(
    parser: *mut YamlParserT,
    string: &mut YamlStringT,
    leading_break: &mut YamlStringT,
    trailing_breaks: &mut YamlStringT,
    whitespaces: &mut YamlStringT,
    leading_blanks: &mut bool,
    start_mark: YamlMarkT,
) -> Result<(), ()> {
    if *leading_blanks || whitespaces.start != whitespaces.pointer {
        if *leading_blanks {
            if *leading_break.start == b'\n' {
                if *trailing_breaks.start == b'\0' {
                    // Check and extend buffer
                    let current_len =
                        string.pointer.offset_from(string.start)
                            as usize;
                    let additional_size = 1; // Space for the next character
                    let required_size = current_len + additional_size;

                    if check_string_buffer_size(required_size, 1)
                        .is_ok()
                    {
                        STRING_EXTEND!(string);
                        *string.pointer = b' ';
                        string.pointer = string.pointer.add(1);
                    } else {
                        yaml_parser_set_scanner_error(
                            parser,
                            b"while scanning a plain scalar\0"
                                .as_ptr()
                                .cast::<i8>(),
                            start_mark,
                            b"while scanning a plain scalar\0"
                                .as_ptr()
                                .cast::<i8>(),
                        );
                        return Err(());
                    }
                } else {
                    if JOIN_SAFE!(string, trailing_breaks).is_err() {
                        yaml_parser_set_scanner_error(
                            parser,
                            b"while scanning a plain scalar\0"
                                .as_ptr()
                                .cast::<i8>(),
                            start_mark,
                            b"while scanning a plain scalar\0"
                                .as_ptr()
                                .cast::<i8>(),
                        );
                        return Err(());
                    }
                    CLEAR!(trailing_breaks);
                }
                CLEAR!(leading_break);
            } else {
                if JOIN_SAFE!(string, leading_break).is_err()
                    || JOIN_SAFE!(string, trailing_breaks).is_err()
                {
                    yaml_parser_set_scanner_error(
                        parser,
                        b"while scanning a plain scalar\0"
                            .as_ptr()
                            .cast::<i8>(),
                        start_mark,
                        b"while scanning a plain scalar\0"
                            .as_ptr()
                            .cast::<i8>(),
                    );
                    return Err(());
                }
                CLEAR!(leading_break);
                CLEAR!(trailing_breaks);
            }
            *leading_blanks = false;
        } else {
            if JOIN_SAFE!(string, whitespaces).is_err() {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a plain scalar\0"
                        .as_ptr()
                        .cast::<i8>(),
                    start_mark,
                    b"while scanning a plain scalar\0"
                        .as_ptr()
                        .cast::<i8>(),
                );
                return Err(());
            }
            CLEAR!(whitespaces);
        }
    }
    Ok(())
}

unsafe fn handle_blank_characters(
    parser: *mut YamlParserT,
    indent: i32,
    leading_blanks: bool,
    start_mark: YamlMarkT,
) -> Result<(), ()> {
    if leading_blanks {
        if let Ok(column_as_i32) =
            libc::c_int::try_from((*parser).mark.column)
        {
            if column_as_i32 < indent && IS_TAB!((*parser).buffer) {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a plain scalar\0".as_ptr().cast::<i8>(),
                    start_mark,
                    b"found a tab character that violates indentation\0".as_ptr().cast::<i8>(),
                );
                return Err(());
            }
        } else {
            // Handle cases where the column value exceeds the range of libc::c_int
            yaml_parser_set_scanner_error(
                parser,
                b"while scanning a plain scalar\0"
                    .as_ptr()
                    .cast::<i8>(),
                start_mark,
                b"column value too large to process\0"
                    .as_ptr()
                    .cast::<i8>(),
            );
            return Err(());
        }
    }
    Ok(())
}

unsafe fn initialize_plain_scalar_token(
    token: *mut YamlTokenT,
    string: &YamlStringT,
    start_mark: YamlMarkT,
    end_mark: YamlMarkT,
    parser: *mut YamlParserT,
    leading_blanks: bool,
    leading_break: &mut YamlStringT,
    trailing_breaks: &mut YamlStringT,
    whitespaces: &mut YamlStringT,
) -> Success {
    // Initialize token
    ptr::write_bytes(
        token.cast::<libc::c_void>(),
        0,
        size_of::<YamlTokenT>(),
    );

    (*token).type_ = YamlScalarToken;
    (*token).start_mark = start_mark;
    (*token).end_mark = end_mark;
    (*token).data.scalar.value = string.start;

    let offset = string.pointer.offset_from(string.start);
    assert!(
        offset >= 0,
        "Pointer offset is negative! This indicates an invalid memory layout."
    );

    (*token).data.scalar.length =
        offset.try_into().expect("Offset cannot be negative");

    (*token).data.scalar.style = YamlPlainScalarStyle;

    if leading_blanks {
        (*parser).simple_key_allowed = true;
    }

    // Cleanup
    STRING_DEL!(leading_break);
    STRING_DEL!(trailing_breaks);
    STRING_DEL!(whitespaces);

    OK
}

unsafe fn yaml_parser_scan_plain_scalar(
    parser: *mut YamlParserT,
    token: *mut YamlTokenT,
) -> Success {
    let (
        mut string,
        mut leading_break,
        mut trailing_breaks,
        mut whitespaces,
        mut end_mark,
        indent,
    ) = scan_plain_scalar_init(parser);

    let mut leading_blanks = false;
    let start_mark = end_mark;

    loop {
        // Check buffer size for scanning
        if cache(parser, 4_u64).fail {
            break_with_cleanup!(
                string,
                leading_break,
                trailing_breaks,
                whitespaces
            );
        }

        // Check for document indicators
        if check_document_indicators(parser) {
            break;
        }

        // Check for comments
        if CHECK!((*parser).buffer, b'#') {
            break;
        }

        // Scan the scalar value
        while !IS_BLANKZ!((*parser).buffer) {
            match handle_flow_indicators(parser, start_mark) {
                Ok(true) => break,
                Ok(false) => {}
                Err(()) => break_with_cleanup!(
                    string,
                    leading_break,
                    trailing_breaks,
                    whitespaces
                ),
            }

            if handle_whitespace_and_breaks(
                parser,
                &mut string,
                &mut leading_break,
                &mut trailing_breaks,
                &mut whitespaces,
                &mut leading_blanks,
                start_mark,
            )
            .is_err()
            {
                break_with_cleanup!(
                    string,
                    leading_break,
                    trailing_breaks,
                    whitespaces
                );
            }

            read!(parser, string);
            end_mark = (*parser).mark;

            if cache(parser, 2_u64).fail {
                break_with_cleanup!(
                    string,
                    leading_break,
                    trailing_breaks,
                    whitespaces
                );
            }
        }

        if !(IS_BLANK!((*parser).buffer) || IS_BREAK!((*parser).buffer))
        {
            break;
        }

        if cache(parser, 1_u64).fail {
            break_with_cleanup!(
                string,
                leading_break,
                trailing_breaks,
                whitespaces
            );
        }

        while IS_BLANK!((*parser).buffer) || IS_BREAK!((*parser).buffer)
        {
            if IS_BLANK!((*parser).buffer) {
                if handle_blank_characters(
                    parser,
                    indent,
                    leading_blanks,
                    start_mark,
                )
                .is_err()
                {
                    break_with_cleanup!(
                        string,
                        leading_break,
                        trailing_breaks,
                        whitespaces
                    );
                }
                if leading_blanks {
                    skip(parser);
                } else {
                    read!(parser, whitespaces);
                }
            } else {
                if cache(parser, 2_u64).fail {
                    break_with_cleanup!(
                        string,
                        leading_break,
                        trailing_breaks,
                        whitespaces
                    );
                }

                if leading_blanks {
                    read_line!(parser, trailing_breaks);
                } else {
                    CLEAR!(whitespaces);
                    read_line!(parser, leading_break);
                    leading_blanks = true;
                }
            }

            if cache(parser, 1_u64).fail {
                break_with_cleanup!(
                    string,
                    leading_break,
                    trailing_breaks,
                    whitespaces
                );
            }
        }

        if (*parser).flow_level == 0
            && libc::c_int::try_from((*parser).mark.column).unwrap()
                < indent
        {
            break;
        }
    }

    initialize_plain_scalar_token(
        token,
        &string,
        start_mark,
        end_mark,
        parser,
        leading_blanks,
        &mut leading_break,
        &mut trailing_breaks,
        &mut whitespaces,
    )
}
