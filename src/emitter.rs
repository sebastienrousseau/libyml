// emitter.rs

use crate::externs::{strcmp, strlen, strncmp};
use crate::internal::{yaml_queue_extend, yaml_stack_extend};
use crate::memory::{yaml_free, yaml_strdup};
use crate::ops::{ForceAdd as _, ForceMul as _};
use crate::success::{Success, FAIL, OK};
use crate::yaml::{size_t, yaml_char_t, YamlStringT};
use crate::{
    libc, yaml_emitter_flush, yaml_event_delete, PointerExt,
    YamlAliasEvent, YamlAnyBreak, YamlAnyEncoding, YamlAnyScalarStyle,
    YamlCrBreak, YamlCrlnBreak, YamlDocumentEndEvent,
    YamlDocumentStartEvent, YamlDoubleQuotedScalarStyle,
    YamlEmitBlockMappingFirstKeyState, YamlEmitBlockMappingKeyState,
    YamlEmitBlockMappingSimpleValueState,
    YamlEmitBlockMappingValueState,
    YamlEmitBlockSequenceFirstItemState,
    YamlEmitBlockSequenceItemState, YamlEmitDocumentContentState,
    YamlEmitDocumentEndState, YamlEmitDocumentStartState,
    YamlEmitEndState, YamlEmitFirstDocumentStartState,
    YamlEmitFlowMappingFirstKeyState, YamlEmitFlowMappingKeyState,
    YamlEmitFlowMappingSimpleValueState, YamlEmitFlowMappingValueState,
    YamlEmitFlowSequenceFirstItemState, YamlEmitFlowSequenceItemState,
    YamlEmitStreamStartState, YamlEmitterError, YamlEmitterT,
    YamlEventT, YamlFlowMappingStyle, YamlFlowSequenceStyle,
    YamlFoldedScalarStyle, YamlLiteralScalarStyle, YamlLnBreak,
    YamlMappingEndEvent, YamlMappingStartEvent, YamlPlainScalarStyle,
    YamlScalarEvent, YamlScalarStyleT, YamlSequenceEndEvent,
    YamlSequenceStartEvent, YamlSingleQuotedScalarStyle,
    YamlStreamEndEvent, YamlStreamStartEvent, YamlTagDirectiveT,
    YamlUtf8Encoding, YamlVersionDirectiveT,
};
use core::ptr::{self, addr_of_mut};

const MAX_SIMPLE_KEY_LENGTH: u64 = 128;

/// Flushes the emitter's buffer if needed.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The buffer fields in `emitter` (start, pointer, end) must be valid and properly aligned
/// - The write handler in `emitter` must be properly initialized and valid
unsafe fn flush(emitter: *mut YamlEmitterT) -> Success {
    if (*emitter).buffer.pointer.wrapping_offset(5_isize)
        < (*emitter).buffer.end
    {
        OK
    } else {
        yaml_emitter_flush(emitter)
    }
}

/// Puts a single byte into the emitter's buffer.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The buffer pointer must have enough space for at least one byte
/// - The buffer fields in `emitter` must be valid and properly aligned
/// - The column field must be valid and not overflow when incremented
unsafe fn put(emitter: *mut YamlEmitterT, value: u8) -> Success {
    if flush(emitter).fail {
        return FAIL;
    }
    let fresh40 = addr_of_mut!((*emitter).buffer.pointer);
    let fresh41 = *fresh40;
    *fresh40 = (*fresh40).wrapping_offset(1);
    *fresh41 = value;
    let fresh42 = addr_of_mut!((*emitter).column);
    *fresh42 += 1;
    OK
}

/// Puts a line break into the emitter's buffer according to the line break style.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The buffer must have enough space for a line break (up to 2 bytes for CRLF)
/// - The line_break field must contain a valid line break style
/// - The line and column fields must be valid and not overflow when modified
unsafe fn put_break(emitter: *mut YamlEmitterT) -> Success {
    if flush(emitter).fail {
        return FAIL;
    }
    if (*emitter).line_break == YamlCrBreak {
        let fresh62 = addr_of_mut!((*emitter).buffer.pointer);
        let fresh63 = *fresh62;
        *fresh62 = (*fresh62).wrapping_offset(1);
        *fresh63 = b'\r';
    } else if (*emitter).line_break == YamlLnBreak {
        let fresh64 = addr_of_mut!((*emitter).buffer.pointer);
        let fresh65 = *fresh64;
        *fresh64 = (*fresh64).wrapping_offset(1);
        *fresh65 = b'\n';
    } else if (*emitter).line_break == YamlCrlnBreak {
        let fresh66 = addr_of_mut!((*emitter).buffer.pointer);
        let fresh67 = *fresh66;
        *fresh66 = (*fresh66).wrapping_offset(1);
        *fresh67 = b'\r';
        let fresh68 = addr_of_mut!((*emitter).buffer.pointer);
        let fresh69 = *fresh68;
        *fresh68 = (*fresh68).wrapping_offset(1);
        *fresh69 = b'\n';
    };
    (*emitter).column = 0;
    let fresh70 = addr_of_mut!((*emitter).line);
    *fresh70 += 1;
    OK
}

/// Writes a string to the emitter's buffer.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `string` must be a valid, non-null pointer to a properly initialized `YamlStringT`
/// - The buffer must have enough space for the string content
/// - All string pointers (start, pointer, end) must be valid and properly aligned
/// - The column field must be valid and not overflow when incremented
unsafe fn write(
    emitter: *mut YamlEmitterT,
    string: *mut YamlStringT,
) -> Success {
    if flush(emitter).fail {
        return FAIL;
    }
    copy!((*emitter).buffer, *string);
    let fresh107 = addr_of_mut!((*emitter).column);
    *fresh107 += 1;
    OK
}

/// Writes a string break to the emitter's buffer.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `string` must be a valid, non-null pointer to a properly initialized `YamlStringT`
/// - The buffer must have enough space for the string content
/// - All string pointers must be valid and properly aligned
/// - The line and column fields must be valid and not overflow when modified
unsafe fn write_break(
    emitter: *mut YamlEmitterT,
    string: *mut YamlStringT,
) -> Success {
    if flush(emitter).fail {
        return FAIL;
    }
    if CHECK!(*string, b'\n') {
        let _ = put_break(emitter);
        (*string).pointer = (*string).pointer.wrapping_offset(1);
    } else {
        copy!((*emitter).buffer, *string);
        (*emitter).column = 0;
        let fresh300 = addr_of_mut!((*emitter).line);
        *fresh300 += 1;
    }
    OK
}

macro_rules! write {
    ($emitter:expr, $string:expr) => {
        write($emitter, addr_of_mut!($string))
    };
}

macro_rules! write_break {
    ($emitter:expr, $string:expr) => {
        write_break($emitter, addr_of_mut!($string))
    };
}

/// Sets an error condition on the emitter.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `problem` must be a valid, null-terminated C string
/// - The problem string must remain valid for the lifetime of the error
unsafe fn yaml_emitter_set_emitter_error(
    emitter: *mut YamlEmitterT,
    problem: *const libc::c_char,
) -> Success {
    (*emitter).error = YamlEmitterError;
    let fresh0 = addr_of_mut!((*emitter).problem);
    *fresh0 = problem;
    FAIL
}

/// Emit an event.
///
/// The event object may be generated using the yaml_parser_parse() function.
/// The emitter takes the responsibility for the event object and destroys its
/// content after it is emitted. The event object is destroyed even if the
/// function fails.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT` struct.
/// - `event` must be a valid, non-null pointer to a `YamlEventT` struct that can be safely read from and will be destroyed by the function.
/// - The `YamlEmitterT` and `YamlEventT` structs must be properly aligned and have the expected memory layout.
/// - The `YamlEmitterT` struct must be in a valid state to emit the provided event.
pub unsafe fn yaml_emitter_emit(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
) -> Success {
    ENQUEUE!((*emitter).events, *event);
    while yaml_emitter_need_more_events(emitter).fail {
        if yaml_emitter_analyze_event(emitter, (*emitter).events.head)
            .fail
        {
            return FAIL;
        }
        if yaml_emitter_state_machine(emitter, (*emitter).events.head)
            .fail
        {
            return FAIL;
        }
        yaml_event_delete(addr_of_mut!(DEQUEUE!((*emitter).events)));
    }
    OK
}

/// Checks if the emitter needs more events to proceed.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The events queue in emitter must be properly initialized
/// - All event pointers in the queue must be valid and properly aligned
unsafe fn yaml_emitter_need_more_events(
    emitter: *mut YamlEmitterT,
) -> Success {
    let mut level: libc::c_int = 0;
    let mut event: *mut YamlEventT;
    if QUEUE_EMPTY!((*emitter).events) {
        return OK;
    }
    let accumulate = match (*(*emitter).events.head).type_ {
        YamlDocumentStartEvent => 1,
        YamlSequenceStartEvent => 2,
        YamlMappingStartEvent => 3,
        _ => return FAIL,
    };
    if (*emitter).events.tail.c_offset_from((*emitter).events.head)
        as libc::c_long
        > accumulate as libc::c_long
    {
        return FAIL;
    }
    event = (*emitter).events.head;
    while event != (*emitter).events.tail {
        match (*event).type_ {
            YamlStreamStartEvent
            | YamlDocumentStartEvent
            | YamlSequenceStartEvent
            | YamlMappingStartEvent => {
                level += 1;
            }
            YamlStreamEndEvent | YamlDocumentEndEvent
            | YamlSequenceEndEvent | YamlMappingEndEvent => {
                level -= 1;
            }
            _ => {}
        }
        if level == 0 {
            return FAIL;
        }
        event = event.wrapping_offset(1);
    }
    OK
}

/// Appends a tag directive to the emitter's tag directives list.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The tag directive's handle and prefix must be valid null-terminated strings
/// - The tag directives stack in emitter must be properly initialized and have capacity
unsafe fn yaml_emitter_append_tag_directive(
    emitter: *mut YamlEmitterT,
    value: YamlTagDirectiveT,
    allow_duplicates: bool,
) -> Success {
    let mut tag_directive: *mut YamlTagDirectiveT;
    let mut copy = YamlTagDirectiveT {
        handle: ptr::null_mut::<yaml_char_t>(),
        prefix: ptr::null_mut::<yaml_char_t>(),
    };
    tag_directive = (*emitter).tag_directives.start;
    while tag_directive != (*emitter).tag_directives.top {
        if strcmp(
            value.handle as *mut libc::c_char,
            (*tag_directive).handle as *mut libc::c_char,
        ) == 0
        {
            if allow_duplicates {
                return OK;
            }
            return yaml_emitter_set_emitter_error(
                emitter,
                b"duplicate %TAG directive\0" as *const u8
                    as *const libc::c_char,
            );
        }
        tag_directive = tag_directive.wrapping_offset(1);
    }
    copy.handle = yaml_strdup(value.handle);
    copy.prefix = yaml_strdup(value.prefix);
    PUSH!((*emitter).tag_directives, copy);
    OK
}

/// Increases the indentation level of the emitter.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The indents stack must be properly initialized and have capacity
/// - The indent field must be valid and not overflow when modified
unsafe fn yaml_emitter_increase_indent(
    emitter: *mut YamlEmitterT,
    flow: bool,
    indentless: bool,
) {
    PUSH!((*emitter).indents, (*emitter).indent);
    if (*emitter).indent < 0 {
        (*emitter).indent =
            if flow { (*emitter).best_indent } else { 0 };
    } else if !indentless {
        (*emitter).indent += (*emitter).best_indent;
    }
}

/// Processes the emitter's state machine.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The emitter must be in a valid state for processing the given event
/// - All event data pointers must be valid and properly aligned
unsafe fn yaml_emitter_state_machine(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
) -> Success {
    match (*emitter).state {
        YamlEmitStreamStartState => {
            yaml_emitter_emit_stream_start(emitter, event)
        }
        YamlEmitFirstDocumentStartState => {
            yaml_emitter_emit_document_start(emitter, event, true)
        }
        YamlEmitDocumentStartState => {
            yaml_emitter_emit_document_start(emitter, event, false)
        }
        YamlEmitDocumentContentState => {
            yaml_emitter_emit_document_content(emitter, event)
        }
        YamlEmitDocumentEndState => {
            yaml_emitter_emit_document_end(emitter, event)
        }
        YamlEmitFlowSequenceFirstItemState => {
            yaml_emitter_emit_flow_sequence_item(emitter, event, true)
        }
        YamlEmitFlowSequenceItemState => {
            yaml_emitter_emit_flow_sequence_item(emitter, event, false)
        }
        YamlEmitFlowMappingFirstKeyState => {
            yaml_emitter_emit_flow_mapping_key(emitter, event, true)
        }
        YamlEmitFlowMappingKeyState => {
            yaml_emitter_emit_flow_mapping_key(emitter, event, false)
        }
        YamlEmitFlowMappingSimpleValueState => {
            yaml_emitter_emit_flow_mapping_value(emitter, event, true)
        }
        YamlEmitFlowMappingValueState => {
            yaml_emitter_emit_flow_mapping_value(emitter, event, false)
        }
        YamlEmitBlockSequenceFirstItemState => {
            yaml_emitter_emit_block_sequence_item(emitter, event, true)
        }
        YamlEmitBlockSequenceItemState => {
            yaml_emitter_emit_block_sequence_item(emitter, event, false)
        }
        YamlEmitBlockMappingFirstKeyState => {
            yaml_emitter_emit_block_mapping_key(emitter, event, true)
        }
        YamlEmitBlockMappingKeyState => {
            yaml_emitter_emit_block_mapping_key(emitter, event, false)
        }
        YamlEmitBlockMappingSimpleValueState => {
            yaml_emitter_emit_block_mapping_value(emitter, event, true)
        }
        YamlEmitBlockMappingValueState => {
            yaml_emitter_emit_block_mapping_value(emitter, event, false)
        }
        YamlEmitEndState => yaml_emitter_set_emitter_error(
            emitter,
            b"expected nothing after STREAM-END\0" as *const u8
                as *const libc::c_char,
        ),
    }
}

/// Emits a stream start event.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The event must be a YamlStreamStartEvent
/// - The encoding fields must contain valid encoding values
/// - The emitter state must be ready to start a new stream
unsafe fn yaml_emitter_emit_stream_start(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
) -> Success {
    (*emitter).open_ended = 0;
    if (*event).type_ == YamlStreamStartEvent {
        if (*emitter).encoding == YamlAnyEncoding {
            (*emitter).encoding = (*event).data.stream_start.encoding;
        }
        if (*emitter).encoding == YamlAnyEncoding {
            (*emitter).encoding = YamlUtf8Encoding;
        }
        if (*emitter).best_indent < 2 || (*emitter).best_indent > 9 {
            (*emitter).best_indent = 2;
        }
        if (*emitter).best_width >= 0
            && (*emitter).best_width
                <= (*emitter).best_indent.force_mul(2)
        {
            (*emitter).best_width = 80;
        }
        if (*emitter).best_width < 0 {
            (*emitter).best_width = libc::c_int::MAX;
        }
        if (*emitter).line_break == YamlAnyBreak {
            (*emitter).line_break = YamlLnBreak;
        }
        (*emitter).indent = -1;
        (*emitter).line = 0;
        (*emitter).column = 0;
        (*emitter).whitespace = true;
        (*emitter).indention = true;
        if (*emitter).encoding != YamlUtf8Encoding
            && yaml_emitter_write_bom(emitter).fail
        {
            return FAIL;
        }
        (*emitter).state = YamlEmitFirstDocumentStartState;
        return OK;
    }
    yaml_emitter_set_emitter_error(
        emitter,
        b"expected STREAM-START\0" as *const u8 as *const libc::c_char,
    )
}

/// Emits a document start event.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The event must be a YamlDocumentStartEvent or YamlStreamEndEvent
/// - All version and tag directive pointers in the event must be valid if present
/// - The tag directives stack must be properly initialized
unsafe fn yaml_emitter_emit_document_start(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
    first: bool,
) -> Success {
    if (*event).type_ == YamlDocumentStartEvent {
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
        let mut tag_directive: *mut YamlTagDirectiveT;
        let mut implicit;
        if !(*event).data.document_start.version_directive.is_null()
            && yaml_emitter_analyze_version_directive(
                emitter,
                *(*event).data.document_start.version_directive,
            )
            .fail
        {
            return FAIL;
        }
        tag_directive =
            (*event).data.document_start.tag_directives.start;
        while tag_directive
            != (*event).data.document_start.tag_directives.end
        {
            if yaml_emitter_analyze_tag_directive(
                emitter,
                *tag_directive,
            )
            .fail
            {
                return FAIL;
            }
            if yaml_emitter_append_tag_directive(
                emitter,
                *tag_directive,
                false,
            )
            .fail
            {
                return FAIL;
            }
            tag_directive = tag_directive.wrapping_offset(1);
        }
        tag_directive = default_tag_directives.as_mut_ptr();
        while !(*tag_directive).handle.is_null() {
            if yaml_emitter_append_tag_directive(
                emitter,
                *tag_directive,
                true,
            )
            .fail
            {
                return FAIL;
            }
            tag_directive = tag_directive.wrapping_offset(1);
        }
        implicit = (*event).data.document_start.implicit;
        if !first || (*emitter).canonical {
            implicit = false;
        }
        if (!(*event).data.document_start.version_directive.is_null()
            || (*event).data.document_start.tag_directives.start
                != (*event).data.document_start.tag_directives.end)
            && (*emitter).open_ended != 0
        {
            if yaml_emitter_write_indicator(
                emitter,
                b"...\0" as *const u8 as *const libc::c_char,
                true,
                false,
                false,
            )
            .fail
            {
                return FAIL;
            }
            if yaml_emitter_write_indent(emitter).fail {
                return FAIL;
            }
        }
        (*emitter).open_ended = 0;
        if !(*event).data.document_start.version_directive.is_null() {
            implicit = false;
            if yaml_emitter_write_indicator(
                emitter,
                b"%YAML\0" as *const u8 as *const libc::c_char,
                true,
                false,
                false,
            )
            .fail
            {
                return FAIL;
            }
            if (*(*event).data.document_start.version_directive).minor
                == 1
            {
                if yaml_emitter_write_indicator(
                    emitter,
                    b"1.1\0" as *const u8 as *const libc::c_char,
                    true,
                    false,
                    false,
                )
                .fail
                {
                    return FAIL;
                }
            } else if yaml_emitter_write_indicator(
                emitter,
                b"1.2\0" as *const u8 as *const libc::c_char,
                true,
                false,
                false,
            )
            .fail
            {
                return FAIL;
            }
            if yaml_emitter_write_indent(emitter).fail {
                return FAIL;
            }
        }
        if (*event).data.document_start.tag_directives.start
            != (*event).data.document_start.tag_directives.end
        {
            implicit = false;
            tag_directive =
                (*event).data.document_start.tag_directives.start;
            while tag_directive
                != (*event).data.document_start.tag_directives.end
            {
                if yaml_emitter_write_indicator(
                    emitter,
                    b"%TAG\0" as *const u8 as *const libc::c_char,
                    true,
                    false,
                    false,
                )
                .fail
                {
                    return FAIL;
                }
                if yaml_emitter_write_tag_handle(
                    emitter,
                    (*tag_directive).handle,
                    strlen(
                        (*tag_directive).handle as *mut libc::c_char,
                    ),
                )
                .fail
                {
                    return FAIL;
                }
                if yaml_emitter_write_tag_content(
                    emitter,
                    (*tag_directive).prefix,
                    strlen(
                        (*tag_directive).prefix as *mut libc::c_char,
                    ),
                    true,
                )
                .fail
                {
                    return FAIL;
                }
                if yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
                tag_directive = tag_directive.wrapping_offset(1);
            }
        }
        if yaml_emitter_check_empty_document(emitter) {
            implicit = false;
        }
        if !implicit {
            if yaml_emitter_write_indent(emitter).fail {
                return FAIL;
            }
            if yaml_emitter_write_indicator(
                emitter,
                b"---\0" as *const u8 as *const libc::c_char,
                true,
                false,
                false,
            )
            .fail
            {
                return FAIL;
            }
            if (*emitter).canonical
                && yaml_emitter_write_indent(emitter).fail
            {
                return FAIL;
            }
        }
        (*emitter).state = YamlEmitDocumentContentState;
        (*emitter).open_ended = 0;
        return OK;
    } else if (*event).type_ == YamlStreamEndEvent {
        if (*emitter).open_ended == 2 {
            if yaml_emitter_write_indicator(
                emitter,
                b"...\0" as *const u8 as *const libc::c_char,
                true,
                false,
                false,
            )
            .fail
            {
                return FAIL;
            }
            (*emitter).open_ended = 0;
            if yaml_emitter_write_indent(emitter).fail {
                return FAIL;
            }
        }
        if yaml_emitter_flush(emitter).fail {
            return FAIL;
        }
        (*emitter).state = YamlEmitEndState;
        return OK;
    }
    yaml_emitter_set_emitter_error(
        emitter,
        b"expected DOCUMENT-START or STREAM-END\0" as *const u8
            as *const libc::c_char,
    )
}

/// Emits document content.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The states stack must have capacity for the new state
/// - The emitter must be in a valid state for document content
/// - All event data must be properly initialized
unsafe fn yaml_emitter_emit_document_content(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
) -> Success {
    PUSH!((*emitter).states, YamlEmitDocumentEndState);
    yaml_emitter_emit_node(emitter, event, true, false, false, false)
}

/// Emits a document end event.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The event must be a YamlDocumentEndEvent
/// - The tag directives stack must be properly initialized for cleanup
/// - The emitter state must be valid for ending a document
unsafe fn yaml_emitter_emit_document_end(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
) -> Success {
    if (*event).type_ == YamlDocumentEndEvent {
        if yaml_emitter_write_indent(emitter).fail {
            return FAIL;
        }
        if !(*event).data.document_end.implicit {
            if yaml_emitter_write_indicator(
                emitter,
                b"...\0" as *const u8 as *const libc::c_char,
                true,
                false,
                false,
            )
            .fail
            {
                return FAIL;
            }
            (*emitter).open_ended = 0;
            if yaml_emitter_write_indent(emitter).fail {
                return FAIL;
            }
        } else if (*emitter).open_ended == 0 {
            (*emitter).open_ended = 1;
        }
        if yaml_emitter_flush(emitter).fail {
            return FAIL;
        }
        (*emitter).state = YamlEmitDocumentStartState;
        while !STACK_EMPTY!((*emitter).tag_directives) {
            let tag_directive = POP!((*emitter).tag_directives);
            yaml_free(tag_directive.handle as *mut libc::c_void);
            yaml_free(tag_directive.prefix as *mut libc::c_void);
        }
        return OK;
    }
    yaml_emitter_set_emitter_error(
        emitter,
        b"expected DOCUMENT-END\0" as *const u8 as *const libc::c_char,
    )
}

/// Emits a flow sequence item.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The flow_level counter must not overflow when modified
/// - The indents stack must be properly initialized
/// - The states stack must be properly initialized
unsafe fn yaml_emitter_emit_flow_sequence_item(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
    first: bool,
) -> Success {
    if first {
        if yaml_emitter_write_indicator(
            emitter,
            b"[\0" as *const u8 as *const libc::c_char,
            true,
            true,
            false,
        )
        .fail
        {
            return FAIL;
        }
        yaml_emitter_increase_indent(emitter, true, false);
        let fresh12 = addr_of_mut!((*emitter).flow_level);
        *fresh12 += 1;
    }
    if (*event).type_ == YamlSequenceEndEvent {
        let fresh13 = addr_of_mut!((*emitter).flow_level);
        *fresh13 -= 1;
        (*emitter).indent = POP!((*emitter).indents);
        if (*emitter).canonical && !first {
            if yaml_emitter_write_indicator(
                emitter,
                b",\0" as *const u8 as *const libc::c_char,
                false,
                false,
                false,
            )
            .fail
            {
                return FAIL;
            }
            if yaml_emitter_write_indent(emitter).fail {
                return FAIL;
            }
        }
        if yaml_emitter_write_indicator(
            emitter,
            b"]\0" as *const u8 as *const libc::c_char,
            false,
            false,
            false,
        )
        .fail
        {
            return FAIL;
        }
        (*emitter).state = POP!((*emitter).states);
        return OK;
    }
    if !first
        && yaml_emitter_write_indicator(
            emitter,
            b",\0" as *const u8 as *const libc::c_char,
            false,
            false,
            false,
        )
        .fail
    {
        return FAIL;
    }
    if ((*emitter).canonical
        || (*emitter).column > (*emitter).best_width)
        && yaml_emitter_write_indent(emitter).fail
    {
        return FAIL;
    }
    PUSH!((*emitter).states, YamlEmitFlowSequenceItemState);
    yaml_emitter_emit_node(emitter, event, false, true, false, false)
}

/// Emits a flow mapping key.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The flow_level counter must not overflow when modified
/// - The indents stack must be properly initialized and have capacity
/// - The states stack must be properly initialized and have capacity
unsafe fn yaml_emitter_emit_flow_mapping_key(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
    first: bool,
) -> Success {
    if first {
        if yaml_emitter_write_indicator(
            emitter,
            b"{\0" as *const u8 as *const libc::c_char,
            true,
            true,
            false,
        )
        .fail
        {
            return FAIL;
        }
        yaml_emitter_increase_indent(emitter, true, false);
        let fresh18 = addr_of_mut!((*emitter).flow_level);
        *fresh18 += 1;
    }
    if (*event).type_ == YamlMappingEndEvent {
        if STACK_EMPTY!((*emitter).indents) {
            return FAIL;
        }
        let fresh19 = addr_of_mut!((*emitter).flow_level);
        *fresh19 -= 1;
        (*emitter).indent = POP!((*emitter).indents);
        if (*emitter).canonical && !first {
            if yaml_emitter_write_indicator(
                emitter,
                b",\0" as *const u8 as *const libc::c_char,
                false,
                false,
                false,
            )
            .fail
            {
                return FAIL;
            }
            if yaml_emitter_write_indent(emitter).fail {
                return FAIL;
            }
        }
        if yaml_emitter_write_indicator(
            emitter,
            b"}\0" as *const u8 as *const libc::c_char,
            false,
            false,
            false,
        )
        .fail
        {
            return FAIL;
        }
        (*emitter).state = POP!((*emitter).states);
        return OK;
    }
    if !first
        && yaml_emitter_write_indicator(
            emitter,
            b",\0" as *const u8 as *const libc::c_char,
            false,
            false,
            false,
        )
        .fail
    {
        return FAIL;
    }
    if ((*emitter).canonical
        || (*emitter).column > (*emitter).best_width)
        && yaml_emitter_write_indent(emitter).fail
    {
        return FAIL;
    }
    if !(*emitter).canonical && yaml_emitter_check_simple_key(emitter) {
        PUSH!((*emitter).states, YamlEmitFlowMappingSimpleValueState);
        yaml_emitter_emit_node(emitter, event, false, false, true, true)
    } else {
        if yaml_emitter_write_indicator(
            emitter,
            b"?\0" as *const u8 as *const libc::c_char,
            true,
            false,
            false,
        )
        .fail
        {
            return FAIL;
        }
        PUSH!((*emitter).states, YamlEmitFlowMappingValueState);
        yaml_emitter_emit_node(
            emitter, event, false, false, true, false,
        )
    }
}

/// Emits a flow mapping value.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The states stack must be properly initialized and have capacity
/// - The emitter state must be valid for a flow mapping value
/// - The column and indentation state must be valid
unsafe fn yaml_emitter_emit_flow_mapping_value(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
    simple: bool,
) -> Success {
    if simple {
        if yaml_emitter_write_indicator(
            emitter,
            b":\0" as *const u8 as *const libc::c_char,
            false,
            false,
            false,
        )
        .fail
        {
            return FAIL;
        }
    } else {
        if ((*emitter).canonical
            || (*emitter).column > (*emitter).best_width)
            && yaml_emitter_write_indent(emitter).fail
        {
            return FAIL;
        }
        if yaml_emitter_write_indicator(
            emitter,
            b":\0" as *const u8 as *const libc::c_char,
            true,
            false,
            false,
        )
        .fail
        {
            return FAIL;
        }
    }
    PUSH!((*emitter).states, YamlEmitFlowMappingKeyState);
    yaml_emitter_emit_node(emitter, event, false, false, true, false)
}

/// Emits a block sequence item.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The indents stack must be properly initialized and have capacity
/// - The states stack must be properly initialized and have capacity
/// - The mapping_context and indention flags must be valid
unsafe fn yaml_emitter_emit_block_sequence_item(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
    first: bool,
) -> Success {
    if first {
        yaml_emitter_increase_indent(
            emitter,
            false,
            (*emitter).mapping_context && !(*emitter).indention,
        );
    }
    if (*event).type_ == YamlSequenceEndEvent {
        (*emitter).indent = POP!((*emitter).indents);
        (*emitter).state = POP!((*emitter).states);
        return OK;
    }
    if yaml_emitter_write_indent(emitter).fail {
        return FAIL;
    }
    if yaml_emitter_write_indicator(
        emitter,
        b"-\0" as *const u8 as *const libc::c_char,
        true,
        false,
        true,
    )
    .fail
    {
        return FAIL;
    }
    PUSH!((*emitter).states, YamlEmitBlockSequenceItemState);
    yaml_emitter_emit_node(emitter, event, false, true, false, false)
}

/// Emits a block mapping key.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The indents stack must be properly initialized and have capacity
/// - The states stack must be properly initialized and have capacity
/// - All event data must be properly initialized
unsafe fn yaml_emitter_emit_block_mapping_key(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
    first: bool,
) -> Success {
    if first {
        yaml_emitter_increase_indent(emitter, false, false);
    }
    if (*event).type_ == YamlMappingEndEvent {
        (*emitter).indent = POP!((*emitter).indents);
        (*emitter).state = POP!((*emitter).states);
        return OK;
    }
    if yaml_emitter_write_indent(emitter).fail {
        return FAIL;
    }
    if yaml_emitter_check_simple_key(emitter) {
        PUSH!((*emitter).states, YamlEmitBlockMappingSimpleValueState);
        yaml_emitter_emit_node(emitter, event, false, false, true, true)
    } else {
        if yaml_emitter_write_indicator(
            emitter,
            b"?\0" as *const u8 as *const libc::c_char,
            true,
            false,
            true,
        )
        .fail
        {
            return FAIL;
        }
        PUSH!((*emitter).states, YamlEmitBlockMappingValueState);
        yaml_emitter_emit_node(
            emitter, event, false, false, true, false,
        )
    }
}

/// Emits a block mapping value.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The states stack must be properly initialized and have capacity
/// - The emitter state must be valid for a block mapping value
/// - The indentation state must be properly tracked
unsafe fn yaml_emitter_emit_block_mapping_value(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
    simple: bool,
) -> Success {
    if simple {
        if yaml_emitter_write_indicator(
            emitter,
            b":\0" as *const u8 as *const libc::c_char,
            false,
            false,
            false,
        )
        .fail
        {
            return FAIL;
        }
    } else {
        if yaml_emitter_write_indent(emitter).fail {
            return FAIL;
        }
        if yaml_emitter_write_indicator(
            emitter,
            b":\0" as *const u8 as *const libc::c_char,
            true,
            false,
            true,
        )
        .fail
        {
            return FAIL;
        }
    }
    PUSH!((*emitter).states, YamlEmitBlockMappingKeyState);
    yaml_emitter_emit_node(emitter, event, false, false, true, false)
}

/// Emits a node with proper context.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - All context flags must be valid boolean values
/// - The event type must be one of: alias, scalar, sequence start, or mapping start
/// - All event data must be properly initialized for the given event type
unsafe fn yaml_emitter_emit_node(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
    root: bool,
    sequence: bool,
    mapping: bool,
    simple_key: bool,
) -> Success {
    (*emitter).root_context = root;
    (*emitter).sequence_context = sequence;
    (*emitter).mapping_context = mapping;
    (*emitter).simple_key_context = simple_key;
    match (*event).type_ {
        YamlAliasEvent => yaml_emitter_emit_alias(emitter, event),
        YamlScalarEvent => yaml_emitter_emit_scalar(emitter, event),
        YamlSequenceStartEvent => yaml_emitter_emit_sequence_start(emitter, event),
        YamlMappingStartEvent => yaml_emitter_emit_mapping_start(emitter, event),
        _ => yaml_emitter_set_emitter_error(
            emitter,
            b"expected SCALAR, SEQUENCE-START, MAPPING-START, or ALIAS\0" as *const u8
                as *const libc::c_char,
        ),
    }
}

/// Emits an alias node.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The anchor data must be properly initialized
/// - The simple_key_context flag must be valid
/// - The states stack must be properly initialized
unsafe fn yaml_emitter_emit_alias(
    emitter: *mut YamlEmitterT,
    _event: *mut YamlEventT,
) -> Success {
    if yaml_emitter_process_anchor(emitter).fail {
        return FAIL;
    }
    if (*emitter).simple_key_context && put(emitter, b' ').fail {
        return FAIL;
    }
    (*emitter).state = POP!((*emitter).states);
    OK
}

/// Emits a scalar node.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The event must be a YamlScalarEvent with valid scalar data
/// - The indents stack must be properly initialized
/// - The states stack must be properly initialized
unsafe fn yaml_emitter_emit_scalar(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
) -> Success {
    if yaml_emitter_select_scalar_style(emitter, event).fail {
        return FAIL;
    }
    if yaml_emitter_process_anchor(emitter).fail {
        return FAIL;
    }
    if yaml_emitter_process_tag(emitter).fail {
        return FAIL;
    }
    yaml_emitter_increase_indent(emitter, true, false);
    if yaml_emitter_process_scalar(emitter).fail {
        return FAIL;
    }
    (*emitter).indent = POP!((*emitter).indents);
    (*emitter).state = POP!((*emitter).states);
    OK
}

/// Emits a sequence start node.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The event must be a YamlSequenceStartEvent
/// - The flow_level and canonical flags must be valid
/// - The sequence start style must be valid
unsafe fn yaml_emitter_emit_sequence_start(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
) -> Success {
    if yaml_emitter_process_anchor(emitter).fail {
        return FAIL;
    }
    if yaml_emitter_process_tag(emitter).fail {
        return FAIL;
    }
    if (*emitter).flow_level != 0
        || (*emitter).canonical
        || (*event).data.sequence_start.style == YamlFlowSequenceStyle
        || yaml_emitter_check_empty_sequence(emitter)
    {
        (*emitter).state = YamlEmitFlowSequenceFirstItemState;
    } else {
        (*emitter).state = YamlEmitBlockSequenceFirstItemState;
    }
    OK
}

/// Emits a mapping start node.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The event must be a YamlMappingStartEvent
/// - The flow_level and canonical flags must be valid
/// - The mapping start style must be valid
unsafe fn yaml_emitter_emit_mapping_start(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
) -> Success {
    if yaml_emitter_process_anchor(emitter).fail {
        return FAIL;
    }
    if yaml_emitter_process_tag(emitter).fail {
        return FAIL;
    }
    if (*emitter).flow_level != 0
        || (*emitter).canonical
        || (*event).data.mapping_start.style == YamlFlowMappingStyle
        || yaml_emitter_check_empty_mapping(emitter)
    {
        (*emitter).state = YamlEmitFlowMappingFirstKeyState;
    } else {
        (*emitter).state = YamlEmitBlockMappingFirstKeyState;
    }
    OK
}

/// Checks if a document is empty.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The events queue must be properly initialized
unsafe fn yaml_emitter_check_empty_document(
    _emitter: *mut YamlEmitterT,
) -> bool {
    false
}

/// Checks if a sequence is empty.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The events queue must be properly initialized with valid head and tail pointers
/// - At least two events must be available in the queue if the sequence is empty
unsafe fn yaml_emitter_check_empty_sequence(
    emitter: *mut YamlEmitterT,
) -> bool {
    if ((*emitter).events.tail.c_offset_from((*emitter).events.head)
        as libc::c_long)
        < 2_i64
    {
        return false;
    }
    (*(*emitter).events.head).type_ == YamlSequenceStartEvent
        && (*(*emitter).events.head.wrapping_offset(1_isize)).type_
            == YamlSequenceEndEvent
}

/// Checks if a mapping is empty.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The events queue must be properly initialized with valid head and tail pointers
/// - At least two events must be available in the queue if the mapping is empty
unsafe fn yaml_emitter_check_empty_mapping(
    emitter: *mut YamlEmitterT,
) -> bool {
    if ((*emitter).events.tail.c_offset_from((*emitter).events.head)
        as libc::c_long)
        < 2_i64
    {
        return false;
    }
    (*(*emitter).events.head).type_ == YamlMappingStartEvent
        && (*(*emitter).events.head.wrapping_offset(1_isize)).type_
            == YamlMappingEndEvent
}

/// Checks if a key can be written as a simple key.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The events queue must be properly initialized with a valid head pointer
/// - All event data in the head event must be properly initialized
/// - The scalar_data fields must be valid for scalar events
unsafe fn yaml_emitter_check_simple_key(
    emitter: *mut YamlEmitterT,
) -> bool {
    // Validate that the emitter and its events queue are properly initialized.
    if emitter.is_null() || (*emitter).events.head.is_null() {
        return false; // Invalid emitter or empty event queue.
    }

    let event = (*emitter).events.head;

    // Ensure the event pointer itself is valid.
    if event.is_null() {
        return false; // Event pointer is null.
    }

    // Check that the event type is one of the supported types.
    if !matches!(
        (*event).type_,
        YamlAliasEvent
            | YamlScalarEvent
            | YamlSequenceStartEvent
            | YamlMappingStartEvent
    ) {
        return false; // Unsupported or uninitialized event type.
    }

    let mut length: size_t = 0;

    // Determine the length based on the event type.
    match (*event).type_ {
        YamlAliasEvent => {
            // Add the length of the alias anchor.
            length =
                length.force_add((*emitter).anchor_data.anchor_length);
        }
        YamlScalarEvent => {
            // For scalar events, check multiline restrictions and calculate total length.
            if (*emitter).scalar_data.multiline {
                return false; // Multiline scalars are not simple keys.
            }
            length = length
                .force_add((*emitter).anchor_data.anchor_length)
                .force_add((*emitter).tag_data.handle_length)
                .force_add((*emitter).tag_data.suffix_length)
                .force_add((*emitter).scalar_data.length);
        }
        YamlSequenceStartEvent => {
            // For sequence start, ensure it's empty and calculate its length.
            if !yaml_emitter_check_empty_sequence(emitter) {
                return false; // Non-empty sequences are not simple keys.
            }
            length = length
                .force_add((*emitter).anchor_data.anchor_length)
                .force_add((*emitter).tag_data.handle_length)
                .force_add((*emitter).tag_data.suffix_length);
        }
        YamlMappingStartEvent => {
            // For mapping start, ensure it's empty and calculate its length.
            if !yaml_emitter_check_empty_mapping(emitter) {
                return false; // Non-empty mappings are not simple keys.
            }
            length = length
                .force_add((*emitter).anchor_data.anchor_length)
                .force_add((*emitter).tag_data.handle_length)
                .force_add((*emitter).tag_data.suffix_length);
        }
        _ => return false, // Unsupported event type (shouldn't reach here due to earlier match).
    }

    // Check if the calculated length is within the maximum allowed for simple keys.
    length <= MAX_SIMPLE_KEY_LENGTH
}

/// Selects appropriate style for a scalar value.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - The event must be a YamlScalarEvent
/// - The scalar_data fields must be properly initialized
/// - The tag_data fields must be properly initialized
unsafe fn yaml_emitter_select_scalar_style(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
) -> Success {
    let mut style: YamlScalarStyleT = (*event).data.scalar.style;
    let no_tag = (*emitter).tag_data.handle.is_null()
        && (*emitter).tag_data.suffix.is_null();
    if no_tag
        && !(*event).data.scalar.plain_implicit
        && !(*event).data.scalar.quoted_implicit
    {
        return yaml_emitter_set_emitter_error(
            emitter,
            b"neither tag nor implicit flags are specified\0"
                as *const u8 as *const libc::c_char,
        );
    }
    if style == YamlAnyScalarStyle {
        style = YamlPlainScalarStyle;
    }
    if (*emitter).canonical {
        style = YamlDoubleQuotedScalarStyle;
    }
    if (*emitter).simple_key_context && (*emitter).scalar_data.multiline
    {
        style = YamlDoubleQuotedScalarStyle;
    }
    if style == YamlPlainScalarStyle {
        if (*emitter).flow_level != 0
            && !(*emitter).scalar_data.flow_plain_allowed
            || (*emitter).flow_level == 0
                && !(*emitter).scalar_data.block_plain_allowed
        {
            style = YamlSingleQuotedScalarStyle;
        }
        if (*emitter).scalar_data.length == 0
            && ((*emitter).flow_level != 0
                || (*emitter).simple_key_context)
        {
            style = YamlSingleQuotedScalarStyle;
        }
        if no_tag && !(*event).data.scalar.plain_implicit {
            style = YamlSingleQuotedScalarStyle;
        }
    }
    if style == YamlSingleQuotedScalarStyle
        && !(*emitter).scalar_data.single_quoted_allowed
    {
        style = YamlDoubleQuotedScalarStyle;
    }
    if (style == YamlLiteralScalarStyle
        || style == YamlFoldedScalarStyle)
        && (!(*emitter).scalar_data.block_allowed
            || (*emitter).flow_level != 0
            || (*emitter).simple_key_context)
    {
        style = YamlDoubleQuotedScalarStyle;
    }
    if no_tag
        && !(*event).data.scalar.quoted_implicit
        && style != YamlPlainScalarStyle
    {
        let fresh46 = addr_of_mut!((*emitter).tag_data.handle);
        *fresh46 = b"!\0" as *const u8 as *const libc::c_char
            as *mut yaml_char_t;
        (*emitter).tag_data.handle_length = 1_u64;
    }
    (*emitter).scalar_data.style = style;
    OK
}

/// Processes an anchor or alias.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The anchor_data fields must be properly initialized
/// - If anchor_data.anchor is non-null, it must point to a valid null-terminated string
/// - The buffer must have enough space for the anchor formatting
unsafe fn yaml_emitter_process_anchor(
    emitter: *mut YamlEmitterT,
) -> Success {
    if (*emitter).anchor_data.anchor.is_null() {
        return OK;
    }
    if yaml_emitter_write_indicator(
        emitter,
        if (*emitter).anchor_data.alias {
            b"*\0" as *const u8 as *const libc::c_char
        } else {
            b"&\0" as *const u8 as *const libc::c_char
        },
        true,
        false,
        false,
    )
    .fail
    {
        return FAIL;
    }
    yaml_emitter_write_anchor(
        emitter,
        (*emitter).anchor_data.anchor,
        (*emitter).anchor_data.anchor_length,
    )
}

/// Processes a tag.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The tag_data fields must be properly initialized
/// - If tag_data handle/suffix are non-null, they must point to valid null-terminated strings
/// - The buffer must have enough space for the tag formatting
unsafe fn yaml_emitter_process_tag(
    emitter: *mut YamlEmitterT,
) -> Success {
    if (*emitter).tag_data.handle.is_null()
        && (*emitter).tag_data.suffix.is_null()
    {
        return OK;
    }
    if !(*emitter).tag_data.handle.is_null() {
        if yaml_emitter_write_tag_handle(
            emitter,
            (*emitter).tag_data.handle,
            (*emitter).tag_data.handle_length,
        )
        .fail
        {
            return FAIL;
        }
        if !(*emitter).tag_data.suffix.is_null()
            && yaml_emitter_write_tag_content(
                emitter,
                (*emitter).tag_data.suffix,
                (*emitter).tag_data.suffix_length,
                false,
            )
            .fail
        {
            return FAIL;
        }
    } else {
        if yaml_emitter_write_indicator(
            emitter,
            b"!<\0" as *const u8 as *const libc::c_char,
            true,
            false,
            false,
        )
        .fail
        {
            return FAIL;
        }
        if yaml_emitter_write_tag_content(
            emitter,
            (*emitter).tag_data.suffix,
            (*emitter).tag_data.suffix_length,
            false,
        )
        .fail
        {
            return FAIL;
        }
        if yaml_emitter_write_indicator(
            emitter,
            b">\0" as *const u8 as *const libc::c_char,
            false,
            false,
            false,
        )
        .fail
        {
            return FAIL;
        }
    }
    OK
}

/// Processes a scalar value for emission.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `value` must be a valid pointer to a scalar value buffer
/// - `length` must accurately reflect the size of the value buffer
/// - The scalar_data fields in emitter must be properly initialized
unsafe fn yaml_emitter_process_scalar(
    emitter: *mut YamlEmitterT,
) -> Success {
    match (*emitter).scalar_data.style {
        YamlPlainScalarStyle => {
            return yaml_emitter_write_plain_scalar(
                emitter,
                (*emitter).scalar_data.value,
                (*emitter).scalar_data.length,
                !(*emitter).simple_key_context,
            );
        }
        YamlSingleQuotedScalarStyle => {
            return yaml_emitter_write_single_quoted_scalar(
                emitter,
                (*emitter).scalar_data.value,
                (*emitter).scalar_data.length,
                !(*emitter).simple_key_context,
            );
        }
        YamlDoubleQuotedScalarStyle => {
            return yaml_emitter_write_double_quoted_scalar(
                emitter,
                (*emitter).scalar_data.value,
                (*emitter).scalar_data.length,
                !(*emitter).simple_key_context,
            );
        }
        YamlLiteralScalarStyle => {
            return yaml_emitter_write_literal_scalar(
                emitter,
                (*emitter).scalar_data.value,
                (*emitter).scalar_data.length,
            );
        }
        YamlFoldedScalarStyle => {
            return yaml_emitter_write_folded_scalar(
                emitter,
                (*emitter).scalar_data.value,
                (*emitter).scalar_data.length,
            );
        }
        _ => {}
    }
    FAIL
}

/// Analyzes a version directive.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The version directive must contain valid major and minor version numbers
unsafe fn yaml_emitter_analyze_version_directive(
    emitter: *mut YamlEmitterT,
    version_directive: YamlVersionDirectiveT,
) -> Success {
    if version_directive.major != 1
        || version_directive.minor != 1 && version_directive.minor != 2
    {
        return yaml_emitter_set_emitter_error(
            emitter,
            b"incompatible %YAML directive\0" as *const u8
                as *const libc::c_char,
        );
    }
    OK
}

/// Analyzes a tag directive for validity.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The tag directive's handle and prefix must be valid null-terminated strings
/// - The strings must remain valid for the duration of the analysis
unsafe fn yaml_emitter_analyze_tag_directive(
    emitter: *mut YamlEmitterT,
    tag_directive: YamlTagDirectiveT,
) -> Success {
    let handle_length: size_t =
        strlen(tag_directive.handle as *mut libc::c_char);
    let prefix_length: size_t =
        strlen(tag_directive.prefix as *mut libc::c_char);
    let mut handle =
        STRING_ASSIGN!(tag_directive.handle, handle_length);
    let prefix = STRING_ASSIGN!(tag_directive.prefix, prefix_length);
    if handle.start == handle.end {
        return yaml_emitter_set_emitter_error(
            emitter,
            b"tag handle must not be empty\0" as *const u8
                as *const libc::c_char,
        );
    }
    if *handle.start != b'!' {
        return yaml_emitter_set_emitter_error(
            emitter,
            b"tag handle must start with '!'\0" as *const u8
                as *const libc::c_char,
        );
    }
    if *handle.end.wrapping_offset(-1_isize) != b'!' {
        return yaml_emitter_set_emitter_error(
            emitter,
            b"tag handle must end with '!'\0" as *const u8
                as *const libc::c_char,
        );
    }
    handle.pointer = handle.pointer.wrapping_offset(1);
    while handle.pointer < handle.end.wrapping_offset(-1_isize) {
        if !IS_ALPHA!(handle) {
            return yaml_emitter_set_emitter_error(
                emitter,
                b"tag handle must contain alphanumerical characters only\0" as *const u8
                    as *const libc::c_char,
            );
        }
        MOVE!(handle);
    }
    if prefix.start == prefix.end {
        return yaml_emitter_set_emitter_error(
            emitter,
            b"tag prefix must not be empty\0" as *const u8
                as *const libc::c_char,
        );
    }
    OK
}

pub(crate) unsafe fn yaml_emitter_analyze_anchor(
    emitter: *mut YamlEmitterT,
    anchor: *mut yaml_char_t,
    alias: bool,
) -> Success {
    let anchor_length: size_t = strlen(anchor as *mut libc::c_char);
    let mut string = STRING_ASSIGN!(anchor, anchor_length);
    if string.start == string.end {
        return yaml_emitter_set_emitter_error(
            emitter,
            if alias {
                b"alias value must not be empty\0" as *const u8
                    as *const libc::c_char
            } else {
                b"anchor value must not be empty\0" as *const u8
                    as *const libc::c_char
            },
        );
    }
    while string.pointer != string.end {
        if !IS_ALPHA!(string) {
            return yaml_emitter_set_emitter_error(
                emitter,
                if alias {
                    b"alias value must contain alphanumerical characters only\0" as *const u8
                        as *const libc::c_char
                } else {
                    b"anchor value must contain alphanumerical characters only\0" as *const u8
                        as *const libc::c_char
                },
            );
        }
        MOVE!(string);
    }
    let fresh47 = addr_of_mut!((*emitter).anchor_data.anchor);
    *fresh47 = string.start;
    (*emitter).anchor_data.anchor_length =
        string.end.c_offset_from(string.start) as size_t;
    (*emitter).anchor_data.alias = alias;
    OK
}

pub(crate) unsafe fn yaml_emitter_analyze_tag(
    emitter: *mut YamlEmitterT,
    tag: *mut yaml_char_t,
) -> Success {
    let mut tag_directive: *mut YamlTagDirectiveT;
    let tag_length: size_t = strlen(tag as *mut libc::c_char);
    let string = STRING_ASSIGN!(tag, tag_length);
    if string.start == string.end {
        return yaml_emitter_set_emitter_error(
            emitter,
            b"tag value must not be empty\0" as *const u8
                as *const libc::c_char,
        );
    }
    tag_directive = (*emitter).tag_directives.start;
    while tag_directive != (*emitter).tag_directives.top {
        let prefix_length: size_t =
            strlen((*tag_directive).prefix as *mut libc::c_char);
        if prefix_length
            < string.end.c_offset_from(string.start) as size_t
            && strncmp(
                (*tag_directive).prefix as *mut libc::c_char,
                string.start as *mut libc::c_char,
                prefix_length,
            ) == 0
        {
            let fresh48 = addr_of_mut!((*emitter).tag_data.handle);
            *fresh48 = (*tag_directive).handle;
            (*emitter).tag_data.handle_length =
                strlen((*tag_directive).handle as *mut libc::c_char);
            let fresh49 = addr_of_mut!((*emitter).tag_data.suffix);
            *fresh49 =
                string.start.wrapping_offset(prefix_length as isize);
            (*emitter).tag_data.suffix_length =
                (string.end.c_offset_from(string.start)
                    as libc::c_ulong)
                    .wrapping_sub(prefix_length);
            return OK;
        }
        tag_directive = tag_directive.wrapping_offset(1);
    }
    let fresh50 = addr_of_mut!((*emitter).tag_data.suffix);
    *fresh50 = string.start;
    (*emitter).tag_data.suffix_length =
        string.end.c_offset_from(string.start) as size_t;
    OK
}

/// Analyzes a scalar value's properties.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `value` must be a valid pointer to a scalar value buffer
/// - `length` must accurately reflect the size of the value buffer
/// - The scalar_data fields in emitter must be properly initialized
unsafe fn yaml_emitter_analyze_scalar(
    emitter: *mut YamlEmitterT,
    value: *mut yaml_char_t,
    length: size_t,
) -> Success {
    let mut block_indicators = false;
    let mut flow_indicators = false;
    let mut line_breaks = false;
    let mut special_characters = false;
    let mut leading_space = false;
    let mut leading_break = false;
    let mut trailing_space = false;
    let mut trailing_break = false;
    let mut break_space = false;
    let mut space_break = false;
    let mut preceded_by_whitespace;
    let mut followed_by_whitespace;
    let mut previous_space = false;
    let mut previous_break = false;
    let mut string = STRING_ASSIGN!(value, length);
    let fresh51 = addr_of_mut!((*emitter).scalar_data.value);
    *fresh51 = value;
    (*emitter).scalar_data.length = length;
    if string.start == string.end {
        (*emitter).scalar_data.multiline = false;
        (*emitter).scalar_data.flow_plain_allowed = false;
        (*emitter).scalar_data.block_plain_allowed = true;
        (*emitter).scalar_data.single_quoted_allowed = true;
        (*emitter).scalar_data.block_allowed = false;
        return OK;
    }
    if CHECK_AT!(string, b'-', 0)
        && CHECK_AT!(string, b'-', 1)
        && CHECK_AT!(string, b'-', 2)
        || CHECK_AT!(string, b'.', 0)
            && CHECK_AT!(string, b'.', 1)
            && CHECK_AT!(string, b'.', 2)
    {
        block_indicators = true;
        flow_indicators = true;
    }
    preceded_by_whitespace = true;
    followed_by_whitespace = IS_BLANKZ_AT!(string, WIDTH!(string));
    while string.pointer != string.end {
        if string.start == string.pointer {
            if CHECK!(string, b'#')
                || CHECK!(string, b',')
                || CHECK!(string, b'[')
                || CHECK!(string, b']')
                || CHECK!(string, b'{')
                || CHECK!(string, b'}')
                || CHECK!(string, b'&')
                || CHECK!(string, b'*')
                || CHECK!(string, b'!')
                || CHECK!(string, b'|')
                || CHECK!(string, b'>')
                || CHECK!(string, b'\'')
                || CHECK!(string, b'"')
                || CHECK!(string, b'%')
                || CHECK!(string, b'@')
                || CHECK!(string, b'`')
            {
                flow_indicators = true;
                block_indicators = true;
            }
            if CHECK!(string, b'?') || CHECK!(string, b':') {
                flow_indicators = true;
                if followed_by_whitespace {
                    block_indicators = true;
                }
            }
            if CHECK!(string, b'-') && followed_by_whitespace {
                flow_indicators = true;
                block_indicators = true;
            }
        } else {
            if CHECK!(string, b',')
                || CHECK!(string, b'?')
                || CHECK!(string, b'[')
                || CHECK!(string, b']')
                || CHECK!(string, b'{')
                || CHECK!(string, b'}')
            {
                flow_indicators = true;
            }
            if CHECK!(string, b':') {
                flow_indicators = true;
                if followed_by_whitespace {
                    block_indicators = true;
                }
            }
            if CHECK!(string, b'#') && preceded_by_whitespace {
                flow_indicators = true;
                block_indicators = true;
            }
        }
        if !IS_PRINTABLE!(string)
            || !IS_ASCII!(string) && !(*emitter).unicode
        {
            special_characters = true;
        }
        if IS_BREAK!(string) {
            line_breaks = true;
        }
        if IS_SPACE!(string) {
            if string.start == string.pointer {
                leading_space = true;
            }
            if string.pointer.wrapping_offset(WIDTH!(string) as isize)
                == string.end
            {
                trailing_space = true;
            }
            if previous_break {
                break_space = true;
            }
            previous_space = true;
            previous_break = false;
        } else if IS_BREAK!(string) {
            if string.start == string.pointer {
                leading_break = true;
            }
            if string.pointer.wrapping_offset(WIDTH!(string) as isize)
                == string.end
            {
                trailing_break = true;
            }
            if previous_space {
                space_break = true;
            }
            previous_space = false;
            previous_break = true;
        } else {
            previous_space = false;
            previous_break = false;
        }
        preceded_by_whitespace = IS_BLANKZ!(string);
        MOVE!(string);
        if string.pointer != string.end {
            followed_by_whitespace =
                IS_BLANKZ_AT!(string, WIDTH!(string));
        }
    }
    (*emitter).scalar_data.multiline = line_breaks;
    (*emitter).scalar_data.flow_plain_allowed = true;
    (*emitter).scalar_data.block_plain_allowed = true;
    (*emitter).scalar_data.single_quoted_allowed = true;
    (*emitter).scalar_data.block_allowed = true;
    if leading_space
        || leading_break
        || trailing_space
        || trailing_break
    {
        (*emitter).scalar_data.flow_plain_allowed = false;
        (*emitter).scalar_data.block_plain_allowed = false;
    }
    if trailing_space {
        (*emitter).scalar_data.block_allowed = false;
    }
    if break_space {
        (*emitter).scalar_data.flow_plain_allowed = false;
        (*emitter).scalar_data.block_plain_allowed = false;
        (*emitter).scalar_data.single_quoted_allowed = false;
    }
    if space_break || special_characters {
        (*emitter).scalar_data.flow_plain_allowed = false;
        (*emitter).scalar_data.block_plain_allowed = false;
        (*emitter).scalar_data.single_quoted_allowed = false;
        (*emitter).scalar_data.block_allowed = false;
    }
    if line_breaks {
        (*emitter).scalar_data.flow_plain_allowed = false;
        (*emitter).scalar_data.block_plain_allowed = false;
    }
    if flow_indicators {
        (*emitter).scalar_data.flow_plain_allowed = false;
    }
    if block_indicators {
        (*emitter).scalar_data.block_plain_allowed = false;
    }
    OK
}

/// Analyzes an event before emission.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `event` must be a valid, non-null pointer to a properly initialized `YamlEventT`
/// - All event data pointers (anchor, tag, scalar value) must be valid if present
/// - The scalar data fields must be properly initialized for scalar events
/// - The tag data fields must be properly initialized for events with tags
unsafe fn yaml_emitter_analyze_event(
    emitter: *mut YamlEmitterT,
    event: *mut YamlEventT,
) -> Success {
    let fresh52 = addr_of_mut!((*emitter).anchor_data.anchor);
    *fresh52 = ptr::null_mut::<yaml_char_t>();
    (*emitter).anchor_data.anchor_length = 0_u64;
    let fresh53 = addr_of_mut!((*emitter).tag_data.handle);
    *fresh53 = ptr::null_mut::<yaml_char_t>();
    (*emitter).tag_data.handle_length = 0_u64;
    let fresh54 = addr_of_mut!((*emitter).tag_data.suffix);
    *fresh54 = ptr::null_mut::<yaml_char_t>();
    (*emitter).tag_data.suffix_length = 0_u64;
    let fresh55 = addr_of_mut!((*emitter).scalar_data.value);
    *fresh55 = ptr::null_mut::<yaml_char_t>();
    (*emitter).scalar_data.length = 0_u64;
    match (*event).type_ {
        YamlAliasEvent => yaml_emitter_analyze_anchor(
            emitter,
            (*event).data.alias.anchor,
            true,
        ),
        YamlScalarEvent => {
            if !(*event).data.scalar.anchor.is_null()
                && yaml_emitter_analyze_anchor(
                    emitter,
                    (*event).data.scalar.anchor,
                    false,
                )
                .fail
            {
                return FAIL;
            }
            if !(*event).data.scalar.tag.is_null()
                && ((*emitter).canonical
                    || !(*event).data.scalar.plain_implicit
                        && !(*event).data.scalar.quoted_implicit)
                && yaml_emitter_analyze_tag(
                    emitter,
                    (*event).data.scalar.tag,
                )
                .fail
            {
                return FAIL;
            }
            yaml_emitter_analyze_scalar(
                emitter,
                (*event).data.scalar.value,
                (*event).data.scalar.length,
            )
        }
        YamlSequenceStartEvent => {
            if !(*event).data.sequence_start.anchor.is_null()
                && yaml_emitter_analyze_anchor(
                    emitter,
                    (*event).data.sequence_start.anchor,
                    false,
                )
                .fail
            {
                return FAIL;
            }
            if !(*event).data.sequence_start.tag.is_null()
                && ((*emitter).canonical
                    || !(*event).data.sequence_start.implicit)
                && yaml_emitter_analyze_tag(
                    emitter,
                    (*event).data.sequence_start.tag,
                )
                .fail
            {
                return FAIL;
            }
            OK
        }
        YamlMappingStartEvent => {
            if !(*event).data.mapping_start.anchor.is_null()
                && yaml_emitter_analyze_anchor(
                    emitter,
                    (*event).data.mapping_start.anchor,
                    false,
                )
                .fail
            {
                return FAIL;
            }
            if !(*event).data.mapping_start.tag.is_null()
                && ((*emitter).canonical
                    || !(*event).data.mapping_start.implicit)
                && yaml_emitter_analyze_tag(
                    emitter,
                    (*event).data.mapping_start.tag,
                )
                .fail
            {
                return FAIL;
            }
            OK
        }
        _ => OK,
    }
}

/// Writes a UTF-8 BOM (Byte Order Mark).
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The buffer must have enough space for the 3-byte BOM sequence
/// - The buffer pointer must be properly aligned for byte writes
/// - The encoding must be set to support BOM writing
unsafe fn yaml_emitter_write_bom(
    emitter: *mut YamlEmitterT,
) -> Success {
    if flush(emitter).fail {
        return FAIL;
    }
    let fresh56 = addr_of_mut!((*emitter).buffer.pointer);
    let fresh57 = *fresh56;
    *fresh56 = (*fresh56).wrapping_offset(1);
    *fresh57 = b'\xEF';
    let fresh58 = addr_of_mut!((*emitter).buffer.pointer);
    let fresh59 = *fresh58;
    *fresh58 = (*fresh58).wrapping_offset(1);
    *fresh59 = b'\xBB';
    let fresh60 = addr_of_mut!((*emitter).buffer.pointer);
    let fresh61 = *fresh60;
    *fresh60 = (*fresh60).wrapping_offset(1);
    *fresh61 = b'\xBF';
    OK
}

/// Writes indentation according to the current indentation level.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The indent field must contain a valid indentation level
/// - The buffer must have enough space for the full indentation
/// - The column tracking must be accurate
/// - The whitespace and indentation flags must be valid
unsafe fn yaml_emitter_write_indent(
    emitter: *mut YamlEmitterT,
) -> Success {
    let indent: libc::c_int = if (*emitter).indent >= 0 {
        (*emitter).indent
    } else {
        0
    };
    if (!(*emitter).indention
        || (*emitter).column > indent
        || (*emitter).column == indent && !(*emitter).whitespace)
        && put_break(emitter).fail
    {
        return FAIL;
    }
    if (*emitter).column < indent {
        loop {
            if put(emitter, b' ').fail {
                return FAIL;
            }
            if (*emitter).column >= indent {
                break;
            }
        }
    }
    (*emitter).whitespace = true;
    (*emitter).indention = true;
    OK
}

/// Writes a YAML indicator token.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `indicator` must be a valid pointer to a null-terminated C string
/// - The indicator string must contain a valid YAML indicator token
/// - The buffer must have enough space for the indicator and any required whitespace
/// - The whitespace and indentation state must be valid
unsafe fn yaml_emitter_write_indicator(
    emitter: *mut YamlEmitterT,
    indicator: *const libc::c_char,
    need_whitespace: bool,
    is_whitespace: bool,
    is_indention: bool,
) -> Success {
    let indicator_length: size_t = strlen(indicator);
    let mut string =
        STRING_ASSIGN!(indicator as *mut yaml_char_t, indicator_length);
    if need_whitespace
        && !(*emitter).whitespace
        && put(emitter, b' ').fail
    {
        return FAIL;
    }
    while string.pointer != string.end {
        if write!(emitter, string).fail {
            return FAIL;
        }
    }
    (*emitter).whitespace = is_whitespace;
    (*emitter).indention = (*emitter).indention && is_indention;
    OK
}

/// Writes an anchor.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `value` must be a valid pointer to a null-terminated string buffer
/// - `length` must accurately reflect the size of the anchor name
/// - The buffer must have enough space for the anchor formatting
/// - The anchor name must contain only valid anchor characters
unsafe fn yaml_emitter_write_anchor(
    emitter: *mut YamlEmitterT,
    value: *mut yaml_char_t,
    length: size_t,
) -> Success {
    let mut string = STRING_ASSIGN!(value, length);
    while string.pointer != string.end {
        if write!(emitter, string).fail {
            return FAIL;
        }
    }
    (*emitter).whitespace = false;
    (*emitter).indention = false;
    OK
}

/// Writes a tag handle.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `value` must be a valid pointer to a null-terminated string buffer
/// - `length` must accurately reflect the size of the tag handle
/// - The buffer must have enough space for the tag handle and any required whitespace
/// - The tag handle must be a valid YAML tag handle
unsafe fn yaml_emitter_write_tag_handle(
    emitter: *mut YamlEmitterT,
    value: *mut yaml_char_t,
    length: size_t,
) -> Success {
    let mut string = STRING_ASSIGN!(value, length);
    if !(*emitter).whitespace && put(emitter, b' ').fail {
        return FAIL;
    }
    while string.pointer != string.end {
        if write!(emitter, string).fail {
            return FAIL;
        }
    }
    (*emitter).whitespace = false;
    (*emitter).indention = false;
    OK
}

/// Writes tag content with proper escaping.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `value` must be a valid pointer to a null-terminated string buffer
/// - `length` must accurately reflect the size of the tag content
/// - The buffer must have enough space for the content and all escape sequences
/// - The tag content must be valid according to YAML tag rules
unsafe fn yaml_emitter_write_tag_content(
    emitter: *mut YamlEmitterT,
    value: *mut yaml_char_t,
    length: size_t,
    need_whitespace: bool,
) -> Success {
    let mut string = STRING_ASSIGN!(value, length);
    if need_whitespace
        && !(*emitter).whitespace
        && put(emitter, b' ').fail
    {
        return FAIL;
    }
    while string.pointer != string.end {
        if IS_ALPHA!(string)
            || CHECK!(string, b';')
            || CHECK!(string, b'/')
            || CHECK!(string, b'?')
            || CHECK!(string, b':')
            || CHECK!(string, b'@')
            || CHECK!(string, b'&')
            || CHECK!(string, b'=')
            || CHECK!(string, b'+')
            || CHECK!(string, b'$')
            || CHECK!(string, b',')
            || CHECK!(string, b'_')
            || CHECK!(string, b'.')
            || CHECK!(string, b'~')
            || CHECK!(string, b'*')
            || CHECK!(string, b'\'')
            || CHECK!(string, b'(')
            || CHECK!(string, b')')
            || CHECK!(string, b'[')
            || CHECK!(string, b']')
        {
            if write!(emitter, string).fail {
                return FAIL;
            }
        } else {
            let mut width = WIDTH!(string);
            loop {
                let fresh207 = width;
                width -= 1;
                if fresh207 == 0 {
                    break;
                }
                let fresh208 = string.pointer;
                string.pointer = string.pointer.wrapping_offset(1);
                let value = *fresh208;
                if put(emitter, b'%').fail {
                    return FAIL;
                }
                if put(
                    emitter,
                    (value >> 4).force_add(if (value >> 4) < 10 {
                        b'0'
                    } else {
                        b'A' - 10
                    }),
                )
                .fail
                {
                    return FAIL;
                }
                if put(
                    emitter,
                    (value & 0x0F).force_add(if (value & 0x0F) < 10 {
                        b'0'
                    } else {
                        b'A' - 10
                    }),
                )
                .fail
                {
                    return FAIL;
                }
            }
        }
    }
    (*emitter).whitespace = false;
    (*emitter).indention = false;
    OK
}

/// Writes a plain scalar value.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `value` must be a valid pointer to a null-terminated string buffer
/// - `length` must accurately reflect the size of the value buffer
/// - The buffer must have enough space for the content and any required line breaks
/// - The flow_level and indentation state must be valid
unsafe fn yaml_emitter_write_plain_scalar(
    emitter: *mut YamlEmitterT,
    value: *mut yaml_char_t,
    length: size_t,
    allow_breaks: bool,
) -> Success {
    let mut spaces = false;
    let mut breaks = false;
    let mut string = STRING_ASSIGN!(value, length);
    if !(*emitter).whitespace
        && (length != 0 || (*emitter).flow_level != 0)
        && put(emitter, b' ').fail
    {
        return FAIL;
    }
    while string.pointer != string.end {
        if IS_SPACE!(string) {
            if allow_breaks
                && !spaces
                && (*emitter).column > (*emitter).best_width
                && !IS_SPACE_AT!(string, 1)
            {
                if yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
                MOVE!(string);
            } else if write!(emitter, string).fail {
                return FAIL;
            }
            spaces = true;
        } else if IS_BREAK!(string) {
            if !breaks
                && CHECK!(string, b'\n')
                && put_break(emitter).fail
            {
                return FAIL;
            }
            if write_break!(emitter, string).fail {
                return FAIL;
            }
            (*emitter).indention = true;
            breaks = true;
        } else {
            if breaks && yaml_emitter_write_indent(emitter).fail {
                return FAIL;
            }
            if write!(emitter, string).fail {
                return FAIL;
            }
            (*emitter).indention = false;
            spaces = false;
            breaks = false;
        }
    }
    (*emitter).whitespace = false;
    (*emitter).indention = false;
    OK
}

/// Writes a single quoted scalar value.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `value` must be a valid pointer to a null-terminated string buffer
/// - `length` must accurately reflect the size of the value buffer
/// - The buffer must have enough space for the quoted content and escaping
/// - All string content must be valid UTF-8 if unicode mode is enabled
unsafe fn yaml_emitter_write_single_quoted_scalar(
    emitter: *mut YamlEmitterT,
    value: *mut yaml_char_t,
    length: size_t,
    allow_breaks: bool,
) -> Success {
    let mut spaces = false;
    let mut breaks = false;
    let mut string = STRING_ASSIGN!(value, length);
    if yaml_emitter_write_indicator(
        emitter,
        b"'\0" as *const u8 as *const libc::c_char,
        true,
        false,
        false,
    )
    .fail
    {
        return FAIL;
    }
    while string.pointer != string.end {
        if IS_SPACE!(string) {
            if allow_breaks
                && !spaces
                && (*emitter).column > (*emitter).best_width
                && string.pointer != string.start
                && string.pointer
                    != string.end.wrapping_offset(-1_isize)
                && !IS_SPACE_AT!(string, 1)
            {
                if yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
                MOVE!(string);
            } else if write!(emitter, string).fail {
                return FAIL;
            }
            spaces = true;
        } else if IS_BREAK!(string) {
            if !breaks
                && CHECK!(string, b'\n')
                && put_break(emitter).fail
            {
                return FAIL;
            }
            if write_break!(emitter, string).fail {
                return FAIL;
            }
            (*emitter).indention = true;
            breaks = true;
        } else {
            if breaks && yaml_emitter_write_indent(emitter).fail {
                return FAIL;
            }
            if CHECK!(string, b'\'') && put(emitter, b'\'').fail {
                return FAIL;
            }
            if write!(emitter, string).fail {
                return FAIL;
            }
            (*emitter).indention = false;
            spaces = false;
            breaks = false;
        }
    }
    if breaks && yaml_emitter_write_indent(emitter).fail {
        return FAIL;
    }
    if yaml_emitter_write_indicator(
        emitter,
        b"'\0" as *const u8 as *const libc::c_char,
        false,
        false,
        false,
    )
    .fail
    {
        return FAIL;
    }
    (*emitter).whitespace = false;
    (*emitter).indention = false;
    OK
}

/// Writes a double quoted scalar value with proper escaping.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `value` must be a valid pointer to a null-terminated string buffer
/// - `length` must accurately reflect the size of the value buffer
/// - The buffer must have enough space for the quoted content and all escape sequences
/// - The unicode flag in emitter must be properly set
unsafe fn yaml_emitter_write_double_quoted_scalar(
    emitter: *mut YamlEmitterT,
    value: *mut yaml_char_t,
    length: size_t,
    allow_breaks: bool,
) -> Success {
    let mut spaces = false;
    let mut string = STRING_ASSIGN!(value, length);
    if yaml_emitter_write_indicator(
        emitter,
        b"\"\0" as *const u8 as *const libc::c_char,
        true,
        false,
        false,
    )
    .fail
    {
        return FAIL;
    }
    while string.pointer != string.end {
        if !IS_PRINTABLE!(string)
            || !(*emitter).unicode && !IS_ASCII!(string)
            || IS_BOM!(string)
            || IS_BREAK!(string)
            || CHECK!(string, b'"')
            || CHECK!(string, b'\\')
        {
            let mut octet: libc::c_uchar;
            let mut width: libc::c_uint;
            let mut value_0: libc::c_uint;
            let mut k: libc::c_int;
            octet = *string.pointer;
            width = if octet & 0x80 == 0x00 {
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
            value_0 = if octet & 0x80 == 0 {
                octet & 0x7F
            } else if octet & 0xE0 == 0xC0 {
                octet & 0x1F
            } else if octet & 0xF0 == 0xE0 {
                octet & 0x0F
            } else if octet & 0xF8 == 0xF0 {
                octet & 0x07
            } else {
                0
            } as libc::c_uint;
            k = 1;
            while k < width as libc::c_int {
                octet = *string.pointer.wrapping_offset(k as isize);
                value_0 = (value_0 << 6)
                    .force_add((octet & 0x3F) as libc::c_uint);
                k += 1;
            }
            string.pointer =
                string.pointer.wrapping_offset(width as isize);
            if put(emitter, b'\\').fail {
                return FAIL;
            }
            match value_0 {
                0x00 => {
                    if put(emitter, b'0').fail {
                        return FAIL;
                    }
                }
                0x07 => {
                    if put(emitter, b'a').fail {
                        return FAIL;
                    }
                }
                0x08 => {
                    if put(emitter, b'b').fail {
                        return FAIL;
                    }
                }
                0x09 => {
                    if put(emitter, b't').fail {
                        return FAIL;
                    }
                }
                0x0A => {
                    if put(emitter, b'n').fail {
                        return FAIL;
                    }
                }
                0x0B => {
                    if put(emitter, b'v').fail {
                        return FAIL;
                    }
                }
                0x0C => {
                    if put(emitter, b'f').fail {
                        return FAIL;
                    }
                }
                0x0D => {
                    if put(emitter, b'r').fail {
                        return FAIL;
                    }
                }
                0x1B => {
                    if put(emitter, b'e').fail {
                        return FAIL;
                    }
                }
                0x22 => {
                    if put(emitter, b'"').fail {
                        return FAIL;
                    }
                }
                0x5C => {
                    if put(emitter, b'\\').fail {
                        return FAIL;
                    }
                }
                0x85 => {
                    if put(emitter, b'N').fail {
                        return FAIL;
                    }
                }
                0xA0 => {
                    if put(emitter, b'_').fail {
                        return FAIL;
                    }
                }
                0x2028 => {
                    if put(emitter, b'L').fail {
                        return FAIL;
                    }
                }
                0x2029 => {
                    if put(emitter, b'P').fail {
                        return FAIL;
                    }
                }
                _ => {
                    if value_0 <= 0xFF {
                        if put(emitter, b'x').fail {
                            return FAIL;
                        }
                        width = 2;
                    } else if value_0 <= 0xFFFF {
                        if put(emitter, b'u').fail {
                            return FAIL;
                        }
                        width = 4;
                    } else {
                        if put(emitter, b'U').fail {
                            return FAIL;
                        }
                        width = 8;
                    }
                    k = width.wrapping_sub(1).wrapping_mul(4)
                        as libc::c_int;
                    while k >= 0 {
                        let digit: libc::c_int =
                            ((value_0 >> k) & 0x0F) as libc::c_int;
                        if put(
                            emitter,
                            (digit
                                + if digit < 10 {
                                    b'0'
                                } else {
                                    b'A' - 10
                                }
                                    as i32)
                                as u8,
                        )
                        .fail
                        {
                            return FAIL;
                        }
                        k -= 4;
                    }
                }
            }
            spaces = false;
        } else if IS_SPACE!(string) {
            if allow_breaks
                && !spaces
                && (*emitter).column > (*emitter).best_width
                && string.pointer != string.start
                && string.pointer
                    != string.end.wrapping_offset(-1_isize)
            {
                if yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
                if IS_SPACE_AT!(string, 1) && put(emitter, b'\\').fail {
                    return FAIL;
                }
                MOVE!(string);
            } else if write!(emitter, string).fail {
                return FAIL;
            }
            spaces = true;
        } else {
            if write!(emitter, string).fail {
                return FAIL;
            }
            spaces = false;
        }
    }
    if yaml_emitter_write_indicator(
        emitter,
        b"\"\0" as *const u8 as *const libc::c_char,
        false,
        false,
        false,
    )
    .fail
    {
        return FAIL;
    }
    (*emitter).whitespace = false;
    (*emitter).indention = false;
    OK
}

/// Writes block scalar hints.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - The string parameter must be a valid `YamlStringT` with properly aligned pointers
/// - The buffer must have enough space for the hints
/// - The best_indent field must contain a valid indentation value
unsafe fn yaml_emitter_write_block_scalar_hints(
    emitter: *mut YamlEmitterT,
    mut string: YamlStringT,
) -> Success {
    let mut indent_hint: [libc::c_char; 2] = [0; 2];
    let mut chomp_hint: *const libc::c_char =
        ptr::null::<libc::c_char>();
    if IS_SPACE!(string) || IS_BREAK!(string) {
        indent_hint[0] = (b'0' as libc::c_int + (*emitter).best_indent)
            as libc::c_char;
        indent_hint[1] = '\0' as libc::c_char;
        if yaml_emitter_write_indicator(
            emitter,
            indent_hint.as_mut_ptr(),
            false,
            false,
            false,
        )
        .fail
        {
            return FAIL;
        }
    }
    (*emitter).open_ended = 0;
    string.pointer = string.end;
    if string.start == string.pointer {
        chomp_hint = b"-\0" as *const u8 as *const libc::c_char;
    } else {
        loop {
            string.pointer = string.pointer.wrapping_offset(-1);
            if *string.pointer & 0xC0 != 0x80 {
                break;
            }
        }
        if !IS_BREAK!(string) {
            chomp_hint = b"-\0" as *const u8 as *const libc::c_char;
        } else if string.start == string.pointer {
            chomp_hint = b"+\0" as *const u8 as *const libc::c_char;
            (*emitter).open_ended = 2;
        } else {
            loop {
                string.pointer = string.pointer.wrapping_offset(-1);
                if *string.pointer & 0xC0 != 0x80 {
                    break;
                }
            }
            if IS_BREAK!(string) {
                chomp_hint = b"+\0" as *const u8 as *const libc::c_char;
                (*emitter).open_ended = 2;
            }
        }
    }
    if !chomp_hint.is_null()
        && yaml_emitter_write_indicator(
            emitter, chomp_hint, false, false, false,
        )
        .fail
    {
        return FAIL;
    }
    OK
}

/// Writes a literal scalar value.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `value` must be a valid pointer to a null-terminated string buffer
/// - `length` must accurately reflect the size of the value buffer
/// - The buffer must have enough space for the scalar content and formatting
/// - All emitter state fields must be properly initialized
unsafe fn yaml_emitter_write_literal_scalar(
    emitter: *mut YamlEmitterT,
    value: *mut yaml_char_t,
    length: size_t,
) -> Success {
    let mut breaks = true;
    let mut string = STRING_ASSIGN!(value, length);
    if yaml_emitter_write_indicator(
        emitter,
        b"|\0" as *const u8 as *const libc::c_char,
        true,
        false,
        false,
    )
    .fail
    {
        return FAIL;
    }
    if yaml_emitter_write_block_scalar_hints(emitter, string).fail {
        return FAIL;
    }
    if put_break(emitter).fail {
        return FAIL;
    }
    (*emitter).indention = true;
    (*emitter).whitespace = true;
    while string.pointer != string.end {
        if IS_BREAK!(string) {
            if write_break!(emitter, string).fail {
                return FAIL;
            }
            (*emitter).indention = true;
            breaks = true;
        } else {
            if breaks && yaml_emitter_write_indent(emitter).fail {
                return FAIL;
            }
            if write!(emitter, string).fail {
                return FAIL;
            }
            (*emitter).indention = false;
            breaks = false;
        }
    }
    OK
}

/// Writes a folded scalar value.
///
/// # Safety
///
/// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT`
/// - `value` must be a valid pointer to a null-terminated string buffer
/// - `length` must accurately reflect the size of the value buffer
/// - The buffer must have enough space for the scalar content and all necessary line breaks
/// - The emitter's line breaking and indentation state must be valid
unsafe fn yaml_emitter_write_folded_scalar(
    emitter: *mut YamlEmitterT,
    value: *mut yaml_char_t,
    length: size_t,
) -> Success {
    let mut breaks = true;
    let mut leading_spaces = true;
    let mut string = STRING_ASSIGN!(value, length);
    if yaml_emitter_write_indicator(
        emitter,
        b">\0" as *const u8 as *const libc::c_char,
        true,
        false,
        false,
    )
    .fail
    {
        return FAIL;
    }
    if yaml_emitter_write_block_scalar_hints(emitter, string).fail {
        return FAIL;
    }
    if put_break(emitter).fail {
        return FAIL;
    }
    (*emitter).indention = true;
    (*emitter).whitespace = true;
    while string.pointer != string.end {
        if IS_BREAK!(string) {
            if !breaks && !leading_spaces && CHECK!(string, b'\n') {
                let mut k: libc::c_int = 0;
                while IS_BREAK_AT!(string, k as isize) {
                    k += WIDTH_AT!(string, k as isize);
                }
                if !IS_BLANKZ_AT!(string, k as isize)
                    && put_break(emitter).fail
                {
                    return FAIL;
                }
            }
            if write_break!(emitter, string).fail {
                return FAIL;
            }
            (*emitter).indention = true;
            breaks = true;
        } else {
            if breaks {
                if yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
                leading_spaces = IS_BLANK!(string);
            }
            if !breaks
                && IS_SPACE!(string)
                && !IS_SPACE_AT!(string, 1)
                && (*emitter).column > (*emitter).best_width
            {
                if yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
                MOVE!(string);
            } else if write!(emitter, string).fail {
                return FAIL;
            }
            (*emitter).indention = false;
            breaks = false;
        }
    }
    OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::yaml_malloc;
    use crate::YamlEmitterStateT;
    use core::ffi::CStr;
    use core::mem::zeroed;
    use core::mem::MaybeUninit;
    use core::ptr::null_mut;

    #[test]
    fn test_flush() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t) as *mut u8;
            if raw_buf.is_null() {
                panic!(
                    "failed to allocate test buffer for YamlEmitterT"
                );
            }

            // Set up the buffer pointers
            emitter.buffer.start = raw_buf;
            emitter.buffer.pointer = raw_buf;
            emitter.buffer.end = (raw_buf).add(capacity);

            // Call the function under test
            let result = flush(&mut emitter);
            assert!(!result.fail, "Expected flush to succeed");

            // Now free the allocated buffer, so we don't leak
            yaml_free(raw_buf as *mut libc::c_void);
        }
    }

    // 2. put
    #[test]
    fn test_put() {
        unsafe {
            // 1) Create a zeroed emitter
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // 2) Allocate a buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            if raw_buf.is_null() {
                panic!(
                    "Failed to allocate test buffer for YamlEmitterT"
                );
            }

            // 3) Initialize the emitter buffer pointers
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // 4) Optionally set other fields to satisfy any asserts
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;

            // 5) Call the function under test
            let byte_value = b'A';
            let result = put(&mut emitter, byte_value);
            assert!(!result.fail, "Expected put to succeed");

            // 6) Free the buffer so Miri doesn't complain about a leak
            yaml_free(raw_buf);
        }
    }

    // 3. put_break
    #[test]
    fn test_put_break() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null(), "Failed to allocate buffer");

            // Set up the buffer and write handler
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Test each line break type
            for break_type in [YamlCrBreak, YamlLnBreak, YamlCrlnBreak]
            {
                emitter.line_break = break_type;
                emitter.column = 5; // Set some initial column value
                emitter.line = 1; // Set initial line

                let result = put_break(&mut emitter);
                assert!(!result.fail, "Expected put_break to succeed");

                // Verify column was reset
                assert_eq!(
                    emitter.column, 0,
                    "Column should be reset after break"
                );

                // Verify line was incremented
                assert_eq!(
                    emitter.line, 2,
                    "Line should be incremented"
                );

                // Reset buffer pointer for next test
                emitter.buffer.pointer = emitter.buffer.start;
            }

            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_put_buffer_verification() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());

            // Set up the buffer pointers
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Test normal write
            let test_byte = b'A';
            let result = put(&mut emitter, test_byte);
            assert!(!result.fail, "Expected put to succeed");
            assert_eq!(
                *emitter.buffer.start, test_byte,
                "Byte wasn't written correctly"
            );
            assert_eq!(
                emitter.column, 1,
                "Column should be incremented"
            );

            // Clean up
            yaml_free(raw_buf);
        }
    }

    // 4. write
    #[test]
    fn test_write_function() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null(), "Failed to allocate buffer");

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;

            let mut yaml_string = YamlStringT {
                start: b"test".as_ptr() as *mut yaml_char_t,
                end: b"test".as_ptr().add(4) as *mut yaml_char_t,
                pointer: b"test".as_ptr() as *mut yaml_char_t,
            };

            let result = write(&mut emitter, &mut yaml_string);
            assert!(!result.fail, "Expected write to succeed");

            yaml_free(raw_buf);
        }
    }

    // 5. write_break
    #[test]
    fn test_write_break_function() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null(), "Failed to allocate buffer");

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;

            let mut yaml_string = YamlStringT {
                start: b"\n".as_ptr() as *mut yaml_char_t,
                end: b"\n".as_ptr().add(1) as *mut yaml_char_t,
                pointer: b"\n".as_ptr() as *mut yaml_char_t,
            };

            let result = write_break(&mut emitter, &mut yaml_string);
            assert!(!result.fail, "Expected write_break to succeed");

            yaml_free(raw_buf);
        }
    }

    // 6. yaml_emitter_set_emitter_error
    #[test]
    fn test_yaml_emitter_set_emitter_error() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;

            let error_str = b"some error\0";
            let result = yaml_emitter_set_emitter_error(
                &mut emitter,
                error_str.as_ptr() as *const libc::c_char,
            );
            assert!(result.fail, "Setting an error should return FAIL");
            assert_eq!(emitter.error, YamlEmitterError);
            assert_eq!(
                emitter.problem,
                error_str.as_ptr() as *const libc::c_char
            );

            yaml_free(raw_buf);
        }
    }

    // 7. yaml_emitter_emit
    #[test]
    fn test_yaml_emitter_emit() {
        unsafe {
            // 1) Zero the emitter
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // 2) Allocate a small output buffer so pointer arithmetic won’t overflow
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(
                !raw_buf.is_null(),
                "Failed to allocate main buffer"
            );
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // 4) (Optional) Allocate space for the events queue
            //    if ENQUEUE!/DEQUEUE! macros don't do dynamic reallocation automatically.
            let events_capacity = 4_usize;
            let events_array = yaml_malloc(
                (events_capacity * size_of::<YamlEventT>())
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEventT;
            assert!(
                !events_array.is_null(),
                "Failed to allocate events array"
            );

            emitter.events.start = events_array;
            emitter.events.head = events_array;
            emitter.events.tail = events_array;
            emitter.events.end = events_array.add(events_capacity);

            // 5) Optionally set an initial emitter state if your code expects it.
            //    Some code requires YamlEmitStreamStartState as a valid starting point:
            // emitter.state = YamlEmitStreamStartState;

            // 6) Fill in fields like encoding, line_break, etc.
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;

            // 7) Create a valid event (rather than all zeros).
            //    For example, a basic StreamStartEvent:
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlStreamStartEvent;
            // If needed, you can set event.data.stream_start.encoding = YamlUtf8Encoding;

            // 8) Call the function under test
            let result = yaml_emitter_emit(&mut emitter, &mut event);
            assert!(
                !result.fail,
                "Expected yaml_emitter_emit to succeed"
            );

            // 9) Free everything so Miri doesn’t see leaks
            yaml_free(events_array as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    // 8. yaml_emitter_need_more_events
    #[test]
    fn test_yaml_emitter_need_more_events() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;

            // In a real test, you’d enqueue some events if needed

            let result = yaml_emitter_need_more_events(&mut emitter);
            assert!(
                !result.fail,
                "Expected no failure for empty event queue"
            );

            yaml_free(raw_buf);
        }
    }

    // 9. yaml_emitter_append_tag_directive
    #[test]
    fn test_yaml_emitter_append_tag_directive() {
        unsafe {
            let mut emitter: YamlEmitterT = zeroed();
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;

            // Allocate tag_directives buffer
            let needed = 4usize;
            let directives_buf = yaml_malloc(
                (size_of::<YamlTagDirectiveT>() * needed)
                    .try_into()
                    .unwrap(),
            )
                as *mut YamlTagDirectiveT;
            assert!(!directives_buf.is_null());
            emitter.tag_directives.start = directives_buf;
            emitter.tag_directives.end = directives_buf.add(needed);
            emitter.tag_directives.top = directives_buf;

            // Allocate strings that we'll free later
            let handle = yaml_strdup(b"!test!\0".as_ptr());
            let prefix =
                yaml_strdup(b"tag:example.com,2023:\0".as_ptr());

            let directive = YamlTagDirectiveT { handle, prefix };

            let result = yaml_emitter_append_tag_directive(
                &mut emitter,
                directive,
                false,
            );
            assert!(!result.fail, "Expected success for new directive");

            // Free all tag directives that were copied
            let mut current = emitter.tag_directives.start;
            while current < emitter.tag_directives.top {
                yaml_free((*current).handle as *mut libc::c_void);
                yaml_free((*current).prefix as *mut libc::c_void);
                current = current.add(1);
            }

            // Free our original strings
            yaml_free(handle as *mut libc::c_void);
            yaml_free(prefix as *mut libc::c_void);

            // Free the buffers
            yaml_free(directives_buf as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_increase_indent() {
        unsafe {
            // 1) Zero-initialize the emitter
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // 2) Allocate a small buffer so pointer arithmetic is safe (in case something needs it)
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(
                !raw_buf.is_null(),
                "Failed to allocate test buffer"
            );
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // 3) Provide some default settings for the emitter
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;

            // 4) Allocate space for the indents stack, since yaml_emitter_increase_indent
            //    does PUSH!((*emitter).indents, (*emitter).indent)
            //    If your macros do dynamic reallocation, you could skip this step.
            let indents_capacity = 4_usize;
            let indents_memory = yaml_malloc(
                (indents_capacity * size_of::<libc::c_int>()) as size_t,
            ) as *mut libc::c_int;
            assert!(
                !indents_memory.is_null(),
                "Failed to allocate indents stack"
            );

            emitter.indents.start = indents_memory;
            emitter.indents.top = indents_memory;
            emitter.indents.end = indents_memory.add(indents_capacity);

            // 5) Set initial indent and best_indent
            emitter.indent = 2;
            emitter.best_indent = 2; // how many spaces to add each time

            // 6) Call the function under test
            //    We'll pass `flow = false` and `indentless = false`.
            yaml_emitter_increase_indent(&mut emitter, false, false);

            // 7) Now check that emitter.indent was incremented by best_indent (2 -> 4)
            assert_eq!(
                emitter.indent, 4,
                "indent should have increased by best_indent"
            );

            // 8) Clean up allocations so that Miri doesn’t report memory leaks
            yaml_free(indents_memory as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_state_machine() {
        unsafe {
            // 1) Zero-initialize the emitter.
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // 2) Allocate a buffer so pointer arithmetic won’t overflow.
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(
                !raw_buf.is_null(),
                "Failed to allocate test buffer for YamlEmitterT"
            );
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // 4) Set a valid initial state. For example, many code paths expect the emitter
            //    to start in YamlEmitStreamStartState before seeing a YamlStreamStartEvent.
            emitter.state = YamlEmitStreamStartState;

            // 5) Fill in any other fields your code might rely on (e.g. encoding).
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            // (If you have states, tag directives, or other stacks that the state machine may push/pop,
            // allocate them similarly to your other tests.)

            // 6) Create a valid event for the current state: YamlStreamStartEvent
            //    so the state machine has something to process.
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlStreamStartEvent;
            // If needed, set event.data.stream_start.encoding = YamlUtf8Encoding, etc.

            // 7) Call the state machine under test
            let result =
                yaml_emitter_state_machine(&mut emitter, &mut event);
            assert!(
            !result.fail,
            "Expected state machine to handle a STREAM-START event successfully"
        );

            // 8) Free the allocated buffer so that Miri (if used) doesn't see a leak.
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_stream_start() {
        unsafe {
            // 1) Zero-initialize the emitter.
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // 2) Allocate a buffer (some functions do flush or pointer arithmetic).
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(
                !raw_buf.is_null(),
                "Failed to allocate test buffer for YamlEmitterT"
            );
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // 3) Set up any other fields or states that might be checked
            //    by yaml_emitter_emit_stream_start. Often it checks 'emitter.encoding',
            //    'emitter.best_indent', etc.
            emitter.encoding = YamlUtf8Encoding;
            emitter.best_indent = 2; // something valid between 2..9
            emitter.best_width = 80;
            emitter.line_break = YamlLnBreak;
            // E.g., state might be YamlEmitStreamStartState if your code expects that:
            // emitter.state = YamlEmitStreamStartState;

            // 4) Create a valid stream-start event
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlStreamStartEvent;
            // Typically, event.data.stream_start.encoding = YamlUtf8Encoding;
            // But if your code doesn't read that, you can skip.

            // 5) Call the function under test
            let result = yaml_emitter_emit_stream_start(
                &mut emitter,
                &mut event,
            );

            // 6) Check that the function succeeded
            assert!(
                !result.fail,
                "Expected yaml_emitter_emit_stream_start to succeed"
            );

            // 8) Free the buffer so Miri doesn't see a leak
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_document_start() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // 1) Allocate the main buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // 2) Allocate space for tag_directives
            use core::mem::size_of;
            let tag_directives_capacity = 4_usize;
            let tag_directives_buf = yaml_malloc(
                (size_of::<YamlTagDirectiveT>()
                    * tag_directives_capacity)
                    .try_into()
                    .unwrap(),
            )
                as *mut YamlTagDirectiveT;
            assert!(!tag_directives_buf.is_null());

            emitter.tag_directives.start = tag_directives_buf;
            emitter.tag_directives.top = tag_directives_buf;
            emitter.tag_directives.end =
                tag_directives_buf.add(tag_directives_capacity);

            // 3) Set up other emitter fields
            emitter.state = YamlEmitFirstDocumentStartState;
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            emitter.best_indent = 2;
            emitter.best_width = 80;

            // 4) Create a document start event
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlDocumentStartEvent;
            event.data.document_start.version_directive = null_mut();
            event.data.document_start.implicit = true;

            // 5) Call the function
            let result = yaml_emitter_emit_document_start(
                &mut emitter,
                &mut event,
                true,
            );
            assert!(!result.fail, "Expected document_start to succeed");

            // 6) **Free** the newly appended tag directives' handle/prefix
            let mut current = emitter.tag_directives.start;
            while current < emitter.tag_directives.top {
                // Both handle and prefix were allocated via yaml_strdup
                yaml_free((*current).handle as *mut libc::c_void);
                yaml_free((*current).prefix as *mut libc::c_void);
                current = current.add(1);
            }

            // 7) Free the tag_directives buffer array
            yaml_free(tag_directives_buf as *mut libc::c_void);

            // 8) Finally free the main buffer
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_flow_sequence_item() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Allocate main buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Allocate states stack
            let states_capacity = 4_usize;
            let states_buf = yaml_malloc(
                (size_of::<YamlEmitterStateT>() * states_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEmitterStateT;
            assert!(!states_buf.is_null());

            emitter.states.start = states_buf;
            emitter.states.top = states_buf;
            emitter.states.end = states_buf.add(states_capacity);

            // Allocate indents stack
            let indents_capacity = 4_usize;
            let indents_buf = yaml_malloc(
                (size_of::<libc::c_int>() * indents_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut libc::c_int;
            assert!(!indents_buf.is_null());

            emitter.indents.start = indents_buf;
            emitter.indents.top = indents_buf;
            emitter.indents.end = indents_buf.add(indents_capacity);

            // Set initial indent and push it onto stack
            emitter.indent = 2;
            PUSH!(emitter.indents, emitter.indent);

            // Set up emitter state
            emitter.state = YamlEmitFlowSequenceFirstItemState;
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            emitter.best_indent = 2;
            emitter.best_width = 80;
            emitter.flow_level = 1; // Since we're in a flow sequence

            // Create sequence end event
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlSequenceEndEvent;

            // Call function under test
            let result = yaml_emitter_emit_flow_sequence_item(
                &mut emitter,
                &mut event,
                true,
            );
            assert!(!result.fail, "Expected flow_sequence_item to succeed (ending sequence)");

            // Cleanup
            yaml_free(indents_buf as *mut libc::c_void);
            yaml_free(states_buf as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_flow_mapping_key() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Allocate main buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Allocate states stack
            let states_capacity = 4_usize;
            let states_buf = yaml_malloc(
                (size_of::<YamlEmitterStateT>() * states_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEmitterStateT;
            assert!(!states_buf.is_null());

            emitter.states.start = states_buf;
            emitter.states.top = states_buf;
            emitter.states.end = states_buf.add(states_capacity);

            // Allocate indents stack
            let indents_capacity = 4_usize;
            let indents_buf = yaml_malloc(
                (size_of::<libc::c_int>() * indents_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut libc::c_int;
            assert!(!indents_buf.is_null());

            emitter.indents.start = indents_buf;
            emitter.indents.top = indents_buf;
            emitter.indents.end = indents_buf.add(indents_capacity);

            // Set initial indent and push it onto stack
            emitter.indent = 2;
            PUSH!(emitter.indents, emitter.indent);

            // Set up emitter state
            emitter.state = YamlEmitFlowMappingFirstKeyState;
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            emitter.best_indent = 2;
            emitter.best_width = 80;
            emitter.flow_level = 1; // Since we're in a flow mapping

            // Create mapping end event
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlMappingEndEvent;

            // Call function under test
            let result = yaml_emitter_emit_flow_mapping_key(
                &mut emitter,
                &mut event,
                true,
            );
            assert!(
                !result.fail,
                "Expected flow_mapping_key to succeed (ending mapping)"
            );

            // Cleanup
            yaml_free(indents_buf as *mut libc::c_void);
            yaml_free(states_buf as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_flow_mapping_value() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Allocate main buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Allocate states stack
            let states_capacity = 4_usize;
            let states_buf = yaml_malloc(
                (size_of::<YamlEmitterStateT>() * states_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEmitterStateT;
            assert!(!states_buf.is_null());

            emitter.states.start = states_buf;
            emitter.states.top = states_buf;
            emitter.states.end = states_buf.add(states_capacity);

            // Push initial state onto states stack
            PUSH!(emitter.states, YamlEmitFlowMappingKeyState);

            // Allocate indents stack
            let indents_capacity = 4_usize;
            let indents_buf = yaml_malloc(
                (size_of::<libc::c_int>() * indents_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut libc::c_int;
            assert!(!indents_buf.is_null());

            emitter.indents.start = indents_buf;
            emitter.indents.top = indents_buf;
            emitter.indents.end = indents_buf.add(indents_capacity);

            // Set initial indent and push it onto stack
            emitter.indent = 2;
            PUSH!(emitter.indents, emitter.indent);

            // Set up emitter state
            emitter.state = YamlEmitFlowMappingKeyState;
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            emitter.best_indent = 2;
            emitter.best_width = 80;
            emitter.flow_level = 1; // Since we're in a flow mapping
            emitter.column = 0; // Start at column 0
            emitter.canonical = false;
            emitter.whitespace = true;

            // Create scalar event for the value
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlScalarEvent;
            let content = b"test\0";
            event.data.scalar.value =
                content.as_ptr() as *mut yaml_char_t;
            event.data.scalar.length = 4;
            event.data.scalar.style = YamlPlainScalarStyle;
            event.data.scalar.plain_implicit = true;
            event.data.scalar.quoted_implicit = true;

            // Call function under test
            let result = yaml_emitter_emit_flow_mapping_value(
                &mut emitter,
                &mut event,
                true, // simple value mode
            );
            assert!(
                !result.fail,
                "Expected flow_mapping_value to succeed"
            );

            // Cleanup
            yaml_free(indents_buf as *mut libc::c_void);
            yaml_free(states_buf as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_block_sequence_item() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Allocate main buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Allocate states stack
            let states_capacity = 4_usize;
            let states_buf = yaml_malloc(
                (size_of::<YamlEmitterStateT>() * states_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEmitterStateT;
            assert!(!states_buf.is_null());

            emitter.states.start = states_buf;
            emitter.states.top = states_buf;
            emitter.states.end = states_buf.add(states_capacity);

            // Push initial state
            PUSH!(emitter.states, YamlEmitBlockSequenceFirstItemState);

            // Allocate indents stack
            let indents_capacity = 4_usize;
            let indents_buf = yaml_malloc(
                (size_of::<libc::c_int>() * indents_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut libc::c_int;
            assert!(!indents_buf.is_null());

            emitter.indents.start = indents_buf;
            emitter.indents.top = indents_buf;
            emitter.indents.end = indents_buf.add(indents_capacity);

            // Set initial indent and push it onto stack
            emitter.indent = 2;
            PUSH!(emitter.indents, emitter.indent);

            // Set up emitter state
            emitter.state = YamlEmitBlockSequenceFirstItemState;
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            emitter.best_indent = 2;
            emitter.best_width = 80;
            emitter.column = 0;
            emitter.whitespace = true;
            emitter.indention = true;
            emitter.flow_level = 0; // Block sequence means flow_level = 0
            emitter.mapping_context = false;

            // Create sequence end event
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlSequenceEndEvent;

            // Call function under test
            let result = yaml_emitter_emit_block_sequence_item(
                &mut emitter,
                &mut event,
                true,
            );
            assert!(
                !result.fail,
                "Expected block_sequence_item to succeed"
            );

            // Cleanup
            yaml_free(indents_buf as *mut libc::c_void);
            yaml_free(states_buf as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_block_mapping_key() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Allocate main buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Allocate states stack
            let states_capacity = 4_usize;
            let states_buf = yaml_malloc(
                (size_of::<YamlEmitterStateT>() * states_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEmitterStateT;
            assert!(!states_buf.is_null());

            emitter.states.start = states_buf;
            emitter.states.top = states_buf;
            emitter.states.end = states_buf.add(states_capacity);

            // Push initial state
            PUSH!(emitter.states, YamlEmitBlockMappingFirstKeyState);

            // Allocate indents stack
            let indents_capacity = 4_usize;
            let indents_buf = yaml_malloc(
                (size_of::<libc::c_int>() * indents_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut libc::c_int;
            assert!(!indents_buf.is_null());

            emitter.indents.start = indents_buf;
            emitter.indents.top = indents_buf;
            emitter.indents.end = indents_buf.add(indents_capacity);

            // Set initial indent and push it onto stack
            emitter.indent = 2;
            PUSH!(emitter.indents, emitter.indent);

            // Set up emitter state
            emitter.state = YamlEmitBlockMappingFirstKeyState;
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            emitter.best_indent = 2;
            emitter.best_width = 80;
            emitter.column = 0;
            emitter.whitespace = true;
            emitter.indention = true;
            emitter.flow_level = 0; // Block mapping means flow_level = 0
            emitter.mapping_context = true;

            // Create mapping end event
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlMappingEndEvent;

            // Call function under test
            let result = yaml_emitter_emit_block_mapping_key(
                &mut emitter,
                &mut event,
                true,
            );
            assert!(
                !result.fail,
                "Expected block_mapping_key to succeed"
            );

            // Cleanup
            yaml_free(indents_buf as *mut libc::c_void);
            yaml_free(states_buf as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_block_mapping_value() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Allocate main buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Allocate states stack
            let states_capacity = 4_usize;
            let states_buf = yaml_malloc(
                (size_of::<YamlEmitterStateT>() * states_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEmitterStateT;
            assert!(!states_buf.is_null());

            emitter.states.start = states_buf;
            emitter.states.top = states_buf;
            emitter.states.end = states_buf.add(states_capacity);

            // Push initial state
            PUSH!(emitter.states, YamlEmitBlockMappingKeyState);

            // Allocate indents stack
            let indents_capacity = 4_usize;
            let indents_buf = yaml_malloc(
                (size_of::<libc::c_int>() * indents_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut libc::c_int;
            assert!(!indents_buf.is_null());

            emitter.indents.start = indents_buf;
            emitter.indents.top = indents_buf;
            emitter.indents.end = indents_buf.add(indents_capacity);

            // Set initial indent and push it onto stack
            emitter.indent = 2;
            PUSH!(emitter.indents, emitter.indent);

            // Set up emitter state
            emitter.state = YamlEmitBlockMappingKeyState;
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            emitter.best_indent = 2;
            emitter.best_width = 80;
            emitter.column = 0;
            emitter.whitespace = true;
            emitter.indention = true;
            emitter.flow_level = 0; // Block mapping means flow_level = 0
            emitter.mapping_context = true;

            // Create scalar event for the value
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlScalarEvent;
            let content = b"test value\0";
            event.data.scalar.value =
                content.as_ptr() as *mut yaml_char_t;
            event.data.scalar.length = 10;
            event.data.scalar.style = YamlPlainScalarStyle;
            event.data.scalar.plain_implicit = true;
            event.data.scalar.quoted_implicit = true;

            // Call function under test
            let result = yaml_emitter_emit_block_mapping_value(
                &mut emitter,
                &mut event,
                false, // non-simple value mode
            );
            assert!(
                !result.fail,
                "Expected block_mapping_value to succeed"
            );

            // Cleanup
            yaml_free(indents_buf as *mut libc::c_void);
            yaml_free(states_buf as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_node() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Allocate main buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Allocate states stack
            let states_capacity = 4_usize;
            let states_buf = yaml_malloc(
                (size_of::<YamlEmitterStateT>() * states_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEmitterStateT;
            assert!(!states_buf.is_null());

            emitter.states.start = states_buf;
            emitter.states.top = states_buf;
            emitter.states.end = states_buf.add(states_capacity);

            // Push initial state
            PUSH!(emitter.states, YamlEmitDocumentContentState);

            // Allocate indents stack
            let indents_capacity = 4_usize;
            let indents_buf = yaml_malloc(
                (size_of::<libc::c_int>() * indents_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut libc::c_int;
            assert!(!indents_buf.is_null());

            emitter.indents.start = indents_buf;
            emitter.indents.top = indents_buf;
            emitter.indents.end = indents_buf.add(indents_capacity);

            // Set initial indent and push it onto stack
            emitter.indent = 2;
            PUSH!(emitter.indents, emitter.indent);

            // Set up emitter state
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            emitter.best_indent = 2;
            emitter.best_width = 80;
            emitter.column = 0;
            emitter.whitespace = true;
            emitter.indention = true;
            emitter.flow_level = 0;

            // Create scalar event
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlScalarEvent;
            let content = b"test scalar\0";
            event.data.scalar.value =
                content.as_ptr() as *mut yaml_char_t;
            event.data.scalar.length = 11;
            event.data.scalar.style = YamlPlainScalarStyle;
            event.data.scalar.plain_implicit = true;
            event.data.scalar.quoted_implicit = true;

            // Initialize scalar data in emitter
            emitter.scalar_data.value = event.data.scalar.value;
            emitter.scalar_data.length = event.data.scalar.length;
            emitter.scalar_data.style = YamlPlainScalarStyle;
            emitter.scalar_data.multiline = false;
            emitter.scalar_data.flow_plain_allowed = true;
            emitter.scalar_data.block_plain_allowed = true;
            emitter.scalar_data.single_quoted_allowed = true;
            emitter.scalar_data.block_allowed = true;

            // Call function under test with root node flags
            let result = yaml_emitter_emit_node(
                &mut emitter,
                &mut event,
                true,  // root
                false, // sequence
                false, // mapping
                false, // simple key
            );
            assert!(!result.fail, "Expected node emission to succeed");

            // Cleanup
            yaml_free(indents_buf as *mut libc::c_void);
            yaml_free(states_buf as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_alias() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Allocate main buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Allocate states stack
            let states_capacity = 4_usize;
            let states_buf = yaml_malloc(
                (size_of::<YamlEmitterStateT>() * states_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEmitterStateT;
            assert!(!states_buf.is_null());

            emitter.states.start = states_buf;
            emitter.states.top = states_buf;
            emitter.states.end = states_buf.add(states_capacity);

            // Push initial state
            PUSH!(emitter.states, YamlEmitDocumentContentState);

            // Set up emitter state
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            emitter.best_indent = 2;
            emitter.best_width = 80;
            emitter.column = 0;
            emitter.whitespace = true;
            emitter.simple_key_context = true;

            // Set up anchor data
            let anchor = b"test_alias\0";
            emitter.anchor_data.anchor =
                anchor.as_ptr() as *mut yaml_char_t;
            emitter.anchor_data.anchor_length = 10;
            emitter.anchor_data.alias = true;

            // Create alias event
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlAliasEvent;
            event.data.alias.anchor =
                anchor.as_ptr() as *mut yaml_char_t;

            // Call function under test
            let result =
                yaml_emitter_emit_alias(&mut emitter, &mut event);
            assert!(!result.fail, "Expected alias emission to succeed");

            // Cleanup
            yaml_free(states_buf as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_scalar() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Allocate main buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Allocate states stack
            let states_capacity = 4_usize;
            let states_buf = yaml_malloc(
                (size_of::<YamlEmitterStateT>() * states_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEmitterStateT;
            assert!(!states_buf.is_null());

            emitter.states.start = states_buf;
            emitter.states.top = states_buf;
            emitter.states.end = states_buf.add(states_capacity);

            // Push initial state
            PUSH!(emitter.states, YamlEmitDocumentContentState);

            // Allocate indents stack
            let indents_capacity = 4_usize;
            let indents_buf = yaml_malloc(
                (size_of::<libc::c_int>() * indents_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut libc::c_int;
            assert!(!indents_buf.is_null());

            emitter.indents.start = indents_buf;
            emitter.indents.top = indents_buf;
            emitter.indents.end = indents_buf.add(indents_capacity);

            // Set initial indent and push it onto stack
            emitter.indent = 2;
            PUSH!(emitter.indents, emitter.indent);

            // Set up emitter state
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            emitter.best_indent = 2;
            emitter.best_width = 80;
            emitter.column = 0;
            emitter.whitespace = true;
            emitter.indention = true;
            emitter.canonical = false;

            // Set up scalar data in emitter
            let content = b"test scalar\0";
            emitter.scalar_data.value =
                content.as_ptr() as *mut yaml_char_t;
            emitter.scalar_data.length = 11;
            emitter.scalar_data.style = YamlPlainScalarStyle;
            emitter.scalar_data.multiline = false;
            emitter.scalar_data.flow_plain_allowed = true;
            emitter.scalar_data.block_plain_allowed = true;
            emitter.scalar_data.single_quoted_allowed = true;
            emitter.scalar_data.block_allowed = true;

            // Create scalar event
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlScalarEvent;
            event.data.scalar.value =
                content.as_ptr() as *mut yaml_char_t;
            event.data.scalar.length = 11;
            event.data.scalar.style = YamlPlainScalarStyle;
            event.data.scalar.plain_implicit = true;
            event.data.scalar.quoted_implicit = true;

            // Initialize empty tag and anchor data
            emitter.tag_data.handle = null_mut();
            emitter.tag_data.suffix = null_mut();
            emitter.anchor_data.anchor = null_mut();
            emitter.anchor_data.anchor_length = 0;

            // Call function under test
            let result =
                yaml_emitter_emit_scalar(&mut emitter, &mut event);
            assert!(
                !result.fail,
                "Expected scalar emission to succeed"
            );

            // Cleanup
            yaml_free(indents_buf as *mut libc::c_void);
            yaml_free(states_buf as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_sequence_start() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Allocate main buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Allocate states stack
            let states_capacity = 4_usize;
            let states_buf = yaml_malloc(
                (size_of::<YamlEmitterStateT>() * states_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEmitterStateT;
            assert!(!states_buf.is_null());

            emitter.states.start = states_buf;
            emitter.states.top = states_buf;
            emitter.states.end = states_buf.add(states_capacity);

            // Push initial state
            PUSH!(emitter.states, YamlEmitDocumentContentState);

            // Allocate events queue
            let events_capacity = 4_usize;
            let events_buf = yaml_malloc(
                (size_of::<YamlEventT>() * events_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEventT;
            assert!(!events_buf.is_null());

            emitter.events.start = events_buf;
            emitter.events.head = events_buf;
            emitter.events.tail = events_buf;
            emitter.events.end = events_buf.add(events_capacity);

            // Set up emitter state
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            emitter.best_indent = 2;
            emitter.best_width = 80;
            emitter.column = 0;
            emitter.whitespace = true;
            emitter.indention = true;
            emitter.canonical = false;
            emitter.flow_level = 0;

            // Initialize empty tag and anchor data
            emitter.tag_data.handle = null_mut();
            emitter.tag_data.suffix = null_mut();
            emitter.anchor_data.anchor = null_mut();
            emitter.anchor_data.anchor_length = 0;

            // Create sequence start event
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlSequenceStartEvent;
            event.data.sequence_start.anchor = null_mut();
            event.data.sequence_start.tag = null_mut();
            event.data.sequence_start.implicit = true;
            event.data.sequence_start.style = YamlFlowSequenceStyle;

            // Push a sequence end event for the empty sequence check
            let mut end_event: YamlEventT = zeroed();
            end_event.type_ = YamlSequenceEndEvent;
            ENQUEUE!(emitter.events, end_event);

            // Call function under test
            let result = yaml_emitter_emit_sequence_start(
                &mut emitter,
                &mut event,
            );
            assert!(
                !result.fail,
                "Expected sequence start emission to succeed"
            );

            // Cleanup
            yaml_free(events_buf as *mut libc::c_void);
            yaml_free(states_buf as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_mapping_start() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Allocate main buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Allocate states stack
            let states_capacity = 4_usize;
            let states_buf = yaml_malloc(
                (size_of::<YamlEmitterStateT>() * states_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEmitterStateT;
            assert!(!states_buf.is_null());

            emitter.states.start = states_buf;
            emitter.states.top = states_buf;
            emitter.states.end = states_buf.add(states_capacity);

            // Push initial state
            PUSH!(emitter.states, YamlEmitDocumentContentState);

            // Allocate events queue
            let events_capacity = 4_usize;
            let events_buf = yaml_malloc(
                (size_of::<YamlEventT>() * events_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEventT;
            assert!(!events_buf.is_null());

            emitter.events.start = events_buf;
            emitter.events.head = events_buf;
            emitter.events.tail = events_buf;
            emitter.events.end = events_buf.add(events_capacity);

            // Set up emitter state
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            emitter.best_indent = 2;
            emitter.best_width = 80;
            emitter.column = 0;
            emitter.whitespace = true;
            emitter.indention = true;
            emitter.canonical = false;
            emitter.flow_level = 0;

            // Initialize empty tag and anchor data
            emitter.tag_data.handle = null_mut();
            emitter.tag_data.suffix = null_mut();
            emitter.anchor_data.anchor = null_mut();
            emitter.anchor_data.anchor_length = 0;

            // Create mapping start event
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlMappingStartEvent;
            event.data.mapping_start.anchor = null_mut();
            event.data.mapping_start.tag = null_mut();
            event.data.mapping_start.implicit = true;
            event.data.mapping_start.style = YamlFlowMappingStyle;

            // Push a mapping end event for the empty mapping check
            let mut end_event: YamlEventT = zeroed();
            end_event.type_ = YamlMappingEndEvent;
            ENQUEUE!(emitter.events, end_event);

            // Call function under test
            let result = yaml_emitter_emit_mapping_start(
                &mut emitter,
                &mut event,
            );
            assert!(
                !result.fail,
                "Expected mapping start emission to succeed"
            );

            // Cleanup
            yaml_free(events_buf as *mut libc::c_void);
            yaml_free(states_buf as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_check_empty_document() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            let is_empty =
                yaml_emitter_check_empty_document(&mut emitter);
            assert!(
                !is_empty,
                "Expected function to return false by default"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_check_empty_sequence() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            let is_empty =
                yaml_emitter_check_empty_sequence(&mut emitter);

            assert!(
                !is_empty,
                "Expected empty sequence to be false in naive test"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_check_empty_mapping() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            let is_empty =
                yaml_emitter_check_empty_mapping(&mut emitter);
            assert!(
                !is_empty,
                "Expected empty mapping to be false in naive test"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_check_simple_key() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            let is_simple = yaml_emitter_check_simple_key(&mut emitter);
            assert!(
                !is_simple,
                "Expected no simple key in naive empty test"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_select_scalar_style() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Set up scalar data in emitter
            let content = b"test scalar content\0";
            emitter.scalar_data.value =
                content.as_ptr() as *mut yaml_char_t;
            emitter.scalar_data.length = 18;
            emitter.scalar_data.multiline = false;
            emitter.scalar_data.flow_plain_allowed = true;
            emitter.scalar_data.block_plain_allowed = true;
            emitter.scalar_data.single_quoted_allowed = true;
            emitter.scalar_data.block_allowed = true;

            // Initialize empty tag data
            emitter.tag_data.handle = null_mut();
            emitter.tag_data.suffix = null_mut();
            emitter.tag_data.handle_length = 0;
            emitter.tag_data.suffix_length = 0;

            // Set up emitter flags
            emitter.canonical = false;
            emitter.simple_key_context = false;
            emitter.flow_level = 0;

            // Create scalar event
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlScalarEvent;
            event.data.scalar.value =
                content.as_ptr() as *mut yaml_char_t;
            event.data.scalar.length = 18;
            event.data.scalar.style = YamlAnyScalarStyle;
            event.data.scalar.plain_implicit = true;
            event.data.scalar.quoted_implicit = true;

            // Call function under test
            let result = yaml_emitter_select_scalar_style(
                &mut emitter,
                &mut event,
            );
            assert!(
                !result.fail,
                "Expected scalar style selection to succeed"
            );

            // Verify the style was selected
            assert!(
                emitter.scalar_data.style != YamlAnyScalarStyle,
                "Expected a specific scalar style to be selected"
            );
        }
    }
    #[test]
    fn test_yaml_emitter_process_anchor() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            // If there's no anchor, function returns OK
            let result = yaml_emitter_process_anchor(&mut emitter);
            assert!(
                !result.fail,
                "Expected anchor process to succeed (none set)"
            );
        }
    }
    #[test]
    fn test_yaml_emitter_process_tag() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let result = yaml_emitter_process_tag(&mut emitter);
            assert!(
                !result.fail,
                "Expected tag process to succeed (none set)"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_process_scalar() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Allocate main buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Set up scalar data
            let content = b"test scalar\0";
            emitter.scalar_data.value =
                content.as_ptr() as *mut yaml_char_t;
            emitter.scalar_data.length = 11;
            emitter.scalar_data.style = YamlPlainScalarStyle;
            emitter.scalar_data.multiline = false;
            emitter.scalar_data.flow_plain_allowed = true;
            emitter.scalar_data.block_plain_allowed = true;
            emitter.scalar_data.single_quoted_allowed = true;
            emitter.scalar_data.block_allowed = true;

            // Set up emitter state
            emitter.column = 0;
            emitter.whitespace = true;
            emitter.indention = true;
            emitter.simple_key_context = false;
            emitter.flow_level = 0;
            emitter.best_width = 80;
            emitter.best_indent = 2;

            // Call function under test
            let result = yaml_emitter_process_scalar(&mut emitter);
            assert!(
                !result.fail,
                "Expected scalar processing to succeed"
            );

            // Verify buffer contains scalar content
            let buf_ptr = raw_buf as *mut u8;
            for (i, &byte) in b"test scalar".iter().enumerate() {
                assert_eq!(
                    *buf_ptr.add(i),
                    byte,
                    "Buffer content mismatch"
                );
            }

            // Cleanup
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_analyze_version_directive() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let version_dir =
                YamlVersionDirectiveT { major: 1, minor: 2 };

            let result = yaml_emitter_analyze_version_directive(
                &mut emitter,
                version_dir,
            );
            assert!(
                !result.fail,
                "Expected analyzing version 1.2 to succeed"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_analyze_version_directive_invalid() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Test invalid major version
            let invalid_major =
                YamlVersionDirectiveT { major: 2, minor: 1 };
            let result = yaml_emitter_analyze_version_directive(
                &mut emitter,
                invalid_major,
            );
            assert!(
                result.fail,
                "Expected failure for invalid major version"
            );

            // Test invalid minor version
            let invalid_minor =
                YamlVersionDirectiveT { major: 1, minor: 3 };
            let result = yaml_emitter_analyze_version_directive(
                &mut emitter,
                invalid_minor,
            );
            assert!(
                result.fail,
                "Expected failure for invalid minor version"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_analyze_tag_directive() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let tag_directive = YamlTagDirectiveT {
                handle: b"!foo!\0".as_ptr() as *mut yaml_char_t,
                prefix: b"tag:example.com,2023:\0".as_ptr()
                    as *mut yaml_char_t,
            };

            let result = yaml_emitter_analyze_tag_directive(
                &mut emitter,
                tag_directive,
            );
            assert!(
                !result.fail,
                "Expected analyzing tag directive to succeed"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_analyze_tag_directive_invalid() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Test empty handle
            let empty_handle = YamlTagDirectiveT {
                handle: b"\0".as_ptr() as *mut yaml_char_t,
                prefix: b"tag:example.com,2023:\0".as_ptr()
                    as *mut yaml_char_t,
            };
            let result = yaml_emitter_analyze_tag_directive(
                &mut emitter,
                empty_handle,
            );
            assert!(result.fail, "Expected failure for empty handle");

            // Test handle without starting !
            let invalid_handle = YamlTagDirectiveT {
                handle: b"invalid!\0".as_ptr() as *mut yaml_char_t,
                prefix: b"tag:example.com,2023:\0".as_ptr()
                    as *mut yaml_char_t,
            };
            let result = yaml_emitter_analyze_tag_directive(
                &mut emitter,
                invalid_handle,
            );
            assert!(
                result.fail,
                "Expected failure for handle without starting !"
            );

            // Test handle without ending !
            let invalid_end = YamlTagDirectiveT {
                handle: b"!invalid\0".as_ptr() as *mut yaml_char_t,
                prefix: b"tag:example.com,2023:\0".as_ptr()
                    as *mut yaml_char_t,
            };
            let result = yaml_emitter_analyze_tag_directive(
                &mut emitter,
                invalid_end,
            );
            assert!(
                result.fail,
                "Expected failure for handle without ending !"
            );

            // Test handle with non-alphanumeric characters
            let invalid_chars = YamlTagDirectiveT {
                handle: b"!inv@lid!\0".as_ptr() as *mut yaml_char_t,
                prefix: b"tag:example.com,2023:\0".as_ptr()
                    as *mut yaml_char_t,
            };
            let result = yaml_emitter_analyze_tag_directive(
                &mut emitter,
                invalid_chars,
            );
            assert!(
                result.fail,
                "Expected failure for handle with invalid characters"
            );

            // Test empty prefix
            let empty_prefix = YamlTagDirectiveT {
                handle: b"!valid!\0".as_ptr() as *mut yaml_char_t,
                prefix: b"\0".as_ptr() as *mut yaml_char_t,
            };
            let result = yaml_emitter_analyze_tag_directive(
                &mut emitter,
                empty_prefix,
            );
            assert!(result.fail, "Expected failure for empty prefix");
        }
    }

    #[test]
    fn test_yaml_emitter_analyze_anchor() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let anchor = b"myAnchor\0";
            let result = yaml_emitter_analyze_anchor(
                &mut emitter,
                anchor.as_ptr() as *mut yaml_char_t,
                false,
            );
            assert!(
                !result.fail,
                "Expected analyzing anchor to succeed"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_analyze_anchor_invalid() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Test empty anchor
            let empty_anchor = b"\0";
            let result = yaml_emitter_analyze_anchor(
                &mut emitter,
                empty_anchor.as_ptr() as *mut yaml_char_t,
                false,
            );
            assert!(result.fail, "Expected failure for empty anchor");

            // Test invalid characters in anchor
            let invalid_anchor = b"my@nchor\0";
            let result = yaml_emitter_analyze_anchor(
                &mut emitter,
                invalid_anchor.as_ptr() as *mut yaml_char_t,
                false,
            );
            assert!(
                result.fail,
                "Expected failure for invalid anchor characters"
            );

            // Test empty alias
            let result = yaml_emitter_analyze_anchor(
                &mut emitter,
                b"\0".as_ptr() as *mut yaml_char_t,
                true,
            );
            assert!(result.fail, "Expected failure for empty alias");
        }
    }

    #[test]
    fn test_yaml_emitter_analyze_tag() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let tag = b"!foo!\0";
            let result = yaml_emitter_analyze_tag(
                &mut emitter,
                tag.as_ptr() as *mut yaml_char_t,
            );
            assert!(!result.fail, "Expected analyzing tag to succeed");
        }
    }

    #[test]
    fn test_yaml_emitter_analyze_tag_variations() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Setup tag directives (needed for prefix matching test)
            let tag_directives_capacity = 4_usize;
            let tag_directives_buf = yaml_malloc(
                (size_of::<YamlTagDirectiveT>()
                    * tag_directives_capacity)
                    .try_into()
                    .unwrap(),
            )
                as *mut YamlTagDirectiveT;
            assert!(!tag_directives_buf.is_null());

            emitter.tag_directives.start = tag_directives_buf;
            emitter.tag_directives.top = tag_directives_buf;
            emitter.tag_directives.end =
                tag_directives_buf.add(tag_directives_capacity);

            // Add a test directive
            let test_directive = YamlTagDirectiveT {
                handle: b"!test!\0".as_ptr() as *mut yaml_char_t,
                prefix: b"tag:test.org,2023:\0".as_ptr()
                    as *mut yaml_char_t,
            };
            *emitter.tag_directives.top = test_directive;
            emitter.tag_directives.top =
                emitter.tag_directives.top.add(1);

            // Test empty tag
            let result = yaml_emitter_analyze_tag(
                &mut emitter,
                b"\0".as_ptr() as *mut yaml_char_t,
            );
            assert!(result.fail, "Expected failure for empty tag");

            // Test tag with matching prefix
            let tag_with_prefix = b"tag:test.org,2023:type\0";
            let result = yaml_emitter_analyze_tag(
                &mut emitter,
                tag_with_prefix.as_ptr() as *mut yaml_char_t,
            );
            assert!(
                !result.fail,
                "Expected success for tag with matching prefix"
            );
            assert!(
                !emitter.tag_data.handle.is_null(),
                "Handle should be set for matching prefix"
            );

            // Test tag without matching prefix
            let tag_without_prefix = b"tag:other.org,2023:type\0";
            let result = yaml_emitter_analyze_tag(
                &mut emitter,
                tag_without_prefix.as_ptr() as *mut yaml_char_t,
            );
            assert!(
                !result.fail,
                "Expected success for tag without matching prefix"
            );
            assert!(
                !emitter.tag_data.suffix.is_null(),
                "Suffix should be set for non-matching prefix"
            );

            // Cleanup
            yaml_free(tag_directives_buf as *mut libc::c_void);
        }
    }

    #[test]
    fn test_yaml_emitter_analyze_scalar() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let value = b"test value\0";
            let length = 10;
            let result = yaml_emitter_analyze_scalar(
                &mut emitter,
                value.as_ptr() as *mut yaml_char_t,
                length,
            );
            assert!(
                !result.fail,
                "Expected analyzing scalar to succeed"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_analyze_scalar_special_cases() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            emitter.unicode = false; // Test non-unicode mode

            // Test empty scalar
            let result = yaml_emitter_analyze_scalar(
                &mut emitter,
                b"\0".as_ptr() as *mut yaml_char_t,
                0,
            );
            assert!(
                !result.fail,
                "Empty scalar analysis should succeed"
            );
            assert!(
                !emitter.scalar_data.multiline,
                "Empty scalar shouldn't be multiline"
            );
            assert!(
                emitter.scalar_data.block_plain_allowed,
                "Empty scalar should allow block plain"
            );

            // Test block indicators
            let result = yaml_emitter_analyze_scalar(
                &mut emitter,
                b"---\0".as_ptr() as *mut yaml_char_t,
                3,
            );
            assert!(
                !result.fail,
                "Block indicators analysis should succeed"
            );
            assert!(!emitter.scalar_data.flow_plain_allowed || !emitter.scalar_data.block_plain_allowed,
               "Plain style should be restricted with block indicators");

            // Test multiline with breaks and spaces
            let result = yaml_emitter_analyze_scalar(
                &mut emitter,
                b"line1\n line2\0".as_ptr() as *mut yaml_char_t,
                11,
            );
            assert!(!result.fail, "Multiline analysis should succeed");
            assert!(
                emitter.scalar_data.multiline,
                "Should detect multiline content"
            );
            assert!(
                !emitter.scalar_data.flow_plain_allowed,
                "Flow plain should not be allowed with line breaks"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_analyze_scalar_special_characters() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            emitter.unicode = false;

            // Test scalar with flow indicators
            let result = yaml_emitter_analyze_scalar(
                &mut emitter,
                b"[test]\0".as_ptr() as *mut yaml_char_t,
                6,
            );
            assert!(
                !result.fail,
                "Flow indicators analysis should succeed"
            );
            assert!(
                !emitter.scalar_data.flow_plain_allowed,
                "Flow plain should not be allowed with flow indicators"
            );

            // Test scalar with special characters
            let result = yaml_emitter_analyze_scalar(
                &mut emitter,
                b"key: value\0".as_ptr() as *mut yaml_char_t,
                10,
            );
            assert!(
                !result.fail,
                "Special characters analysis should succeed"
            );
            assert!(!emitter.scalar_data.flow_plain_allowed || !emitter.scalar_data.block_plain_allowed,
               "Plain style should be restricted with special characters");
        }
    }

    #[test]
    fn test_yaml_emitter_write_indent() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            emitter.indent = 2;
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;

            let result = yaml_emitter_write_indent(&mut emitter);
            assert!(!result.fail, "Expected write_indent to succeed");

            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_write_indicator() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;

            let indicator = b"---\0";
            let result = yaml_emitter_write_indicator(
                &mut emitter,
                indicator.as_ptr() as *const libc::c_char,
                true,
                false,
                false,
            );
            assert!(
                !result.fail,
                "Expected writing indicator to succeed"
            );

            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_write_indicator_whitespace() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Setup buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Test with need_whitespace=true and whitespace=false
            emitter.whitespace = false;
            emitter.indention = true;
            let indicator = b"---\0";
            let result = yaml_emitter_write_indicator(
                &mut emitter,
                indicator.as_ptr() as *const libc::c_char,
                true,  // need_whitespace
                false, // is_whitespace
                true,  // is_indention
            );
            assert!(
                !result.fail,
                "Expected success with need_whitespace=true"
            );
            assert!(
                !emitter.whitespace,
                "Whitespace flag should not be set"
            );
            assert!(
                emitter.indention,
                "Indention flag should remain set"
            );

            // Test with need_whitespace=true and is_whitespace=true
            emitter.buffer.pointer = emitter.buffer.start; // Reset pointer
            emitter.whitespace = false;
            emitter.indention = true;
            let result = yaml_emitter_write_indicator(
                &mut emitter,
                indicator.as_ptr() as *const libc::c_char,
                true, // need_whitespace
                true, // is_whitespace
                true, // is_indention
            );
            assert!(!result.fail, "Expected success with need_whitespace=true and is_whitespace=true");
            assert!(
                emitter.whitespace,
                "Whitespace flag should be set"
            );
            assert!(
                emitter.indention,
                "Indention flag should remain set"
            );

            // Test preserving indention state
            emitter.buffer.pointer = emitter.buffer.start; // Reset pointer
            emitter.whitespace = true;
            emitter.indention = false;
            let result = yaml_emitter_write_indicator(
                &mut emitter,
                indicator.as_ptr() as *const libc::c_char,
                false, // need_whitespace
                false, // is_whitespace
                false, // is_indention
            );
            assert!(
                !result.fail,
                "Expected success with indention preservation"
            );
            assert!(
                !emitter.whitespace,
                "Whitespace flag should not be set"
            );
            assert!(
                !emitter.indention,
                "Indention flag should remain unset"
            );

            // Cleanup
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_write_anchor() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            let anchor = b"myAnchor\0";
            let length = 8;
            let result = yaml_emitter_write_anchor(
                &mut emitter,
                anchor.as_ptr() as *mut yaml_char_t,
                length,
            );
            assert!(!result.fail, "Expected anchor writing to succeed");

            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_write_tag_handle() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            let handle = b"!foo!\0";
            let length = 5;
            let result = yaml_emitter_write_tag_handle(
                &mut emitter,
                handle.as_ptr() as *mut yaml_char_t,
                length,
            );
            assert!(
                !result.fail,
                "Expected writing a tag handle to succeed"
            );

            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_write_tag_content() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            let content = b"tag:example.com,2023:\0";
            let length = 20;
            let result = yaml_emitter_write_tag_content(
                &mut emitter,
                content.as_ptr() as *mut yaml_char_t,
                length,
                true,
            );
            assert!(
                !result.fail,
                "Expected tag content write to succeed"
            );

            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_write_plain_scalar() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            let value = b"some plain text\0";
            let length = 15;
            let result = yaml_emitter_write_plain_scalar(
                &mut emitter,
                value.as_ptr() as *mut yaml_char_t,
                length,
                true,
            );
            assert!(
                !result.fail,
                "Expected plain scalar write to succeed"
            );

            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_write_single_quoted_scalar() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            let value = b"single quoted text\0";
            let length = 19;
            let result = yaml_emitter_write_single_quoted_scalar(
                &mut emitter,
                value.as_ptr() as *mut yaml_char_t,
                length,
                true,
            );
            assert!(
                !result.fail,
                "Expected single-quoted scalar to succeed"
            );

            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_write_double_quoted_scalar() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            let value = b"double \"quoted\" text\n\0";
            let length = 22;
            let result = yaml_emitter_write_double_quoted_scalar(
                &mut emitter,
                value.as_ptr() as *mut yaml_char_t,
                length,
                true,
            );
            assert!(
                !result.fail,
                "Expected double-quoted scalar to succeed"
            );

            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_write_block_scalar_hints() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            let test_string = YamlStringT {
                start: b"\n".as_ptr() as *mut yaml_char_t,
                end: b"\n".as_ptr().add(1) as *mut yaml_char_t,
                pointer: b"\n".as_ptr() as *mut yaml_char_t,
            };

            let result = yaml_emitter_write_block_scalar_hints(
                &mut emitter,
                test_string,
            );
            assert!(
                !result.fail,
                "Expected block scalar hints write to succeed"
            );

            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_write_literal_scalar() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            let value = b"multi\nline\nliteral\n\0";
            let length = 19;
            let result = yaml_emitter_write_literal_scalar(
                &mut emitter,
                value.as_ptr() as *mut yaml_char_t,
                length,
            );
            assert!(
                !result.fail,
                "Expected literal scalar write to succeed"
            );

            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_write_folded_scalar() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());

            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            let value = b"multi\nline\nfolded\n\0";
            let length = 18;
            let result = yaml_emitter_write_folded_scalar(
                &mut emitter,
                value.as_ptr() as *mut yaml_char_t,
                length,
            );
            assert!(
                !result.fail,
                "Expected folded scalar write to succeed"
            );

            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_document_content() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Allocate main buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Allocate states stack
            let states_capacity = 4_usize;
            let states_buf = yaml_malloc(
                (size_of::<YamlEmitterStateT>() * states_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEmitterStateT;
            assert!(!states_buf.is_null());

            emitter.states.start = states_buf;
            emitter.states.top = states_buf;
            emitter.states.end = states_buf.add(states_capacity);

            // Allocate indents stack
            let indents_capacity = 4_usize;
            let indents_buf = yaml_malloc(
                (size_of::<libc::c_int>() * indents_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut libc::c_int;
            assert!(!indents_buf.is_null());

            emitter.indents.start = indents_buf;
            emitter.indents.top = indents_buf;
            emitter.indents.end = indents_buf.add(indents_capacity);

            // Setup scalar data
            let content = b"test content\0";
            emitter.scalar_data.value =
                content.as_ptr() as *mut yaml_char_t;
            emitter.scalar_data.length = 12;
            emitter.scalar_data.style = YamlPlainScalarStyle;
            emitter.scalar_data.multiline = false;
            emitter.scalar_data.flow_plain_allowed = true;
            emitter.scalar_data.block_plain_allowed = true;
            emitter.scalar_data.single_quoted_allowed = true;
            emitter.scalar_data.block_allowed = true;

            // Set up emitter state
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            emitter.indent = 2;
            emitter.best_indent = 2;
            emitter.best_width = 80;
            emitter.whitespace = true;
            emitter.indention = true;
            emitter.root_context = true;
            emitter.simple_key_context = false;

            // Create scalar event
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlScalarEvent;
            event.data.scalar.value =
                content.as_ptr() as *mut yaml_char_t;
            event.data.scalar.length = 12;
            event.data.scalar.style = YamlPlainScalarStyle;
            event.data.scalar.plain_implicit = true;
            event.data.scalar.quoted_implicit = true;

            // Call function under test
            let result = yaml_emitter_emit_document_content(
                &mut emitter,
                &mut event,
            );
            assert!(
                !result.fail,
                "Expected document_content emission to succeed"
            );

            // Cleanup
            yaml_free(indents_buf as *mut libc::c_void);
            yaml_free(states_buf as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }

    #[test]
    fn test_yaml_emitter_emit_document_end_variations() {
        unsafe {
            let mut emitter: YamlEmitterT =
                MaybeUninit::zeroed().assume_init();

            // Setup write handler
            unsafe fn write_handler(
                _data: *mut libc::c_void,
                _buffer: *mut u8,
                _size: size_t,
            ) -> libc::c_int {
                1 // Return success (1 for success, 0 for failure in libyaml)
            }

            emitter.write_handler = Some(write_handler);
            emitter.write_handler_data = null_mut(); // Explicitly set to null

            // Setup buffer
            let capacity = 128_usize;
            let raw_buf = yaml_malloc(capacity as size_t);
            assert!(!raw_buf.is_null());
            emitter.buffer.start = raw_buf as *mut yaml_char_t;
            emitter.buffer.pointer = raw_buf as *mut yaml_char_t;
            emitter.buffer.end =
                (raw_buf as *mut yaml_char_t).add(capacity);

            // Setup states stack
            let states_capacity = 4_usize;
            let states_buf = yaml_malloc(
                (size_of::<YamlEmitterStateT>() * states_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut YamlEmitterStateT;
            assert!(!states_buf.is_null());

            emitter.states.start = states_buf;
            emitter.states.top = states_buf;
            emitter.states.end = states_buf.add(states_capacity);

            // Push initial state
            PUSH!(emitter.states, YamlEmitDocumentStartState);

            // Setup indents stack
            let indents_capacity = 4_usize;
            let indents_buf = yaml_malloc(
                (size_of::<libc::c_int>() * indents_capacity)
                    .try_into()
                    .unwrap(),
            ) as *mut libc::c_int;
            assert!(!indents_buf.is_null());

            emitter.indents.start = indents_buf;
            emitter.indents.top = indents_buf;
            emitter.indents.end = indents_buf.add(indents_capacity);

            // Push initial indent
            PUSH!(emitter.indents, 0);

            // Setup tag directives stack
            let tag_directives_capacity = 4_usize;
            let tag_directives_buf = yaml_malloc(
                (size_of::<YamlTagDirectiveT>()
                    * tag_directives_capacity)
                    .try_into()
                    .unwrap(),
            )
                as *mut YamlTagDirectiveT;
            assert!(!tag_directives_buf.is_null());

            emitter.tag_directives.start = tag_directives_buf;
            emitter.tag_directives.top = tag_directives_buf;
            emitter.tag_directives.end =
                tag_directives_buf.add(tag_directives_capacity);

            // Required emitter state
            emitter.state = YamlEmitDocumentEndState;
            emitter.encoding = YamlUtf8Encoding;
            emitter.line_break = YamlLnBreak;
            emitter.indent = 2;
            emitter.best_indent = 2;
            emitter.best_width = 80;
            emitter.whitespace = true;
            emitter.indention = true;
            emitter.column = 0;
            emitter.line = 0;
            emitter.root_context = false;
            emitter.sequence_context = false;
            emitter.mapping_context = false;
            emitter.simple_key_context = false;
            emitter.flow_level = 0;

            // Test with implicit document end
            let mut event: YamlEventT = zeroed();
            event.type_ = YamlDocumentEndEvent;
            event.data.document_end.implicit = true;
            emitter.open_ended = 0;

            let result = yaml_emitter_emit_document_end(
                &mut emitter,
                &mut event,
            );
            if result.fail {
                assert!(
                    !result.fail,
                    "Failed with implicit end. Error: {}",
                    if !emitter.problem.is_null() {
                        CStr::from_ptr(emitter.problem)
                            .to_string_lossy()
                    } else {
                        "Unknown error".into()
                    }
                );
            }
            assert_eq!(
                emitter.open_ended, 1,
                "Open ended should be set to 1 for implicit end"
            );

            // Test with explicit document end
            emitter.buffer.pointer = emitter.buffer.start;
            event.data.document_end.implicit = false;

            let result = yaml_emitter_emit_document_end(
                &mut emitter,
                &mut event,
            );
            assert!(!result.fail, "Expected success with explicit end");
            assert_eq!(
                emitter.open_ended, 0,
                "Open ended should be 0 for explicit end"
            );

            // Test with wrong event type
            event.type_ = YamlScalarEvent;
            let result = yaml_emitter_emit_document_end(
                &mut emitter,
                &mut event,
            );
            assert!(
                result.fail,
                "Expected failure with wrong event type"
            );

            // Cleanup
            yaml_free(indents_buf as *mut libc::c_void);
            yaml_free(states_buf as *mut libc::c_void);
            yaml_free(tag_directives_buf as *mut libc::c_void);
            yaml_free(raw_buf);
        }
    }
}
