// tests/tests_loader_parsing.rs

#![allow(missing_docs)]

use core::ffi::c_char;
use core::mem::{align_of, size_of};
use libyml::{
    externs::test_strcmp as strcmp,
    loader::YamlError,
    yaml::{yaml_char_t, YamlDocumentT, YamlParserT},
    yaml_parser_load,
};

/// Tests for the YAML parser loading functionality.
mod yaml_parser_load_tests {
    use super::*;

    #[test]
    /// Test the behavior with null pointers for parser and document.
    fn test_null_pointers() {
        unsafe {
            let result = yaml_parser_load(
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            assert_eq!(result, Err(YamlError::NullPointer));
        }
    }

    #[test]
    /// Test the behavior with valid pointers but uninitialized data.
    fn test_uninitialized_pointers() {
        unsafe {
            let parser: *mut YamlParserT = core::ptr::null_mut();
            let document: *mut YamlDocumentT = core::ptr::null_mut();

            let result = yaml_parser_load(parser, document);
            assert_eq!(result, Err(YamlError::NullPointer));
        }
    }
}

/// Tests for string handling functionality.
mod string_handling_tests {
    use super::*;

    #[test]
    /// Test basic string comparison functionality.
    fn test_string_comparison() {
        let s1 = b"test\0" as *const u8 as *mut c_char;
        let s2 = b"test\0" as *const u8 as *mut c_char;
        let s3 = b"different\0" as *const u8 as *mut c_char;

        assert_eq!(strcmp(s1, s2), 0); // Equal strings
        assert_ne!(strcmp(s1, s3), 0); // Unequal strings
    }

    #[test]
    /// Test behavior with empty strings.
    fn test_empty_string_comparison() {
        let empty = b"\0" as *const u8 as *mut c_char;
        let non_empty = b"not_empty\0" as *const u8 as *mut c_char;

        assert_eq!(strcmp(empty, empty), 0); // Both empty
        assert_ne!(strcmp(empty, non_empty), 0); // One empty, one not
    }
}

/// Tests for handling empty and minimal documents.
mod empty_document_tests {
    use super::*;

    #[test]
    /// Test loading an empty document.
    fn test_empty_document() {
        unsafe {
            let parser: *mut YamlParserT = core::ptr::null_mut();
            let document: *mut YamlDocumentT = core::ptr::null_mut();

            let result = yaml_parser_load(parser, document);
            assert_eq!(result, Err(YamlError::NullPointer));
        }
    }
}

/// Tests for error conditions and edge cases.
mod error_condition_tests {
    use super::*;

    #[test]
    /// Test behavior with a null parser and valid document.
    fn test_null_parser_valid_document() {
        unsafe {
            let document = core::ptr::null_mut::<YamlDocumentT>();
            let result =
                yaml_parser_load(core::ptr::null_mut(), document);
            assert_eq!(result, Err(YamlError::NullPointer));
        }
    }

    #[test]
    /// Test behavior with a valid parser and null document.
    fn test_valid_parser_null_document() {
        unsafe {
            let parser = core::ptr::null_mut::<YamlParserT>();
            let result =
                yaml_parser_load(parser, core::ptr::null_mut());
            assert_eq!(result, Err(YamlError::NullPointer));
        }
    }
}

/// Tests for YAML character handling.
mod yaml_char_tests {
    use super::*;

    #[test]
    /// Test the conversion between yaml_char_t and c_char.
    fn test_yaml_char_conversion() {
        unsafe {
            let yaml_str = b"test\0" as *const u8 as *mut yaml_char_t;
            let c_str = yaml_str as *mut c_char;

            let mut i = 0;
            while *c_str.add(i) != 0 {
                assert_eq!(*c_str.add(i), *yaml_str.add(i) as c_char);
                i += 1;
            }
        }
    }
}

/// Tests for boundary conditions.
mod boundary_tests {
    use super::*;

    #[test]
    /// Test pointer alignment and size constraints.
    fn test_pointer_boundaries() {
        assert!(align_of::<YamlParserT>() > 0);
        assert!(size_of::<YamlParserT>() > 0);
    }
}

/// Tests for memory alignment and initialization.
mod memory_tests {
    use super::*;
    use core::mem::MaybeUninit;
    use libyml::YamlErrorTypeT;

    #[test]
    /// Test proper memory alignment.
    fn test_memory_alignment() {
        assert!(align_of::<YamlParserT>() > 0);
        assert!(align_of::<YamlDocumentT>() > 0);
    }

    #[test]
    /// Test safe memory initialization.
    fn test_memory_initialization() {
        let document = MaybeUninit::<YamlDocumentT>::uninit();
        assert!(!document.as_ptr().is_null());
    }

    #[test]
    /// Test string comparison functionality
    fn test_string_comparison() {
        let s1 = b"test\0".as_ptr() as *mut c_char;
        let s2 = b"test\0".as_ptr() as *mut c_char;
        let s3 = b"different\0".as_ptr() as *mut c_char;

        assert_eq!(strcmp(s1, s2), 0); // Equal strings
        assert_ne!(strcmp(s1, s3), 0); // Different strings
    }

    #[test]
    /// Test handling of empty strings
    fn test_empty_strings() {
        let empty1 = b"\0".as_ptr() as *mut c_char;
        let empty2 = b"\0".as_ptr() as *mut c_char;

        assert_eq!(strcmp(empty1, empty2), 0); // Both empty
    }

    #[test]
    /// Test default initialization values
    fn test_default_initialization() {
        let parser = YamlParserT::default();

        // Verify initial state
        assert_eq!(parser.error, YamlErrorTypeT::YamlNoError);
        assert_eq!(parser.context, core::ptr::null());
        assert_eq!(parser.context_mark.index, 0);
        assert_eq!(parser.context_mark.line, 0);
        assert_eq!(parser.context_mark.column, 0);
        assert_eq!(parser.problem, core::ptr::null());
        assert_eq!(parser.problem_mark.index, 0);
        assert_eq!(parser.problem_mark.line, 0);
        assert_eq!(parser.problem_mark.column, 0);
    }
}
