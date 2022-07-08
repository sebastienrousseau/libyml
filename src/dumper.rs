use crate::api::{yaml_free, yaml_malloc};
use crate::externs::{memset, strcmp};
use crate::fmt::WriteToPtr;
use crate::yaml::{
    yaml_anchors_t, yaml_char_t, yaml_document_t, yaml_emitter_t, yaml_event_t, yaml_mark_t,
    yaml_node_item_t, yaml_node_pair_t, yaml_node_t, YAML_ALIAS_EVENT, YAML_ANY_ENCODING,
    YAML_DOCUMENT_END_EVENT, YAML_DOCUMENT_START_EVENT, YAML_MAPPING_END_EVENT, YAML_MAPPING_NODE,
    YAML_MAPPING_START_EVENT, YAML_SCALAR_EVENT, YAML_SCALAR_NODE, YAML_SEQUENCE_END_EVENT,
    YAML_SEQUENCE_NODE, YAML_SEQUENCE_START_EVENT, YAML_STREAM_END_EVENT, YAML_STREAM_START_EVENT,
};
use crate::{libc, yaml_document_delete, yaml_emitter_emit, PointerExt};
use core::mem::{size_of, MaybeUninit};
use core::ptr::{self, addr_of_mut};
/// Start a YAML stream.
///
/// This function should be used before yaml_emitter_dump() is called.
///
/// Returns 1 if the function succeeded, 0 on error.
pub unsafe fn yaml_emitter_open(mut emitter: *mut yaml_emitter_t) -> libc::c_int {
    let mut event = MaybeUninit::<yaml_event_t>::uninit();
    let event = event.as_mut_ptr();
    let mark = yaml_mark_t {
        index: 0_u64,
        line: 0_u64,
        column: 0_u64,
    };
    __assert!(!emitter.is_null());
    __assert!((*emitter).opened == 0);
    memset(
        event as *mut libc::c_void,
        0_i32,
        size_of::<yaml_event_t>() as libc::c_ulong,
    );
    (*event).type_ = YAML_STREAM_START_EVENT;
    (*event).start_mark = mark;
    (*event).end_mark = mark;
    (*event).data.stream_start.encoding = YAML_ANY_ENCODING;
    if yaml_emitter_emit(emitter, event) == 0 {
        return 0_i32;
    }
    (*emitter).opened = 1_i32;
    1_i32
}
/// Finish a YAML stream.
///
/// This function should be used after yaml_emitter_dump() is called.
///
/// Returns 1 if the function succeeded, 0 on error.
pub unsafe fn yaml_emitter_close(mut emitter: *mut yaml_emitter_t) -> libc::c_int {
    let mut event = MaybeUninit::<yaml_event_t>::uninit();
    let event = event.as_mut_ptr();
    let mark = yaml_mark_t {
        index: 0_u64,
        line: 0_u64,
        column: 0_u64,
    };
    __assert!(!emitter.is_null());
    __assert!((*emitter).opened != 0);
    if (*emitter).closed != 0 {
        return 1_i32;
    }
    memset(
        event as *mut libc::c_void,
        0_i32,
        size_of::<yaml_event_t>() as libc::c_ulong,
    );
    (*event).type_ = YAML_STREAM_END_EVENT;
    (*event).start_mark = mark;
    (*event).end_mark = mark;
    if yaml_emitter_emit(emitter, event) == 0 {
        return 0_i32;
    }
    (*emitter).closed = 1_i32;
    1_i32
}
/// Emit a YAML document.
///
/// The documen object may be generated using the yaml_parser_load() function or
/// the yaml_document_initialize() function. The emitter takes the
/// responsibility for the document object and destroys its content after it is
/// emitted. The document object is destroyed even if the function fails.
///
/// Returns 1 if the function succeeded, 0 on error.
pub unsafe fn yaml_emitter_dump(
    emitter: *mut yaml_emitter_t,
    document: *mut yaml_document_t,
) -> libc::c_int {
    let current_block: u64;
    let mut event = MaybeUninit::<yaml_event_t>::uninit();
    let event = event.as_mut_ptr();
    let mark = yaml_mark_t {
        index: 0_u64,
        line: 0_u64,
        column: 0_u64,
    };
    __assert!(!emitter.is_null());
    __assert!(!document.is_null());
    let fresh0 = addr_of_mut!((*emitter).document);
    *fresh0 = document;
    if (*emitter).opened == 0 {
        if yaml_emitter_open(emitter) == 0 {
            current_block = 5018439318894558507;
        } else {
            current_block = 15619007995458559411;
        }
    } else {
        current_block = 15619007995458559411;
    }
    match current_block {
        15619007995458559411 => {
            if (*document).nodes.start == (*document).nodes.top {
                if !(yaml_emitter_close(emitter) == 0) {
                    yaml_emitter_delete_document_and_anchors(emitter);
                    return 1_i32;
                }
            } else {
                __assert!((*emitter).opened != 0);
                let fresh1 = addr_of_mut!((*emitter).anchors);
                *fresh1 = yaml_malloc((size_of::<yaml_anchors_t>() as libc::c_ulong).wrapping_mul(
                    ((*document).nodes.top).c_offset_from((*document).nodes.start) as libc::c_long
                        as libc::c_ulong,
                )) as *mut yaml_anchors_t;
                if !((*emitter).anchors).is_null() {
                    memset(
                        (*emitter).anchors as *mut libc::c_void,
                        0_i32,
                        (size_of::<yaml_anchors_t>() as libc::c_ulong).wrapping_mul(
                            ((*document).nodes.top).c_offset_from((*document).nodes.start)
                                as libc::c_long as libc::c_ulong,
                        ),
                    );
                    memset(
                        event as *mut libc::c_void,
                        0_i32,
                        size_of::<yaml_event_t>() as libc::c_ulong,
                    );
                    (*event).type_ = YAML_DOCUMENT_START_EVENT;
                    (*event).start_mark = mark;
                    (*event).end_mark = mark;
                    (*event).data.document_start.version_directive = (*document).version_directive;
                    (*event).data.document_start.tag_directives.start =
                        (*document).tag_directives.start;
                    (*event).data.document_start.tag_directives.end =
                        (*document).tag_directives.end;
                    (*event).data.document_start.implicit = (*document).start_implicit;
                    if !(yaml_emitter_emit(emitter, event) == 0) {
                        yaml_emitter_anchor_node(emitter, 1_i32);
                        if !(yaml_emitter_dump_node(emitter, 1_i32) == 0) {
                            memset(
                                event as *mut libc::c_void,
                                0_i32,
                                size_of::<yaml_event_t>() as libc::c_ulong,
                            );
                            (*event).type_ = YAML_DOCUMENT_END_EVENT;
                            (*event).start_mark = mark;
                            (*event).end_mark = mark;
                            (*event).data.document_end.implicit = (*document).end_implicit;
                            if !(yaml_emitter_emit(emitter, event) == 0) {
                                yaml_emitter_delete_document_and_anchors(emitter);
                                return 1_i32;
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    yaml_emitter_delete_document_and_anchors(emitter);
    0_i32
}
unsafe fn yaml_emitter_delete_document_and_anchors(mut emitter: *mut yaml_emitter_t) {
    let mut index: libc::c_int;
    if ((*emitter).anchors).is_null() {
        yaml_document_delete((*emitter).document);
        let fresh2 = addr_of_mut!((*emitter).document);
        *fresh2 = ptr::null_mut::<yaml_document_t>();
        return;
    }
    index = 0_i32;
    while ((*(*emitter).document).nodes.start).wrapping_offset(index as isize)
        < (*(*emitter).document).nodes.top
    {
        let mut node: yaml_node_t =
            *((*(*emitter).document).nodes.start).wrapping_offset(index as isize);
        if (*((*emitter).anchors).wrapping_offset(index as isize)).serialized == 0 {
            yaml_free(node.tag as *mut libc::c_void);
            if node.type_ as libc::c_uint == YAML_SCALAR_NODE as libc::c_int as libc::c_uint {
                yaml_free(node.data.scalar.value as *mut libc::c_void);
            }
        }
        if node.type_ as libc::c_uint == YAML_SEQUENCE_NODE as libc::c_int as libc::c_uint {
            yaml_free(node.data.sequence.items.start as *mut libc::c_void);
            node.data.sequence.items.end = ptr::null_mut::<yaml_node_item_t>();
            node.data.sequence.items.top = node.data.sequence.items.end;
            node.data.sequence.items.start = node.data.sequence.items.top;
        }
        if node.type_ as libc::c_uint == YAML_MAPPING_NODE as libc::c_int as libc::c_uint {
            yaml_free(node.data.mapping.pairs.start as *mut libc::c_void);
            node.data.mapping.pairs.end = ptr::null_mut::<yaml_node_pair_t>();
            node.data.mapping.pairs.top = node.data.mapping.pairs.end;
            node.data.mapping.pairs.start = node.data.mapping.pairs.top;
        }
        index += 1;
    }
    yaml_free((*(*emitter).document).nodes.start as *mut libc::c_void);
    let fresh3 = addr_of_mut!((*(*emitter).document).nodes.end);
    *fresh3 = ptr::null_mut::<yaml_node_t>();
    let fresh4 = addr_of_mut!((*(*emitter).document).nodes.top);
    *fresh4 = *fresh3;
    let fresh5 = addr_of_mut!((*(*emitter).document).nodes.start);
    *fresh5 = *fresh4;
    yaml_free((*emitter).anchors as *mut libc::c_void);
    let fresh6 = addr_of_mut!((*emitter).anchors);
    *fresh6 = ptr::null_mut::<yaml_anchors_t>();
    (*emitter).last_anchor_id = 0_i32;
    let fresh7 = addr_of_mut!((*emitter).document);
    *fresh7 = ptr::null_mut::<yaml_document_t>();
}
unsafe fn yaml_emitter_anchor_node(emitter: *mut yaml_emitter_t, index: libc::c_int) {
    let node: *mut yaml_node_t = ((*(*emitter).document).nodes.start)
        .wrapping_offset(index as isize)
        .wrapping_offset(-(1_isize));
    let mut item: *mut yaml_node_item_t;
    let mut pair: *mut yaml_node_pair_t;
    let fresh8 =
        addr_of_mut!((*((*emitter).anchors).wrapping_offset((index - 1_i32) as isize)).references);
    *fresh8 += 1;
    if (*((*emitter).anchors).wrapping_offset((index - 1_i32) as isize)).references == 1_i32 {
        match (*node).type_ as libc::c_uint {
            2 => {
                item = (*node).data.sequence.items.start;
                while item < (*node).data.sequence.items.top {
                    yaml_emitter_anchor_node(emitter, *item);
                    item = item.wrapping_offset(1);
                }
            }
            3 => {
                pair = (*node).data.mapping.pairs.start;
                while pair < (*node).data.mapping.pairs.top {
                    yaml_emitter_anchor_node(emitter, (*pair).key);
                    yaml_emitter_anchor_node(emitter, (*pair).value);
                    pair = pair.wrapping_offset(1);
                }
            }
            _ => {}
        }
    } else if (*((*emitter).anchors).wrapping_offset((index - 1_i32) as isize)).references == 2_i32
    {
        let fresh9 = addr_of_mut!((*emitter).last_anchor_id);
        *fresh9 += 1;
        (*((*emitter).anchors).wrapping_offset((index - 1_i32) as isize)).anchor = *fresh9;
    }
}
unsafe fn yaml_emitter_generate_anchor(
    _emitter: *mut yaml_emitter_t,
    anchor_id: libc::c_int,
) -> *mut yaml_char_t {
    let anchor: *mut yaml_char_t = yaml_malloc(16_u64) as *mut yaml_char_t;
    if anchor.is_null() {
        return ptr::null_mut::<yaml_char_t>();
    }
    write!(WriteToPtr::new(anchor), "id{:03}\0", anchor_id);
    anchor
}
unsafe fn yaml_emitter_dump_node(emitter: *mut yaml_emitter_t, index: libc::c_int) -> libc::c_int {
    let node: *mut yaml_node_t = ((*(*emitter).document).nodes.start)
        .wrapping_offset(index as isize)
        .wrapping_offset(-(1_isize));
    let anchor_id: libc::c_int =
        (*((*emitter).anchors).wrapping_offset((index - 1_i32) as isize)).anchor;
    let mut anchor: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
    if anchor_id != 0 {
        anchor = yaml_emitter_generate_anchor(emitter, anchor_id);
        if anchor.is_null() {
            return 0_i32;
        }
    }
    if (*((*emitter).anchors).wrapping_offset((index - 1_i32) as isize)).serialized != 0 {
        return yaml_emitter_dump_alias(emitter, anchor);
    }
    (*((*emitter).anchors).wrapping_offset((index - 1_i32) as isize)).serialized = 1_i32;
    match (*node).type_ as libc::c_uint {
        1 => yaml_emitter_dump_scalar(emitter, node, anchor),
        2 => yaml_emitter_dump_sequence(emitter, node, anchor),
        3 => yaml_emitter_dump_mapping(emitter, node, anchor),
        _ => __assert!(false),
    }
}
unsafe fn yaml_emitter_dump_alias(
    emitter: *mut yaml_emitter_t,
    anchor: *mut yaml_char_t,
) -> libc::c_int {
    let mut event = MaybeUninit::<yaml_event_t>::uninit();
    let event = event.as_mut_ptr();
    let mark = yaml_mark_t {
        index: 0_u64,
        line: 0_u64,
        column: 0_u64,
    };
    memset(
        event as *mut libc::c_void,
        0_i32,
        size_of::<yaml_event_t>() as libc::c_ulong,
    );
    (*event).type_ = YAML_ALIAS_EVENT;
    (*event).start_mark = mark;
    (*event).end_mark = mark;
    (*event).data.alias.anchor = anchor;
    yaml_emitter_emit(emitter, event)
}
unsafe fn yaml_emitter_dump_scalar(
    emitter: *mut yaml_emitter_t,
    node: *mut yaml_node_t,
    anchor: *mut yaml_char_t,
) -> libc::c_int {
    let mut event = MaybeUninit::<yaml_event_t>::uninit();
    let event = event.as_mut_ptr();
    let mark = yaml_mark_t {
        index: 0_u64,
        line: 0_u64,
        column: 0_u64,
    };
    let plain_implicit: libc::c_int = (strcmp(
        (*node).tag as *mut libc::c_char,
        b"tag:yaml.org,2002:str\0" as *const u8 as *const libc::c_char,
    ) == 0_i32) as libc::c_int;
    let quoted_implicit: libc::c_int = (strcmp(
        (*node).tag as *mut libc::c_char,
        b"tag:yaml.org,2002:str\0" as *const u8 as *const libc::c_char,
    ) == 0_i32) as libc::c_int;
    memset(
        event as *mut libc::c_void,
        0_i32,
        size_of::<yaml_event_t>() as libc::c_ulong,
    );
    (*event).type_ = YAML_SCALAR_EVENT;
    (*event).start_mark = mark;
    (*event).end_mark = mark;
    (*event).data.scalar.anchor = anchor;
    (*event).data.scalar.tag = (*node).tag;
    (*event).data.scalar.value = (*node).data.scalar.value;
    (*event).data.scalar.length = (*node).data.scalar.length;
    (*event).data.scalar.plain_implicit = plain_implicit;
    (*event).data.scalar.quoted_implicit = quoted_implicit;
    (*event).data.scalar.style = (*node).data.scalar.style;
    yaml_emitter_emit(emitter, event)
}
unsafe fn yaml_emitter_dump_sequence(
    emitter: *mut yaml_emitter_t,
    node: *mut yaml_node_t,
    anchor: *mut yaml_char_t,
) -> libc::c_int {
    let mut event = MaybeUninit::<yaml_event_t>::uninit();
    let event = event.as_mut_ptr();
    let mark = yaml_mark_t {
        index: 0_u64,
        line: 0_u64,
        column: 0_u64,
    };
    let implicit: libc::c_int = (strcmp(
        (*node).tag as *mut libc::c_char,
        b"tag:yaml.org,2002:seq\0" as *const u8 as *const libc::c_char,
    ) == 0_i32) as libc::c_int;
    let mut item: *mut yaml_node_item_t;
    memset(
        event as *mut libc::c_void,
        0_i32,
        size_of::<yaml_event_t>() as libc::c_ulong,
    );
    (*event).type_ = YAML_SEQUENCE_START_EVENT;
    (*event).start_mark = mark;
    (*event).end_mark = mark;
    (*event).data.sequence_start.anchor = anchor;
    (*event).data.sequence_start.tag = (*node).tag;
    (*event).data.sequence_start.implicit = implicit;
    (*event).data.sequence_start.style = (*node).data.sequence.style;
    if yaml_emitter_emit(emitter, event) == 0 {
        return 0_i32;
    }
    item = (*node).data.sequence.items.start;
    while item < (*node).data.sequence.items.top {
        if yaml_emitter_dump_node(emitter, *item) == 0 {
            return 0_i32;
        }
        item = item.wrapping_offset(1);
    }
    memset(
        event as *mut libc::c_void,
        0_i32,
        size_of::<yaml_event_t>() as libc::c_ulong,
    );
    (*event).type_ = YAML_SEQUENCE_END_EVENT;
    (*event).start_mark = mark;
    (*event).end_mark = mark;
    if yaml_emitter_emit(emitter, event) == 0 {
        return 0_i32;
    }
    1_i32
}
unsafe fn yaml_emitter_dump_mapping(
    emitter: *mut yaml_emitter_t,
    node: *mut yaml_node_t,
    anchor: *mut yaml_char_t,
) -> libc::c_int {
    let mut event = MaybeUninit::<yaml_event_t>::uninit();
    let event = event.as_mut_ptr();
    let mark = yaml_mark_t {
        index: 0_u64,
        line: 0_u64,
        column: 0_u64,
    };
    let implicit: libc::c_int = (strcmp(
        (*node).tag as *mut libc::c_char,
        b"tag:yaml.org,2002:map\0" as *const u8 as *const libc::c_char,
    ) == 0_i32) as libc::c_int;
    let mut pair: *mut yaml_node_pair_t;
    memset(
        event as *mut libc::c_void,
        0_i32,
        size_of::<yaml_event_t>() as libc::c_ulong,
    );
    (*event).type_ = YAML_MAPPING_START_EVENT;
    (*event).start_mark = mark;
    (*event).end_mark = mark;
    (*event).data.mapping_start.anchor = anchor;
    (*event).data.mapping_start.tag = (*node).tag;
    (*event).data.mapping_start.implicit = implicit;
    (*event).data.mapping_start.style = (*node).data.mapping.style;
    if yaml_emitter_emit(emitter, event) == 0 {
        return 0_i32;
    }
    pair = (*node).data.mapping.pairs.start;
    while pair < (*node).data.mapping.pairs.top {
        if yaml_emitter_dump_node(emitter, (*pair).key) == 0 {
            return 0_i32;
        }
        if yaml_emitter_dump_node(emitter, (*pair).value) == 0 {
            return 0_i32;
        }
        pair = pair.wrapping_offset(1);
    }
    memset(
        event as *mut libc::c_void,
        0_i32,
        size_of::<yaml_event_t>() as libc::c_ulong,
    );
    (*event).type_ = YAML_MAPPING_END_EVENT;
    (*event).start_mark = mark;
    (*event).end_mark = mark;
    if yaml_emitter_emit(emitter, event) == 0 {
        return 0_i32;
    }
    1_i32
}
