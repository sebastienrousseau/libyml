// src/loader/parsing.rs

use crate::YamlEventTypeT::YamlDocumentEndEvent;
use crate::{
    internal::yaml_stack_extend,
    libc,
    loader::{
        error_handling::{yaml_parser_set_error, YamlError},
        initialize_yaml_node, yaml_parser_delete_aliases, ParserState,
    },
    memory::{yaml_free, yaml_malloc, yaml_strdup},
    success::FAIL,
    success::OK,
    yaml::yaml_char_t,
    yaml_document_delete, yaml_parser_parse, YamlAliasDataT,
    YamlDocumentT,
    YamlErrorTypeT::YamlMemoryError,
    YamlEventT,
    YamlEventTypeT::{
        YamlDocumentStartEvent, YamlStreamEndEvent,
        YamlStreamStartEvent,
    },
    YamlNodeItemT, YamlNodePairT, YamlNodeT,
    YamlNodeTypeT::YamlMappingNode,
    YamlNodeTypeT::YamlScalarNode,
    YamlNodeTypeT::YamlSequenceNode,
    YamlParserT,
};
use core::ffi::c_char;
use core::{
    mem::{size_of, MaybeUninit},
    ptr::{self, addr_of_mut},
};

/// Struct representing the loader context.
#[repr(C)]
struct LoaderContext {
    start: *mut i32,
    end: *mut i32,
    top: *mut i32,
}

/// Parses the input stream and produces the next YAML document.
///
/// Call this function subsequently to produce a sequence of documents
/// constituting the input stream.
///
/// If the produced document has no root node, it means that the document end
/// has been reached.
///
/// An application is responsible for freeing any data associated with the
/// produced document object using the `yaml_document_delete()` function.
///
/// An application must not alternate the calls of `yaml_parser_load()` with the
/// calls of `yaml_parser_scan()` or `yaml_parser_parse()`. Doing this will break
/// the parser.
///
/// # Safety
///
/// - `parser` must be a valid, non-null pointer to a properly initialized `YamlParserT` struct.
/// - `document` must be a valid, non-null pointer to a `YamlDocumentT` struct that can be safely written to.
/// - The `YamlParserT` and `YamlDocumentT` structs must be properly aligned and have the expected memory layout.
/// - The caller must call `yaml_document_delete` to free any data associated with the produced document object.
/// - The caller must not alternate calls to `yaml_parser_load` with calls to `yaml_parser_scan` or `yaml_parser_parse` on the same `YamlParserT` instance.
pub unsafe fn yaml_parser_load(
    parser: *mut YamlParserT,
    document: *mut YamlDocumentT,
) -> Result<(), YamlError> {
    if parser.is_null() || document.is_null() {
        return Err(YamlError::NullPointer);
    }

    // Initialize the document memory safely
    ptr::write_bytes(
        document as *mut u8,
        0,
        size_of::<YamlDocumentT>(),
    );
    STACK_INIT!((*document).nodes, YamlNodeT);

    let mut event = MaybeUninit::<YamlEventT>::uninit();
    let event_ptr = event.as_mut_ptr();

    if !(*parser).stream_start_produced {
        if yaml_parser_parse(parser, event_ptr).fail {
            return Err(YamlError::InvalidEventType);
        }
        assert!(
            (*event_ptr).type_ == YamlStreamStartEvent,
            "Expected YamlStreamStartEvent"
        );
    }

    if (*parser).stream_end_produced {
        return Ok(());
    }

    if yaml_parser_parse(parser, event_ptr) == OK {
        if (*event_ptr).type_ == YamlStreamEndEvent {
            return Ok(());
        }
        STACK_INIT!((*parser).aliases, YamlAliasDataT);
        (*parser).document = document;
        if yaml_parser_load_document(parser, event_ptr).is_ok() {
            yaml_parser_delete_aliases(parser);
            (*parser).document = ptr::null_mut::<YamlDocumentT>();
            return Ok(());
        }
    }

    yaml_parser_delete_aliases(parser);
    yaml_document_delete(document);
    (*parser).document = ptr::null_mut::<YamlDocumentT>();
    Err(YamlError::MemoryAllocationFailed)
}

/// Loads a YAML document based on the provided event.
///
/// # Arguments
///
/// * `parser` - A mutable pointer to the `YamlParserT` struct.
/// * `event` - A mutable pointer to the `YamlEventT` struct.
///
/// # Returns
///
/// * `Result<(), YamlError>` indicating the outcome of the operation.
///
/// # Safety
///
/// - All pointers must be valid and properly initialized.
unsafe fn yaml_parser_load_document(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
) -> Result<(), YamlError> {
    let mut ctx = LoaderContext {
        start: ptr::null_mut::<i32>(),
        end: ptr::null_mut::<i32>(),
        top: ptr::null_mut::<i32>(),
    };

    // Verify we have a valid document start
    if (*event).type_ != YamlDocumentStartEvent {
        return yaml_parser_set_error(
            parser,
            None,
            b"Expected document start event\0" as *const u8
                as *const c_char,
            (*event).start_mark,
        )
        .and(Err(YamlError::InvalidDocumentStructure));
    }

    let document = (*parser).document;

    // Initialize version directive and tag directives
    (*document).version_directive =
        (*event).data.document_start.version_directive;
    (*document).tag_directives.start =
        (*event).data.document_start.tag_directives.start;
    (*document).tag_directives.end =
        (*event).data.document_start.tag_directives.end;
    (*document).start_implicit = (*event).data.document_start.implicit;
    (*document).start_mark = (*event).start_mark;

    STACK_INIT!(ctx, i32);
    if yaml_parser_load_nodes(parser, addr_of_mut!(ctx)).is_err() {
        STACK_DEL!(ctx);
        return Err(YamlError::InvalidDocumentStructure);
    }

    // Verify document end marker
    let mut end_event = MaybeUninit::<YamlEventT>::uninit();
    let end_event_ptr = end_event.as_mut_ptr();

    if yaml_parser_parse(parser, end_event_ptr).fail {
        STACK_DEL!(ctx);
        return yaml_parser_set_error(
            parser,
            None,
            b"Failed to parse document end\0" as *const u8
                as *const c_char,
            (*event).end_mark,
        )
        .and(Err(YamlError::InvalidDocumentStructure));
    }

    if (*end_event_ptr).type_ != YamlDocumentEndEvent {
        STACK_DEL!(ctx);
        return yaml_parser_set_error(
            parser,
            None,
            b"Expected document end event\0" as *const u8
                as *const c_char,
            (*end_event_ptr).start_mark,
        )
        .and(Err(YamlError::InvalidDocumentStructure));
    }

    (*document).end_implicit =
        (*end_event_ptr).data.document_end.implicit;
    (*document).end_mark = (*end_event_ptr).end_mark;

    STACK_DEL!(ctx);
    Ok(())
}

/// Loads YAML nodes based on parser events.
///
/// # Arguments
///
/// * `parser` - A mutable pointer to the `YamlParserT` struct.
/// * `ctx` - A mutable pointer to the `LoaderContext` struct.
///
/// # Returns
///
/// * `Result<(), YamlError>` indicating the outcome of the operation.
///
/// # Safety
///
/// - All pointers must be valid and properly initialized.
unsafe fn yaml_parser_load_nodes(
    parser: *mut YamlParserT,
    ctx: *mut LoaderContext,
) -> Result<(), YamlError> {
    let mut event = MaybeUninit::<YamlEventT>::uninit();
    let event_ptr = event.as_mut_ptr();

    loop {
        if yaml_parser_parse(parser, event_ptr).fail {
            return Err(YamlError::InvalidEventType);
        }

        let current_state: ParserState = (*event_ptr).type_.into();
        match current_state {
            ParserState::Alias => {
                if yaml_parser_load_alias(parser, event_ptr, ctx)
                    .is_err()
                {
                    return Err(YamlError::UndefinedAlias);
                }
            }
            ParserState::Scalar => {
                if yaml_parser_load_scalar(parser, event_ptr, ctx)
                    .is_err()
                {
                    return Err(YamlError::MemoryAllocationFailed);
                }
            }
            ParserState::SequenceStart => {
                if yaml_parser_load_sequence(parser, event_ptr, ctx)
                    .is_err()
                {
                    return Err(YamlError::InvalidDocumentStructure);
                }
            }
            ParserState::SequenceEnd => {
                if yaml_parser_load_sequence_end(parser, event_ptr, ctx)
                    .is_err()
                {
                    return Err(YamlError::InvalidDocumentStructure);
                }
            }
            ParserState::MappingStart => {
                if yaml_parser_load_mapping(parser, event_ptr, ctx)
                    .is_err()
                {
                    return Err(YamlError::InvalidDocumentStructure);
                }
            }
            ParserState::MappingEnd => {
                if yaml_parser_load_mapping_end(parser, event_ptr, ctx)
                    .is_err()
                {
                    return Err(YamlError::InvalidDocumentStructure);
                }
            }
            ParserState::DocumentEnd => {
                let document = (*parser).document;
                (*document).end_implicit =
                    (*event_ptr).data.document_end.implicit;
                (*document).end_mark = (*event_ptr).end_mark;
                break;
            }
            _ => {
                return Err(YamlError::InvalidEventType);
            }
        }

        if current_state == ParserState::DocumentEnd {
            break;
        }
    }

    Ok(())
}

/// Registers an anchor in the parser.
///
/// # Arguments
///
/// * `parser` - A mutable pointer to the `YamlParserT` struct.
/// * `index` - The index of the node.
/// * `anchor` - A pointer to the YAML character array representing the anchor.
///
/// # Returns
///
/// * `Result<(), YamlError>` indicating the outcome of the operation.
///
/// # Safety
///
/// - All pointers must be valid and properly initialized.
unsafe fn yaml_parser_register_anchor(
    parser: *mut YamlParserT,
    index: i32,
    anchor: *mut yaml_char_t,
) -> Result<(), YamlError> {
    if anchor.is_null() {
        return Ok(());
    }

    let mut data = MaybeUninit::<YamlAliasDataT>::uninit();
    let data_ptr = data.as_mut_ptr();
    (*data_ptr).anchor = anchor;
    (*data_ptr).index = index;

    let node_ptr = (*(*parser).document)
        .nodes
        .start
        .offset((index - 1) as isize);
    let node = &*node_ptr;
    (*data_ptr).mark = node.start_mark;

    let mut alias_data = (*parser).aliases.start;
    while alias_data != (*parser).aliases.top {
        if strcmp(
            (*alias_data).anchor as *mut c_char,
            anchor as *mut c_char,
        ) == 0
        {
            yaml_free(anchor as *mut libc::c_void);

            return yaml_parser_set_error(
                parser,
                Some((
                    b"found duplicate anchor\0" as *const u8
                        as *const c_char,
                    (*alias_data).mark,
                )),
                b"second occurrence\0" as *const u8 as *const c_char,
                (*data_ptr).mark,
            )
            .map(|_| ());
        }
        alias_data = alias_data.offset(1);
    }

    if !PUSH!((*parser).aliases, *data_ptr) {
        return Err(YamlError::MemoryAllocationFailed);
    }
    Ok(())
}

/// Adds a node to the current context.
///
/// # Arguments
///
/// * `parser` - A mutable pointer to the `YamlParserT` struct.
/// * `ctx` - A mutable pointer to the `LoaderContext` struct.
/// * `index` - The index of the node to add.
///
/// # Returns
///
/// * `Result<(), YamlError>` indicating the outcome of the operation.
///
/// # Safety
///
/// - All pointers must be valid and properly initialized.
unsafe fn yaml_parser_load_node_add(
    parser: *mut YamlParserT,
    ctx: *mut LoaderContext,
    index: i32,
) -> Result<(), YamlError> {
    if STACK_EMPTY!(*ctx) {
        return Ok(());
    }

    let parent_index: i32 = *(*ctx).top.offset(-1);
    let parent = &mut *(*(*parser).document)
        .nodes
        .start
        .offset((parent_index - 1) as isize);

    match parent.type_ {
        YamlSequenceNode => {
            if !PUSH!(parent.data.sequence.items, index) {
                return Err(YamlError::MemoryAllocationFailed);
            }
        }
        YamlMappingNode => {
            let mut pair = MaybeUninit::<YamlNodePairT>::uninit();
            let pair_ptr = pair.as_mut_ptr();

            if !STACK_EMPTY!(parent.data.mapping.pairs) {
                let p = &mut *parent.data.mapping.pairs.top.offset(-1);
                if p.key != 0 && p.value == 0 {
                    p.value = index;
                } else {
                    (*pair_ptr).key = index;
                    (*pair_ptr).value = 0;
                    if !PUSH!(parent.data.mapping.pairs, *pair_ptr) {
                        return Err(YamlError::MemoryAllocationFailed);
                    }
                }
            } else {
                (*pair_ptr).key = index;
                (*pair_ptr).value = 0;
                if !PUSH!(parent.data.mapping.pairs, *pair_ptr) {
                    return Err(YamlError::MemoryAllocationFailed);
                }
            }
        }
        _ => {
            return Err(YamlError::InvalidDocumentStructure);
        }
    }

    Ok(())
}

/// Loads an alias event.
///
/// # Arguments
///
/// * `parser` - A mutable pointer to the `YamlParserT` struct.
/// * `event` - A mutable pointer to the `YamlEventT` struct.
/// * `ctx` - A mutable pointer to the `LoaderContext` struct.
///
/// # Returns
///
/// * `Result<(), YamlError>` indicating the outcome of the operation.
///
/// # Safety
///
/// - All pointers must be valid and properly initialized.
unsafe fn yaml_parser_load_alias(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
    ctx: *mut LoaderContext,
) -> Result<(), YamlError> {
    let anchor: *mut yaml_char_t = (*event).data.alias.anchor;
    let mut alias_data = (*parser).aliases.start;

    while alias_data != (*parser).aliases.top {
        if strcmp(
            (*alias_data).anchor as *mut c_char,
            anchor as *mut c_char,
        ) == 0
        {
            yaml_free(anchor as *mut libc::c_void);
            return yaml_parser_load_node_add(
                parser,
                ctx,
                (*alias_data).index,
            );
        }
        alias_data = alias_data.offset(1);
    }

    yaml_free(anchor as *mut libc::c_void);
    yaml_parser_set_error(
        parser,
        None,
        b"found undefined alias\0" as *const u8 as *const c_char,
        (*event).start_mark,
    )
    .map(|_| ())
}

/// Loads a scalar event.
///
/// # Arguments
///
/// * `parser` - A mutable pointer to the `YamlParserT` struct.
/// * `event` - A mutable pointer to the `YamlEventT` struct.
/// * `ctx` - A mutable pointer to the `LoaderContext` struct.
///
/// # Returns
///
/// * `Result<(), YamlError>` indicating the outcome of the operation.
///
/// # Safety
///
/// - All pointers must be valid and properly initialized.
unsafe fn yaml_parser_load_scalar(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
    ctx: *mut LoaderContext,
) -> Result<(), YamlError> {
    let mut node = MaybeUninit::<YamlNodeT>::uninit();
    let node_ptr = node.as_mut_ptr();
    let index: i32;
    let mut tag: *mut yaml_char_t = (*event).data.scalar.tag;

    if STACK_LIMIT!(parser, (*(*parser).document).nodes).ok {
        if tag.is_null()
            || strcmp(
                tag as *mut c_char,
                b"!\0" as *const u8 as *mut c_char,
            ) == 0
        {
            yaml_free(tag as *mut libc::c_void);
            tag = yaml_strdup(
                b"tag:yaml.org,2002:str\0" as *const u8 as *const c_char
                    as *mut yaml_char_t,
            );
            if tag.is_null() {
                return Err(YamlError::MemoryAllocationFailed);
            }
        }

        initialize_yaml_node(node_ptr);
        (*node_ptr).type_ = YamlScalarNode;
        (*node_ptr).tag = tag;
        (*node_ptr).start_mark = (*event).start_mark;
        (*node_ptr).end_mark = (*event).end_mark;
        (*node_ptr).data.scalar.value = (*event).data.scalar.value;
        (*node_ptr).data.scalar.length = (*event).data.scalar.length;
        (*node_ptr).data.scalar.style = (*event).data.scalar.style;

        if !PUSH!((*(*parser).document).nodes, *node_ptr) {
            return Err(YamlError::MemoryAllocationFailed);
        }
        index = (*(*parser).document)
            .nodes
            .top
            .offset_from((*(*parser).document).nodes.start)
            as i32
            + 1;

        yaml_parser_register_anchor(
            parser,
            index,
            (*event).data.scalar.anchor,
        )?;

        yaml_parser_load_node_add(parser, ctx, index)?;
        return Ok(());
    }

    yaml_free(tag as *mut libc::c_void);
    yaml_free((*event).data.scalar.anchor as *mut libc::c_void);
    yaml_free((*event).data.scalar.value as *mut libc::c_void);
    Err(YamlError::MemoryAllocationFailed)
}

/// Loads a sequence start event.
///
/// # Arguments
///
/// * `parser` - A mutable pointer to the `YamlParserT` struct.
/// * `event` - A mutable pointer to the `YamlEventT` struct.
/// * `ctx` - A mutable pointer to the `LoaderContext` struct.
///
/// # Returns
///
/// * `Result<(), YamlError>` indicating the outcome of the operation.
///
/// # Safety
///
/// - All pointers must be valid and properly initialized.
unsafe fn yaml_parser_load_sequence(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
    ctx: *mut LoaderContext,
) -> Result<(), YamlError> {
    let mut node = MaybeUninit::<YamlNodeT>::uninit();
    let node_ptr = node.as_mut_ptr();
    struct Items {
        start: *mut YamlNodeItemT,
        end: *mut YamlNodeItemT,
        top: *mut YamlNodeItemT,
    }
    let mut items = Items {
        start: ptr::null_mut::<YamlNodeItemT>(),
        end: ptr::null_mut::<YamlNodeItemT>(),
        top: ptr::null_mut::<YamlNodeItemT>(),
    };
    let index: i32;
    let mut tag: *mut yaml_char_t = (*event).data.sequence_start.tag;

    if STACK_LIMIT!(parser, (*(*parser).document).nodes).ok {
        if tag.is_null()
            || strcmp(
                tag as *mut c_char,
                b"!\0" as *const u8 as *mut c_char,
            ) == 0
        {
            yaml_free(tag as *mut libc::c_void);
            tag = yaml_strdup(
                b"tag:yaml.org,2002:seq\0" as *const u8 as *const c_char
                    as *mut yaml_char_t,
            );
            if tag.is_null() {
                return Err(YamlError::MemoryAllocationFailed);
            }
        }

        STACK_INIT!(items, YamlNodeItemT);
        initialize_yaml_node(node_ptr);
        (*node_ptr).type_ = YamlSequenceNode;
        (*node_ptr).tag = tag;
        (*node_ptr).start_mark = (*event).start_mark;
        (*node_ptr).end_mark = (*event).end_mark;
        (*node_ptr).data.sequence.items.start = items.start;
        (*node_ptr).data.sequence.items.end = items.end;
        (*node_ptr).data.sequence.items.top = items.start;
        (*node_ptr).data.sequence.style =
            (*event).data.sequence_start.style;

        if !PUSH!((*(*parser).document).nodes, *node_ptr) {
            return Err(YamlError::MemoryAllocationFailed);
        }
        index = (*(*parser).document)
            .nodes
            .top
            .offset_from((*(*parser).document).nodes.start)
            as i32
            + 1;

        yaml_parser_register_anchor(
            parser,
            index,
            (*event).data.sequence_start.anchor,
        )?;

        yaml_parser_load_node_add(parser, ctx, index)?;

        if STACK_LIMIT!(parser, *ctx).fail {
            return Err(YamlError::MemoryAllocationFailed);
        }

        if !PUSH!(*ctx, index) {
            return Err(YamlError::MemoryAllocationFailed);
        }
        return Ok(());
    }

    yaml_free(tag as *mut libc::c_void);
    yaml_free((*event).data.sequence_start.anchor as *mut libc::c_void);
    Err(YamlError::MemoryAllocationFailed)
}

/// Deletes the loading of a sequence.
///
/// # Arguments
///
/// * `parser` - A mutable pointer to the `YamlParserT` struct.
/// * `event` - A mutable pointer to the `YamlEventT` struct.
/// * `ctx` - A mutable pointer to the `LoaderContext` struct.
///
/// # Returns
///
/// * `Result<(), YamlError>` indicating the outcome of the operation.
///
/// # Safety
///
/// - All pointers must be valid and properly initialized.
unsafe fn yaml_parser_load_sequence_end(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
    ctx: *mut LoaderContext,
) -> Result<(), YamlError> {
    if ctx.is_null() || parser.is_null() || event.is_null() {
        return Err(YamlError::NullPointer);
    }

    // Ensure the stack is not underflowed
    if (*ctx).top.offset_from((*ctx).start) <= 0 {
        return Err(YamlError::InvalidDocumentStructure);
    }

    // Retrieve the index from the top of the stack
    let index: i32 = *(*ctx).top.offset(-1);

    // Dereference the parser's document to access `nodes`
    let document = &*(*parser).document;

    // Calculate the pointer to the desired YamlNodeT
    let node_ptr = document.nodes.start.offset((index - 1) as isize);

    // Dereference the YamlNodeT pointer to access its fields
    let node = &mut *node_ptr;

    // Check if the node type is valid
    if node.type_ != YamlSequenceNode {
        let _ = yaml_parser_set_error(
            parser,
            None,
            b"Expected a sequence node but found a different type\0"
                as *const u8 as *const c_char,
            node.start_mark,
        );
        return Err(YamlError::InvalidDocumentStructure);
    }

    // Set the `end_mark` of the node based on the event
    node.end_mark = (*event).end_mark;

    // Pop the index from the context stack
    POP!(*ctx);

    Ok(())
}

/// Loads a mapping start event.
///
/// # Arguments
///
/// * `parser` - A mutable pointer to the `YamlParserT` struct.
/// * `event` - A mutable pointer to the `YamlEventT` struct.
/// * `ctx` - A mutable pointer to the `LoaderContext` struct.
///
/// # Returns
///
/// * `Result<(), YamlError>` indicating the outcome of the operation.
///
/// # Safety
///
/// - All pointers must be valid and properly initialized.
unsafe fn yaml_parser_load_mapping(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
    ctx: *mut LoaderContext,
) -> Result<(), YamlError> {
    let mut node = MaybeUninit::<YamlNodeT>::uninit();
    let node_ptr = node.as_mut_ptr();
    struct Pairs {
        start: *mut YamlNodePairT,
        end: *mut YamlNodePairT,
        top: *mut YamlNodePairT,
    }
    let mut pairs = Pairs {
        start: ptr::null_mut::<YamlNodePairT>(),
        end: ptr::null_mut::<YamlNodePairT>(),
        top: ptr::null_mut::<YamlNodePairT>(),
    };
    let index: i32;
    let mut tag: *mut yaml_char_t = (*event).data.mapping_start.tag;

    if STACK_LIMIT!(parser, (*(*parser).document).nodes).ok {
        if tag.is_null()
            || strcmp(
                tag as *mut c_char,
                b"!\0" as *const u8 as *mut c_char,
            ) == 0
        {
            yaml_free(tag as *mut libc::c_void);
            tag = yaml_strdup(
                b"tag:yaml.org,2002:map\0" as *const u8 as *const c_char
                    as *mut yaml_char_t,
            );
            if tag.is_null() {
                return Err(YamlError::MemoryAllocationFailed);
            }
        }

        STACK_INIT!(pairs, YamlNodePairT);
        initialize_yaml_node(node_ptr);
        (*node_ptr).type_ = YamlMappingNode;
        (*node_ptr).tag = tag;
        (*node_ptr).start_mark = (*event).start_mark;
        (*node_ptr).end_mark = (*event).end_mark;
        (*node_ptr).data.mapping.pairs.start = pairs.start;
        (*node_ptr).data.mapping.pairs.end = pairs.end;
        (*node_ptr).data.mapping.pairs.top = pairs.start;
        (*node_ptr).data.mapping.style =
            (*event).data.mapping_start.style;

        if !PUSH!((*(*parser).document).nodes, *node_ptr) {
            return Err(YamlError::MemoryAllocationFailed);
        }
        index = (*(*parser).document)
            .nodes
            .top
            .offset_from((*(*parser).document).nodes.start)
            as i32
            + 1;

        yaml_parser_register_anchor(
            parser,
            index,
            (*event).data.mapping_start.anchor,
        )?;

        yaml_parser_load_node_add(parser, ctx, index)?;

        if STACK_LIMIT!(parser, *ctx).fail {
            return Err(YamlError::MemoryAllocationFailed);
        }

        if !PUSH!(*ctx, index) {
            return Err(YamlError::MemoryAllocationFailed);
        }
        return Ok(());
    }

    yaml_free(tag as *mut libc::c_void);
    yaml_free((*event).data.mapping_start.anchor as *mut libc::c_void);
    Err(YamlError::MemoryAllocationFailed)
}

/// Ends the loading of a mapping.
///
/// # Arguments
///
/// * `parser` - A mutable pointer to the `YamlParserT` struct.
/// * `event` - A mutable pointer to the `YamlEventT` struct.
/// * `ctx` - A mutable pointer to the `LoaderContext` struct.
///
/// # Returns
///
/// * `Result<(), YamlError>` indicating the outcome of the operation.
///
/// # Safety
///
/// - All pointers must be valid and properly initialized.
unsafe fn yaml_parser_load_mapping_end(
    parser: *mut YamlParserT,
    event: *mut YamlEventT,
    ctx: *mut LoaderContext,
) -> Result<(), YamlError> {
    // Ensure the stack is not underflowed
    assert!(
        (*ctx).top.offset_from((*ctx).start) > 0,
        "LoaderContext stack underflow"
    );

    // Retrieve the index from the top of the stack
    let index: i32 = *(*ctx).top.offset(-1);

    // Dereference parser.document to get a reference to YamlDocumentT
    let document: &YamlDocumentT = &*(*parser).document;

    // Calculate the pointer to the desired YamlNodeT
    let node_ptr: *mut YamlNodeT =
        document.nodes.start.offset((index - 1) as isize);

    // Dereference node_ptr to get mutable reference to YamlNodeT
    let node: &mut YamlNodeT = &mut *node_ptr;

    // Assert that the node is of type YamlMappingNode
    assert!(node.type_ == YamlMappingNode, "Expected YamlMappingNode");

    // Set the end_mark of the node based on the event
    node.end_mark = (*event).end_mark;

    // Pop the index from the context stack
    POP!(*ctx);

    Ok(())
}

/// Compares two C strings.
///
/// # Arguments
///
/// * `s1` - A pointer to the first C string.
/// * `s2` - A pointer to the second C string.
///
/// # Returns
///
/// * `0` if both strings are equal.
/// * A negative value if `s1` is less than `s2`.
/// * A positive value if `s1` is greater than `s2`.
///
/// # Safety
///
/// - Both `s1` and `s2` must be valid, null-terminated C strings.
unsafe fn strcmp(s1: *mut c_char, s2: *mut c_char) -> i32 {
    let mut i = 0;
    loop {
        let c1 = *s1.add(i);
        let c2 = *s2.add(i);
        if c1 != c2 {
            return c1 as i32 - c2 as i32;
        }
        if c1 == 0 {
            break;
        }
        i += 1;
    }
    0
}
