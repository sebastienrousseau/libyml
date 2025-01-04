#![allow(missing_docs)]
#![no_std]
#[cfg(test)]
mod tests {
    use core::mem::MaybeUninit;
    use libyml::success::is_success;
    use libyml::*;

    /// Tests the initialization and deletion of the YAML parser.
    #[test]
    fn test_parser_initialize_and_delete() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            assert!(is_success(yaml_parser_initialize(
                parser.as_mut_ptr()
            )));
            let mut parser = parser.assume_init();
            yaml_parser_delete(&mut parser);
        }
    }

    /// Tests setting the input string for the YAML parser.
    #[test]
    fn test_parser_set_input_string() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            assert!(is_success(yaml_parser_initialize(
                parser.as_mut_ptr()
            )));
            let mut parser = parser.assume_init();

            let input = b"key: value\n";
            yaml_parser_set_input_string(
                &mut parser,
                input.as_ptr(),
                input.len() as u64,
            );

            yaml_parser_delete(&mut parser);
        }
    }

    /// Tests parsing of a complex YAML document with nested structures.
    #[test]
    fn test_complex_document() {
        unsafe {
            // 1) Create uninitialized parser
            let mut parser_uninit =
                MaybeUninit::<YamlParserT>::uninit();

            // 2) Initialize parser using a raw pointer
            let parser_ptr = parser_uninit.as_mut_ptr();
            assert!(is_success(yaml_parser_initialize(parser_ptr)));

            // 3) Provide the input
            let input = b"
        parent:
            child1: value1
            child2:
            - list_item1
            - list_item2
        ";
            // Passing `parser_ptr` rather than `&mut parser`
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len() as u64,
            );

            // 4) Prepare an event
            let mut event_uninit = MaybeUninit::<YamlEventT>::uninit();

            // 5) Parse: again pass `parser_ptr` as a raw pointer
            assert!(is_success(yaml_parser_parse(
                parser_ptr,
                event_uninit.as_mut_ptr()
            )));
            let _event = event_uninit.assume_init();

            // 6) Finally delete
            yaml_parser_delete(parser_ptr);
        }
    }
}
