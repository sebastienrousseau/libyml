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

/// Helper function to clean up a YAML document.
///
/// # Safety
/// - `document` must be a valid pointer to a `YamlDocumentT`.
unsafe fn cleanup_document(document: *mut YamlDocumentT) {
    if !document.is_null() {
        STACK_DEL!((*document).nodes);
        yaml_document_delete(document);
    }
}

/// Helper function to clean up a YAML parser.
///
/// # Safety
/// - `parser` must be a valid pointer to a `YamlParserT`.
unsafe fn cleanup_parser(parser: *mut YamlParserT) {
    if !parser.is_null() {
        yaml_parser_delete_aliases(parser);
        (*parser).document = ptr::null_mut();
    }
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

    // Handle the case where the stream end is already produced
    if (*parser).stream_end_produced {
        cleanup_document(document);
        return Ok(());
    }

    // Initialize the document memory safely
    ptr::write_bytes(
        document as *mut u8,
        0,
        size_of::<YamlDocumentT>(),
    );
    if !STACK_INIT!((*document).nodes, YamlNodeT) {
        return Err(YamlError::MemoryAllocationFailed);
    }

    let mut event = MaybeUninit::<YamlEventT>::uninit();
    let event_ptr = event.as_mut_ptr();

    // Ensure the token queue is properly initialized and non-empty
    if (*parser).tokens.start.is_null()
        || (*parser).tokens.head.is_null()
        || (*parser).tokens.head == (*parser).tokens.tail
    {
        cleanup_document(document);
        return Err(YamlError::InvalidEventType);
    }

    // Parse the stream start event if not already produced
    if !(*parser).stream_start_produced {
        if yaml_parser_parse(parser, event_ptr).fail {
            cleanup_document(document);
            return Err(YamlError::InvalidEventType);
        }
        if (*event_ptr).type_ != YamlStreamStartEvent {
            cleanup_document(document);
            return Err(YamlError::InvalidEventType);
        }
        (*parser).stream_start_produced = true;
    }

    // Parse events and load the document
    if yaml_parser_parse(parser, event_ptr) == OK {
        if (*event_ptr).type_ == YamlStreamEndEvent {
            (*parser).stream_end_produced = true;
            cleanup_document(document);
            return Ok(());
        }

        // Initialize aliases stack for the document
        if !STACK_INIT!((*parser).aliases, YamlAliasDataT) {
            cleanup_document(document);
            return Err(YamlError::MemoryAllocationFailed);
        }

        (*parser).document = document;

        // Attempt to load the document
        if yaml_parser_load_document(parser, event_ptr).is_ok() {
            cleanup_parser(parser);
            return Ok(());
        }
    }

    // Cleanup in case of failure
    cleanup_parser(parser);
    cleanup_document(document);
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
        b"found undefined anchor\0" as *const u8 as *const c_char,
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

/// Compares two null-terminated C strings.
///
/// # Arguments
///
/// * `s1` - A mutable pointer to the first C string.
/// * `s2` - A mutable pointer to the second C string.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::YamlSimpleKeyT;
    use crate::{
        YamlEventTypeT, YamlMarkT, YamlParserStateT, YamlTokenT,
    };
    use core::ffi::CStr;

    // =========================================================================
    // Helper Functions
    // =========================================================================

    /// Creates a properly initialized scanner state for testing.
    ///
    /// # Safety
    ///
    /// - `parser` must be a valid pointer to a `YamlParserT`
    /// - The parser must not have any existing initialized stacks
    unsafe fn initialize_scanner_state(parser: *mut YamlParserT) {
        // Initialize required stacks
        STACK_INIT!((*parser).simple_keys, YamlSimpleKeyT);
        STACK_INIT!((*parser).states, YamlParserStateT);
        STACK_INIT!((*parser).marks, YamlMarkT);
        STACK_INIT!((*parser).indents, libc::c_int);

        // Initialize the parser state
        PUSH!(
            (*parser).states,
            YamlParserStateT::YamlParseStreamStartState
        );

        // Set initial flags
        (*parser).tokens_parsed = 0;
        (*parser).stream_start_produced = false;
        (*parser).stream_end_produced = false;
    }

    /// Helper function to properly clean up a node and its resources
    unsafe fn cleanup_node(node: &mut YamlNodeT) {
        match node.type_ {
            YamlSequenceNode => {
                if !node.data.sequence.items.start.is_null() {
                    STACK_DEL!(node.data.sequence.items);
                }
            }
            YamlMappingNode => {
                if !node.data.mapping.pairs.start.is_null() {
                    STACK_DEL!(node.data.mapping.pairs);
                }
            }
            YamlScalarNode => {
                if !node.data.scalar.value.is_null() {
                    yaml_free(
                        node.data.scalar.value as *mut libc::c_void,
                    );
                }
            }
            _ => {}
        }

        if !node.tag.is_null() {
            yaml_free(node.tag as *mut libc::c_void);
            node.tag = ptr::null_mut();
        }
    }

    /// Creates a null-terminated byte array from a string literal.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to convert, must be null-terminated
    ///
    /// # Returns
    ///
    /// A pointer to a null-terminated C string
    ///
    /// # Panics
    ///
    /// Panics if the input string is not null-terminated
    fn c_string(s: &str) -> *mut c_char {
        let cstr = CStr::from_bytes_with_nul(s.as_bytes())
            .expect("String must be null-terminated");
        cstr.as_ptr() as *mut c_char
    }

    /// Creates a test parser and document with proper initialization.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - Pointer to initialized parser
    /// - Pointer to initialized document
    /// - Cleanup closure
    ///
    /// # Safety
    ///
    /// Caller must:
    /// - Not use the pointers after calling the cleanup closure
    /// - Call the cleanup closure exactly once
    unsafe fn setup_test_parser(
    ) -> (*mut YamlParserT, *mut YamlDocumentT, impl FnOnce()) {
        // Allocate parser and document
        let parser_ptr = yaml_malloc(size_of::<YamlParserT>() as u64)
            as *mut YamlParserT;
        let document_ptr = yaml_malloc(size_of::<YamlDocumentT>() as u64)
            as *mut YamlDocumentT;

        // Verify allocations
        if parser_ptr.is_null() || document_ptr.is_null() {
            cleanup_failed_setup(parser_ptr, document_ptr);
            panic!("Failed to allocate parser or document");
        }

        // Zero initialize structures
        ptr::write_bytes(
            parser_ptr as *mut u8,
            0,
            size_of::<YamlParserT>(),
        );
        ptr::write_bytes(
            document_ptr as *mut u8,
            0,
            size_of::<YamlDocumentT>(),
        );

        // Initialize token queue
        (*parser_ptr).tokens.start =
            yaml_malloc((size_of::<YamlTokenT>() * 10) as u64)
                as *mut YamlTokenT;
        if (*parser_ptr).tokens.start.is_null() {
            cleanup_failed_setup(parser_ptr, document_ptr);
            panic!("Failed to allocate token queue");
        }

        // Setup token queue pointers
        (*parser_ptr).tokens.end = (*parser_ptr).tokens.start.add(10);
        (*parser_ptr).tokens.head = (*parser_ptr).tokens.start;
        (*parser_ptr).tokens.tail = (*parser_ptr).tokens.start;

        // Initialize scanner state
        initialize_scanner_state(parser_ptr);

        // Initialize document nodes stack
        STACK_INIT!((*document_ptr).nodes, YamlNodeT);

        // Create cleanup closure
        let cleanup = move || {
            cleanup_resources(parser_ptr, document_ptr);
        };

        (parser_ptr, document_ptr, cleanup)
    }

    /// Cleans up resources from failed setup
    ///
    /// # Safety
    ///
    /// - Pointers must either be null or valid pointers obtained from yaml_malloc
    unsafe fn cleanup_failed_setup(
        parser: *mut YamlParserT,
        document: *mut YamlDocumentT,
    ) {
        if !parser.is_null() {
            yaml_free(parser as *mut libc::c_void);
        }
        if !document.is_null() {
            yaml_free(document as *mut libc::c_void);
        }
    }

    /// Cleans up all resources associated with a parser and document
    ///
    /// # Safety
    ///
    /// - Pointers must be valid and obtained from setup_test_parser
    unsafe fn cleanup_resources(
        parser: *mut YamlParserT,
        document: *mut YamlDocumentT,
    ) {
        // Clean up document
        if !document.is_null() {
            if !(*document).nodes.start.is_null() {
                STACK_DEL!((*document).nodes);
            }
            yaml_free(document as *mut libc::c_void);
        }

        // Clean up parser
        if !parser.is_null() {
            if !(*parser).tokens.start.is_null() {
                yaml_free((*parser).tokens.start as *mut libc::c_void);
            }

            // Clean up scanner stacks
            STACK_DEL!((*parser).simple_keys);
            STACK_DEL!((*parser).states);
            STACK_DEL!((*parser).marks);
            STACK_DEL!((*parser).indents);

            yaml_free(parser as *mut libc::c_void);
        }
    }

    /// Creates a test event with proper initialization
    ///
    /// # Safety
    ///
    /// - The returned pointer must be freed with yaml_free
    unsafe fn create_test_event(
        event_type: YamlEventTypeT,
    ) -> *mut YamlEventT {
        let event = yaml_malloc(size_of::<YamlEventT>() as u64)
            as *mut YamlEventT;
        if event.is_null() {
            panic!("Failed to allocate event");
        }

        // Zero initialize
        ptr::write_bytes(event as *mut u8, 0, size_of::<YamlEventT>());

        // Set basic fields
        (*event).type_ = event_type;
        (*event).start_mark = YamlMarkT::default();
        (*event).end_mark = YamlMarkT::default();

        event
    }

    /// Creates a test anchor with proper null termination
    ///
    /// # Safety
    ///
    /// - The returned pointer must be freed with yaml_free
    unsafe fn create_test_anchor(name: &str) -> *mut yaml_char_t {
        let len = name.len();
        let anchor = yaml_malloc((len + 1) as u64) as *mut yaml_char_t;
        if anchor.is_null() {
            panic!("Failed to allocate anchor");
        }

        // Copy content and null terminate
        ptr::copy_nonoverlapping(name.as_bytes().as_ptr(), anchor, len);
        *anchor.add(len) = 0;

        anchor
    }

    // =========================================================================
    // String Comparison Tests
    // =========================================================================

    /// Tests string comparison with equal strings
    #[test]
    fn test_strcmp_equal_strings() {
        unsafe {
            let s1 = c_string("test\0");
            let s2 = c_string("test\0");
            assert_eq!(strcmp(s1, s2), 0);
        }
    }

    /// Tests string comparison when first string is lexicographically less
    #[test]
    fn test_strcmp_first_less() {
        unsafe {
            let s1 = c_string("abc\0");
            let s2 = c_string("abd\0");
            assert!(strcmp(s1, s2) < 0);
        }
    }

    /// Tests string comparison when first string is lexicographically greater
    #[test]
    fn test_strcmp_first_greater() {
        unsafe {
            let s1 = c_string("abd\0");
            let s2 = c_string("abc\0");
            assert!(strcmp(s1, s2) > 0);
        }
    }

    /// Tests string comparison with empty strings
    #[test]
    fn test_strcmp_empty_strings() {
        unsafe {
            let s1 = c_string("\0");
            let s2 = c_string("\0");
            assert_eq!(strcmp(s1, s2), 0);
        }
    }

    /// Tests string comparison with prefix relationships
    #[test]
    fn test_strcmp_prefix() {
        unsafe {
            let s1 = c_string("abc\0");
            let s2 = c_string("abcd\0");
            assert!(strcmp(s1, s2) < 0);
        }
    }

    /// Tests string comparison with non-ASCII characters
    #[test]
    fn test_strcmp_non_ascii() {
        unsafe {
            let s1 = c_string("tést\0");
            let s2 = c_string("tést\0");
            let s3 = c_string("tèst\0");
            assert_eq!(strcmp(s1, s2), 0);
            assert!(strcmp(s1, s3) > 0);
        }
    }

    // =========================================================================
    // Parser Loading Tests
    // =========================================================================

    /// Tests parser loading with null pointers
    #[test]
    fn test_yaml_parser_load_null_pointers() {
        unsafe {
            let result =
                yaml_parser_load(ptr::null_mut(), ptr::null_mut());
            assert!(matches!(result, Err(YamlError::NullPointer)));
        }
    }

    /// Tests parser loading with stream end condition
    #[test]
    fn test_yaml_parser_load_stream_end() {
        unsafe {
            let (parser, document, cleanup) = setup_test_parser();

            // Set stream state
            (*parser).stream_start_produced = true;
            (*parser).stream_end_produced = true;

            let result = yaml_parser_load(parser, document);
            assert!(matches!(result, Ok(())));

            cleanup();
        }
    }

    /// Tests loading of scalar nodes
    #[test]
    fn test_yaml_parser_load_scalar() {
        unsafe {
            let (parser, document, cleanup) = setup_test_parser();
            (*parser).document = document;

            // Initialize context
            let mut ctx = LoaderContext {
                start: yaml_malloc(16 * size_of::<i32>() as u64)
                    as *mut i32,
                end: ptr::null_mut(),
                top: ptr::null_mut(),
            };
            ctx.end = ctx.start.add(16);
            ctx.top = ctx.start;

            // Create and test scalar event
            let event =
                create_test_event(YamlEventTypeT::YamlScalarEvent);
            let value = create_test_anchor("test_value");
            let tag = create_test_anchor("tag:yaml.org,2002:str");

            (*event).data.scalar.value = value;
            (*event).data.scalar.length = 10;
            (*event).data.scalar.tag = tag;

            let result =
                yaml_parser_load_scalar(parser, event, &mut ctx);
            assert!(result.is_ok());

            // Manual cleanup of event and its contents
            if !(*event).data.scalar.value.is_null() {
                yaml_free(
                    (*event).data.scalar.value as *mut libc::c_void,
                );
            }
            if !(*event).data.scalar.tag.is_null() {
                yaml_free(
                    (*event).data.scalar.tag as *mut libc::c_void,
                );
            }
            yaml_free(event as *mut libc::c_void);
            yaml_free(ctx.start as *mut libc::c_void);
            cleanup();
        }
    }

    // =========================================================================
    // Document Structure Tests
    // =========================================================================

    /// Tests sequence end handling with null pointers
    #[test]
    fn test_sequence_end_null_pointers() {
        unsafe {
            let result = yaml_parser_load_sequence_end(
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
            assert!(matches!(result, Err(YamlError::NullPointer)));
        }
    }

    /// Tests sequence end handling with empty stack
    #[test]
    fn test_sequence_end_empty_stack() {
        unsafe {
            let (parser, document, cleanup) = setup_test_parser();
            (*parser).document = document;

            let mut event = MaybeUninit::<YamlEventT>::uninit();
            let mut ctx = LoaderContext {
                start: ptr::null_mut(),
                end: ptr::null_mut(),
                top: ptr::null_mut(),
            };
            STACK_INIT!(ctx, i32);

            let result = yaml_parser_load_sequence_end(
                parser,
                event.as_mut_ptr(),
                &mut ctx,
            );
            assert!(matches!(
                result,
                Err(YamlError::InvalidDocumentStructure)
            ));

            STACK_DEL!(ctx);
            cleanup();
        }
    }

    /// Tests sequence end handling with valid sequence node
    #[test]
    fn test_sequence_end_valid() {
        unsafe {
            let (parser, document, cleanup) = setup_test_parser();
            (*parser).document = document;

            // Create context with a sequence node
            let mut ctx = LoaderContext {
                start: ptr::null_mut(),
                end: ptr::null_mut(),
                top: ptr::null_mut(),
            };
            STACK_INIT!(ctx, i32);

            // Create and push a sequence node
            let node = create_sequence_node();
            PUSH!((*document).nodes, node);
            let node_index = (*document)
                .nodes
                .top
                .offset_from((*document).nodes.start)
                as i32;
            PUSH!(ctx, node_index);

            // Create event and test
            let mut event = MaybeUninit::<YamlEventT>::uninit();
            let event_ptr = event.as_mut_ptr();
            ptr::write_bytes(
                event_ptr as *mut u8,
                0,
                size_of::<YamlEventT>(),
            );
            (*event_ptr).end_mark = YamlMarkT::default();

            let result = yaml_parser_load_sequence_end(
                parser, event_ptr, &mut ctx,
            );
            assert!(result.is_ok());

            // Cleanup sequence items
            while !STACK_EMPTY!((*document).nodes) {
                let mut node = POP!((*document).nodes);
                cleanup_node(&mut node);
            }

            STACK_DEL!(ctx);
            cleanup();
        }
    }

    /// Tests the proper handling of mapping end events
    #[test]
    fn test_mapping_end() {
        unsafe {
            let (parser, document, cleanup) = setup_test_parser();
            (*parser).document = document;

            // Initialize context
            let mut ctx = LoaderContext {
                start: yaml_malloc(16 * size_of::<i32>() as u64)
                    as *mut i32,
                end: ptr::null_mut(),
                top: ptr::null_mut(),
            };
            ctx.end = ctx.start.add(16);
            ctx.top = ctx.start;

            // Create and push a mapping node
            let node = create_mapping_node();
            PUSH!((*document).nodes, node);
            let node_index = (*document)
                .nodes
                .top
                .offset_from((*document).nodes.start)
                as i32;
            PUSH!(ctx, node_index);

            // Create mapping end event
            let event =
                create_test_event(YamlEventTypeT::YamlMappingEndEvent);
            let result =
                yaml_parser_load_mapping_end(parser, event, &mut ctx);
            assert!(result.is_ok());

            // Cleanup mapping pairs
            while !STACK_EMPTY!((*document).nodes) {
                let mut node = POP!((*document).nodes);
                if !node.data.mapping.pairs.start.is_null() {
                    STACK_DEL!(node.data.mapping.pairs);
                }
            }

            // Cleanup remaining resources
            yaml_free(event as *mut libc::c_void);
            yaml_free(ctx.start as *mut libc::c_void);
            cleanup();
        }
    }

    /// Tests document loading with invalid event types
    #[test]
    fn test_load_document_invalid_events() {
        unsafe {
            let (parser, document, cleanup) = setup_test_parser();
            (*parser).document = document;

            let invalid_events = [
                YamlEventTypeT::YamlScalarEvent,
                YamlEventTypeT::YamlSequenceStartEvent,
                YamlEventTypeT::YamlMappingStartEvent,
            ];

            for &event_type in &invalid_events {
                let event = create_test_event(event_type);
                let result = yaml_parser_load_document(parser, event);
                assert!(matches!(
                    result,
                    Err(YamlError::InvalidDocumentStructure)
                ));
                yaml_free(event as *mut libc::c_void);
            }

            cleanup();
        }
    }

    /// Tests node addition to different parent types
    #[test]
    fn test_node_addition() {
        unsafe {
            let (parser, document, cleanup) = setup_test_parser();
            (*parser).document = document;

            // Initialize context
            let mut ctx = LoaderContext {
                start: yaml_malloc(16 * size_of::<i32>() as u64)
                    as *mut i32,
                end: ptr::null_mut(),
                top: ptr::null_mut(),
            };
            ctx.end = ctx.start.add(16);
            ctx.top = ctx.start;

            // Test sequence node
            let seq_node = create_sequence_node();
            PUSH!((*document).nodes, seq_node);
            let seq_index = (*document)
                .nodes
                .top
                .offset_from((*document).nodes.start)
                as i32;
            PUSH!(ctx, seq_index);

            let result = yaml_parser_load_node_add(parser, &mut ctx, 2);
            assert!(result.is_ok());

            // Test mapping node
            let map_node = create_mapping_node();
            PUSH!((*document).nodes, map_node);
            let map_index = (*document)
                .nodes
                .top
                .offset_from((*document).nodes.start)
                as i32;
            *ctx.top = map_index;

            let result = yaml_parser_load_node_add(parser, &mut ctx, 3);
            assert!(result.is_ok());

            // Cleanup
            while !STACK_EMPTY!((*document).nodes) {
                let mut node = POP!((*document).nodes);
                cleanup_node(&mut node);
            }
            yaml_free(ctx.start as *mut libc::c_void);
            cleanup();
        }
    }

    // Helper function to create a sequence node with proper cleanup
    unsafe fn create_sequence_node() -> YamlNodeT {
        let mut node = MaybeUninit::<YamlNodeT>::uninit();
        let node_ptr = node.as_mut_ptr();
        initialize_yaml_node(node_ptr);
        (*node_ptr).type_ = YamlSequenceNode;

        STACK_INIT!((*node_ptr).data.sequence.items, YamlNodeItemT);

        *node_ptr
    }

    /// Helper function to create a mapping node for testing
    unsafe fn create_mapping_node() -> YamlNodeT {
        let mut node = MaybeUninit::<YamlNodeT>::uninit();
        let node_ptr = node.as_mut_ptr();
        initialize_yaml_node(node_ptr);
        (*node_ptr).type_ = YamlMappingNode;

        STACK_INIT!((*node_ptr).data.mapping.pairs, YamlNodePairT);

        *node_ptr
    }

    /// Tests the cleanup of resources in various error conditions
    #[test]
    fn test_cleanup_on_error() {
        unsafe {
            let (parser, document, _) = setup_test_parser();
            (*parser).document = document;

            // Initialize test resources
            STACK_INIT!((*parser).aliases, YamlAliasDataT);
            let anchor = create_test_anchor("test");
            let alias_data = YamlAliasDataT {
                anchor,
                index: 1,
                mark: YamlMarkT::default(),
            };
            PUSH!((*parser).aliases, alias_data);

            // Cleanup
            while !STACK_EMPTY!((*parser).aliases) {
                let alias = POP!((*parser).aliases);
                yaml_free(alias.anchor as *mut libc::c_void);
            }
            STACK_DEL!((*parser).aliases);
            cleanup_resources(parser, document);
        }
    }
}
