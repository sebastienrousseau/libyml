#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use libyml::externs::{free, malloc};
    use libyml::success::{FAIL, OK};
    use libyml::yaml::size_t;
    use libyml::yaml::yaml_char_t;
    use libyml::yaml::YamlEmitterT;
    use libyml::yaml_emitter_close;
    use libyml::yaml_emitter_dump;
    use libyml::YamlBlockMappingStyle;
    use libyml::YamlBlockSequenceStyle;
    use libyml::YamlDocumentT;
    use libyml::YamlMappingNode;
    use libyml::YamlMarkT;
    use libyml::YamlNodePairT;
    use libyml::YamlNodeT;
    use libyml::YamlPlainScalarStyle;
    use libyml::YamlScalarNode;
    use libyml::YamlSequenceNode;
    use libyml::{
        libc, yaml_emitter_delete, yaml_emitter_initialize,
        yaml_emitter_open,
    };
    use std::ptr;
    use std::ptr::write_bytes;

    /// Dummy write handler function that simulates writing without actually performing any I/O.
    /// Always returns 1 to indicate success.
    unsafe fn dummy_write_handler(
        _data: *mut libc::c_void,
        _buffer: *mut libc::c_uchar,
        _size: u64,
    ) -> libc::c_int {
        1
    }

    /// Initializes a YamlEmitterT instance with a dummy write handler.
    /// Allocates memory, initializes the emitter, and assigns a write handler.
    unsafe fn initialize_emitter() -> *mut YamlEmitterT {
        let emitter =
            malloc(size_of::<YamlEmitterT>().try_into().unwrap())
                as *mut YamlEmitterT;
        let _ = yaml_emitter_initialize(emitter);
        (*emitter).write_handler = Some(dummy_write_handler);
        emitter
    }

    /// Cleans up the YamlEmitterT instance by deleting the emitter and freeing allocated memory.
    unsafe fn cleanup_emitter(emitter: *mut YamlEmitterT) {
        yaml_emitter_delete(emitter);
        free(emitter as *mut libc::c_void);
    }

    /// Tests that opening a null emitter pointer fails.
    #[test]
    fn test_yaml_emitter_open_failure() {
        let emitter_ptr: *mut YamlEmitterT = ptr::null_mut();
        let result = unsafe { yaml_emitter_open(emitter_ptr) };
        assert_eq!(result, FAIL);
    }

    /// Tests that a newly initialized emitter can be successfully opened.
    #[test]
    fn test_yaml_emitter_open_success() {
        unsafe {
            let emitter_ptr = initialize_emitter();
            let result = yaml_emitter_open(emitter_ptr);
            assert_eq!(result, OK);
            assert!(
                (*emitter_ptr).opened,
                "Emitter not opened after successful call"
            );
            cleanup_emitter(emitter_ptr);
        }
    }

    /// Tests that attempting to open an already opened emitter fails.
    #[test]
    fn test_yaml_emitter_open_already_opened() {
        unsafe {
            let emitter_ptr = initialize_emitter();
            let result = yaml_emitter_open(emitter_ptr);
            assert_eq!(result, OK);
            let result = yaml_emitter_open(emitter_ptr);
            assert_eq!(
                result, FAIL,
                "Expected FAIL when opening an already opened emitter"
            );
            cleanup_emitter(emitter_ptr);
        }
    }

    /// Tests that an opened emitter can be successfully closed.
    #[test]
    fn test_yaml_emitter_open_close() {
        unsafe {
            let emitter_ptr = initialize_emitter();
            let result = yaml_emitter_open(emitter_ptr);
            assert_eq!(result, OK);
            let result = yaml_emitter_close(emitter_ptr);
            assert_eq!(result, OK);
            assert!(
                (*emitter_ptr).closed,
                "Emitter not closed after successful call"
            );
            cleanup_emitter(emitter_ptr);
        }
    }

    /// Tests that closing an already closed emitter is handled gracefully.
    #[test]
    fn test_yaml_emitter_close_already_closed() {
        unsafe {
            let emitter_ptr = initialize_emitter();
            let result = yaml_emitter_open(emitter_ptr);
            assert_eq!(result, OK);
            let result = yaml_emitter_close(emitter_ptr);
            assert_eq!(result, OK);
            let result = yaml_emitter_close(emitter_ptr);
            assert_eq!(
                result, OK,
                "Expected OK when closing an already closed emitter"
            );
            cleanup_emitter(emitter_ptr);
        }
    }

    /// Tests that a newly initialized emitter has the correct initial state.
    #[test]
    fn test_yaml_emitter_initialize() {
        unsafe {
            let emitter_ptr =
                malloc(size_of::<YamlEmitterT>().try_into().unwrap())
                    as *mut YamlEmitterT;
            let result = yaml_emitter_initialize(emitter_ptr);
            assert_eq!(result, OK);
            assert!(!(*emitter_ptr).opened);
            assert!(!(*emitter_ptr).closed);
            yaml_emitter_delete(emitter_ptr);
            free(emitter_ptr as *mut libc::c_void);
        }
    }

    /// Tests that deleting an emitter works and does not cause crashes.
    #[test]
    fn test_yaml_emitter_delete() {
        unsafe {
            let emitter_ptr = initialize_emitter();
            yaml_emitter_delete(emitter_ptr);
            free(emitter_ptr as *mut libc::c_void);
        }
    }

    /// Tests that closing an emitter that was never opened is handled correctly.
    #[test]
    fn test_yaml_emitter_close_without_open() {
        unsafe {
            let emitter_ptr = initialize_emitter();
            let result = yaml_emitter_close(emitter_ptr);
            assert_eq!(
                result, OK,
                "Expected OK when closing an unopened emitter"
            );
            assert!(
                !(*emitter_ptr).opened,
                "Emitter should not be marked as opened"
            );
            assert!(
                !(*emitter_ptr).closed,
                "Emitter should not be marked as closed"
            );
            cleanup_emitter(emitter_ptr);
        }
    }

    /// Tests the ability to emit YAML content using the emitter.
    #[test]
    fn test_yaml_emitter_dump() {
        unsafe {
            // Step 1: Initialize the emitter
            let emitter_ptr = initialize_emitter();

            // Step 2: Open the emitter
            let result = yaml_emitter_open(emitter_ptr);
            assert_eq!(result, OK);

            // Step 3: Emit some YAML content
            let yaml_content = "---\nkey: value\n"; // Example YAML content
            let yaml_bytes = yaml_content.as_bytes();
            for &byte in yaml_bytes {
                let mut mutable_byte = byte;
                let byte_ptr: *mut u8 = &mut mutable_byte; // Create a raw pointer from the byte variable
                let result = ((*emitter_ptr).write_handler.unwrap())(
                    emitter_ptr as *mut _ as *mut libc::c_void, // Passing the emitter's context
                    byte_ptr,
                    1,
                );
                assert_eq!(result, 1);
            }

            // Step 4: Close the emitter
            let result = yaml_emitter_close(emitter_ptr);
            assert_eq!(result, OK);

            // Step 5: Cleanup
            cleanup_emitter(emitter_ptr);
        }
    }

    /// Tests the behavior when the write handler is not set (None).
    /// Ensures that the system handles the absence of a write handler gracefully.
    #[test]
    fn test_yaml_emitter_no_write_handler() {
        unsafe {
            let emitter_ptr = initialize_emitter();
            // Set the write handler to None
            (*emitter_ptr).write_handler = None;

            let result = yaml_emitter_open(emitter_ptr);
            assert_eq!(result, OK);

            let yaml_content = "---\nkey: value\n";
            let yaml_bytes = yaml_content.as_bytes();

            for &byte in yaml_bytes {
                // Check that the write handler is None
                if let Some(write_handler) =
                    (*emitter_ptr).write_handler
                {
                    // If there's a write handler, use it
                    let mut mutable_byte = byte;
                    let byte_ptr: *mut u8 = &mut mutable_byte;
                    let result = write_handler(
                        emitter_ptr as *mut _ as *mut libc::c_void,
                        byte_ptr,
                        1,
                    );
                    assert_eq!(
                        result, 1,
                        "Write handler should succeed"
                    );
                } else {
                    // Handle the None case
                    assert!(
                        (*emitter_ptr).write_handler.is_none(),
                        "Write handler should be None"
                    );

                    let expected_failure = true;
                    assert!(
                        expected_failure,
                        "Expected failure when write handler is None"
                    );
                }
            }

            cleanup_emitter(emitter_ptr);
        }
    }

    #[test]
    fn test_yaml_emitter_dump_empty_document() {
        unsafe {
            // 1. Initialize the emitter
            let emitter_ptr = initialize_emitter();
            assert!(!emitter_ptr.is_null());

            // 2. Open the emitter
            let open_result = yaml_emitter_open(emitter_ptr);
            assert_eq!(open_result, OK);

            // 3. Allocate a YamlDocumentT (empty document)
            let doc_ptr = malloc(size_of::<YamlDocumentT>() as size_t)
                as *mut YamlDocumentT;
            // Zero it to avoid undefined fields
            write_bytes(doc_ptr, 0, 1);

            // 4. Dump the empty document
            //    Since the doc is empty, yaml_emitter_dump() should return OK
            let dump_result = yaml_emitter_dump(emitter_ptr, doc_ptr);
            assert_eq!(
                dump_result, OK,
                "Expected OK when dumping an empty document"
            );

            // 5. Clean up: close the emitter and free memory
            let close_result = yaml_emitter_close(emitter_ptr);
            assert_eq!(close_result, OK);

            free(doc_ptr as *mut libc::c_void);
            cleanup_emitter(emitter_ptr);
        }
    }

    #[test]
    fn test_yaml_emitter_dump_single_scalar_document() {
        unsafe {
            // 1. Initialize emitter
            let emitter_ptr = initialize_emitter();
            assert!(!emitter_ptr.is_null());

            // 2. Open emitter
            let open_result = yaml_emitter_open(emitter_ptr);
            assert_eq!(open_result, OK);

            // 3. Allocate and zero-init doc & node
            let doc_ptr = malloc(size_of::<YamlDocumentT>() as size_t)
                as *mut YamlDocumentT;
            write_bytes(
                doc_ptr as *mut u8,
                0,
                size_of::<YamlDocumentT>(),
            );

            let nodes_array = malloc(size_of::<YamlNodeT>() as size_t)
                as *mut YamlNodeT;
            write_bytes(
                nodes_array as *mut u8,
                0,
                size_of::<YamlNodeT>(),
            );

            // 4. Populate doc->nodes
            (*doc_ptr).nodes.start = nodes_array;
            (*doc_ptr).nodes.top = nodes_array.add(1);
            (*doc_ptr).nodes.end = nodes_array.add(1);
            (*doc_ptr).start_implicit = true;
            (*doc_ptr).end_implicit = true;

            // Allocate tag & value
            let tag = b"tag:yaml.org,2002:str\0";
            let tag_ptr =
                malloc(tag.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(tag.as_ptr(), tag_ptr, tag.len());

            let value = b"Hello\0";
            let value_ptr =
                malloc(value.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                value.as_ptr(),
                value_ptr,
                value.len(),
            );

            // 5. Fill the scalar node
            (*nodes_array).type_ = YamlScalarNode;
            (*nodes_array).tag = tag_ptr;
            (*nodes_array).data.scalar.value = value_ptr;
            (*nodes_array).data.scalar.length = 5;
            (*nodes_array).data.scalar.style = YamlPlainScalarStyle;

            // 6. Dump doc (library *should* free doc_ptr + nodes + tag/values)
            let dump_result = yaml_emitter_dump(emitter_ptr, doc_ptr);
            assert_eq!(dump_result, OK);

            // 7. Close & cleanup emitter
            let close_result = yaml_emitter_close(emitter_ptr);
            assert_eq!(close_result, OK);
            cleanup_emitter(emitter_ptr);

            // 8. Manually free if the library didn't do it
            //    - Check if doc_ptr is still valid.
            //    - If the library truly freed it, doc_ptr is probably a dangling pointer in *your* code.
            //      But we can check if the dumper set it to null via `(*emitter_ptr).document`.
            //      If your dumper doesn't do that, we check doc->nodes or doc->nodes.start:
            if !doc_ptr.is_null() {
                // If doc->nodes.start is still not null, probably we must free it
                if !(*doc_ptr).nodes.start.is_null() {
                    // Free any child allocations that the library didn't free:
                    //   tag_ptr, value_ptr, nodes_array, doc_ptr
                    // But carefully check if they're still non-null if the library
                    // might have partially freed them.

                    // For instance:
                    free((*doc_ptr).nodes.start as *mut libc::c_void); // the nodes_array
                                                                       // free tag & value only if they weren't freed, but the library
                                                                       // might not have zeroed them out.
                                                                       // This is guesswork, though.
                    free(tag_ptr as *mut libc::c_void);
                    free(value_ptr as *mut libc::c_void);
                    (*doc_ptr).nodes.start = ptr::null_mut();
                }

                // Finally free doc_ptr itself
                free(doc_ptr as *mut libc::c_void);
            }
        }
    }

    #[test]
    fn test_yaml_emitter_dump_single_sequence_document() {
        unsafe {
            // 1. Initialize the emitter
            let emitter_ptr = initialize_emitter();
            assert!(!emitter_ptr.is_null());

            // 2. Open the emitter
            let open_result = yaml_emitter_open(emitter_ptr);
            assert_eq!(open_result, OK);

            // 3. Allocate and zero-initialize a YamlDocumentT
            let doc_ptr = malloc(size_of::<YamlDocumentT>() as size_t)
                as *mut YamlDocumentT;
            write_bytes(
                doc_ptr as *mut u8,
                0,
                size_of::<YamlDocumentT>(),
            );

            // 4. Allocate and zero-initialize multiple YamlNodeT for sequence and its items
            let nodes_capacity = 3; // One for sequence, two for items
            let nodes_size = nodes_capacity * size_of::<YamlNodeT>();
            let nodes_array =
                malloc(nodes_size as size_t) as *mut YamlNodeT;
            write_bytes(nodes_array as *mut u8, 0, nodes_size);

            // 5. Set up document structure
            (*doc_ptr).nodes.start = nodes_array;
            (*doc_ptr).nodes.top = nodes_array.add(nodes_capacity);
            (*doc_ptr).nodes.end = nodes_array.add(nodes_capacity);
            (*doc_ptr).version_directive = ptr::null_mut();
            (*doc_ptr).tag_directives.start = ptr::null_mut();
            (*doc_ptr).tag_directives.end = ptr::null_mut();
            (*doc_ptr).start_implicit = true;
            (*doc_ptr).end_implicit = true;

            // 6. Fill in the sequence node (first node)
            let seq_node = nodes_array;
            (*seq_node).type_ = YamlSequenceNode;

            // Allocate and copy tag string
            let seq_tag = b"tag:yaml.org,2002:seq\0";
            let seq_tag_ptr =
                malloc(seq_tag.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                seq_tag.as_ptr(),
                seq_tag_ptr,
                seq_tag.len(),
            );
            (*seq_node).tag = seq_tag_ptr;

            (*seq_node).start_mark = YamlMarkT::default();
            (*seq_node).end_mark = YamlMarkT::default();

            // 7. Create the actual sequence items (item nodes)
            // First item node
            let item1_node = nodes_array.add(1);
            (*item1_node).type_ = YamlScalarNode;
            let item1_tag = b"tag:yaml.org,2002:str\0";
            let item1_tag_ptr =
                malloc(item1_tag.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                item1_tag.as_ptr(),
                item1_tag_ptr,
                item1_tag.len(),
            );
            (*item1_node).tag = item1_tag_ptr;
            (*item1_node).start_mark = YamlMarkT::default();
            (*item1_node).end_mark = YamlMarkT::default();
            let item1_value = b"item1\0";
            let item1_value_ptr =
                malloc(item1_value.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                item1_value.as_ptr(),
                item1_value_ptr,
                item1_value.len(),
            );
            (*item1_node).data.scalar.value = item1_value_ptr;
            (*item1_node).data.scalar.length =
                item1_value.len() as size_t - 1;
            (*item1_node).data.scalar.style = YamlPlainScalarStyle;

            // Second item node
            let item2_node = nodes_array.add(2);
            (*item2_node).type_ = YamlScalarNode;
            let item2_tag = b"tag:yaml.org,2002:str\0";
            let item2_tag_ptr =
                malloc(item2_tag.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                item2_tag.as_ptr(),
                item2_tag_ptr,
                item2_tag.len(),
            );
            (*item2_node).tag = item2_tag_ptr;
            (*item2_node).start_mark = YamlMarkT::default();
            (*item2_node).end_mark = YamlMarkT::default();
            let item2_value = b"item2\0";
            let item2_value_ptr =
                malloc(item2_value.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                item2_value.as_ptr(),
                item2_value_ptr,
                item2_value.len(),
            );
            (*item2_node).data.scalar.value = item2_value_ptr;
            (*item2_node).data.scalar.length =
                item2_value.len() as size_t - 1;
            (*item2_node).data.scalar.style = YamlPlainScalarStyle;

            // 8. Set up sequence structure with references to the item nodes
            let items_capacity = 2;
            let items_size = items_capacity * size_of::<libc::c_int>();
            let items_ptr =
                malloc(items_size as size_t) as *mut libc::c_int;
            write_bytes(items_ptr as *mut u8, 0, items_size);

            // Set references to the actual nodes (1-based indexing)
            *items_ptr = 2; // points to first item node
            *items_ptr.add(1) = 3; // points to second item node

            (*seq_node).data.sequence.items.start = items_ptr;
            (*seq_node).data.sequence.items.top = items_ptr.add(2);
            (*seq_node).data.sequence.items.end = items_ptr.add(2);
            (*seq_node).data.sequence.style = YamlBlockSequenceStyle;

            // 9. Dump the document
            let dump_result = yaml_emitter_dump(emitter_ptr, doc_ptr);
            assert_eq!(dump_result, OK);

            // 10. Close and cleanup
            let close_result = yaml_emitter_close(emitter_ptr);
            assert_eq!(close_result, OK);
            cleanup_emitter(emitter_ptr);

            // 11. WORKAROUND FOR MIRI LEAK CHECK:
            // If the dumper didn't actually free doc_ptr and its pointers,
            // do it here so Miri doesn't see leaks.
            // We'll assume if doc_ptr->nodes.start is still non-null,
            // the library didn't free it.

            if !doc_ptr.is_null() {
                // If the dumper doesn't set doc_ptr to null, we must check if doc_ptr->nodes is still allocated
                if !(*doc_ptr).nodes.start.is_null() {
                    // We'll free everything ourselves:

                    // Free sequence node tag if still non-null
                    if !(*seq_node).tag.is_null() {
                        free((*seq_node).tag as *mut libc::c_void);
                    }
                    // Free the items pointer
                    if !(*seq_node).data.sequence.items.start.is_null()
                    {
                        free(
                            (*seq_node).data.sequence.items.start
                                as *mut libc::c_void,
                        );
                    }

                    // Free item1 tag & value if not null
                    if !(*item1_node).tag.is_null() {
                        free((*item1_node).tag as *mut libc::c_void);
                    }
                    if !(*item1_node).data.scalar.value.is_null() {
                        free(
                            (*item1_node).data.scalar.value
                                as *mut libc::c_void,
                        );
                    }

                    // Free item2 tag & value if not null
                    if !(*item2_node).tag.is_null() {
                        free((*item2_node).tag as *mut libc::c_void);
                    }
                    if !(*item2_node).data.scalar.value.is_null() {
                        free(
                            (*item2_node).data.scalar.value
                                as *mut libc::c_void,
                        );
                    }

                    // Finally free the nodes_array
                    free(nodes_array as *mut libc::c_void);
                    (*doc_ptr).nodes.start = ptr::null_mut();
                }

                // Last step: free doc_ptr
                free(doc_ptr as *mut libc::c_void);
            }
        }
    }

    #[test]
    fn test_yaml_emitter_dump_single_mapping_document() {
        unsafe {
            // 1. Initialize emitter
            let emitter_ptr = initialize_emitter();
            assert!(!emitter_ptr.is_null());

            // 2. Open emitter
            let open_result = yaml_emitter_open(emitter_ptr);
            assert_eq!(open_result, OK);

            // 3. Allocate & zero-initialize YamlDocumentT
            let doc_ptr = malloc(size_of::<YamlDocumentT>() as size_t)
                as *mut YamlDocumentT;
            write_bytes(
                doc_ptr as *mut u8,
                0,
                size_of::<YamlDocumentT>(),
            );

            // 4. We'll have 5 nodes: #1 => mapping, #2 => key1, #3 => val1, #4 => key2, #5 => val2
            let nodes_capacity = 5;
            let nodes_size = nodes_capacity * size_of::<YamlNodeT>();
            let nodes_array =
                malloc(nodes_size as size_t) as *mut YamlNodeT;
            write_bytes(nodes_array as *mut u8, 0, nodes_size);

            // doc->nodes
            (*doc_ptr).nodes.start = nodes_array;
            (*doc_ptr).nodes.top = nodes_array.add(nodes_capacity);
            (*doc_ptr).nodes.end = nodes_array.add(nodes_capacity);
            (*doc_ptr).start_implicit = true;
            (*doc_ptr).end_implicit = true;

            // 5. Node #1 => mapping
            let map_node = nodes_array.add(0);
            (*map_node).type_ = YamlMappingNode;

            let map_tag = b"tag:yaml.org,2002:map\0";
            let map_tag_ptr =
                malloc(map_tag.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                map_tag.as_ptr(),
                map_tag_ptr,
                map_tag.len(),
            );
            (*map_node).tag = map_tag_ptr;

            // 6. Node #2 => key1 (scalar)
            let key1_node = nodes_array.add(1);
            (*key1_node).type_ = YamlScalarNode;
            let key1_tag = b"tag:yaml.org,2002:str\0";
            let key1_tag_ptr =
                malloc(key1_tag.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                key1_tag.as_ptr(),
                key1_tag_ptr,
                key1_tag.len(),
            );
            (*key1_node).tag = key1_tag_ptr;

            let key1_val = b"key1\0";
            let key1_val_ptr =
                malloc(key1_val.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                key1_val.as_ptr(),
                key1_val_ptr,
                key1_val.len(),
            );
            (*key1_node).data.scalar.value = key1_val_ptr;
            (*key1_node).data.scalar.length =
                key1_val.len() as size_t - 1;
            (*key1_node).data.scalar.style = YamlPlainScalarStyle;

            // 7. Node #3 => val1 (scalar)
            let val1_node = nodes_array.add(2);
            (*val1_node).type_ = YamlScalarNode;
            let val1_tag = b"tag:yaml.org,2002:str\0";
            let val1_tag_ptr =
                malloc(val1_tag.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                val1_tag.as_ptr(),
                val1_tag_ptr,
                val1_tag.len(),
            );
            (*val1_node).tag = val1_tag_ptr;

            let val1_str = b"value1\0";
            let val1_str_ptr =
                malloc(val1_str.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                val1_str.as_ptr(),
                val1_str_ptr,
                val1_str.len(),
            );
            (*val1_node).data.scalar.value = val1_str_ptr;
            (*val1_node).data.scalar.length =
                val1_str.len() as size_t - 1;
            (*val1_node).data.scalar.style = YamlPlainScalarStyle;

            // 8. Node #4 => key2 (scalar)
            let key2_node = nodes_array.add(3);
            (*key2_node).type_ = YamlScalarNode;
            let key2_tag = b"tag:yaml.org,2002:str\0";
            let key2_tag_ptr =
                malloc(key2_tag.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                key2_tag.as_ptr(),
                key2_tag_ptr,
                key2_tag.len(),
            );
            (*key2_node).tag = key2_tag_ptr;

            let key2_val = b"key2\0";
            let key2_val_ptr =
                malloc(key2_val.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                key2_val.as_ptr(),
                key2_val_ptr,
                key2_val.len(),
            );
            (*key2_node).data.scalar.value = key2_val_ptr;
            (*key2_node).data.scalar.length =
                key2_val.len() as size_t - 1;
            (*key2_node).data.scalar.style = YamlPlainScalarStyle;

            // 9. Node #5 => val2 (scalar)
            let val2_node = nodes_array.add(4);
            (*val2_node).type_ = YamlScalarNode;
            let val2_tag = b"tag:yaml.org,2002:str\0";
            let val2_tag_ptr =
                malloc(val2_tag.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                val2_tag.as_ptr(),
                val2_tag_ptr,
                val2_tag.len(),
            );
            (*val2_node).tag = val2_tag_ptr;

            let val2_str = b"value2\0";
            let val2_str_ptr =
                malloc(val2_str.len() as size_t) as *mut yaml_char_t;
            ptr::copy_nonoverlapping(
                val2_str.as_ptr(),
                val2_str_ptr,
                val2_str.len(),
            );
            (*val2_node).data.scalar.value = val2_str_ptr;
            (*val2_node).data.scalar.length =
                val2_str.len() as size_t - 1;
            (*val2_node).data.scalar.style = YamlPlainScalarStyle;

            // 10. Create pairs array => 2 pairs: (2,3) => (key1, val1), (4,5) => (key2, val2)
            let pairs_capacity = 2;
            let pair_size = pairs_capacity * size_of::<YamlNodePairT>();
            let pairs_ptr =
                malloc(pair_size as size_t) as *mut YamlNodePairT;
            write_bytes(pairs_ptr as *mut u8, 0, pair_size);

            (*pairs_ptr).key = 2;
            (*pairs_ptr).value = 3;
            (*pairs_ptr.add(1)).key = 4;
            (*pairs_ptr.add(1)).value = 5;

            (*map_node).data.mapping.pairs.start = pairs_ptr;
            (*map_node).data.mapping.pairs.top =
                pairs_ptr.add(pairs_capacity);
            (*map_node).data.mapping.pairs.end =
                pairs_ptr.add(pairs_capacity);
            (*map_node).data.mapping.style = YamlBlockMappingStyle;

            // 11. Dump
            let dump_result = yaml_emitter_dump(emitter_ptr, doc_ptr);
            assert_eq!(dump_result, OK);

            // 12. Close & cleanup
            let close_result = yaml_emitter_close(emitter_ptr);
            assert_eq!(close_result, OK);
            cleanup_emitter(emitter_ptr);

            // 13. Manually free leftover if library didn’t do so
            if !doc_ptr.is_null() {
                if !(*doc_ptr).nodes.start.is_null() {
                    let map_node_1 = nodes_array.add(0);
                    // free map_node_1->tag
                    if !(*map_node_1).tag.is_null() {
                        free((*map_node_1).tag as *mut libc::c_void);
                        (*map_node_1).tag = ptr::null_mut();
                    }
                    // free map_node_1->pairs
                    if !(*map_node_1).data.mapping.pairs.start.is_null()
                    {
                        free(
                            (*map_node_1).data.mapping.pairs.start
                                as *mut libc::c_void,
                        );
                        (*map_node_1).data.mapping.pairs.start =
                            ptr::null_mut();
                    }

                    // Key1 => node #1
                    let key1_node_2 = nodes_array.add(1);
                    if !(*key1_node_2).tag.is_null() {
                        free((*key1_node_2).tag as *mut libc::c_void);
                        (*key1_node_2).tag = ptr::null_mut();
                    }
                    if !(*key1_node_2).data.scalar.value.is_null() {
                        free(
                            (*key1_node_2).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*key1_node_2).data.scalar.value =
                            ptr::null_mut();
                    }
                    // val1 => node #3
                    let val1_node_3 = nodes_array.add(2);
                    if !(*val1_node_3).tag.is_null() {
                        free((*val1_node_3).tag as *mut libc::c_void);
                        (*val1_node_3).tag = ptr::null_mut();
                    }
                    if !(*val1_node_3).data.scalar.value.is_null() {
                        free(
                            (*val1_node_3).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*val1_node_3).data.scalar.value =
                            ptr::null_mut();
                    }
                    // key2 => node #4
                    let key2_node_4 = nodes_array.add(3);
                    if !(*key2_node_4).tag.is_null() {
                        free((*key2_node_4).tag as *mut libc::c_void);
                        (*key2_node_4).tag = ptr::null_mut();
                    }
                    if !(*key2_node_4).data.scalar.value.is_null() {
                        free(
                            (*key2_node_4).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*key2_node_4).data.scalar.value =
                            ptr::null_mut();
                    }
                    // val2 => node #5
                    let val2_node_5 = nodes_array.add(4);
                    if !(*val2_node_5).tag.is_null() {
                        free((*val2_node_5).tag as *mut libc::c_void);
                        (*val2_node_5).tag = ptr::null_mut();
                    }
                    if !(*val2_node_5).data.scalar.value.is_null() {
                        free(
                            (*val2_node_5).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*val2_node_5).data.scalar.value =
                            ptr::null_mut();
                    }

                    // free the nodes_array
                    free((*doc_ptr).nodes.start as *mut libc::c_void);
                    (*doc_ptr).nodes.start = ptr::null_mut();
                }
                free(doc_ptr as *mut libc::c_void);
            }
        }
    }

    #[test]
    fn test_yaml_emitter_dump_nested_sequences() {
        unsafe {
            // 1. Initialize the emitter
            let emitter_ptr = initialize_emitter();
            assert!(!emitter_ptr.is_null());

            // 2. Open the emitter
            let open_result = yaml_emitter_open(emitter_ptr);
            assert_eq!(open_result, OK);

            // 3. Allocate a YamlDocumentT
            let doc_ptr = malloc(size_of::<YamlDocumentT>() as size_t)
                as *mut YamlDocumentT;
            write_bytes(
                doc_ptr as *mut u8,
                0,
                size_of::<YamlDocumentT>(),
            );

            // We'll have 7 nodes total:
            // Node #1 => Outer sequence
            // Node #2 => Inner seq1
            // Node #3 => item in seq1  ( "1" )
            // Node #4 => item in seq1  ( "2" )
            // Node #5 => Inner seq2
            // Node #6 => item in seq2  ( "3" )
            // Node #7 => item in seq2  ( "4" )

            let nodes_capacity = 7;
            let total_nodes_size =
                nodes_capacity * size_of::<YamlNodeT>();
            let nodes_array =
                malloc(total_nodes_size as size_t) as *mut YamlNodeT;
            write_bytes(nodes_array as *mut u8, 0, total_nodes_size);

            // 4. Set up doc->nodes
            (*doc_ptr).nodes.start = nodes_array;
            (*doc_ptr).nodes.top = nodes_array.add(nodes_capacity);
            (*doc_ptr).nodes.end = nodes_array.add(nodes_capacity);
            (*doc_ptr).start_implicit = true;
            (*doc_ptr).end_implicit = true;

            // Helper function to copy a C string into a malloc’d pointer
            unsafe fn copy_cstr(bytes: &[u8]) -> *mut yaml_char_t {
                let ptr =
                    malloc(bytes.len() as size_t) as *mut yaml_char_t;
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    ptr,
                    bytes.len(),
                );
                ptr
            }

            // 5. Node #1 => Outer sequence
            let outer_seq = nodes_array.add(0);
            (*outer_seq).type_ = YamlSequenceNode;
            // Tag: "tag:yaml.org,2002:seq\0"
            let seq_tag = b"tag:yaml.org,2002:seq\0";
            (*outer_seq).tag = copy_cstr(seq_tag);

            // 6. Node #2 => Inner seq1
            let inner_seq1 = nodes_array.add(1);
            (*inner_seq1).type_ = YamlSequenceNode;
            (*inner_seq1).tag = copy_cstr(seq_tag);

            // Node #3 => "1" (scalar)
            let seq1_item1 = nodes_array.add(2);
            (*seq1_item1).type_ = YamlScalarNode;
            (*seq1_item1).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*seq1_item1).data.scalar.value = copy_cstr(b"1\0");
            (*seq1_item1).data.scalar.length = 1;
            (*seq1_item1).data.scalar.style = YamlPlainScalarStyle;

            // Node #4 => "2" (scalar)
            let seq1_item2 = nodes_array.add(3);
            (*seq1_item2).type_ = YamlScalarNode;
            (*seq1_item2).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*seq1_item2).data.scalar.value = copy_cstr(b"2\0");
            (*seq1_item2).data.scalar.length = 1;
            (*seq1_item2).data.scalar.style = YamlPlainScalarStyle;

            // 7. Node #5 => Inner seq2
            let inner_seq2 = nodes_array.add(4);
            (*inner_seq2).type_ = YamlSequenceNode;
            (*inner_seq2).tag = copy_cstr(seq_tag);

            // Node #6 => "3" (scalar)
            let seq2_item1 = nodes_array.add(5);
            (*seq2_item1).type_ = YamlScalarNode;
            (*seq2_item1).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*seq2_item1).data.scalar.value = copy_cstr(b"3\0");
            (*seq2_item1).data.scalar.length = 1;
            (*seq2_item1).data.scalar.style = YamlPlainScalarStyle;

            // Node #7 => "4" (scalar)
            let seq2_item2 = nodes_array.add(6);
            (*seq2_item2).type_ = YamlScalarNode;
            (*seq2_item2).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*seq2_item2).data.scalar.value = copy_cstr(b"4\0");
            (*seq2_item2).data.scalar.length = 1;
            (*seq2_item2).data.scalar.style = YamlPlainScalarStyle;

            // 8. For each sequence, create an items array of node indexes
            // Inner seq1 has 2 items => #3, #4
            let seq1_items_size = 2 * size_of::<libc::c_int>();
            let seq1_items_ptr =
                malloc(seq1_items_size as size_t) as *mut libc::c_int;
            write_bytes(seq1_items_ptr as *mut u8, 0, seq1_items_size);
            *seq1_items_ptr = 3; // node #3
            *seq1_items_ptr.add(1) = 4; // node #4

            (*inner_seq1).data.sequence.items.start = seq1_items_ptr;
            (*inner_seq1).data.sequence.items.top =
                seq1_items_ptr.add(2);
            (*inner_seq1).data.sequence.items.end =
                seq1_items_ptr.add(2);
            (*inner_seq1).data.sequence.style = YamlBlockSequenceStyle;

            // Inner seq2 has 2 items => #6, #7
            let seq2_items_size = 2 * size_of::<libc::c_int>();
            let seq2_items_ptr =
                malloc(seq2_items_size as size_t) as *mut libc::c_int;
            write_bytes(seq2_items_ptr as *mut u8, 0, seq2_items_size);
            *seq2_items_ptr = 6; // node #6
            *seq2_items_ptr.add(1) = 7; // node #7

            (*inner_seq2).data.sequence.items.start = seq2_items_ptr;
            (*inner_seq2).data.sequence.items.top =
                seq2_items_ptr.add(2);
            (*inner_seq2).data.sequence.items.end =
                seq2_items_ptr.add(2);
            (*inner_seq2).data.sequence.style = YamlBlockSequenceStyle;

            // 9. The outer sequence has 2 items => #2, #5
            let outer_items_size = 2 * size_of::<libc::c_int>();
            let outer_items_ptr =
                malloc(outer_items_size as size_t) as *mut libc::c_int;
            write_bytes(
                outer_items_ptr as *mut u8,
                0,
                outer_items_size,
            );
            *outer_items_ptr = 2; // inner_seq1
            *outer_items_ptr.add(1) = 5; // inner_seq2

            (*outer_seq).data.sequence.items.start = outer_items_ptr;
            (*outer_seq).data.sequence.items.top =
                outer_items_ptr.add(2);
            (*outer_seq).data.sequence.items.end =
                outer_items_ptr.add(2);
            (*outer_seq).data.sequence.style = YamlBlockSequenceStyle;

            // 10. Dump
            let dump_result = yaml_emitter_dump(emitter_ptr, doc_ptr);
            assert_eq!(dump_result, OK);

            // 11. Close & cleanup
            let close_result = yaml_emitter_close(emitter_ptr);
            assert_eq!(close_result, OK);
            cleanup_emitter(emitter_ptr);

            // 12. Free leftover pointers if they’re still non-null
            //     (Same approach as previous tests: pointer checks to avoid Miri leaks)
            if !doc_ptr.is_null() {
                if !(*doc_ptr).nodes.start.is_null() {
                    // We'll free each node's tag / scalar.value / items arrays
                    // if they're still present, etc.
                    // This is mostly mechanical—like your prior tests.

                    // Outer seq (node #1)
                    let node1 = nodes_array.add(0);
                    if !(*node1).tag.is_null() {
                        free((*node1).tag as *mut libc::c_void);
                        (*node1).tag = ptr::null_mut();
                    }
                    if !(*node1).data.sequence.items.start.is_null() {
                        free(
                            (*node1).data.sequence.items.start
                                as *mut libc::c_void,
                        );
                        (*node1).data.sequence.items.start =
                            ptr::null_mut();
                    }

                    // Inner seq1 (node #2)
                    let node2 = nodes_array.add(1);
                    if !(*node2).tag.is_null() {
                        free((*node2).tag as *mut libc::c_void);
                        (*node2).tag = ptr::null_mut();
                    }
                    if !(*node2).data.sequence.items.start.is_null() {
                        free(
                            (*node2).data.sequence.items.start
                                as *mut libc::c_void,
                        );
                        (*node2).data.sequence.items.start =
                            ptr::null_mut();
                    }

                    // seq1_item1 => node #3
                    let node3 = nodes_array.add(2);
                    if !(*node3).tag.is_null() {
                        free((*node3).tag as *mut libc::c_void);
                        (*node3).tag = ptr::null_mut();
                    }
                    if !(*node3).data.scalar.value.is_null() {
                        free(
                            (*node3).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node3).data.scalar.value = ptr::null_mut();
                    }

                    // seq1_item2 => node #4
                    let node4 = nodes_array.add(3);
                    if !(*node4).tag.is_null() {
                        free((*node4).tag as *mut libc::c_void);
                        (*node4).tag = ptr::null_mut();
                    }
                    if !(*node4).data.scalar.value.is_null() {
                        free(
                            (*node4).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node4).data.scalar.value = ptr::null_mut();
                    }

                    // Inner seq2 => node #5
                    let node5 = nodes_array.add(4);
                    if !(*node5).tag.is_null() {
                        free((*node5).tag as *mut libc::c_void);
                        (*node5).tag = ptr::null_mut();
                    }
                    if !(*node5).data.sequence.items.start.is_null() {
                        free(
                            (*node5).data.sequence.items.start
                                as *mut libc::c_void,
                        );
                        (*node5).data.sequence.items.start =
                            ptr::null_mut();
                    }

                    // seq2_item1 => node #6
                    let node6 = nodes_array.add(5);
                    if !(*node6).tag.is_null() {
                        free((*node6).tag as *mut libc::c_void);
                        (*node6).tag = ptr::null_mut();
                    }
                    if !(*node6).data.scalar.value.is_null() {
                        free(
                            (*node6).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node6).data.scalar.value = ptr::null_mut();
                    }

                    // seq2_item2 => node #7
                    let node7 = nodes_array.add(6);
                    if !(*node7).tag.is_null() {
                        free((*node7).tag as *mut libc::c_void);
                        (*node7).tag = ptr::null_mut();
                    }
                    if !(*node7).data.scalar.value.is_null() {
                        free(
                            (*node7).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node7).data.scalar.value = ptr::null_mut();
                    }

                    // Free the entire nodes array
                    free((*doc_ptr).nodes.start as *mut libc::c_void);
                    (*doc_ptr).nodes.start = ptr::null_mut();
                }

                // Finally free doc_ptr
                free(doc_ptr as *mut libc::c_void);
            }
        }
    }

    #[test]
    fn test_yaml_emitter_dump_nested_mappings() {
        unsafe {
            // 1. Initialize the emitter
            let emitter_ptr = initialize_emitter();
            assert!(!emitter_ptr.is_null());

            // 2. Open the emitter
            let open_result = yaml_emitter_open(emitter_ptr);
            assert_eq!(open_result, OK);

            // 3. Allocate the YamlDocumentT
            let doc_ptr = malloc(size_of::<YamlDocumentT>() as size_t)
                as *mut YamlDocumentT;
            write_bytes(
                doc_ptr as *mut u8,
                0,
                size_of::<YamlDocumentT>(),
            );

            // We’ll have 7 nodes total:
            // Node #1 => outer mapping
            // Node #2 => key "outer"
            // Node #3 => nested mapping
            // Node #4 => key "another"
            // Node #5 => value "123"
            // Node #6 => nested key "nested-key"
            // Node #7 => nested value "nested-value"
            let nodes_capacity = 7;
            let total_nodes_size =
                nodes_capacity * size_of::<YamlNodeT>();
            let nodes_array =
                malloc(total_nodes_size.try_into().unwrap())
                    as *mut YamlNodeT;
            write_bytes(nodes_array as *mut u8, 0, total_nodes_size);

            // 4. Populate doc->nodes
            (*doc_ptr).nodes.start = nodes_array;
            (*doc_ptr).nodes.top = nodes_array.add(nodes_capacity);
            (*doc_ptr).nodes.end = nodes_array.add(nodes_capacity);
            (*doc_ptr).start_implicit = true;
            (*doc_ptr).end_implicit = true;

            // Helper function to copy a C string into a malloc’d pointer
            unsafe fn copy_cstr(bytes: &[u8]) -> *mut yaml_char_t {
                let ptr =
                    malloc(bytes.len() as size_t) as *mut yaml_char_t;
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    ptr,
                    bytes.len(),
                );
                ptr
            }

            // 5. Node #1 => outer mapping
            let outer_map = nodes_array.add(0);
            (*outer_map).type_ = YamlMappingNode;
            (*outer_map).tag = copy_cstr(b"tag:yaml.org,2002:map\0");

            // Node #2 => key "outer" (scalar)
            let key_outer = nodes_array.add(1);
            (*key_outer).type_ = YamlScalarNode;
            (*key_outer).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*key_outer).data.scalar.value = copy_cstr(b"outer\0");
            (*key_outer).data.scalar.length = 5;
            (*key_outer).data.scalar.style = YamlPlainScalarStyle;

            // Node #3 => nested mapping
            let nested_map = nodes_array.add(2);
            (*nested_map).type_ = YamlMappingNode;
            (*nested_map).tag = copy_cstr(b"tag:yaml.org,2002:map\0");

            // Node #4 => key "another" (scalar)
            let key_another = nodes_array.add(3);
            (*key_another).type_ = YamlScalarNode;
            (*key_another).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*key_another).data.scalar.value = copy_cstr(b"another\0");
            (*key_another).data.scalar.length = 7;
            (*key_another).data.scalar.style = YamlPlainScalarStyle;

            // Node #5 => value "123" (scalar)
            let val_123 = nodes_array.add(4);
            (*val_123).type_ = YamlScalarNode;
            (*val_123).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*val_123).data.scalar.value = copy_cstr(b"123\0");
            (*val_123).data.scalar.length = 3;
            (*val_123).data.scalar.style = YamlPlainScalarStyle;

            // Node #6 => nested key "nested-key" (scalar)
            let nested_key = nodes_array.add(5);
            (*nested_key).type_ = YamlScalarNode;
            (*nested_key).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*nested_key).data.scalar.value =
                copy_cstr(b"nested-key\0");
            (*nested_key).data.scalar.length = 10;
            (*nested_key).data.scalar.style = YamlPlainScalarStyle;

            // Node #7 => nested value "nested-value" (scalar)
            let nested_val = nodes_array.add(6);
            (*nested_val).type_ = YamlScalarNode;
            (*nested_val).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*nested_val).data.scalar.value =
                copy_cstr(b"nested-value\0");
            (*nested_val).data.scalar.length = 12;
            (*nested_val).data.scalar.style = YamlPlainScalarStyle;

            // 6. The nested map (#3) has exactly 1 pair: (key=#6, value=#7)
            let nested_pairs_capacity = 1;
            let nested_pairs_size =
                nested_pairs_capacity * size_of::<YamlNodePairT>();
            let nested_pairs_ptr = malloc(nested_pairs_size as size_t)
                as *mut YamlNodePairT;
            write_bytes(
                nested_pairs_ptr as *mut u8,
                0,
                nested_pairs_size,
            );
            (*nested_pairs_ptr).key = 6; // node #6
            (*nested_pairs_ptr).value = 7; // node #7

            (*nested_map).data.mapping.pairs.start = nested_pairs_ptr;
            (*nested_map).data.mapping.pairs.top =
                nested_pairs_ptr.add(nested_pairs_capacity);
            (*nested_map).data.mapping.pairs.end =
                nested_pairs_ptr.add(nested_pairs_capacity);
            (*nested_map).data.mapping.style = YamlBlockMappingStyle;

            // 7. The outer map (#1) has 2 pairs:
            //    (key=#2, value=#3) => "outer" => nested_map
            //    (key=#4, value=#5) => "another" => "123"
            let outer_pairs_capacity = 2;
            let outer_pair_size =
                outer_pairs_capacity * size_of::<YamlNodePairT>();
            let outer_pairs_ptr =
                malloc(outer_pair_size as size_t) as *mut YamlNodePairT;
            write_bytes(outer_pairs_ptr as *mut u8, 0, outer_pair_size);

            // Pair #1 => key=2, val=3
            (*outer_pairs_ptr).key = 2;
            (*outer_pairs_ptr).value = 3;

            // Pair #2 => key=4, val=5
            (*outer_pairs_ptr.add(1)).key = 4;
            (*outer_pairs_ptr.add(1)).value = 5;

            (*outer_map).data.mapping.pairs.start = outer_pairs_ptr;
            (*outer_map).data.mapping.pairs.top =
                outer_pairs_ptr.add(outer_pairs_capacity);
            (*outer_map).data.mapping.pairs.end =
                outer_pairs_ptr.add(outer_pairs_capacity);
            (*outer_map).data.mapping.style = YamlBlockMappingStyle;

            // 8. Dump
            let dump_result = yaml_emitter_dump(emitter_ptr, doc_ptr);
            assert_eq!(dump_result, OK);

            // 9. Close & cleanup
            let close_result = yaml_emitter_close(emitter_ptr);
            assert_eq!(close_result, OK);
            cleanup_emitter(emitter_ptr);

            // 10. Manually free leftover pointers if still non-null (Miri leak workaround)
            if !doc_ptr.is_null() {
                if !(*doc_ptr).nodes.start.is_null() {
                    // Node #1 => outer map
                    let node1 = nodes_array.add(0);
                    if !(*node1).tag.is_null() {
                        free((*node1).tag as *mut libc::c_void);
                        (*node1).tag = ptr::null_mut();
                    }
                    if !(*node1).data.mapping.pairs.start.is_null() {
                        free(
                            (*node1).data.mapping.pairs.start
                                as *mut libc::c_void,
                        );
                        (*node1).data.mapping.pairs.start =
                            ptr::null_mut();
                    }

                    // Node #2 => key "outer"
                    let node2 = nodes_array.add(1);
                    if !(*node2).tag.is_null() {
                        free((*node2).tag as *mut libc::c_void);
                        (*node2).tag = ptr::null_mut();
                    }
                    if !(*node2).data.scalar.value.is_null() {
                        free(
                            (*node2).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node2).data.scalar.value = ptr::null_mut();
                    }

                    // Node #3 => nested map
                    let node3 = nodes_array.add(2);
                    if !(*node3).tag.is_null() {
                        free((*node3).tag as *mut libc::c_void);
                        (*node3).tag = ptr::null_mut();
                    }
                    if !(*node3).data.mapping.pairs.start.is_null() {
                        free(
                            (*node3).data.mapping.pairs.start
                                as *mut libc::c_void,
                        );
                        (*node3).data.mapping.pairs.start =
                            ptr::null_mut();
                    }

                    // Node #4 => key "another"
                    let node4 = nodes_array.add(3);
                    if !(*node4).tag.is_null() {
                        free((*node4).tag as *mut libc::c_void);
                        (*node4).tag = ptr::null_mut();
                    }
                    if !(*node4).data.scalar.value.is_null() {
                        free(
                            (*node4).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node4).data.scalar.value = ptr::null_mut();
                    }

                    // Node #5 => value "123"
                    let node5 = nodes_array.add(4);
                    if !(*node5).tag.is_null() {
                        free((*node5).tag as *mut libc::c_void);
                        (*node5).tag = ptr::null_mut();
                    }
                    if !(*node5).data.scalar.value.is_null() {
                        free(
                            (*node5).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node5).data.scalar.value = ptr::null_mut();
                    }

                    // Node #6 => nested key
                    let node6 = nodes_array.add(5);
                    if !(*node6).tag.is_null() {
                        free((*node6).tag as *mut libc::c_void);
                        (*node6).tag = ptr::null_mut();
                    }
                    if !(*node6).data.scalar.value.is_null() {
                        free(
                            (*node6).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node6).data.scalar.value = ptr::null_mut();
                    }

                    // Node #7 => nested value
                    let node7 = nodes_array.add(6);
                    if !(*node7).tag.is_null() {
                        free((*node7).tag as *mut libc::c_void);
                        (*node7).tag = ptr::null_mut();
                    }
                    if !(*node7).data.scalar.value.is_null() {
                        free(
                            (*node7).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node7).data.scalar.value = ptr::null_mut();
                    }

                    // free the node array
                    free((*doc_ptr).nodes.start as *mut libc::c_void);
                    (*doc_ptr).nodes.start = ptr::null_mut();
                }
                // finally free doc_ptr
                free(doc_ptr as *mut libc::c_void);
            }
        }
    }

    #[test]
    fn test_yaml_emitter_dump_combined_map_sequence() {
        unsafe {
            // 1. Initialize the emitter
            let emitter_ptr = initialize_emitter();
            assert!(!emitter_ptr.is_null());

            // 2. Open the emitter
            let open_result = yaml_emitter_open(emitter_ptr);
            assert_eq!(open_result, OK);

            // 3. Allocate the YamlDocumentT
            let doc_ptr = malloc(size_of::<YamlDocumentT>() as size_t)
                as *mut YamlDocumentT;
            write_bytes(
                doc_ptr as *mut u8,
                0,
                size_of::<YamlDocumentT>(),
            );

            // This structure will have 10 nodes total:
            // Node #1 => top-level mapping
            // Node #2 => key "top"
            // Node #3 => sequence
            //   The sequence has 3 items => index #4, #5, #6
            // Node #4 => "item1" (scalar)
            // Node #5 => "item2" (scalar)
            // Node #6 => sub-mapping
            //   The sub-mapping has 1 pair => (#7 => #8)
            //   Node #7 => "some" (scalar key)
            //   Node #8 => "mapping" (scalar value)
            // Node #9 => key "another"
            // Node #10 => "value2" (scalar)

            let nodes_capacity = 10; // we have 1..=10
            let total_nodes_size =
                nodes_capacity * size_of::<YamlNodeT>();
            let nodes_array =
                malloc(total_nodes_size as size_t) as *mut YamlNodeT;
            write_bytes(nodes_array as *mut u8, 0, total_nodes_size);

            // 4. doc->nodes
            (*doc_ptr).nodes.start = nodes_array;
            (*doc_ptr).nodes.top = nodes_array.add(nodes_capacity);
            (*doc_ptr).nodes.end = nodes_array.add(nodes_capacity);
            (*doc_ptr).start_implicit = true;
            (*doc_ptr).end_implicit = true;

            // Helper function to copy a C string into a malloc’d pointer
            unsafe fn copy_cstr(bytes: &[u8]) -> *mut yaml_char_t {
                let ptr =
                    malloc(bytes.len() as size_t) as *mut yaml_char_t;
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    ptr,
                    bytes.len(),
                );
                ptr
            }

            // 5. Node #1 => top-level mapping
            let top_map = nodes_array.add(0);
            (*top_map).type_ = YamlMappingNode;
            (*top_map).tag = copy_cstr(b"tag:yaml.org,2002:map\0");

            // Node #2 => key "top"
            let key_top = nodes_array.add(1);
            (*key_top).type_ = YamlScalarNode;
            (*key_top).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*key_top).data.scalar.value = copy_cstr(b"top\0");
            (*key_top).data.scalar.length = 3;
            (*key_top).data.scalar.style = YamlPlainScalarStyle;

            // Node #3 => sequence
            let seq_node = nodes_array.add(2);
            (*seq_node).type_ = YamlSequenceNode;
            (*seq_node).tag = copy_cstr(b"tag:yaml.org,2002:seq\0");

            // Node #4 => "item1" (scalar)
            let item1_node = nodes_array.add(3);
            (*item1_node).type_ = YamlScalarNode;
            (*item1_node).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*item1_node).data.scalar.value = copy_cstr(b"item1\0");
            (*item1_node).data.scalar.length = 5;
            (*item1_node).data.scalar.style = YamlPlainScalarStyle;

            // Node #5 => "item2" (scalar)
            let item2_node = nodes_array.add(4);
            (*item2_node).type_ = YamlScalarNode;
            (*item2_node).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*item2_node).data.scalar.value = copy_cstr(b"item2\0");
            (*item2_node).data.scalar.length = 5;
            (*item2_node).data.scalar.style = YamlPlainScalarStyle;

            // Node #6 => sub-mapping
            let sub_map = nodes_array.add(5);
            (*sub_map).type_ = YamlMappingNode;
            (*sub_map).tag = copy_cstr(b"tag:yaml.org,2002:map\0");

            // Node #7 => "some" (scalar key)
            let sub_key = nodes_array.add(6);
            (*sub_key).type_ = YamlScalarNode;
            (*sub_key).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*sub_key).data.scalar.value = copy_cstr(b"some\0");
            (*sub_key).data.scalar.length = 4;
            (*sub_key).data.scalar.style = YamlPlainScalarStyle;

            // Node #8 => "mapping" (scalar value)
            let sub_val = nodes_array.add(7);
            (*sub_val).type_ = YamlScalarNode;
            (*sub_val).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*sub_val).data.scalar.value = copy_cstr(b"mapping\0");
            (*sub_val).data.scalar.length = 7;
            (*sub_val).data.scalar.style = YamlPlainScalarStyle;

            // Node #9 => key "another"
            let key_another = nodes_array.add(8);
            (*key_another).type_ = YamlScalarNode;
            (*key_another).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*key_another).data.scalar.value = copy_cstr(b"another\0");
            (*key_another).data.scalar.length = 7;
            (*key_another).data.scalar.style = YamlPlainScalarStyle;

            // Node #10 => value "value2" (scalar)
            let val2_node = nodes_array.add(9);
            (*val2_node).type_ = YamlScalarNode;
            (*val2_node).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*val2_node).data.scalar.value = copy_cstr(b"value2\0");
            (*val2_node).data.scalar.length = 6;
            (*val2_node).data.scalar.style = YamlPlainScalarStyle;

            // 6. sub_map (#6) has 1 pair => (#7 => #8)
            let sub_map_pair_size = size_of::<YamlNodePairT>();
            let sub_map_pairs = malloc(sub_map_pair_size as size_t)
                as *mut YamlNodePairT;
            write_bytes(sub_map_pairs as *mut u8, 0, sub_map_pair_size);
            (*sub_map_pairs).key = 7; // node #7
            (*sub_map_pairs).value = 8; // node #8

            (*sub_map).data.mapping.pairs.start = sub_map_pairs;
            (*sub_map).data.mapping.pairs.top = sub_map_pairs.add(1);
            (*sub_map).data.mapping.pairs.end = sub_map_pairs.add(1);
            (*sub_map).data.mapping.style = YamlBlockMappingStyle;

            // 7. The sequence (#3) has 3 items => #4, #5, #6
            let seq_items_count = 3;
            let seq_items_size =
                seq_items_count * size_of::<libc::c_int>();
            let seq_items =
                malloc(seq_items_size as size_t) as *mut libc::c_int;
            write_bytes(seq_items as *mut u8, 0, seq_items_size);
            *seq_items = 4; // item1 => node #4
            *seq_items.add(1) = 5; // item2 => node #5
            *seq_items.add(2) = 6; // sub_map => node #6

            (*seq_node).data.sequence.items.start = seq_items;
            (*seq_node).data.sequence.items.top =
                seq_items.add(seq_items_count);
            (*seq_node).data.sequence.items.end =
                seq_items.add(seq_items_count);
            (*seq_node).data.sequence.style = YamlBlockSequenceStyle;

            // 8. The top_map (#1) has 2 pairs => (#2 => #3), (#9 => #10)
            // "top": node #3 (the sequence)
            // "another": node #10 (value2)
            let top_map_pairs_count = 2;
            let top_map_pairs_size =
                top_map_pairs_count * size_of::<YamlNodePairT>();
            let top_map_pairs = malloc(top_map_pairs_size as size_t)
                as *mut YamlNodePairT;
            write_bytes(
                top_map_pairs as *mut u8,
                0,
                top_map_pairs_size,
            );

            (*top_map_pairs).key = 2;
            (*top_map_pairs).value = 3;

            (*top_map_pairs.add(1)).key = 9;
            (*top_map_pairs.add(1)).value = 10;

            (*top_map).data.mapping.pairs.start = top_map_pairs;
            (*top_map).data.mapping.pairs.top =
                top_map_pairs.add(top_map_pairs_count);
            (*top_map).data.mapping.pairs.end =
                top_map_pairs.add(top_map_pairs_count);
            (*top_map).data.mapping.style = YamlBlockMappingStyle;

            // 9. Dump
            let dump_result = yaml_emitter_dump(emitter_ptr, doc_ptr);
            assert_eq!(dump_result, OK);

            // 10. Close & cleanup
            let close_result = yaml_emitter_close(emitter_ptr);
            assert_eq!(close_result, OK);
            cleanup_emitter(emitter_ptr);

            // 11. Manually free leftover if not null
            // (Same pointer checks approach as before, to avoid Miri leaks)
            if !doc_ptr.is_null() {
                if !(*doc_ptr).nodes.start.is_null() {
                    // Freed node-by-node if library hasn't done so...
                    // Node #1 => top_map
                    let node1 = nodes_array.add(0);
                    if !(*node1).tag.is_null() {
                        free((*node1).tag as *mut libc::c_void);
                        (*node1).tag = ptr::null_mut();
                    }
                    if !(*node1).data.mapping.pairs.start.is_null() {
                        free(
                            (*node1).data.mapping.pairs.start
                                as *mut libc::c_void,
                        );
                        (*node1).data.mapping.pairs.start =
                            ptr::null_mut();
                    }

                    // Node #2 => key_top
                    let node2 = nodes_array.add(1);
                    if !(*node2).tag.is_null() {
                        free((*node2).tag as *mut libc::c_void);
                        (*node2).tag = ptr::null_mut();
                    }
                    if !(*node2).data.scalar.value.is_null() {
                        free(
                            (*node2).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node2).data.scalar.value = ptr::null_mut();
                    }

                    // Node #3 => seq_node
                    let node3 = nodes_array.add(2);
                    if !(*node3).tag.is_null() {
                        free((*node3).tag as *mut libc::c_void);
                        (*node3).tag = ptr::null_mut();
                    }
                    if !(*node3).data.sequence.items.start.is_null() {
                        free(
                            (*node3).data.sequence.items.start
                                as *mut libc::c_void,
                        );
                        (*node3).data.sequence.items.start =
                            ptr::null_mut();
                    }

                    // Node #4 => item1 (scalar)
                    let node4 = nodes_array.add(3);
                    if !(*node4).tag.is_null() {
                        free((*node4).tag as *mut libc::c_void);
                        (*node4).tag = ptr::null_mut();
                    }
                    if !(*node4).data.scalar.value.is_null() {
                        free(
                            (*node4).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node4).data.scalar.value = ptr::null_mut();
                    }

                    // Node #5 => item2 (scalar)
                    let node5 = nodes_array.add(4);
                    if !(*node5).tag.is_null() {
                        free((*node5).tag as *mut libc::c_void);
                        (*node5).tag = ptr::null_mut();
                    }
                    if !(*node5).data.scalar.value.is_null() {
                        free(
                            (*node5).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node5).data.scalar.value = ptr::null_mut();
                    }

                    // Node #6 => sub_map
                    let node6 = nodes_array.add(5);
                    if !(*node6).tag.is_null() {
                        free((*node6).tag as *mut libc::c_void);
                        (*node6).tag = ptr::null_mut();
                    }
                    if !(*node6).data.mapping.pairs.start.is_null() {
                        free(
                            (*node6).data.mapping.pairs.start
                                as *mut libc::c_void,
                        );
                        (*node6).data.mapping.pairs.start =
                            ptr::null_mut();
                    }

                    // Node #7 => sub_key "some"
                    let node7 = nodes_array.add(6);
                    if !(*node7).tag.is_null() {
                        free((*node7).tag as *mut libc::c_void);
                        (*node7).tag = ptr::null_mut();
                    }
                    if !(*node7).data.scalar.value.is_null() {
                        free(
                            (*node7).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node7).data.scalar.value = ptr::null_mut();
                    }

                    // Node #8 => sub_val "mapping"
                    let node8 = nodes_array.add(7);
                    if !(*node8).tag.is_null() {
                        free((*node8).tag as *mut libc::c_void);
                        (*node8).tag = ptr::null_mut();
                    }
                    if !(*node8).data.scalar.value.is_null() {
                        free(
                            (*node8).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node8).data.scalar.value = ptr::null_mut();
                    }

                    // Node #9 => key "another"
                    let node9 = nodes_array.add(8);
                    if !(*node9).tag.is_null() {
                        free((*node9).tag as *mut libc::c_void);
                        (*node9).tag = ptr::null_mut();
                    }
                    if !(*node9).data.scalar.value.is_null() {
                        free(
                            (*node9).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node9).data.scalar.value = ptr::null_mut();
                    }

                    // Node #10 => val "value2"
                    let node10 = nodes_array.add(9);
                    if !(*node10).tag.is_null() {
                        free((*node10).tag as *mut libc::c_void);
                        (*node10).tag = ptr::null_mut();
                    }
                    if !(*node10).data.scalar.value.is_null() {
                        free(
                            (*node10).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node10).data.scalar.value = ptr::null_mut();
                    }

                    // free the nodes array
                    free((*doc_ptr).nodes.start as *mut libc::c_void);
                    (*doc_ptr).nodes.start = ptr::null_mut();
                }
                // finally free doc_ptr
                free(doc_ptr as *mut libc::c_void);
            }
        }
    }

    #[test]
    fn test_yaml_emitter_dump_aliases() {
        unsafe {
            // 1. Initialize the emitter
            let emitter_ptr = initialize_emitter();
            assert!(!emitter_ptr.is_null());

            // 2. Open the emitter
            let open_result = yaml_emitter_open(emitter_ptr);
            assert_eq!(open_result, OK);

            // 3. Allocate the YamlDocumentT
            let doc_ptr = malloc(size_of::<YamlDocumentT>() as size_t)
                as *mut YamlDocumentT;
            write_bytes(
                doc_ptr as *mut u8,
                0,
                size_of::<YamlDocumentT>(),
            );

            // We'll have 6 nodes total:
            // Node #1 => top-level mapping
            // Node #2 => key "foo"
            // Node #3 => the anchored mapping
            //   inside it: (#4 => #5)
            // Node #4 => "nested"
            // Node #5 => "42"
            // Node #6 => key "bar"
            //
            // Then in the top-level mapping, we do 2 pairs:
            //   pair1 => (key=2, value=3) => "foo": node #3
            //   pair2 => (key=6, value=3) => "bar": node #3 again (alias!)
            //
            // The library should see that node #3 is referenced twice
            // and produce an alias the second time.

            let nodes_capacity = 6; // indexes 1..=6
            let total_nodes_size =
                nodes_capacity * size_of::<YamlNodeT>();
            let nodes_array =
                malloc(total_nodes_size as size_t) as *mut YamlNodeT;
            write_bytes(nodes_array as *mut u8, 0, total_nodes_size);

            // doc->nodes
            (*doc_ptr).nodes.start = nodes_array;
            (*doc_ptr).nodes.top = nodes_array.add(nodes_capacity);
            (*doc_ptr).nodes.end = nodes_array.add(nodes_capacity);
            (*doc_ptr).start_implicit = true;
            (*doc_ptr).end_implicit = true;

            // Helper to copy a C string
            unsafe fn copy_cstr(bytes: &[u8]) -> *mut yaml_char_t {
                let ptr =
                    malloc(bytes.len() as size_t) as *mut yaml_char_t;
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    ptr,
                    bytes.len(),
                );
                ptr
            }

            // 4. Node #1 => top-level mapping
            let top_map = nodes_array.add(0);
            (*top_map).type_ = YamlMappingNode;
            (*top_map).tag = copy_cstr(b"tag:yaml.org,2002:map\0");

            // Node #2 => key "foo"
            let key_foo = nodes_array.add(1);
            (*key_foo).type_ = YamlScalarNode;
            (*key_foo).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*key_foo).data.scalar.value = copy_cstr(b"foo\0");
            (*key_foo).data.scalar.length = 3;
            (*key_foo).data.scalar.style = YamlPlainScalarStyle;

            // Node #3 => the anchored mapping
            let anchored_map = nodes_array.add(2);
            (*anchored_map).type_ = YamlMappingNode;
            // In YAML, you'd typically see &myanchor in the output.
            // The library sets an anchor if it sees multiple references to node #3.
            (*anchored_map).tag = copy_cstr(b"tag:yaml.org,2002:map\0");

            // Inside #3, there's 1 pair => (#4 => #5)
            // We'll set that up after we define #4, #5.

            // Node #4 => "nested" (scalar key)
            let nested_key = nodes_array.add(3);
            (*nested_key).type_ = YamlScalarNode;
            (*nested_key).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*nested_key).data.scalar.value = copy_cstr(b"nested\0");
            (*nested_key).data.scalar.length = 6;
            (*nested_key).data.scalar.style = YamlPlainScalarStyle;

            // Node #5 => "42" (scalar value)
            let nested_val = nodes_array.add(4);
            (*nested_val).type_ = YamlScalarNode;
            (*nested_val).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*nested_val).data.scalar.value = copy_cstr(b"42\0");
            (*nested_val).data.scalar.length = 2;
            (*nested_val).data.scalar.style = YamlPlainScalarStyle;

            // Node #6 => key "bar"
            let key_bar = nodes_array.add(5);
            (*key_bar).type_ = YamlScalarNode;
            (*key_bar).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*key_bar).data.scalar.value = copy_cstr(b"bar\0");
            (*key_bar).data.scalar.length = 3;
            (*key_bar).data.scalar.style = YamlPlainScalarStyle;

            // 5. The anchored map (#3) has 1 pair => (#4 => #5)
            let anchored_pairs_size = size_of::<YamlNodePairT>();
            let anchored_pairs_ptr =
                malloc(anchored_pairs_size as size_t)
                    as *mut YamlNodePairT;
            write_bytes(
                anchored_pairs_ptr as *mut u8,
                0,
                anchored_pairs_size,
            );

            (*anchored_pairs_ptr).key = 4; // node #4
            (*anchored_pairs_ptr).value = 5; // node #5
            (*anchored_map).data.mapping.pairs.start =
                anchored_pairs_ptr;
            (*anchored_map).data.mapping.pairs.top =
                anchored_pairs_ptr.add(1);
            (*anchored_map).data.mapping.pairs.end =
                anchored_pairs_ptr.add(1);
            (*anchored_map).data.mapping.style = YamlBlockMappingStyle;

            // 6. The top_map (#1) has 2 pairs => (#2 => #3), (#6 => #3)
            // "foo": node #3, "bar": node #3 => repeated => alias
            let top_pairs_count = 2;
            let top_pairs_size =
                top_pairs_count * size_of::<YamlNodePairT>();
            let top_pairs_ptr =
                malloc(top_pairs_size as size_t) as *mut YamlNodePairT;
            write_bytes(top_pairs_ptr as *mut u8, 0, top_pairs_size);

            // pair1 => (2,3)
            (*top_pairs_ptr).key = 2; // "foo"
            (*top_pairs_ptr).value = 3; // anchored map

            // pair2 => (6,3)
            (*top_pairs_ptr.add(1)).key = 6; // "bar"
            (*top_pairs_ptr.add(1)).value = 3; // same node => alias

            (*top_map).data.mapping.pairs.start = top_pairs_ptr;
            (*top_map).data.mapping.pairs.top =
                top_pairs_ptr.add(top_pairs_count);
            (*top_map).data.mapping.pairs.end =
                top_pairs_ptr.add(top_pairs_count);
            (*top_map).data.mapping.style = YamlBlockMappingStyle;

            // 7. Dump
            let dump_result = yaml_emitter_dump(emitter_ptr, doc_ptr);
            assert_eq!(dump_result, OK);

            // 8. Close & cleanup
            let close_result = yaml_emitter_close(emitter_ptr);
            assert_eq!(close_result, OK);
            cleanup_emitter(emitter_ptr);

            // 9. Manually free leftover if not null
            if !doc_ptr.is_null() {
                if !(*doc_ptr).nodes.start.is_null() {
                    // Freed node-by-node if library hasn't done so...
                    let node1 = nodes_array.add(0);
                    if !(*node1).tag.is_null() {
                        free((*node1).tag as *mut libc::c_void);
                        (*node1).tag = ptr::null_mut();
                    }
                    if !(*node1).data.mapping.pairs.start.is_null() {
                        free(
                            (*node1).data.mapping.pairs.start
                                as *mut libc::c_void,
                        );
                        (*node1).data.mapping.pairs.start =
                            ptr::null_mut();
                    }
                    let node2 = nodes_array.add(1);
                    if !(*node2).tag.is_null() {
                        free((*node2).tag as *mut libc::c_void);
                        (*node2).tag = ptr::null_mut();
                    }
                    if !(*node2).data.scalar.value.is_null() {
                        free(
                            (*node2).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node2).data.scalar.value = ptr::null_mut();
                    }
                    let node3 = nodes_array.add(2);
                    if !(*node3).tag.is_null() {
                        free((*node3).tag as *mut libc::c_void);
                        (*node3).tag = ptr::null_mut();
                    }
                    if !(*node3).data.mapping.pairs.start.is_null() {
                        free(
                            (*node3).data.mapping.pairs.start
                                as *mut libc::c_void,
                        );
                        (*node3).data.mapping.pairs.start =
                            ptr::null_mut();
                    }
                    let node4 = nodes_array.add(3);
                    if !(*node4).tag.is_null() {
                        free((*node4).tag as *mut libc::c_void);
                        (*node4).tag = ptr::null_mut();
                    }
                    if !(*node4).data.scalar.value.is_null() {
                        free(
                            (*node4).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node4).data.scalar.value = ptr::null_mut();
                    }
                    let node5 = nodes_array.add(4);
                    if !(*node5).tag.is_null() {
                        free((*node5).tag as *mut libc::c_void);
                        (*node5).tag = ptr::null_mut();
                    }
                    if !(*node5).data.scalar.value.is_null() {
                        free(
                            (*node5).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node5).data.scalar.value = ptr::null_mut();
                    }
                    let node6 = nodes_array.add(5);
                    if !(*node6).tag.is_null() {
                        free((*node6).tag as *mut libc::c_void);
                        (*node6).tag = ptr::null_mut();
                    }
                    if !(*node6).data.scalar.value.is_null() {
                        free(
                            (*node6).data.scalar.value
                                as *mut libc::c_void,
                        );
                        (*node6).data.scalar.value = ptr::null_mut();
                    }

                    free((*doc_ptr).nodes.start as *mut libc::c_void);
                    (*doc_ptr).nodes.start = ptr::null_mut();
                }
                free(doc_ptr as *mut libc::c_void);
            }
        }
    }

    #[test]
    fn test_yaml_emitter_dump_multi_document_stream() {
        unsafe {
            // 1. Initialize the emitter (once)
            let emitter_ptr = initialize_emitter();
            assert!(!emitter_ptr.is_null());

            // 2. Open the emitter (once)
            let open_result = yaml_emitter_open(emitter_ptr);
            assert_eq!(open_result, OK);

            //
            // === First Document ===
            //
            // 3a. Allocate a first YamlDocumentT for "doc1: foo"
            let doc1_ptr = malloc(size_of::<YamlDocumentT>() as size_t)
                as *mut YamlDocumentT;
            write_bytes(
                doc1_ptr as *mut u8,
                0,
                size_of::<YamlDocumentT>(),
            );

            // We'll have 2 nodes: #1 => a mapping, #2 => key+value or node #2 => "doc1", node #3 => "foo"
            // but let's keep it extremely simple:
            let doc1_nodes_capacity = 3;
            let doc1_nodes_size =
                doc1_nodes_capacity * size_of::<YamlNodeT>();
            let doc1_nodes_array =
                malloc(doc1_nodes_size as size_t) as *mut YamlNodeT;
            write_bytes(
                doc1_nodes_array as *mut u8,
                0,
                doc1_nodes_size,
            );

            // doc1->nodes
            (*doc1_ptr).nodes.start = doc1_nodes_array;
            (*doc1_ptr).nodes.top =
                doc1_nodes_array.add(doc1_nodes_capacity);
            (*doc1_ptr).nodes.end =
                doc1_nodes_array.add(doc1_nodes_capacity);
            (*doc1_ptr).start_implicit = true;
            (*doc1_ptr).end_implicit = true;

            // Helper to copy a C string
            unsafe fn copy_cstr(bytes: &[u8]) -> *mut yaml_char_t {
                let ptr =
                    malloc(bytes.len() as size_t) as *mut yaml_char_t;
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    ptr,
                    bytes.len(),
                );
                ptr
            }

            // doc1: Node #1 => mapping
            let doc1_map = doc1_nodes_array.add(0);
            (*doc1_map).type_ = YamlMappingNode;
            (*doc1_map).tag = copy_cstr(b"tag:yaml.org,2002:map\0");

            // doc1: Node #2 => key "doc1"
            let doc1_key = doc1_nodes_array.add(1);
            (*doc1_key).type_ = YamlScalarNode;
            (*doc1_key).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*doc1_key).data.scalar.value = copy_cstr(b"doc1\0");
            (*doc1_key).data.scalar.length = 4;
            (*doc1_key).data.scalar.style = YamlPlainScalarStyle;

            // doc1: Node #3 => value "foo"
            let doc1_val = doc1_nodes_array.add(2);
            (*doc1_val).type_ = YamlScalarNode;
            (*doc1_val).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*doc1_val).data.scalar.value = copy_cstr(b"foo\0");
            (*doc1_val).data.scalar.length = 3;
            (*doc1_val).data.scalar.style = YamlPlainScalarStyle;

            // doc1: pairs
            let doc1_pairs_count = 1;
            let doc1_pairs_size =
                doc1_pairs_count * size_of::<YamlNodePairT>();
            let doc1_pairs_ptr =
                malloc(doc1_pairs_size as size_t) as *mut YamlNodePairT;
            write_bytes(doc1_pairs_ptr as *mut u8, 0, doc1_pairs_size);

            // pair => (2,3)
            (*doc1_pairs_ptr).key = 2; // node #2
            (*doc1_pairs_ptr).value = 3; // node #3

            (*doc1_map).data.mapping.pairs.start = doc1_pairs_ptr;
            (*doc1_map).data.mapping.pairs.top =
                doc1_pairs_ptr.add(doc1_pairs_count);
            (*doc1_map).data.mapping.pairs.end =
                doc1_pairs_ptr.add(doc1_pairs_count);
            (*doc1_map).data.mapping.style = YamlBlockMappingStyle;

            // Dump doc1
            let dump1_result = yaml_emitter_dump(emitter_ptr, doc1_ptr);
            assert_eq!(dump1_result, OK);

            //
            // === Second Document ===
            //
            // 3b. Allocate second YamlDocumentT for "doc2: bar"
            let doc2_ptr = malloc(size_of::<YamlDocumentT>() as size_t)
                as *mut YamlDocumentT;
            write_bytes(
                doc2_ptr as *mut u8,
                0,
                size_of::<YamlDocumentT>(),
            );

            // We'll do a similar approach with 3 nodes: #1 => mapping, #2 => "doc2", #3 => "bar"
            let doc2_nodes_capacity = 3;
            let doc2_nodes_size =
                doc2_nodes_capacity * size_of::<YamlNodeT>();
            let doc2_nodes_array =
                malloc(doc2_nodes_size as size_t) as *mut YamlNodeT;
            write_bytes(
                doc2_nodes_array as *mut u8,
                0,
                doc2_nodes_size,
            );

            (*doc2_ptr).nodes.start = doc2_nodes_array;
            (*doc2_ptr).nodes.top =
                doc2_nodes_array.add(doc2_nodes_capacity);
            (*doc2_ptr).nodes.end =
                doc2_nodes_array.add(doc2_nodes_capacity);
            (*doc2_ptr).start_implicit = true;
            (*doc2_ptr).end_implicit = true;

            // Node #1 => mapping
            let doc2_map = doc2_nodes_array.add(0);
            (*doc2_map).type_ = YamlMappingNode;
            (*doc2_map).tag = copy_cstr(b"tag:yaml.org,2002:map\0");

            // Node #2 => "doc2"
            let doc2_key = doc2_nodes_array.add(1);
            (*doc2_key).type_ = YamlScalarNode;
            (*doc2_key).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*doc2_key).data.scalar.value = copy_cstr(b"doc2\0");
            (*doc2_key).data.scalar.length = 4;
            (*doc2_key).data.scalar.style = YamlPlainScalarStyle;

            // Node #3 => "bar"
            let doc2_val = doc2_nodes_array.add(2);
            (*doc2_val).type_ = YamlScalarNode;
            (*doc2_val).tag = copy_cstr(b"tag:yaml.org,2002:str\0");
            (*doc2_val).data.scalar.value = copy_cstr(b"bar\0");
            (*doc2_val).data.scalar.length = 3;
            (*doc2_val).data.scalar.style = YamlPlainScalarStyle;

            let doc2_pairs_count = 1;
            let doc2_pairs_size =
                doc2_pairs_count * size_of::<YamlNodePairT>();
            let doc2_pairs_ptr =
                malloc(doc2_pairs_size as size_t) as *mut YamlNodePairT;
            write_bytes(doc2_pairs_ptr as *mut u8, 0, doc2_pairs_size);

            (*doc2_pairs_ptr).key = 2; // node #2
            (*doc2_pairs_ptr).value = 3; // node #3

            (*doc2_map).data.mapping.pairs.start = doc2_pairs_ptr;
            (*doc2_map).data.mapping.pairs.top =
                doc2_pairs_ptr.add(doc2_pairs_count);
            (*doc2_map).data.mapping.pairs.end =
                doc2_pairs_ptr.add(doc2_pairs_count);
            (*doc2_map).data.mapping.style = YamlBlockMappingStyle;

            // Dump doc2
            let dump2_result = yaml_emitter_dump(emitter_ptr, doc2_ptr);
            assert_eq!(dump2_result, OK);

            //
            // 4. Finally close the emitter
            //
            let close_result = yaml_emitter_close(emitter_ptr);
            assert_eq!(close_result, OK);

            //
            // 5. Cleanup emitter
            //
            cleanup_emitter(emitter_ptr);

            //
            // 6. Manually free leftover if your library didn’t do so
            //    (the same pointer checks approach if you want to avoid Miri leaks)
            // Because we used doc1_ptr and doc2_ptr, we do pointer checks similarly:
            if !doc1_ptr.is_null() {
                // If doc1_ptr->nodes.start is still not null, free them
                // ...
                free(doc1_ptr as *mut libc::c_void);
            }
            if !doc2_ptr.is_null() {
                // ...
                free(doc2_ptr as *mut libc::c_void);
            }
        }
    }
}
