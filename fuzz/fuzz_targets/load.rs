#![no_main]

use libfuzzer_sys::fuzz_target;
use libyml::api::yaml_parser_set_input;
use libyml::decode::{yaml_parser_delete, yaml_parser_initialize};
use libyml::document::{
    yaml_document_delete, yaml_document_get_root_node,
};
use libyml::loader::yaml_parser_load;
use libyml::success::OK;
use libyml::yaml::YamlDocumentT;
use libyml::yaml::YamlParserT;
use std::cmp;
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::ptr;
use std::ptr::addr_of_mut;

fuzz_target!(|data: &[u8]| unsafe { fuzz_target(data) });

unsafe fn fuzz_target(mut data: &[u8]) {
    // Initialize the parser
    let mut parser = MaybeUninit::<YamlParserT>::uninit();
    let init_result = yaml_parser_initialize(parser.as_mut_ptr());
    assert_eq!(init_result, OK, "Parser initialization failed");

    // Set input for the parser
    yaml_parser_set_input(
        parser.as_mut_ptr(),
        read_from_slice,
        addr_of_mut!(data).cast(),
    );

    // Initialize the document
    let mut document = MaybeUninit::<YamlDocumentT>::uninit();
    let document_ptr = document.as_mut_ptr();

    // Parse the document
    while yaml_parser_load(parser.as_mut_ptr(), document_ptr).is_ok() {
        let done = yaml_document_get_root_node(document_ptr).is_null();
        yaml_document_delete(document_ptr);
        if done {
            break;
        }
    }

    // Cleanup parser
    yaml_parser_delete(parser.as_mut_ptr());
}

/// Reads data from a slice into a buffer.
///
/// # Safety
/// This function assumes valid pointers and properly sized buffers.
unsafe fn read_from_slice(
    data: *mut c_void,
    buffer: *mut u8,
    size: u64,
    size_read: *mut u64,
) -> i32 {
    let data = data.cast::<&[u8]>();
    let input = data.read();
    let n = cmp::min(input.len(), size as usize);
    ptr::copy_nonoverlapping(input.as_ptr(), buffer, n);
    data.write(&input[n..]);
    *size_read = n as u64;
    1
}
