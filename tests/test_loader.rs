#[cfg(test)]
mod tests {
    use core::mem::MaybeUninit;
    use libyml::*;
    use libyml::api::{yaml_parser_initialize, yaml_parser_set_input_string};
    use libyml::success::is_success;

    #[test]
    fn test_yaml_parser_load() {
        unsafe {
            let mut parser = MaybeUninit::<YamlParserT>::uninit();
            let mut document = MaybeUninit::<YamlDocumentT>::uninit();
            let input = b"key: value\n";

            // Initialize the parser
            assert!(is_success(yaml_parser_initialize(parser.as_mut_ptr())));
            let parser = parser.assume_init_mut();

            // Set the input string
            yaml_parser_set_input_string(parser, input.as_ptr(), input.len() as u64);

            // Load the document
            let result = yaml_parser_load(parser, document.as_mut_ptr());
            assert!(is_success(result));

            // Clean up
            yaml_document_delete(document.as_mut_ptr());
            yaml_parser_delete(parser);
        }
    }
}
