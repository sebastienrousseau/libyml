// parser.rs

use crate::{
    externs::{memcpy, memset, strcmp, strlen},
    internal::yaml_stack_extend,
    libc,
    memory::{yaml_free, yaml_malloc, yaml_strdup},
    ops::ForceAdd as _,
    scanner::yaml_parser_fetch_more_tokens,
    success::{Success, FAIL, OK},
    yaml::{size_t, yaml_char_t},
    YamlAliasEvent, YamlAliasToken, YamlAnchorToken, YamlBlockEndToken,
    YamlBlockEntryToken, YamlBlockMappingStartToken,
    YamlBlockMappingStyle, YamlBlockSequenceStartToken,
    YamlBlockSequenceStyle, YamlDocumentEndEvent, YamlDocumentEndToken,
    YamlDocumentStartEvent, YamlDocumentStartToken, YamlEventT,
    YamlFlowEntryToken, YamlFlowMappingEndToken,
    YamlFlowMappingStartToken, YamlFlowMappingStyle,
    YamlFlowSequenceEndToken, YamlFlowSequenceStartToken,
    YamlFlowSequenceStyle, YamlKeyToken, YamlMappingEndEvent,
    YamlMappingStartEvent, YamlMarkT, YamlNoError,
    YamlParseBlockMappingFirstKeyState, YamlParseBlockMappingKeyState,
    YamlParseBlockMappingValueState,
    YamlParseBlockNodeOrIndentlessSequenceState,
    YamlParseBlockNodeState, YamlParseBlockSequenceEntryState,
    YamlParseBlockSequenceFirstEntryState,
    YamlParseDocumentContentState, YamlParseDocumentEndState,
    YamlParseDocumentStartState, YamlParseEndState,
    YamlParseFlowMappingEmptyValueState,
    YamlParseFlowMappingFirstKeyState, YamlParseFlowMappingKeyState,
    YamlParseFlowMappingValueState, YamlParseFlowNodeState,
    YamlParseFlowSequenceEntryMappingEndState,
    YamlParseFlowSequenceEntryMappingKeyState,
    YamlParseFlowSequenceEntryMappingValueState,
    YamlParseFlowSequenceEntryState,
    YamlParseFlowSequenceFirstEntryState,
    YamlParseImplicitDocumentStartState,
    YamlParseIndentlessSequenceEntryState, YamlParseStreamStartState,
    YamlParserError, YamlParserT, YamlPlainScalarStyle,
    YamlScalarEvent, YamlScalarToken, YamlSequenceEndEvent,
    YamlSequenceStartEvent, YamlStreamEndEvent, YamlStreamEndToken,
    YamlStreamStartEvent, YamlStreamStartToken, YamlTagDirectiveT,
    YamlTagDirectiveToken, YamlTagToken, YamlTokenT, YamlValueToken,
    YamlVersionDirectiveT, YamlVersionDirectiveToken,
};
use core::{
    mem::size_of,
    ptr::{self, addr_of_mut, write_bytes},
};

/// An optional error type for demonstrating how error handling could be improved to return `Result`.
///
/// This is *not* used in the existing parser to avoid breaking changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParserError {
    /// General parser error.
    GeneralError(&'static str),
}

/// Safely peek at the current token without consuming it.
///
/// # Safety
/// - Checks if `parser` is null.
/// - Checks if more tokens need to be fetched.
/// - Returns `null_mut()` if fetching fails or if `parser` is null.
///
/// # Returns
/// Pointer to the current `YamlTokenT`, or `null_mut` if unavailable.
unsafe fn peek_token(parser: *mut YamlParserT) -> *mut YamlTokenT {
    if parser.is_null() {
        return ptr::null_mut();
    }
    if (*parser).token_available
        || yaml_parser_fetch_more_tokens(parser).ok
    {
        (*parser).tokens.head
    } else {
        ptr::null_mut::<YamlTokenT>()
    }
}

/// Consume the current token, advancing the internal parser state.
///
/// # Safety
/// - Assumes `parser` is valid and non-null.
unsafe fn skip_token(parser: *mut YamlParserT) {
    // Optional safety check: do nothing if parser is null
    if parser.is_null() {
        return;
    }

    (*parser).token_available = false;
    let fresh3 = addr_of_mut!((*parser).tokens_parsed);
    *fresh3 = (*fresh3).wrapping_add(1);
    (*parser).stream_end_produced =
        (*(*parser).tokens.head).type_ == YamlStreamEndToken;
    let fresh4 = addr_of_mut!((*parser).tokens.head);
    *fresh4 = (*fresh4).wrapping_offset(1);
}

/**
Parse the input stream and produce the next parsing event.

This function should be called repeatedly to produce a sequence of events
corresponding to the input stream. The initial event will be of type
`YamlStreamStartEvent`, and the final event will be of type `YamlStreamEndEvent`.

# Safety

- Operates on raw pointers (`parser`, `event`).
- Assumes certain memory layouts and alignments.
- May cause undefined behavior if pointers are invalid.

# Arguments

- `parser`: A pointer to a properly initialized `YamlParserT` struct.
- `event`: A pointer to a `YamlEventT` struct that will be filled with the next event.

# Returns

`OK` if an event was successfully parsed, or `FAIL` if:
- The stream has ended (`stream_end_produced` is true).
- There's an existing error in the parser (`parser.error != YamlNoError`).
- The parser is in the end state.

# Errors

Returns `FAIL` if any error conditions are met. Check the parser’s error state for details.

# Notes

- The caller is responsible for freeing buffers associated with the produced
  event using `yaml_event_delete()`.
- Do not alternate calls to `yaml_parser_parse()` with calls to `yaml_parser_scan()`
  or `yaml_parser_load()`. Doing so will break the parser.
*/
pub unsafe fn yaml_parser_parse(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
) -> Success {
    __assert!(!parser.is_null());
    __assert!(!event.is_null());
    let _ = memset(
        event as *mut libc::c_void,
        0,
        size_of::<YamlEventT>() as libc::c_ulong,
    );
    if (*parser).stream_end_produced {
        return FAIL;
    }
    if (*parser).error != YamlNoError {
        return FAIL;
    }
    if (*parser).state == YamlParseEndState {
        return FAIL;
    }
    yaml_parser_state_machine(parser, event)
}

/**
Set a parser error state internally, without returning `Result`.

Retains the original signature to avoid breaking changes.

# Safety

- Operates on raw pointers (`parser`, `problem`).
- Writes to `parser.error`, `parser.problem`, and `parser.problem_mark`.
*/
unsafe fn yaml_parser_set_parser_error(
    parser: *mut YamlParserT,
    problem: *const libc::c_char,
    problem_mark: YamlMarkT,
) {
    (*parser).error = YamlParserError;
    let fresh0 = addr_of_mut!((*parser).problem);
    *fresh0 = problem;
    (*parser).problem_mark = problem_mark;
}

/**
An optional replacement for `yaml_parser_set_parser_error` returning `Result`.

This function demonstrates how error propagation could be more idiomatic in Rust,
but is unused to avoid breaking the current API.

# Safety

- Same constraints as `yaml_parser_set_parser_error`.

# Returns

`Ok(())` on success, or `Err(ParserError)` on failure.
*/
#[allow(dead_code)]
unsafe fn yaml_parser_set_parser_error_result(
    parser: *mut YamlParserT,
    problem: *const libc::c_char,
    problem_mark: YamlMarkT,
) -> Result<(), ParserError> {
    if parser.is_null() {
        return Err(ParserError::GeneralError("Null parser pointer"));
    }
    (*parser).error = YamlParserError;
    let fresh0 = addr_of_mut!((*parser).problem);
    *fresh0 = problem;
    (*parser).problem_mark = problem_mark;
    Ok(())
}

/// Sets a parser error state with additional context.
///
/// # Safety
/// - Similar pointer considerations as `yaml_parser_set_parser_error`.
/// - Writes to `parser.context`, `parser.context_mark`, `parser.problem`, `parser.problem_mark`.
unsafe fn yaml_parser_set_parser_error_context(
    parser: *mut YamlParserT,
    context: *const libc::c_char,
    context_mark: YamlMarkT,
    problem: *const libc::c_char,
    problem_mark: YamlMarkT,
) {
    (*parser).error = YamlParserError;
    let fresh1 = addr_of_mut!((*parser).context);
    *fresh1 = context;
    (*parser).context_mark = context_mark;
    let fresh2 = addr_of_mut!((*parser).problem);
    *fresh2 = problem;
    (*parser).problem_mark = problem_mark;
}

/**
Main parser state machine.

This function dispatches to various parse handlers depending on `parser.state`.
It is called by `yaml_parser_parse()`.

# Safety

- `parser` and `event` must be valid pointers.
- The parser must be in a well-defined state.
*/
unsafe fn yaml_parser_state_machine(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
) -> Success {
    match (*parser).state {
        YamlParseStreamStartState => {
            yaml_parser_parse_stream_start(parser, event)
        }
        YamlParseImplicitDocumentStartState => {
            yaml_parser_parse_document_start(parser, event, true)
        }
        YamlParseDocumentStartState => {
            yaml_parser_parse_document_start(parser, event, false)
        }
        YamlParseDocumentContentState => {
            yaml_parser_parse_document_content(parser, event)
        }
        YamlParseDocumentEndState => {
            yaml_parser_parse_document_end(parser, event)
        }
        YamlParseBlockNodeState => {
            yaml_parser_parse_node(parser, event, true, false)
        }
        YamlParseBlockNodeOrIndentlessSequenceState => {
            yaml_parser_parse_node(parser, event, true, true)
        }
        YamlParseFlowNodeState => {
            yaml_parser_parse_node(parser, event, false, false)
        }
        YamlParseBlockSequenceFirstEntryState => {
            yaml_parser_parse_block_sequence_entry(parser, event, true)
        }
        YamlParseBlockSequenceEntryState => {
            yaml_parser_parse_block_sequence_entry(parser, event, false)
        }
        YamlParseIndentlessSequenceEntryState => {
            yaml_parser_parse_indentless_sequence_entry(parser, event)
        }
        YamlParseBlockMappingFirstKeyState => {
            yaml_parser_parse_block_mapping_key(parser, event, true)
        }
        YamlParseBlockMappingKeyState => {
            yaml_parser_parse_block_mapping_key(parser, event, false)
        }
        YamlParseBlockMappingValueState => {
            yaml_parser_parse_block_mapping_value(parser, event)
        }
        YamlParseFlowSequenceFirstEntryState => {
            yaml_parser_parse_flow_sequence_entry(parser, event, true)
        }
        YamlParseFlowSequenceEntryState => {
            yaml_parser_parse_flow_sequence_entry(parser, event, false)
        }
        YamlParseFlowSequenceEntryMappingKeyState => {
            yaml_parser_parse_flow_sequence_entry_mapping_key(
                parser, event,
            )
        }
        YamlParseFlowSequenceEntryMappingValueState => {
            yaml_parser_parse_flow_sequence_entry_mapping_value(
                parser, event,
            )
        }
        YamlParseFlowSequenceEntryMappingEndState => {
            yaml_parser_parse_flow_sequence_entry_mapping_end(
                parser, event,
            )
        }
        YamlParseFlowMappingFirstKeyState => {
            yaml_parser_parse_flow_mapping_key(parser, event, true)
        }
        YamlParseFlowMappingKeyState => {
            yaml_parser_parse_flow_mapping_key(parser, event, false)
        }
        YamlParseFlowMappingValueState => {
            yaml_parser_parse_flow_mapping_value(parser, event, false)
        }
        YamlParseFlowMappingEmptyValueState => {
            yaml_parser_parse_flow_mapping_value(parser, event, true)
        }
        _ => FAIL,
    }
}

/**
Parse a `StreamStartToken`, transitioning to the next state.

# Safety
- `parser` must not be null.
- `event` must not be null.
*/
unsafe fn yaml_parser_parse_stream_start(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
) -> Success {
    let token: *mut YamlTokenT = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }
    if (*token).type_ != YamlStreamStartToken {
        yaml_parser_set_parser_error(
            parser,
            b"did not find expected <stream-start>\0" as *const u8
                as *const libc::c_char,
            (*token).start_mark,
        );
        return FAIL;
    }
    (*parser).state = YamlParseImplicitDocumentStartState;
    let _ = memset(
        event as *mut libc::c_void,
        0,
        size_of::<YamlEventT>() as libc::c_ulong,
    );
    (*event).type_ = YamlStreamStartEvent;
    (*event).start_mark = (*token).start_mark;
    (*event).end_mark = (*token).start_mark;
    (*event).data.stream_start.encoding =
        (*token).data.stream_start.encoding;
    skip_token(parser);
    OK
}

/**
Parse the start of a YAML document.

Depending on whether `implicit` is true, handles different YAML constructs
(e.g., version/tag directives, document indicators).

# Safety
- `parser` must not be null.
- `event` must not be null.
*/
unsafe fn yaml_parser_parse_document_start(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
    implicit: bool,
) -> Success {
    let mut token: *mut YamlTokenT;
    let mut version_directive: *mut YamlVersionDirectiveT =
        ptr::null_mut::<YamlVersionDirectiveT>();
    struct TagDirectives {
        start: *mut YamlTagDirectiveT,
        end: *mut YamlTagDirectiveT,
    }
    let mut tag_directives = TagDirectives {
        start: ptr::null_mut::<YamlTagDirectiveT>(),
        end: ptr::null_mut::<YamlTagDirectiveT>(),
    };

    token = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }

    // Skip extra document end tokens if not implicit
    if !implicit {
        while (*token).type_ == YamlDocumentEndToken {
            skip_token(parser);
            token = peek_token(parser);
            if token.is_null() {
                return FAIL;
            }
        }
    }

    // Implicit start?
    if implicit
        && (*token).type_ != YamlVersionDirectiveToken
        && (*token).type_ != YamlTagDirectiveToken
        && (*token).type_ != YamlDocumentStartToken
        && (*token).type_ != YamlStreamEndToken
    {
        if yaml_parser_process_directives(
            parser,
            ptr::null_mut::<*mut YamlVersionDirectiveT>(),
            ptr::null_mut::<*mut YamlTagDirectiveT>(),
            ptr::null_mut::<*mut YamlTagDirectiveT>(),
        )
        .fail
        {
            return FAIL;
        }
        PUSH!((*parser).states, YamlParseDocumentEndState);
        (*parser).state = YamlParseBlockNodeState;
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlDocumentStartEvent;
        (*event).start_mark = (*token).start_mark;
        (*event).end_mark = (*token).start_mark;
        let fresh9 = addr_of_mut!(
            (*event).data.document_start.version_directive
        );
        *fresh9 = ptr::null_mut::<YamlVersionDirectiveT>();
        let fresh10 = addr_of_mut!(
            (*event).data.document_start.tag_directives.start
        );
        *fresh10 = ptr::null_mut::<YamlTagDirectiveT>();
        let fresh11 = addr_of_mut!(
            (*event).data.document_start.tag_directives.end
        );
        *fresh11 = ptr::null_mut::<YamlTagDirectiveT>();
        (*event).data.document_start.implicit = true;
        OK
    } else if (*token).type_ != YamlStreamEndToken {
        let end_mark: YamlMarkT;
        let start_mark: YamlMarkT = (*token).start_mark;

        if yaml_parser_process_directives(
            parser,
            addr_of_mut!(version_directive),
            addr_of_mut!(tag_directives.start),
            addr_of_mut!(tag_directives.end),
        )
        .fail
        {
            return FAIL;
        }
        token = peek_token(parser);
        if !token.is_null() {
            if (*token).type_ != YamlDocumentStartToken {
                yaml_parser_set_parser_error(
                    parser,
                    b"did not find expected <document start>\0"
                        as *const u8
                        as *const libc::c_char,
                    (*token).start_mark,
                );
            } else {
                PUSH!((*parser).states, YamlParseDocumentEndState);
                (*parser).state = YamlParseDocumentContentState;
                end_mark = (*token).end_mark;
                let _ = memset(
                    event as *mut libc::c_void,
                    0,
                    size_of::<YamlEventT>() as libc::c_ulong,
                );
                (*event).type_ = YamlDocumentStartEvent;
                (*event).start_mark = start_mark;
                (*event).end_mark = end_mark;
                let fresh14 = addr_of_mut!(
                    (*event).data.document_start.version_directive
                );
                *fresh14 = version_directive;
                let fresh15 = addr_of_mut!(
                    (*event).data.document_start.tag_directives.start
                );
                *fresh15 = tag_directives.start;
                let fresh16 = addr_of_mut!(
                    (*event).data.document_start.tag_directives.end
                );
                *fresh16 = tag_directives.end;
                (*event).data.document_start.implicit = false;
                skip_token(parser);
                tag_directives.end =
                    ptr::null_mut::<YamlTagDirectiveT>();
                tag_directives.start = tag_directives.end;
                return OK;
            }
        }
        yaml_free(version_directive as *mut libc::c_void);
        while tag_directives.start != tag_directives.end {
            yaml_free(
                (*tag_directives.end.wrapping_offset(-1_isize)).handle
                    as *mut libc::c_void,
            );
            yaml_free(
                (*tag_directives.end.wrapping_offset(-1_isize)).prefix
                    as *mut libc::c_void,
            );
            tag_directives.end = tag_directives.end.wrapping_offset(-1);
        }
        yaml_free(tag_directives.start as *mut libc::c_void);
        FAIL
    } else {
        (*parser).state = YamlParseEndState;
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlStreamEndEvent;
        (*event).start_mark = (*token).start_mark;
        (*event).end_mark = (*token).end_mark;
        skip_token(parser);
        OK
    }
}

/**
Parse the content of a YAML document.

# Safety
- `parser` and `event` must be valid.
*/
unsafe fn yaml_parser_parse_document_content(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
) -> Success {
    let token: *mut YamlTokenT = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }
    if (*token).type_ == YamlVersionDirectiveToken
        || (*token).type_ == YamlTagDirectiveToken
        || (*token).type_ == YamlDocumentStartToken
        || (*token).type_ == YamlDocumentEndToken
        || (*token).type_ == YamlStreamEndToken
    {
        (*parser).state = POP!((*parser).states);
        yaml_parser_process_empty_scalar(event, (*token).start_mark)
    } else {
        yaml_parser_parse_node(parser, event, true, false)
    }
}

/**
Parse the end of a YAML document.

This function checks for explicit document-end tokens (`...`) and processes
any remaining tag directives.

# Safety
- `parser` and `event` must be valid.
*/
unsafe fn yaml_parser_parse_document_end(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
) -> Success {
    let mut end_mark: YamlMarkT;
    let mut implicit = true;
    let token: *mut YamlTokenT = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }
    end_mark = (*token).start_mark;
    let start_mark: YamlMarkT = end_mark;
    if (*token).type_ == YamlDocumentEndToken {
        end_mark = (*token).end_mark;
        skip_token(parser);
        implicit = false;
    }
    while !STACK_EMPTY!((*parser).tag_directives) {
        let tag_directive = POP!((*parser).tag_directives);
        yaml_free(tag_directive.handle as *mut libc::c_void);
        yaml_free(tag_directive.prefix as *mut libc::c_void);
    }
    (*parser).state = YamlParseDocumentStartState;
    let _ = memset(
        event as *mut libc::c_void,
        0,
        size_of::<YamlEventT>() as libc::c_ulong,
    );
    (*event).type_ = YamlDocumentEndEvent;
    (*event).start_mark = start_mark;
    (*event).end_mark = end_mark;
    (*event).data.document_end.implicit = implicit;
    OK
}

/**
Parse a YAML node, handling anchors, tags, scalars, and complex constructs.

# Safety
- `parser` and `event` must be valid pointers.
- The `block` and `indentless_sequence` flags indicate the context of the node.
*/
unsafe fn yaml_parser_parse_node(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
    block: bool,
    indentless_sequence: bool,
) -> Success {
    let mut current_block: u64;
    let mut token: *mut YamlTokenT;
    let mut anchor: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
    let mut tag_handle: *mut yaml_char_t =
        ptr::null_mut::<yaml_char_t>();
    let mut tag_suffix: *mut yaml_char_t =
        ptr::null_mut::<yaml_char_t>();
    let mut tag: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
    let mut start_mark: YamlMarkT;
    let mut end_mark: YamlMarkT;
    let mut tag_mark = YamlMarkT {
        index: 0,
        line: 0,
        column: 0,
    };
    let implicit;

    token = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }

    // Handle Alias
    if (*token).type_ == YamlAliasToken {
        (*parser).state = POP!((*parser).states);
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlAliasEvent;
        (*event).start_mark = (*token).start_mark;
        (*event).end_mark = (*token).end_mark;
        let fresh26 = addr_of_mut!((*event).data.alias.anchor);
        *fresh26 = (*token).data.alias.value;
        skip_token(parser);
        OK
    } else {
        end_mark = (*token).start_mark;
        start_mark = end_mark;

        // Gather anchor or tag if present
        if (*token).type_ == YamlAnchorToken {
            anchor = (*token).data.anchor.value;
            start_mark = (*token).start_mark;
            end_mark = (*token).end_mark;
            skip_token(parser);
            token = peek_token(parser);
            if token.is_null() {
                current_block = 17786380918591080555;
            } else if (*token).type_ == YamlTagToken {
                tag_handle = (*token).data.tag.handle;
                tag_suffix = (*token).data.tag.suffix;
                tag_mark = (*token).start_mark;
                end_mark = (*token).end_mark;
                skip_token(parser);
                token = peek_token(parser);
                if token.is_null() {
                    current_block = 17786380918591080555;
                } else {
                    current_block = 11743904203796629665;
                }
            } else {
                current_block = 11743904203796629665;
            }
        } else if (*token).type_ == YamlTagToken {
            tag_handle = (*token).data.tag.handle;
            tag_suffix = (*token).data.tag.suffix;
            tag_mark = (*token).start_mark;
            start_mark = tag_mark;
            end_mark = (*token).end_mark;
            skip_token(parser);
            token = peek_token(parser);
            if token.is_null() {
                current_block = 17786380918591080555;
            } else if (*token).type_ == YamlAnchorToken {
                anchor = (*token).data.anchor.value;
                end_mark = (*token).end_mark;
                skip_token(parser);
                token = peek_token(parser);
                if token.is_null() {
                    current_block = 17786380918591080555;
                } else {
                    current_block = 11743904203796629665;
                }
            } else {
                current_block = 11743904203796629665;
            }
        } else {
            current_block = 11743904203796629665;
        }

        // Build the actual tag
        if current_block == 11743904203796629665 {
            if !tag_handle.is_null() {
                if *tag_handle == 0 {
                    tag = tag_suffix;
                    yaml_free(tag_handle as *mut libc::c_void);
                    tag_suffix = ptr::null_mut::<yaml_char_t>();
                    tag_handle = tag_suffix;
                    current_block = 9437013279121998969;
                } else {
                    let mut tag_directive: *mut YamlTagDirectiveT;
                    tag_directive = (*parser).tag_directives.start;
                    loop {
                        if tag_directive == (*parser).tag_directives.top
                        {
                            current_block = 17728966195399430138;
                            break;
                        }
                        if strcmp(
                            (*tag_directive).handle
                                as *mut libc::c_char,
                            tag_handle as *mut libc::c_char,
                        ) == 0
                        {
                            let prefix_len: size_t = strlen(
                                (*tag_directive).prefix
                                    as *mut libc::c_char,
                            );
                            let suffix_len: size_t =
                                strlen(tag_suffix as *mut libc::c_char);
                            tag = yaml_malloc(
                                prefix_len
                                    .force_add(suffix_len)
                                    .force_add(1_u64),
                            )
                                as *mut yaml_char_t;
                            let _ = memcpy(
                                tag as *mut libc::c_void,
                                (*tag_directive).prefix
                                    as *const libc::c_void,
                                prefix_len,
                            );
                            let _ = memcpy(
                                tag.wrapping_offset(prefix_len as isize)
                                    as *mut libc::c_void,
                                tag_suffix as *const libc::c_void,
                                suffix_len,
                            );
                            *tag.wrapping_offset(
                                prefix_len.force_add(suffix_len)
                                    as isize,
                            ) = b'\0';
                            yaml_free(tag_handle as *mut libc::c_void);
                            yaml_free(tag_suffix as *mut libc::c_void);
                            tag_suffix = ptr::null_mut::<yaml_char_t>();
                            tag_handle = tag_suffix;
                            current_block = 17728966195399430138;
                            break;
                        } else {
                            tag_directive =
                                tag_directive.wrapping_offset(1);
                        }
                    }
                    if current_block != 17786380918591080555 {
                        if tag.is_null() {
                            yaml_parser_set_parser_error_context(
                                parser,
                                b"while parsing a node\0" as *const u8
                                    as *const libc::c_char,
                                start_mark,
                                b"found undefined tag handle\0"
                                    as *const u8
                                    as *const libc::c_char,
                                tag_mark,
                            );
                            current_block = 17786380918591080555;
                        } else {
                            current_block = 9437013279121998969;
                        }
                    }
                }
            } else {
                current_block = 9437013279121998969;
            }
            if current_block != 17786380918591080555 {
                implicit = tag.is_null() || *tag == 0;

                // Indentless sequence
                if indentless_sequence
                    && (*token).type_ == YamlBlockEntryToken
                {
                    end_mark = (*token).end_mark;
                    (*parser).state =
                        YamlParseIndentlessSequenceEntryState;
                    let _ = memset(
                        event as *mut libc::c_void,
                        0,
                        size_of::<YamlEventT>() as libc::c_ulong,
                    );
                    (*event).type_ = YamlSequenceStartEvent;
                    (*event).start_mark = start_mark;
                    (*event).end_mark = end_mark;
                    let fresh37 = addr_of_mut!(
                        (*event).data.sequence_start.anchor
                    );
                    *fresh37 = anchor;
                    let fresh38 =
                        addr_of_mut!((*event).data.sequence_start.tag);
                    *fresh38 = tag;
                    (*event).data.sequence_start.implicit = implicit;
                    (*event).data.sequence_start.style =
                        YamlBlockSequenceStyle;
                    return OK;
                }
                // Scalar
                else if (*token).type_ == YamlScalarToken {
                    let mut plain_implicit = false;
                    let mut quoted_implicit = false;
                    end_mark = (*token).end_mark;
                    if (*token).data.scalar.style
                        == YamlPlainScalarStyle
                        && tag.is_null()
                        || !tag.is_null()
                            && strcmp(
                                tag as *mut libc::c_char,
                                b"!\0" as *const u8
                                    as *const libc::c_char,
                            ) == 0
                    {
                        plain_implicit = true;
                    } else if tag.is_null() {
                        quoted_implicit = true;
                    }
                    (*parser).state = POP!((*parser).states);
                    let _ = memset(
                        event as *mut libc::c_void,
                        0,
                        size_of::<YamlEventT>() as libc::c_ulong,
                    );
                    (*event).type_ = YamlScalarEvent;
                    (*event).start_mark = start_mark;
                    (*event).end_mark = end_mark;
                    let fresh40 =
                        addr_of_mut!((*event).data.scalar.anchor);
                    *fresh40 = anchor;
                    let fresh41 =
                        addr_of_mut!((*event).data.scalar.tag);
                    *fresh41 = tag;
                    let fresh42 =
                        addr_of_mut!((*event).data.scalar.value);
                    *fresh42 = (*token).data.scalar.value;
                    (*event).data.scalar.length =
                        (*token).data.scalar.length;
                    (*event).data.scalar.plain_implicit =
                        plain_implicit;
                    (*event).data.scalar.quoted_implicit =
                        quoted_implicit;
                    (*event).data.scalar.style =
                        (*token).data.scalar.style;
                    skip_token(parser);
                    return OK;
                }
                // Flow Sequence Start
                else if (*token).type_ == YamlFlowSequenceStartToken {
                    end_mark = (*token).end_mark;
                    (*parser).state =
                        YamlParseFlowSequenceFirstEntryState;
                    let _ = memset(
                        event as *mut libc::c_void,
                        0,
                        size_of::<YamlEventT>() as libc::c_ulong,
                    );
                    (*event).type_ = YamlSequenceStartEvent;
                    (*event).start_mark = start_mark;
                    (*event).end_mark = end_mark;
                    let fresh45 = addr_of_mut!(
                        (*event).data.sequence_start.anchor
                    );
                    *fresh45 = anchor;
                    let fresh46 =
                        addr_of_mut!((*event).data.sequence_start.tag);
                    *fresh46 = tag;
                    (*event).data.sequence_start.implicit = implicit;
                    (*event).data.sequence_start.style =
                        YamlFlowSequenceStyle;
                    return OK;
                }
                // Flow Mapping Start
                else if (*token).type_ == YamlFlowMappingStartToken {
                    end_mark = (*token).end_mark;
                    (*parser).state = YamlParseFlowMappingFirstKeyState;
                    let _ = memset(
                        event as *mut libc::c_void,
                        0,
                        size_of::<YamlEventT>() as libc::c_ulong,
                    );
                    (*event).type_ = YamlMappingStartEvent;
                    (*event).start_mark = start_mark;
                    (*event).end_mark = end_mark;
                    let fresh47 = addr_of_mut!(
                        (*event).data.mapping_start.anchor
                    );
                    *fresh47 = anchor;
                    let fresh48 =
                        addr_of_mut!((*event).data.mapping_start.tag);
                    *fresh48 = tag;
                    (*event).data.mapping_start.implicit = implicit;
                    (*event).data.mapping_start.style =
                        YamlFlowMappingStyle;
                    return OK;
                }
                // Block Sequence
                else if block
                    && (*token).type_ == YamlBlockSequenceStartToken
                {
                    end_mark = (*token).end_mark;
                    (*parser).state =
                        YamlParseBlockSequenceFirstEntryState;
                    let _ = memset(
                        event as *mut libc::c_void,
                        0,
                        size_of::<YamlEventT>() as libc::c_ulong,
                    );
                    (*event).type_ = YamlSequenceStartEvent;
                    (*event).start_mark = start_mark;
                    (*event).end_mark = end_mark;
                    let fresh49 = addr_of_mut!(
                        (*event).data.sequence_start.anchor
                    );
                    *fresh49 = anchor;
                    let fresh50 =
                        addr_of_mut!((*event).data.sequence_start.tag);
                    *fresh50 = tag;
                    (*event).data.sequence_start.implicit = implicit;
                    (*event).data.sequence_start.style =
                        YamlBlockSequenceStyle;
                    return OK;
                }
                // Block Mapping
                else if block
                    && (*token).type_ == YamlBlockMappingStartToken
                {
                    end_mark = (*token).end_mark;
                    (*parser).state =
                        YamlParseBlockMappingFirstKeyState;
                    let _ = memset(
                        event as *mut libc::c_void,
                        0,
                        size_of::<YamlEventT>() as libc::c_ulong,
                    );
                    (*event).type_ = YamlMappingStartEvent;
                    (*event).start_mark = start_mark;
                    (*event).end_mark = end_mark;
                    let fresh51 = addr_of_mut!(
                        (*event).data.mapping_start.anchor
                    );
                    *fresh51 = anchor;
                    let fresh52 =
                        addr_of_mut!((*event).data.mapping_start.tag);
                    *fresh52 = tag;
                    (*event).data.mapping_start.implicit = implicit;
                    (*event).data.mapping_start.style =
                        YamlBlockMappingStyle;
                    return OK;
                }
                // Plain scalar with anchor or tag
                else if !anchor.is_null() || !tag.is_null() {
                    let value: *mut yaml_char_t =
                        yaml_malloc(1_u64) as *mut yaml_char_t;
                    *value = b'\0';
                    (*parser).state = POP!((*parser).states);
                    let _ = memset(
                        event as *mut libc::c_void,
                        0,
                        size_of::<YamlEventT>() as libc::c_ulong,
                    );
                    (*event).type_ = YamlScalarEvent;
                    (*event).start_mark = start_mark;
                    (*event).end_mark = end_mark;
                    let fresh54 =
                        addr_of_mut!((*event).data.scalar.anchor);
                    *fresh54 = anchor;
                    let fresh55 =
                        addr_of_mut!((*event).data.scalar.tag);
                    *fresh55 = tag;
                    let fresh56 =
                        addr_of_mut!((*event).data.scalar.value);
                    *fresh56 = value;
                    (*event).data.scalar.length = 0_u64;
                    (*event).data.scalar.plain_implicit = implicit;
                    (*event).data.scalar.quoted_implicit = false;
                    (*event).data.scalar.style = YamlPlainScalarStyle;
                    return OK;
                } else {
                    yaml_parser_set_parser_error_context(
                        parser,
                        if block {
                            b"while parsing a block node\0" as *const u8
                                as *const libc::c_char
                        } else {
                            b"while parsing a flow node\0" as *const u8
                                as *const libc::c_char
                        },
                        start_mark,
                        b"did not find expected node content\0"
                            as *const u8
                            as *const libc::c_char,
                        (*token).start_mark,
                    );
                }
            }
        }
        yaml_free(anchor as *mut libc::c_void);
        yaml_free(tag_handle as *mut libc::c_void);
        yaml_free(tag_suffix as *mut libc::c_void);
        yaml_free(tag as *mut libc::c_void);
        FAIL
    }
}

/**
Parse a block sequence entry. This function handles both the first entry and
subsequent entries depending on `first`.

# Safety
- `parser` and `event` must be valid pointers.
*/
unsafe fn yaml_parser_parse_block_sequence_entry(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
    first: bool,
) -> Success {
    let mut token: *mut YamlTokenT;
    if first {
        token = peek_token(parser);
        PUSH!((*parser).marks, (*token).start_mark);
        skip_token(parser);
    }
    token = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }
    if (*token).type_ == YamlBlockEntryToken {
        let mark: YamlMarkT = (*token).end_mark;
        skip_token(parser);
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ != YamlBlockEntryToken
            && (*token).type_ != YamlBlockEndToken
        {
            PUSH!((*parser).states, YamlParseBlockSequenceEntryState);
            yaml_parser_parse_node(parser, event, true, false)
        } else {
            (*parser).state = YamlParseBlockSequenceEntryState;
            yaml_parser_process_empty_scalar(event, mark)
        }
    } else if (*token).type_ == YamlBlockEndToken {
        (*parser).state = POP!((*parser).states);
        let _ = POP!((*parser).marks);
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlSequenceEndEvent;
        (*event).start_mark = (*token).start_mark;
        (*event).end_mark = (*token).end_mark;
        skip_token(parser);
        OK
    } else {
        yaml_parser_set_parser_error_context(
            parser,
            b"while parsing a block collection\0" as *const u8
                as *const libc::c_char,
            POP!((*parser).marks),
            b"did not find expected '-' indicator\0" as *const u8
                as *const libc::c_char,
            (*token).start_mark,
        );
        FAIL
    }
}

/**
Parse an indentless sequence entry in a block context.

# Safety
- `parser` and `event` must be valid pointers.
*/
unsafe fn yaml_parser_parse_indentless_sequence_entry(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
) -> Success {
    let mut token: *mut YamlTokenT;
    token = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }
    if (*token).type_ == YamlBlockEntryToken {
        let mark: YamlMarkT = (*token).end_mark;
        skip_token(parser);
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ != YamlBlockEntryToken
            && (*token).type_ != YamlKeyToken
            && (*token).type_ != YamlValueToken
            && (*token).type_ != YamlBlockEndToken
        {
            PUSH!(
                (*parser).states,
                YamlParseIndentlessSequenceEntryState
            );
            yaml_parser_parse_node(parser, event, true, false)
        } else {
            (*parser).state = YamlParseIndentlessSequenceEntryState;
            yaml_parser_process_empty_scalar(event, mark)
        }
    } else {
        (*parser).state = POP!((*parser).states);
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlSequenceEndEvent;
        (*event).start_mark = (*token).start_mark;
        (*event).end_mark = (*token).start_mark;
        OK
    }
}

/**
Parse a key in a block mapping.

# Safety
- `parser` and `event` must be valid pointers.
*/
unsafe fn yaml_parser_parse_block_mapping_key(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
    first: bool,
) -> Success {
    let mut token: *mut YamlTokenT;
    if first {
        token = peek_token(parser);
        PUSH!((*parser).marks, (*token).start_mark);
        skip_token(parser);
    }
    token = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }
    if (*token).type_ == YamlKeyToken {
        let mark: YamlMarkT = (*token).end_mark;
        skip_token(parser);
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ != YamlKeyToken
            && (*token).type_ != YamlValueToken
            && (*token).type_ != YamlBlockEndToken
        {
            PUSH!((*parser).states, YamlParseBlockMappingValueState);
            yaml_parser_parse_node(parser, event, true, true)
        } else {
            (*parser).state = YamlParseBlockMappingValueState;
            yaml_parser_process_empty_scalar(event, mark)
        }
    } else if (*token).type_ == YamlBlockEndToken {
        (*parser).state = POP!((*parser).states);
        let _ = POP!((*parser).marks);
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlMappingEndEvent;
        (*event).start_mark = (*token).start_mark;
        (*event).end_mark = (*token).end_mark;
        skip_token(parser);
        OK
    } else {
        yaml_parser_set_parser_error_context(
            parser,
            b"while parsing a block mapping\0" as *const u8
                as *const libc::c_char,
            POP!((*parser).marks),
            b"did not find expected key\0" as *const u8
                as *const libc::c_char,
            (*token).start_mark,
        );
        FAIL
    }
}

/**
Parse a value in a block mapping.

# Safety
- `parser` and `event` must be valid pointers.
*/
unsafe fn yaml_parser_parse_block_mapping_value(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
) -> Success {
    let mut token: *mut YamlTokenT;
    token = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }
    if (*token).type_ == YamlValueToken {
        let mark: YamlMarkT = (*token).end_mark;
        skip_token(parser);
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ != YamlKeyToken
            && (*token).type_ != YamlValueToken
            && (*token).type_ != YamlBlockEndToken
        {
            PUSH!((*parser).states, YamlParseBlockMappingKeyState);
            yaml_parser_parse_node(parser, event, true, true)
        } else {
            (*parser).state = YamlParseBlockMappingKeyState;
            yaml_parser_process_empty_scalar(event, mark)
        }
    } else {
        (*parser).state = YamlParseBlockMappingKeyState;
        yaml_parser_process_empty_scalar(event, (*token).start_mark)
    }
}

/**
Parse an entry in a flow sequence.

# Safety
- `parser` and `event` must be valid pointers.
*/
unsafe fn yaml_parser_parse_flow_sequence_entry(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
    first: bool,
) -> Success {
    let mut token: *mut YamlTokenT;
    if first {
        token = peek_token(parser);
        PUSH!((*parser).marks, (*token).start_mark);
        skip_token(parser);
    }
    token = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }
    if (*token).type_ != YamlFlowSequenceEndToken {
        if !first {
            if (*token).type_ == YamlFlowEntryToken {
                skip_token(parser);
                token = peek_token(parser);
                if token.is_null() {
                    return FAIL;
                }
            } else {
                yaml_parser_set_parser_error_context(
                    parser,
                    b"while parsing a flow sequence\0" as *const u8
                        as *const libc::c_char,
                    POP!((*parser).marks),
                    b"did not find expected ',' or ']'\0" as *const u8
                        as *const libc::c_char,
                    (*token).start_mark,
                );
                return FAIL;
            }
        }
        if (*token).type_ == YamlKeyToken {
            (*parser).state = YamlParseFlowSequenceEntryMappingKeyState;
            let _ = memset(
                event as *mut libc::c_void,
                0,
                size_of::<YamlEventT>() as libc::c_ulong,
            );
            (*event).type_ = YamlMappingStartEvent;
            (*event).start_mark = (*token).start_mark;
            (*event).end_mark = (*token).end_mark;
            let fresh99 =
                addr_of_mut!((*event).data.mapping_start.anchor);
            *fresh99 = ptr::null_mut::<yaml_char_t>();
            let fresh100 =
                addr_of_mut!((*event).data.mapping_start.tag);
            *fresh100 = ptr::null_mut::<yaml_char_t>();
            (*event).data.mapping_start.implicit = true;
            (*event).data.mapping_start.style = YamlFlowMappingStyle;
            skip_token(parser);
            return OK;
        } else if (*token).type_ != YamlFlowSequenceEndToken {
            PUSH!((*parser).states, YamlParseFlowSequenceEntryState);
            return yaml_parser_parse_node(parser, event, false, false);
        }
    }
    (*parser).state = POP!((*parser).states);
    let _ = POP!((*parser).marks);
    let _ = memset(
        event as *mut libc::c_void,
        0,
        size_of::<YamlEventT>() as libc::c_ulong,
    );
    (*event).type_ = YamlSequenceEndEvent;
    (*event).start_mark = (*token).start_mark;
    (*event).end_mark = (*token).end_mark;
    skip_token(parser);
    OK
}

/**
Parse the key part of a mapping within a flow sequence entry.

# Safety
- `parser` and `event` must be valid pointers.
*/
unsafe fn yaml_parser_parse_flow_sequence_entry_mapping_key(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
) -> Success {
    let token: *mut YamlTokenT = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }
    if (*token).type_ != YamlValueToken
        && (*token).type_ != YamlFlowEntryToken
        && (*token).type_ != YamlFlowSequenceEndToken
    {
        PUSH!(
            (*parser).states,
            YamlParseFlowSequenceEntryMappingValueState
        );
        yaml_parser_parse_node(parser, event, false, false)
    } else {
        let mark: YamlMarkT = (*token).end_mark;
        skip_token(parser);
        (*parser).state = YamlParseFlowSequenceEntryMappingValueState;
        yaml_parser_process_empty_scalar(event, mark)
    }
}

/**
Parse the value part of a mapping within a flow sequence entry.

# Safety
- `parser` and `event` must be valid pointers.
*/
unsafe fn yaml_parser_parse_flow_sequence_entry_mapping_value(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
) -> Success {
    let mut token: *mut YamlTokenT;
    token = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }
    if (*token).type_ == YamlValueToken {
        skip_token(parser);
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ != YamlFlowEntryToken
            && (*token).type_ != YamlFlowSequenceEndToken
        {
            PUSH!(
                (*parser).states,
                YamlParseFlowSequenceEntryMappingEndState
            );
            return yaml_parser_parse_node(parser, event, false, false);
        }
    }
    (*parser).state = YamlParseFlowSequenceEntryMappingEndState;
    yaml_parser_process_empty_scalar(event, (*token).start_mark)
}

/**
Finish a mapping inside a flow sequence entry.

# Safety
- `parser` and `event` must be valid pointers.
*/
unsafe fn yaml_parser_parse_flow_sequence_entry_mapping_end(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
) -> Success {
    let token: *mut YamlTokenT = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }
    (*parser).state = YamlParseFlowSequenceEntryState;
    let _ = memset(
        event as *mut libc::c_void,
        0,
        size_of::<YamlEventT>() as libc::c_ulong,
    );
    (*event).type_ = YamlMappingEndEvent;
    (*event).start_mark = (*token).start_mark;
    (*event).end_mark = (*token).start_mark;
    OK
}

/**
Parse a key in a flow mapping.

Handles the transition between comma (',') separated key-value pairs and
the flow mapping end ('}').

# Safety
- `parser` and `event` must be valid pointers.
*/
unsafe fn yaml_parser_parse_flow_mapping_key(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
    first: bool,
) -> Success {
    let mut token: *mut YamlTokenT;
    if first {
        token = peek_token(parser);
        PUSH!((*parser).marks, (*token).start_mark);
        skip_token(parser);
    }
    token = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }
    if (*token).type_ != YamlFlowMappingEndToken {
        if !first {
            if (*token).type_ == YamlFlowEntryToken {
                skip_token(parser);
                token = peek_token(parser);
                if token.is_null() {
                    return FAIL;
                }
            } else {
                yaml_parser_set_parser_error_context(
                    parser,
                    b"while parsing a flow mapping\0" as *const u8
                        as *const libc::c_char,
                    POP!((*parser).marks),
                    b"did not find expected ',' or '}'\0" as *const u8
                        as *const libc::c_char,
                    (*token).start_mark,
                );
                return FAIL;
            }
        }
        if (*token).type_ == YamlKeyToken {
            skip_token(parser);
            token = peek_token(parser);
            if token.is_null() {
                return FAIL;
            }
            if (*token).type_ != YamlValueToken
                && (*token).type_ != YamlFlowEntryToken
                && (*token).type_ != YamlFlowMappingEndToken
            {
                PUSH!((*parser).states, YamlParseFlowMappingValueState);
                return yaml_parser_parse_node(
                    parser, event, false, false,
                );
            } else {
                (*parser).state = YamlParseFlowMappingValueState;
                return yaml_parser_process_empty_scalar(
                    event,
                    (*token).start_mark,
                );
            }
        } else if (*token).type_ != YamlFlowMappingEndToken {
            PUSH!(
                (*parser).states,
                YamlParseFlowMappingEmptyValueState
            );
            return yaml_parser_parse_node(parser, event, false, false);
        }
    }
    (*parser).state = POP!((*parser).states);
    let _ = POP!((*parser).marks);
    let _ = memset(
        event as *mut libc::c_void,
        0,
        size_of::<YamlEventT>() as libc::c_ulong,
    );
    (*event).type_ = YamlMappingEndEvent;
    (*event).start_mark = (*token).start_mark;
    (*event).end_mark = (*token).end_mark;
    skip_token(parser);
    OK
}

/**
Parse a value in a flow mapping.

If `empty` is true, parses an empty value (e.g., missing after a key token).

# Safety
- `parser` and `event` must be valid pointers.
*/
unsafe fn yaml_parser_parse_flow_mapping_value(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
    empty: bool,
) -> Success {
    let mut token: *mut YamlTokenT;
    token = peek_token(parser);
    if token.is_null() {
        return FAIL;
    }
    if empty {
        (*parser).state = YamlParseFlowMappingKeyState;
        return yaml_parser_process_empty_scalar(
            event,
            (*token).start_mark,
        );
    }
    if (*token).type_ == YamlValueToken {
        skip_token(parser);
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ != YamlFlowEntryToken
            && (*token).type_ != YamlFlowMappingEndToken
        {
            PUSH!((*parser).states, YamlParseFlowMappingKeyState);
            return yaml_parser_parse_node(parser, event, false, false);
        }
    }
    (*parser).state = YamlParseFlowMappingKeyState;
    yaml_parser_process_empty_scalar(event, (*token).start_mark)
}

/**
Produce an empty scalar event (e.g., when a key or value is omitted).

# Safety
- `event` must be a valid pointer.
*/
unsafe fn yaml_parser_process_empty_scalar(
    event: *mut YamlEventT,
    mark: YamlMarkT,
) -> Success {
    let value: *mut yaml_char_t =
        yaml_malloc(1_u64) as *mut yaml_char_t;
    *value = b'\0';
    let _ = memset(
        event as *mut libc::c_void,
        0,
        size_of::<YamlEventT>() as libc::c_ulong,
    );
    (*event).type_ = YamlScalarEvent;
    (*event).start_mark = mark;
    (*event).end_mark = mark;
    let fresh138 = addr_of_mut!((*event).data.scalar.anchor);
    *fresh138 = ptr::null_mut::<yaml_char_t>();
    let fresh139 = addr_of_mut!((*event).data.scalar.tag);
    *fresh139 = ptr::null_mut::<yaml_char_t>();
    let fresh140 = addr_of_mut!((*event).data.scalar.value);
    *fresh140 = value;
    (*event).data.scalar.length = 0_u64;
    (*event).data.scalar.plain_implicit = true;
    (*event).data.scalar.quoted_implicit = false;
    (*event).data.scalar.style = YamlPlainScalarStyle;
    OK
}

/**
Process YAML directives (e.g., `%YAML 1.2`, `%TAG !yaml! tag:...`).

- Stores found version directives and tag directives.
- Sets parser errors if duplicates or invalid directives are found.

# Safety
- `parser` must be valid.
- `version_directive_ref`, `tag_directives_start_ref`, `tag_directives_end_ref`
  may be null if the caller does not need the data.
*/
unsafe fn yaml_parser_process_directives(
    parser: *mut YamlParserT,
    version_directive_ref: *mut *mut YamlVersionDirectiveT,
    tag_directives_start_ref: *mut *mut YamlTagDirectiveT,
    tag_directives_end_ref: *mut *mut YamlTagDirectiveT,
) -> Success {
    let mut current_block: u64;
    let mut default_tag_directives: [YamlTagDirectiveT; 3] = [
        YamlTagDirectiveT {
            handle: b"!\0" as *const u8 as *const libc::c_char
                as *mut yaml_char_t,
            prefix: b"!\0" as *const u8 as *const libc::c_char
                as *mut yaml_char_t,
        },
        YamlTagDirectiveT {
            handle: b"!!\0" as *const u8 as *const libc::c_char
                as *mut yaml_char_t,
            prefix: b"tag:yaml.org,2002:\0" as *const u8
                as *const libc::c_char
                as *mut yaml_char_t,
        },
        YamlTagDirectiveT {
            handle: ptr::null_mut::<yaml_char_t>(),
            prefix: ptr::null_mut::<yaml_char_t>(),
        },
    ];
    let mut default_tag_directive: *mut YamlTagDirectiveT;
    let mut version_directive: *mut YamlVersionDirectiveT =
        ptr::null_mut::<YamlVersionDirectiveT>();

    struct TagDirectives {
        start: *mut YamlTagDirectiveT,
        end: *mut YamlTagDirectiveT,
        top: *mut YamlTagDirectiveT,
    }
    let mut tag_directives = TagDirectives {
        start: ptr::null_mut::<YamlTagDirectiveT>(),
        end: ptr::null_mut::<YamlTagDirectiveT>(),
        top: ptr::null_mut::<YamlTagDirectiveT>(),
    };
    let mut token: *mut YamlTokenT;
    STACK_INIT!(tag_directives, YamlTagDirectiveT);

    token = peek_token(parser);
    if !token.is_null() {
        loop {
            if !((*token).type_ == YamlVersionDirectiveToken
                || (*token).type_ == YamlTagDirectiveToken)
            {
                current_block = 16924917904204750491;
                break;
            }
            if (*token).type_ == YamlVersionDirectiveToken {
                if !version_directive.is_null() {
                    yaml_parser_set_parser_error(
                        parser,
                        b"found duplicate %YAML directive\0"
                            as *const u8
                            as *const libc::c_char,
                        (*token).start_mark,
                    );
                    current_block = 17143798186130252483;
                    break;
                } else if (*token).data.version_directive.major != 1
                    || (*token).data.version_directive.minor != 1
                        && (*token).data.version_directive.minor != 2
                {
                    yaml_parser_set_parser_error(
                        parser,
                        b"found incompatible YAML document\0"
                            as *const u8
                            as *const libc::c_char,
                        (*token).start_mark,
                    );
                    current_block = 17143798186130252483;
                    break;
                } else {
                    version_directive =
                        yaml_malloc(size_of::<YamlVersionDirectiveT>()
                            as libc::c_ulong)
                            as *mut YamlVersionDirectiveT;
                    (*version_directive).major =
                        (*token).data.version_directive.major;
                    (*version_directive).minor =
                        (*token).data.version_directive.minor;
                }
            } else if (*token).type_ == YamlTagDirectiveToken {
                let value = YamlTagDirectiveT {
                    handle: (*token).data.tag_directive.handle,
                    prefix: (*token).data.tag_directive.prefix,
                };
                if yaml_parser_append_tag_directive(
                    parser,
                    value,
                    false,
                    (*token).start_mark,
                )
                .fail
                {
                    current_block = 17143798186130252483;
                    break;
                }
                PUSH!(tag_directives, value);
            }
            skip_token(parser);
            token = peek_token(parser);
            if token.is_null() {
                current_block = 17143798186130252483;
                break;
            }
        }
        if current_block != 17143798186130252483 {
            default_tag_directive = default_tag_directives.as_mut_ptr();
            loop {
                if (*default_tag_directive).handle.is_null() {
                    current_block = 18377268871191777778;
                    break;
                }
                if yaml_parser_append_tag_directive(
                    parser,
                    *default_tag_directive,
                    true,
                    (*token).start_mark,
                )
                .fail
                {
                    current_block = 17143798186130252483;
                    break;
                }
                default_tag_directive =
                    default_tag_directive.wrapping_offset(1);
            }
            if current_block != 17143798186130252483 {
                if !version_directive_ref.is_null() {
                    *version_directive_ref = version_directive;
                }
                if !tag_directives_start_ref.is_null() {
                    if STACK_EMPTY!(tag_directives) {
                        *tag_directives_end_ref =
                            ptr::null_mut::<YamlTagDirectiveT>();
                        *tag_directives_start_ref =
                            *tag_directives_end_ref;
                        STACK_DEL!(tag_directives);
                    } else {
                        *tag_directives_start_ref =
                            tag_directives.start;
                        *tag_directives_end_ref = tag_directives.top;
                    }
                } else {
                    STACK_DEL!(tag_directives);
                }
                if version_directive_ref.is_null() {
                    yaml_free(version_directive as *mut libc::c_void);
                }
                return OK;
            }
        }
    }
    yaml_free(version_directive as *mut libc::c_void);
    while !STACK_EMPTY!(tag_directives) {
        let tag_directive = POP!(tag_directives);
        yaml_free(tag_directive.handle as *mut libc::c_void);
        yaml_free(tag_directive.prefix as *mut libc::c_void);
    }
    STACK_DEL!(tag_directives);
    FAIL
}

/**
Append a new tag directive to the parser’s list of tag directives.

# Safety
- `parser` must be valid.
- `value` must have valid handle and prefix pointers or be null.
- If `allow_duplicates` is false, duplicates will result in an error.
*/
unsafe fn yaml_parser_append_tag_directive(
    parser: *mut YamlParserT,
    value: YamlTagDirectiveT,
    allow_duplicates: bool,
    mark: YamlMarkT,
) -> Success {
    let mut tag_directive: *mut YamlTagDirectiveT;
    let mut copy = YamlTagDirectiveT {
        handle: ptr::null_mut::<yaml_char_t>(),
        prefix: ptr::null_mut::<yaml_char_t>(),
    };
    tag_directive = (*parser).tag_directives.start;
    while tag_directive != (*parser).tag_directives.top {
        if strcmp(
            value.handle as *mut libc::c_char,
            (*tag_directive).handle as *mut libc::c_char,
        ) == 0
        {
            if allow_duplicates {
                return OK;
            }
            yaml_parser_set_parser_error(
                parser,
                b"found duplicate %TAG directive\0" as *const u8
                    as *const libc::c_char,
                mark,
            );
            return FAIL;
        }
        tag_directive = tag_directive.wrapping_offset(1);
    }
    copy.handle = yaml_strdup(value.handle);
    copy.prefix = yaml_strdup(value.prefix);
    PUSH!((*parser).tag_directives, copy);
    OK
}

/// Frees all heap allocations associated with a YAML event and zeroes out its memory.
///
/// This function is responsible for properly cleaning up all dynamically allocated memory within a `YamlEventT` structure. It handles various event types differently based on their internal structure:
///
/// - Document Start events: Frees version directives and tag directives
/// - Alias events: Frees anchor values
/// - Scalar events: Frees anchors, tags, and scalar values
/// - Sequence Start events: Frees anchors and tags
/// - Mapping Start events: Frees anchors and tags
/// - Other events: No heap allocations to free
///
/// After freeing all allocations, it zeros out the entire event structure to prevent use-after-free issues.
///
/// # Safety
///
/// The caller must ensure that:
/// - `event` points to a valid, properly aligned `YamlEventT` structure
/// - The event was initialized by the YAML parser
/// - The event is not currently in use by any other part of the program
/// - All pointers within the event structure point to validly allocated memory or are null
/// - The event will not be used after calling this function
/// - This function is not called multiple times on the same event
///
/// # Memory Safety
///
/// This function:
/// - Checks for null pointers before attempting to free memory
/// - Sets all freed pointers to null to prevent double-frees
/// - Zeroes out the entire event structure after freeing all allocations
///
/// # Examples
///
/// ```ignore
/// unsafe {
///     let mut event = YamlEventT::default();
///     // ... parser fills event with data ...
///
///     // Clean up when done
///     yaml_event_delete(&mut event);
///     // event should not be used after this point
/// }
/// ```
///
#[inline(never)]
pub unsafe fn yaml_event_delete(event: *mut YamlEventT) {
    if event.is_null() {
        return;
    }

    match (*event).type_ {
        // 1) Document Start: possibly has a version directive and tag directives
        YamlDocumentStartEvent => {
            // Free version directive (if any)
            let vd = (*event).data.document_start.version_directive;
            if !vd.is_null() {
                yaml_free(vd as *mut libc::c_void);
                (*event).data.document_start.version_directive =
                    ptr::null_mut();
            }

            // Free each tag directive's handle and prefix
            let mut start =
                (*event).data.document_start.tag_directives.start;
            let end = (*event).data.document_start.tag_directives.end;
            while start < end {
                yaml_free((*start).handle as *mut libc::c_void);
                yaml_free((*start).prefix as *mut libc::c_void);
                start = start.add(1);
            }

            // Free the tag_directives array itself
            let tag_array =
                (*event).data.document_start.tag_directives.start;
            if !tag_array.is_null() {
                yaml_free(tag_array as *mut libc::c_void);
            }

            // Null out pointers to be safe
            (*event).data.document_start.tag_directives.start =
                ptr::null_mut();
            (*event).data.document_start.tag_directives.end =
                ptr::null_mut();
        }

        // 2) Document End: typically no heap allocations
        YamlDocumentEndEvent => {
            // nothing to free
        }

        // 3) Alias Event: only an anchor is allocated
        YamlAliasEvent => {
            let anchor = (*event).data.alias.anchor;
            if !anchor.is_null() {
                yaml_free(anchor as *mut libc::c_void);
                (*event).data.alias.anchor = ptr::null_mut();
            }
        }

        // 4) Scalar Event: anchor, tag, and value can be allocated
        YamlScalarEvent => {
            let anchor = (*event).data.scalar.anchor;
            let tag = (*event).data.scalar.tag;
            let value = (*event).data.scalar.value;

            if !anchor.is_null() {
                yaml_free(anchor as *mut libc::c_void);
                (*event).data.scalar.anchor = ptr::null_mut();
            }
            if !tag.is_null() {
                yaml_free(tag as *mut libc::c_void);
                (*event).data.scalar.tag = ptr::null_mut();
            }
            if !value.is_null() {
                yaml_free(value as *mut libc::c_void);
                (*event).data.scalar.value = ptr::null_mut();
            }
        }

        // 5) Sequence Start: anchor and tag
        YamlSequenceStartEvent => {
            let anchor = (*event).data.sequence_start.anchor;
            let tag = (*event).data.sequence_start.tag;
            if !anchor.is_null() {
                yaml_free(anchor as *mut libc::c_void);
                (*event).data.sequence_start.anchor = ptr::null_mut();
            }
            if !tag.is_null() {
                yaml_free(tag as *mut libc::c_void);
                (*event).data.sequence_start.tag = ptr::null_mut();
            }
        }

        // 6) Sequence End: no heap allocations by default
        YamlSequenceEndEvent => {
            // nothing to free
        }

        // 7) Mapping Start: anchor and tag
        YamlMappingStartEvent => {
            let anchor = (*event).data.mapping_start.anchor;
            let tag = (*event).data.mapping_start.tag;
            if !anchor.is_null() {
                yaml_free(anchor as *mut libc::c_void);
                (*event).data.mapping_start.anchor = ptr::null_mut();
            }
            if !tag.is_null() {
                yaml_free(tag as *mut libc::c_void);
                (*event).data.mapping_start.tag = ptr::null_mut();
            }
        }

        // 8) Mapping End: no heap allocations by default
        YamlMappingEndEvent => {
            // nothing to free
        }

        // 9) Stream Start / End: no heap allocations by default
        YamlStreamStartEvent | YamlStreamEndEvent => {
            // nothing to free
        }

        // 10) Fallback: ignore unrecognized event types (if any)
        _ => {
            // Do nothing
        }
    }

    // zero out the event
    write_bytes(event, 0, 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml::{YamlQueueT, YamlSimpleKeyT, YamlStackT};
    use crate::YamlParserStateT;
    use crate::YamlTokenTypeT;
    use core::mem::size_of;
    use core::ptr;

    // ========================================================================
    // Helper Functions
    // ========================================================================

    /// Creates a fully initialized parser for testing.
    ///
    /// This function does the following:
    ///  1. Allocates a YamlParserT using `yaml_malloc`.
    ///  2. Zeros out the entire struct with `memset`.
    ///  3. Initializes every field—this includes setting `not_simple_keys` to a known
    ///     value (e.g., `false`) to avoid undefined behavior in the scanner.
    ///  4. Allocates and initializes the token queue, the simple keys stack,
    ///     and the necessary state stacks.
    ///  5. Sets `eof = false` so we can feed tokens manually.
    ///
    /// # Safety
    /// - The returned pointer must be freed with `teardown_parser`.
    pub(super) unsafe fn setup_parser() -> *mut YamlParserT {
        // 1) Allocate memory for the parser struct
        let p =
            yaml_malloc(size_of::<YamlParserT>().try_into().unwrap())
                as *mut YamlParserT;

        // 2) Zero out the entire parser struct
        memset(
            p as *mut core::ffi::c_void,
            0,
            size_of::<YamlParserT>() as libc::c_ulong,
        );

        // 3) Initialize the basic fields
        (*p).error = YamlNoError;
        (*p).stream_end_produced = false;
        (*p).not_simple_keys = 0;
        (*p).state = YamlParseStreamStartState;
        (*p).token_available = false;
        (*p).tokens_parsed = 0;

        // 4) Allocate and initialize the token queue
        let queue_size = size_of::<YamlTokenT>() * 16;
        let tokens = yaml_malloc(queue_size.try_into().unwrap())
            as *mut YamlTokenT;
        // Optional: zero out the token array
        memset(
            tokens as *mut core::ffi::c_void,
            0,
            queue_size as libc::c_ulong,
        );

        (*p).tokens = YamlQueueT {
            start: tokens,
            head: tokens,
            tail: tokens,
            end: tokens.add(16),
        };

        // 5) Allocate and initialize the simple keys stack
        let simple_keys_size = size_of::<YamlSimpleKeyT>() * 16;
        let simple_keys =
            yaml_malloc(simple_keys_size.try_into().unwrap())
                as *mut YamlSimpleKeyT;
        // Optional: zero out the simple keys array
        memset(
            simple_keys as *mut core::ffi::c_void,
            0,
            simple_keys_size as libc::c_ulong,
        );

        (*p).simple_keys = YamlStackT {
            start: simple_keys,
            end: simple_keys.add(16),
            top: simple_keys,
        };
        (*p).simple_key_allowed = true;

        // 6) Initialize state stacks
        STACK_INIT!((*p).states, YamlParserStateT);
        STACK_INIT!((*p).marks, YamlMarkT);
        STACK_INIT!((*p).tag_directives, YamlTagDirectiveT);

        // 7) Initialize stream buffers and marks
        (*p).raw_buffer.start = ptr::null_mut();
        (*p).raw_buffer.end = ptr::null_mut();
        (*p).raw_buffer.pointer = ptr::null_mut();
        (*p).eof = false; // set false if you plan to inject tokens manually
        (*p).mark = YamlMarkT {
            index: 0,
            line: 0,
            column: 0,
        };

        p
    }

    /// Cleans up all resources associated with a parser.
    ///
    /// This function frees:
    /// - Token queue
    /// - Simple keys queue
    /// - All stacks
    /// - The parser structure itself
    ///
    /// # Safety
    /// - `parser` must be a valid pointer returned by `setup_parser`
    /// - Must not be called twice on the same parser
    fn teardown_parser(parser: *mut YamlParserT) {
        unsafe {
            // 1) Manually free each tag_directive that was pushed.
            while !STACK_EMPTY!((*parser).tag_directives) {
                let td = POP!((*parser).tag_directives);
                yaml_free(td.handle as *mut libc::c_void);
                yaml_free(td.prefix as *mut libc::c_void);
            }
            // 2) Then free the tag_directives array itself
            STACK_DEL!((*parser).tag_directives);

            // 3) Other stacks
            STACK_DEL!((*parser).states);
            STACK_DEL!((*parser).marks);

            // 4) Free token queue, parser struct, etc.
            yaml_free((*parser).tokens.start as *mut libc::c_void);
            yaml_free((*parser).simple_keys.start as *mut libc::c_void);
            yaml_free(parser as *mut libc::c_void);
        }
    }

    /// Creates a new token with default marks.
    ///
    /// # Arguments
    /// * `token_type` - The type of token to create
    ///
    /// # Returns
    /// A pointer to the newly allocated token
    ///
    /// # Safety
    /// The returned pointer must be freed using yaml_free
    fn create_token(token_type: YamlTokenTypeT) -> *mut YamlTokenT {
        unsafe {
            let t = yaml_malloc(
                size_of::<YamlTokenT>().try_into().unwrap(),
            ) as *mut YamlTokenT;

            // <-- Zero out the entire token to avoid uninitialized data
            memset(
                t as *mut core::ffi::c_void,
                0,
                size_of::<YamlTokenT>() as libc::c_ulong,
            );

            (*t).type_ = token_type;
            (*t).start_mark = YamlMarkT {
                index: 0,
                line: 0,
                column: 0,
            };
            (*t).end_mark = (*t).start_mark;

            t
        }
    }

    /// Adds a token to the parser's token queue.
    ///
    /// This function creates a token, copies it into the parser's queue,
    /// and handles cleanup of the temporary token.
    ///
    /// # Arguments
    /// * `parser` - The parser to add the token to
    /// * `token_type` - The type of token to add
    ///
    /// # Returns
    /// A pointer to the token in the parser's queue
    ///
    /// # Safety
    /// - `parser` must be valid
    /// - The returned pointer is owned by the parser and must not be freed directly
    fn add_token_to_parser(
        parser: *mut YamlParserT,
        token_type: YamlTokenTypeT,
    ) -> *mut YamlTokenT {
        // 1) Create the temporary token
        let temp_token = create_token(token_type);

        unsafe {
            // 2) Where to store this token in the parser’s queue
            //    This example uses `tail` as the “write” pointer.
            //    (Some code uses `head` for writes, but typically `head` is the read pointer.)
            let dest = (*parser).tokens.tail;

            // 3) Copy from the temp token to the parser’s buffer
            memcpy(
                dest as *mut core::ffi::c_void,
                temp_token as *const core::ffi::c_void,
                core::mem::size_of::<YamlTokenT>().try_into().unwrap(),
            );

            // 4) Free the temporary
            yaml_free(temp_token as *mut core::ffi::c_void);

            // 5) Mark that we have tokens available; also advance `tail`
            (*parser).token_available = true;
            (*parser).tokens.tail = (*parser).tokens.tail.add(1);

            // 6) Return the pointer *inside the parser's buffer*
            //    This ensures `peek_token(parser)` will match.
            dest
        }
    }

    // ========================================================================
    // Basic Token Tests
    // ========================================================================

    #[test]
    fn test_peek_token_null_parser() {
        let result = unsafe { peek_token(ptr::null_mut()) };
        assert!(result.is_null());
    }

    #[test]
    fn test_peek_token_available() {
        let parser = unsafe { setup_parser() };
        unsafe {
            // This now returns the token’s location in the parser’s own token array
            let token_in_queue =
                add_token_to_parser(parser, YamlStreamStartToken);

            let peeked = peek_token(parser);
            // Both should point to the same YamlTokenT in the parser buffer
            assert_eq!(peeked, token_in_queue);
            assert_eq!((*peeked).type_, YamlStreamStartToken);
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_skip_token() {
        let parser = unsafe { setup_parser() };
        unsafe {
            add_token_to_parser(parser, YamlStreamStartToken);
            (*parser).tokens_parsed = 0;
            skip_token(parser);
            assert!(!(*parser).token_available);
            assert_eq!((*parser).tokens_parsed, 1);
        }
        teardown_parser(parser);
    }

    // ========================================================================
    // Stream Tests
    // ========================================================================

    #[test]
    fn test_parse_stream_start() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            add_token_to_parser(parser, YamlStreamStartToken);
            let result =
                yaml_parser_parse_stream_start(parser, &mut event);
            assert_eq!(result, OK);
            assert_eq!(event.type_, YamlStreamStartEvent);
            assert_eq!(
                (*parser).state,
                YamlParseImplicitDocumentStartState
            );
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_parse_stream_start_invalid_token() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            add_token_to_parser(parser, YamlScalarToken);
            let result =
                yaml_parser_parse_stream_start(parser, &mut event);
            assert_eq!(result, FAIL);
            assert_eq!((*parser).error, YamlParserError);
        }
        teardown_parser(parser);
    }

    // ========================================================================
    // Document Tests
    // ========================================================================

    #[test]
    fn test_parse_document_content() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            let scalar = add_token_to_parser(parser, YamlScalarToken);
            (*scalar).data.scalar.value = yaml_strdup(
                b"content\0" as *const u8 as *mut yaml_char_t,
            );
            (*scalar).data.scalar.length = 7;

            PUSH!((*parser).states, YamlParseDocumentContentState);

            let result =
                yaml_parser_parse_document_content(parser, &mut event);
            assert_eq!(result, OK);

            yaml_event_delete(&mut event);
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_parse_indentless_sequence_entry() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            add_token_to_parser(parser, YamlBlockEntryToken);

            let scalar = add_token_to_parser(parser, YamlScalarToken);
            (*scalar).data.scalar.value = yaml_strdup(
                b"sequence_item\0" as *const u8 as *mut yaml_char_t,
            );
            (*scalar).data.scalar.length = 12;

            let result = yaml_parser_parse_indentless_sequence_entry(
                parser, &mut event,
            );
            assert_eq!(result, OK);

            yaml_event_delete(&mut event);
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_document_start_implicit() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            // Add a token that triggers a DocumentStart event.
            add_token_to_parser(parser, YamlScalarToken);

            let result = yaml_parser_parse_document_start(
                parser, &mut event, true,
            );
            assert_eq!(result, OK);
            assert_eq!(event.type_, YamlDocumentStartEvent);

            // <-- Miri requires a matching free for all allocations in `event`
            yaml_event_delete(&mut event);
        }

        teardown_parser(parser);
    }

    #[test]
    fn test_document_end() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            add_token_to_parser(parser, YamlDocumentEndToken);
            let result =
                yaml_parser_parse_document_end(parser, &mut event);
            assert_eq!(result, OK);
            assert_eq!(event.type_, YamlDocumentEndEvent);
            assert!(!event.data.document_end.implicit);
        }
        teardown_parser(parser);
    }

    // ========================================================================
    // Sequence Tests
    // ========================================================================

    #[test]
    fn test_parse_block_sequence_entry() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            // First token (needed for first==true case)
            add_token_to_parser(parser, YamlBlockEntryToken);

            // Second token - the actual block entry we'll parse
            add_token_to_parser(parser, YamlBlockEntryToken);

            // Next token after block entry (not BlockEntry or BlockEnd)
            add_token_to_parser(parser, YamlScalarToken);

            let result = yaml_parser_parse_block_sequence_entry(
                parser, &mut event, true,
            );
            assert_eq!(result, OK);
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_parse_flow_sequence_entry() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            // Initial token for first==true case
            add_token_to_parser(parser, YamlFlowEntryToken);

            // End token to generate sequence end event
            add_token_to_parser(parser, YamlFlowSequenceEndToken);

            // Push parser state to avoid popping from empty stack
            PUSH!((*parser).states, YamlParseFlowSequenceEntryState);

            let result = yaml_parser_parse_flow_sequence_entry(
                parser, &mut event, true,
            );
            assert_eq!(result, OK);
            assert_eq!(event.type_, YamlSequenceEndEvent);
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_parse_flow_sequence_entry_mapping_key() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            let scalar = add_token_to_parser(parser, YamlScalarToken);
            (*scalar).data.scalar.value = yaml_strdup(
                b"mapping_key\0" as *const u8 as *mut yaml_char_t,
            );
            (*scalar).data.scalar.length = 10;

            let result =
                yaml_parser_parse_flow_sequence_entry_mapping_key(
                    parser, &mut event,
                );
            assert_eq!(result, OK);

            yaml_event_delete(&mut event);
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_parse_flow_sequence_entry_mapping_value() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            add_token_to_parser(parser, YamlValueToken);

            let scalar = add_token_to_parser(parser, YamlScalarToken);
            (*scalar).data.scalar.value = yaml_strdup(
                b"mapping_value\0" as *const u8 as *mut yaml_char_t,
            );
            (*scalar).data.scalar.length = 12;

            let result =
                yaml_parser_parse_flow_sequence_entry_mapping_value(
                    parser, &mut event,
                );
            assert_eq!(result, OK);

            yaml_event_delete(&mut event);
        }
        teardown_parser(parser);
    }

    // ========================================================================
    // Scalar Tests
    // ========================================================================

    #[test]
    fn test_process_empty_scalar() {
        let mut event = YamlEventT::default();
        let mark = YamlMarkT {
            index: 0,
            line: 0,
            column: 0,
        };

        unsafe {
            let result =
                yaml_parser_process_empty_scalar(&mut event, mark);
            assert_eq!(result, OK);
            assert_eq!(event.type_, YamlScalarEvent);
            assert_eq!((*event.data.scalar.value), b'\0');
            assert_eq!(event.data.scalar.length, 0);
            assert!(event.data.scalar.plain_implicit);
            assert!(!event.data.scalar.quoted_implicit);

            yaml_free(event.data.scalar.value as *mut libc::c_void);
        }
    }

    // ========================================================================
    // Tag Directive Tests
    // ========================================================================

    #[test]
    fn test_append_tag_directive() {
        let parser = unsafe { setup_parser() };
        let mark = YamlMarkT {
            index: 0,
            line: 0,
            column: 0,
        };
        let handle = b"!test!\0" as *const u8 as *mut yaml_char_t;
        let prefix = b"tag:test.org\0" as *const u8 as *mut yaml_char_t;
        let directive = YamlTagDirectiveT { handle, prefix };

        unsafe {
            let result = yaml_parser_append_tag_directive(
                parser, directive, false, mark,
            );
            assert_eq!(result, OK);
            assert_eq!(
                (*parser)
                    .tag_directives
                    .top
                    .offset_from((*parser).tag_directives.start),
                1
            );
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_append_tag_directive_duplicate() {
        let parser = unsafe { setup_parser() };
        let mark = YamlMarkT {
            index: 0,
            line: 0,
            column: 0,
        };
        let handle = b"!test!\0" as *const u8 as *mut yaml_char_t;
        let prefix = b"tag:test.org\0" as *const u8 as *mut yaml_char_t;
        let directive = YamlTagDirectiveT { handle, prefix };

        unsafe {
            let result1 = yaml_parser_append_tag_directive(
                parser, directive, false, mark,
            );
            assert_eq!(result1, OK);

            let result2 = yaml_parser_append_tag_directive(
                parser, directive, false, mark,
            );
            assert_eq!(result2, FAIL);
            assert_eq!((*parser).error, YamlParserError);
        }
        teardown_parser(parser);
    }

    // ========================================================================
    // Error Tests
    // ========================================================================

    #[test]
    fn test_parser_set_error() {
        let parser = unsafe { setup_parser() };
        let mark = YamlMarkT {
            index: 0,
            line: 0,
            column: 0,
        };
        let error_msg =
            b"test error\0" as *const u8 as *const libc::c_char;

        unsafe {
            yaml_parser_set_parser_error(parser, error_msg, mark);
            assert_eq!((*parser).error, YamlParserError);
            assert_eq!((*parser).problem, error_msg);
            assert_eq!((*parser).problem_mark.index, mark.index);
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_parser_error_context() {
        let parser = unsafe { setup_parser() };
        let mark = YamlMarkT {
            index: 0,
            line: 0,
            column: 0,
        };

        unsafe {
            let context =
                b"test context\0" as *const u8 as *const libc::c_char;
            let problem =
                b"test problem\0" as *const u8 as *const libc::c_char;

            yaml_parser_set_parser_error_context(
                parser, context, mark, problem, mark,
            );

            assert_eq!((*parser).error, YamlParserError);
            assert_eq!((*parser).context, context);
            assert_eq!((*parser).problem, problem);
            assert_eq!((*parser).context_mark.index, mark.index);
            assert_eq!((*parser).problem_mark.index, mark.index);
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_parser_error_result() {
        let parser = unsafe { setup_parser() };
        let mark = YamlMarkT {
            index: 0,
            line: 0,
            column: 0,
        };

        unsafe {
            let problem =
                b"test problem\0" as *const u8 as *const libc::c_char;

            let result = yaml_parser_set_parser_error_result(
                parser, problem, mark,
            );
            assert!(result.is_ok());
            assert_eq!((*parser).error, YamlParserError);
            assert_eq!((*parser).problem, problem);

            // Test null parser case
            let null_result = yaml_parser_set_parser_error_result(
                ptr::null_mut(),
                problem,
                mark,
            );
            assert!(matches!(
                null_result,
                Err(ParserError::GeneralError(_))
            ));
        }
        teardown_parser(parser);
    }

    // ========================================================================
    // Node Tests
    // ========================================================================
    #[test]
    fn test_parse_node_alias() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            let token = add_token_to_parser(parser, YamlAliasToken);
            (*token).data.alias.value = yaml_strdup(
                b"test_alias\0" as *const u8 as *mut yaml_char_t,
            );

            let result =
                yaml_parser_parse_node(parser, &mut event, true, false);
            assert_eq!(result, OK);
            assert_eq!(event.type_, YamlAliasEvent);

            yaml_event_delete(&mut event);
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_parse_node_scalar() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            let token = add_token_to_parser(parser, YamlScalarToken);
            (*token).data.scalar.value = yaml_strdup(
                b"test_value\0" as *const u8 as *mut yaml_char_t,
            );
            (*token).data.scalar.length = 10;
            (*token).data.scalar.style = YamlPlainScalarStyle;

            let result =
                yaml_parser_parse_node(parser, &mut event, true, false);
            assert_eq!(result, OK);
            assert_eq!(event.type_, YamlScalarEvent);

            yaml_event_delete(&mut event);
        }
        teardown_parser(parser);
    }

    // ========================================================================
    // Flow Sequence Tests
    // ========================================================================
    #[test]
    fn test_parse_flow_mapping_key() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            // Initial token for first==true case
            add_token_to_parser(parser, YamlKeyToken);

            // Add a scalar token after the key
            let scalar = add_token_to_parser(parser, YamlScalarToken);
            (*scalar).data.scalar.value = yaml_strdup(
                b"key_value\0" as *const u8 as *mut yaml_char_t,
            );
            (*scalar).data.scalar.length = 9;

            PUSH!((*parser).states, YamlParseFlowMappingKeyState);

            let result = yaml_parser_parse_flow_mapping_key(
                parser, &mut event, true,
            );
            assert_eq!(result, OK);

            yaml_event_delete(&mut event);
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_parse_flow_mapping_value() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            add_token_to_parser(parser, YamlValueToken);

            let scalar = add_token_to_parser(parser, YamlScalarToken);
            (*scalar).data.scalar.value = yaml_strdup(
                b"test_value\0" as *const u8 as *mut yaml_char_t,
            );
            (*scalar).data.scalar.length = 10;

            let result = yaml_parser_parse_flow_mapping_value(
                parser, &mut event, false,
            );
            assert_eq!(result, OK);

            yaml_event_delete(&mut event);
        }
        teardown_parser(parser);
    }

    // ========================================================================
    // YAML Tests
    // ========================================================================
    #[test]
    fn test_yaml_parser_parse() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            // Test failure cases first
            (*parser).stream_end_produced = true;
            assert_eq!(yaml_parser_parse(parser, &mut event), FAIL);
            (*parser).stream_end_produced = false;

            (*parser).error = YamlParserError;
            assert_eq!(yaml_parser_parse(parser, &mut event), FAIL);
            (*parser).error = YamlNoError;

            (*parser).state = YamlParseEndState;
            assert_eq!(yaml_parser_parse(parser, &mut event), FAIL);
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_parse_document_start_with_directives() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            // Add version directive
            let version =
                add_token_to_parser(parser, YamlVersionDirectiveToken);
            (*version).data.version_directive.major = 1;
            (*version).data.version_directive.minor = 2;

            // Add tag directive
            let tag =
                add_token_to_parser(parser, YamlTagDirectiveToken);
            (*tag).data.tag_directive.handle = yaml_strdup(
                b"!test!\0" as *const u8 as *mut yaml_char_t,
            );
            (*tag).data.tag_directive.prefix = yaml_strdup(
                b"tag:test.org\0" as *const u8 as *mut yaml_char_t,
            );

            // Add document start token
            add_token_to_parser(parser, YamlDocumentStartToken);

            let result = yaml_parser_parse_document_start(
                parser, &mut event, false,
            );
            assert_eq!(result, OK);
            assert_eq!(event.type_, YamlDocumentStartEvent);
            assert!(!event.data.document_start.implicit);

            yaml_event_delete(&mut event);
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_parse_block_mapping_value_with_token() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            add_token_to_parser(parser, YamlValueToken);

            let scalar = add_token_to_parser(parser, YamlScalarToken);
            (*scalar).data.scalar.value = yaml_strdup(
                b"test_value\0" as *const u8 as *mut yaml_char_t,
            );
            (*scalar).data.scalar.length = 10;

            let result = yaml_parser_parse_block_mapping_value(
                parser, &mut event,
            );
            assert_eq!(result, OK);

            yaml_event_delete(&mut event);
        }
        teardown_parser(parser);
    }

    #[test]
    fn test_parse_flow_sequence_entry_mapping_end() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            // Add a token to help with mark setting
            add_token_to_parser(parser, YamlFlowEntryToken);

            // Set initial state
            (*parser).state = YamlParseFlowSequenceEntryMappingEndState;

            let result =
                yaml_parser_parse_flow_sequence_entry_mapping_end(
                    parser, &mut event,
                );
            assert_eq!(result, OK);
            assert_eq!(event.type_, YamlMappingEndEvent);
            assert_eq!(
                (*parser).state,
                YamlParseFlowSequenceEntryState
            );

            yaml_event_delete(&mut event);
        }
        teardown_parser(parser);
    }
    #[test]
    fn test_parser_state_machine_invalid_state() {
        let parser = unsafe { setup_parser() };
        let mut event = YamlEventT::default();

        unsafe {
            (*parser).state = YamlParseEndState;
            let result = yaml_parser_state_machine(parser, &mut event);
            assert_eq!(result, FAIL);
        }
        teardown_parser(parser);
    }
}
