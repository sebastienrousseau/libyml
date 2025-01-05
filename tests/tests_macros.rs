#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use core::ptr;
    use core::ptr::{addr_of_mut, null_mut};
    use libyml::externs::{free, malloc, memset};
    use libyml::libc;
    use libyml::string::{yaml_string_extend, yaml_string_join};
    use libyml::yaml::{size_t, yaml_char_t};
    use libyml::{
        AS_DIGIT, AS_HEX_AT, BUFFER_DEL, BUFFER_INIT, CLEAR, IS_ALPHA,
        IS_ASCII, IS_DIGIT, IS_HEX_AT, JOIN, STRING_ASSIGN, STRING_DEL,
        STRING_EXTEND, STRING_INIT,
    };

    #[derive(Debug, PartialEq)]
    struct YamlBufferT {
        start: *mut yaml_char_t,
        pointer: *mut yaml_char_t,
        last: *mut yaml_char_t,
        end: *mut yaml_char_t,
    }

    #[derive(Debug, PartialEq)]
    struct YamlStringT {
        start: *mut yaml_char_t,
        pointer: *mut yaml_char_t,
        end: *mut yaml_char_t,
    }

    // Mock `yaml_malloc(...)` => calls our unsafe `malloc`
    #[no_mangle]
    extern "C" fn yaml_malloc(size: size_t) -> *mut libc::c_void {
        unsafe { malloc(size) }
    }

    // Mock `yaml_free(...)` => calls our unsafe `free`
    #[no_mangle]
    extern "C" fn yaml_free(ptr: *mut libc::c_void) {
        unsafe { free(ptr) }
    }

    // ----------------------------------------------------------------------
    //  1) Tests for Buffers
    // ----------------------------------------------------------------------

    #[test]
    fn test_buffer_init() {
        let mut buffer = YamlBufferT {
            start: null_mut(),
            pointer: null_mut(),
            last: null_mut(),
            end: null_mut(),
        };
        let size = 16;

        unsafe {
            BUFFER_INIT!(buffer, size);
        }

        assert!(!buffer.start.is_null());
        assert_eq!(buffer.pointer, buffer.start);
        assert_eq!(buffer.last, buffer.start);
        assert_eq!(buffer.end, unsafe { buffer.start.add(size) });

        // Free the buffer
        BUFFER_DEL!(buffer);
    }

    #[test]
    fn test_buffer_del() {
        let mut buffer = YamlBufferT {
            start: yaml_malloc(16) as *mut yaml_char_t,
            pointer: null_mut(),
            last: null_mut(),
            end: null_mut(),
        };
        buffer.pointer = buffer.start;
        buffer.last = buffer.start;
        buffer.end = unsafe { buffer.start.add(16) };

        BUFFER_DEL!(buffer);

        assert!(buffer.start.is_null());
        assert_eq!(buffer.pointer, buffer.start);
        assert_eq!(buffer.last, buffer.start);
        assert_eq!(buffer.end, buffer.start);
    }

    // ----------------------------------------------------------------------
    //  2) Tests for Strings
    // ----------------------------------------------------------------------

    #[test]
    fn test_string_assign() {
        // Use a stack buffer so we don't need free here
        let mut buffer = [0u8; 10];
        let start_ptr = buffer.as_mut_ptr();
        let length: isize = buffer.len() as isize;

        let yaml_str = STRING_ASSIGN!(start_ptr, length);

        assert_eq!(yaml_str.start, start_ptr);
        assert_eq!(yaml_str.end, unsafe {
            start_ptr.add(length as usize)
        });
        assert_eq!(yaml_str.pointer, start_ptr);
        // No `yaml_free` needed, because we used a stack buffer
    }

    #[test]
    fn test_string_init() {
        let mut yaml_str = YamlStringT {
            start: null_mut(),
            pointer: null_mut(),
            end: null_mut(),
        };

        unsafe {
            STRING_INIT!(yaml_str);
        }

        assert!(!yaml_str.start.is_null());
        assert_eq!(yaml_str.pointer, yaml_str.start);
        assert_eq!(yaml_str.end, unsafe { yaml_str.start.add(16) });

        // Freed at the end
        STRING_DEL!(yaml_str);
    }

    #[test]
    fn test_string_del() {
        let mut yaml_str = YamlStringT {
            start: yaml_malloc(16) as *mut yaml_char_t,
            pointer: null_mut(),
            end: null_mut(),
        };
        yaml_str.pointer = yaml_str.start;
        yaml_str.end = unsafe { yaml_str.start.add(16) };

        STRING_DEL!(yaml_str);

        assert!(yaml_str.start.is_null());
        assert_eq!(yaml_str.pointer, yaml_str.start);
        assert_eq!(yaml_str.end, yaml_str.start);
    }

    #[test]
    fn test_string_extend() {
        let mut yaml_str = YamlStringT {
            start: yaml_malloc(5) as *mut yaml_char_t,
            pointer: null_mut(),
            end: null_mut(),
        };
        yaml_str.pointer = yaml_str.start;
        yaml_str.end = unsafe { yaml_str.start.add(5) };

        unsafe {
            STRING_EXTEND!(yaml_str);
        }

        unsafe {
            assert!(yaml_str.end > yaml_str.start.add(5));
        }

        // Freed at the end
        STRING_DEL!(yaml_str);
    }

    #[test]
    fn test_clear() {
        let mut yaml_str = YamlStringT {
            start: yaml_malloc(16) as *mut yaml_char_t,
            pointer: null_mut(),
            end: null_mut(),
        };
        yaml_str.pointer = unsafe { yaml_str.start.add(8) };
        yaml_str.end = unsafe { yaml_str.start.add(16) };

        unsafe {
            CLEAR!(yaml_str);
        }

        assert_eq!(yaml_str.pointer, yaml_str.start);
        // Check if memory is zeroed
        for i in 0..16 {
            assert_eq!(unsafe { *yaml_str.start.add(i) }, 0);
        }

        // Freed
        STRING_DEL!(yaml_str);
    }

    #[test]
    fn test_join() {
        let mut string_a = YamlStringT {
            start: yaml_malloc(16) as *mut yaml_char_t,
            pointer: null_mut(),
            end: null_mut(),
        };
        string_a.pointer = string_a.start;
        unsafe {
            string_a.end = string_a.start.add(16);
        }

        let mut string_b = YamlStringT {
            start: yaml_malloc(8) as *mut yaml_char_t,
            pointer: null_mut(),
            end: null_mut(),
        };
        string_b.pointer = string_b.start;
        unsafe {
            string_b.end = string_b.start.add(8);
            ptr::copy_nonoverlapping(
                "Hello".as_ptr(),
                string_b.start,
                5,
            );
            string_b.pointer = string_b.pointer.add(5);
            JOIN!(string_a, string_b);
        }

        assert_eq!(
            unsafe { string_a.pointer.offset_from(string_a.start) },
            5
        );
        assert_eq!(unsafe { *string_a.start as char }, 'H');
        assert_eq!(unsafe { *string_a.start.add(4) as char }, 'o');

        // Freed
        STRING_DEL!(string_a);
        STRING_DEL!(string_b);
    }

    // ----------------------------------------------------------------------
    //  3) Tests for char classification
    // ----------------------------------------------------------------------

    #[test]
    fn test_is_alpha() {
        let mut yaml_str = YamlStringT {
            start: yaml_malloc(3) as *mut yaml_char_t,
            pointer: null_mut(),
            end: null_mut(),
        };
        unsafe {
            yaml_str.pointer = yaml_str.start;
            yaml_str.end = yaml_str.start.add(3);

            *yaml_str.pointer = b'A';
            assert!(IS_ALPHA!(yaml_str));

            *yaml_str.pointer = b'z';
            assert!(IS_ALPHA!(yaml_str));

            *yaml_str.pointer = b'5';
            assert!(IS_ALPHA!(yaml_str)); // Or if you expect false, change your macro.

            *yaml_str.pointer = b'_';
            assert!(IS_ALPHA!(yaml_str)); // Or false if underscore not alpha

            *yaml_str.pointer = b'-';
            assert!(IS_ALPHA!(yaml_str)); // Or false

            *yaml_str.pointer = b'!';
            assert!(!IS_ALPHA!(yaml_str));

            // Freed
            STRING_DEL!(yaml_str);
        }
    }

    #[test]
    fn test_is_digit() {
        let mut yaml_str = YamlStringT {
            start: yaml_malloc(2) as *mut yaml_char_t,
            pointer: null_mut(),
            end: null_mut(),
        };
        unsafe {
            yaml_str.pointer = yaml_str.start;
            yaml_str.end = yaml_str.start.add(2);

            *yaml_str.pointer = b'5';
            assert!(IS_DIGIT!(yaml_str));

            *yaml_str.pointer = b'A';
            assert!(!IS_DIGIT!(yaml_str));

            STRING_DEL!(yaml_str);
        }
    }

    #[test]
    fn test_as_digit() {
        let mut yaml_str = YamlStringT {
            start: yaml_malloc(1) as *mut yaml_char_t,
            pointer: null_mut(),
            end: null_mut(),
        };
        unsafe {
            yaml_str.pointer = yaml_str.start;
            yaml_str.end = yaml_str.start.add(1);

            *yaml_str.pointer = b'5';
            assert_eq!(AS_DIGIT!(yaml_str), 5);

            STRING_DEL!(yaml_str);
        }
    }

    #[test]
    fn test_is_hex_at() {
        let mut yaml_str = YamlStringT {
            start: yaml_malloc(3) as *mut yaml_char_t,
            pointer: null_mut(),
            end: null_mut(),
        };
        unsafe {
            yaml_str.pointer = yaml_str.start;
            yaml_str.end = yaml_str.start.add(3);

            ptr::copy_nonoverlapping("A5f".as_ptr(), yaml_str.start, 3);
            assert!(IS_HEX_AT!(yaml_str, 0));
            assert!(IS_HEX_AT!(yaml_str, 1));
            assert!(IS_HEX_AT!(yaml_str, 2));

            STRING_DEL!(yaml_str);
        }
    }

    #[test]
    fn test_as_hex_at() {
        let mut yaml_str = YamlStringT {
            start: yaml_malloc(3) as *mut yaml_char_t,
            pointer: null_mut(),
            end: null_mut(),
        };
        unsafe {
            yaml_str.pointer = yaml_str.start;
            yaml_str.end = yaml_str.start.add(3);

            *yaml_str.start = b'A';
            assert_eq!(AS_HEX_AT!(yaml_str, 0), 10);

            STRING_DEL!(yaml_str);
        }
    }

    #[test]
    fn test_is_ascii() {
        let mut yaml_str = YamlStringT {
            start: yaml_malloc(2) as *mut yaml_char_t,
            pointer: null_mut(),
            end: null_mut(),
        };
        unsafe {
            yaml_str.pointer = yaml_str.start;
            yaml_str.end = yaml_str.start.add(2);

            *yaml_str.pointer = 0x7F;
            assert!(IS_ASCII!(yaml_str));

            *yaml_str.pointer = 0x80;
            assert!(!IS_ASCII!(yaml_str));

            STRING_DEL!(yaml_str);
        }
    }
}
