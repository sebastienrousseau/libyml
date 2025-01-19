// writer.rs

//! Writer module for YAML emission
//!
//! This module provides functionality for writing YAML content with support for
//! different character encodings (UTF-8 and UTF-16) in a no_std environment.

use crate::{
    libc,
    success::{Success, FAIL, OK},
    yaml::size_t,
    PointerExt, YamlAnyEncoding, YamlEmitterT, YamlUtf16leEncoding,
    YamlUtf8Encoding, YamlWriterError,
};
use core::ptr::addr_of_mut;

/// Error messages as null-terminated byte strings
const WRITE_ERROR: &[u8] = b"write error\0";
const BUFFER_OVERFLOW: &[u8] = b"buffer overflow\0";
const INVALID_UTF8: &[u8] = b"invalid utf-8 sequence\0";

/// Maximum valid Unicode code point
const MAX_UNICODE: u32 = 0x10FFFF;

/// Sets the writer error for the emitter and returns FAIL.
///
/// # Safety
///
/// The caller must ensure:
/// * `emitter` is a valid pointer to a `YamlEmitterT`
/// * `problem` is a valid pointer to a null-terminated string
///
/// # Arguments
///
/// * `emitter` - Pointer to the YAML emitter
/// * `problem` - Pointer to the error message
///
/// # Returns
///
/// Always returns `Success::FAIL`
#[inline]
pub unsafe fn yaml_emitter_set_writer_error(
    emitter: *mut YamlEmitterT,
    problem: *const libc::c_char,
) -> Success {
    (*emitter).error = YamlWriterError;
    *addr_of_mut!((*emitter).problem) = problem;
    FAIL
}

/// Checks if a UTF-8 sequence starting at the given pointer is valid.
///
/// # Safety
///
/// The caller must ensure:
/// * `ptr` points to valid memory containing UTF-8 data
/// * There are enough bytes available to read based on the first byte
///
/// # Arguments
///
/// * `ptr` - Pointer to the start of the UTF-8 sequence
/// * `remaining` - Number of bytes remaining in the buffer
///
/// # Returns
///
/// `Some(len)` if valid, where len is the sequence length (1-4), or `None` if invalid
#[inline]
unsafe fn check_utf8_sequence(
    ptr: *const u8,
    remaining: isize,
) -> Option<usize> {
    // Add validation for overlong sequences
    let first = *ptr;
    let len = if first < 0x80 {
        1
    } else if first & 0xE0 == 0xC0 && first >= 0xC2 {
        // Prevent overlong sequences
        2
    } else if first & 0xF0 == 0xE0 {
        // Additional validation for 3-byte sequences
        if remaining >= 2 {
            let second = *ptr.offset(1);
            // Prevent overlong sequences and surrogates
            if (first == 0xE0 && second < 0xA0)
                || (first == 0xED && second >= 0xA0)
            {
                return None;
            }
        }
        3
    } else if first & 0xF8 == 0xF0 && first <= 0xF4 {
        // Restrict to valid range
        4
    } else {
        return None;
    };

    if remaining < len as isize {
        return None;
    }

    // Validate continuation bytes
    for i in 1..len {
        if *ptr.add(i) & 0xC0 != 0x80 {
            return None;
        }
    }

    Some(len)
}

/// Converts a UTF-8 sequence to a Unicode code point.
///
/// # Safety
///
/// The caller must ensure:
/// * `ptr` points to a valid UTF-8 sequence
/// * The sequence has already been validated
///
/// # Arguments
///
/// * `ptr` - Pointer to the start of the UTF-8 sequence
/// * `len` - Length of the sequence (1-4 bytes)
///
/// # Returns
///
/// The Unicode code point
#[inline]
unsafe fn utf8_to_codepoint(ptr: *const u8, len: usize) -> u32 {
    match len {
        1 => *ptr as u32,
        2 => {
            (((*ptr & 0x1F) as u32) << 6)
                | ((*ptr.offset(1) & 0x3F) as u32)
        }
        3 => {
            (((*ptr & 0x0F) as u32) << 12)
                | (((*ptr.offset(1) & 0x3F) as u32) << 6)
                | ((*ptr.offset(2) & 0x3F) as u32)
        }
        4 => {
            (((*ptr & 0x07) as u32) << 18)
                | (((*ptr.offset(1) & 0x3F) as u32) << 12)
                | (((*ptr.offset(2) & 0x3F) as u32) << 6)
                | ((*ptr.offset(3) & 0x3F) as u32)
        }
        _ => unreachable!(),
    }
}

/// Writes a UTF-16 code unit to the raw buffer.
///
/// # Safety
///
/// The caller must ensure:
/// * `emitter` points to a valid `YamlEmitterT`
/// * There is enough space in the raw buffer for 2 bytes
///
/// # Returns
///
/// `OK` if successful, `FAIL` if buffer overflow
#[inline]
unsafe fn write_utf16_unit(
    emitter: *mut YamlEmitterT,
    unit: u16,
    low: isize,
    high: isize,
) -> Success {
    if (*emitter).raw_buffer.last.offset(2) > (*emitter).raw_buffer.end
    {
        return yaml_emitter_set_writer_error(
            emitter,
            BUFFER_OVERFLOW.as_ptr() as *const libc::c_char,
        );
    }

    *(*emitter).raw_buffer.last.offset(high) = (unit >> 8) as u8;
    *(*emitter).raw_buffer.last.offset(low) = (unit & 0xFF) as u8;
    (*emitter).raw_buffer.last = (*emitter).raw_buffer.last.offset(2);
    OK
}

/// Flushes the emitter's buffer to the output stream.
///
/// Handles UTF-8 and UTF-16 encodings with proper validation and conversion.
///
/// # Safety
///
/// The caller must ensure:
/// * `emitter` points to a valid `YamlEmitterT`
/// * The write handler and its data are valid
///
/// # Returns
///
/// `OK` if successful, `FAIL` on write error or invalid UTF-8
pub unsafe fn yaml_emitter_flush(
    emitter: *mut YamlEmitterT,
) -> Success {
    debug_assert!(!emitter.is_null());
    debug_assert!(((*emitter).write_handler).is_some());
    debug_assert!((*emitter).encoding != YamlAnyEncoding);

    // Update buffer pointers
    *addr_of_mut!((*emitter).buffer.last) = (*emitter).buffer.pointer;
    *addr_of_mut!((*emitter).buffer.pointer) = (*emitter).buffer.start;

    // Check if buffer is empty
    if (*emitter).buffer.start == (*emitter).buffer.last {
        return OK;
    }

    // Handle UTF-8 encoding
    if (*emitter).encoding == YamlUtf8Encoding {
        let write_result = (*emitter)
            .write_handler
            .expect("non-null function pointer")(
            (*emitter).write_handler_data,
            (*emitter).buffer.start,
            (*emitter)
                .buffer
                .last
                .c_offset_from((*emitter).buffer.start)
                as size_t,
        );

        if write_result != 0 {
            *addr_of_mut!((*emitter).buffer.last) =
                (*emitter).buffer.start;
            *addr_of_mut!((*emitter).buffer.pointer) =
                (*emitter).buffer.start;
            return OK;
        } else {
            return yaml_emitter_set_writer_error(
                emitter,
                WRITE_ERROR.as_ptr().cast::<libc::c_char>(),
            );
        }
    }

    // Handle UTF-16 encoding
    let (low, high) = if (*emitter).encoding == YamlUtf16leEncoding {
        (0, 1)
    } else {
        (1, 0)
    };

    while (*emitter).buffer.pointer != (*emitter).buffer.last {
        let remaining = (*emitter)
            .buffer
            .last
            .offset_from((*emitter).buffer.pointer);
        let seq_len = match check_utf8_sequence(
            (*emitter).buffer.pointer,
            remaining,
        ) {
            Some(len) => len,
            None => {
                return yaml_emitter_set_writer_error(
                    emitter,
                    INVALID_UTF8.as_ptr().cast::<libc::c_char>(),
                )
            }
        };

        let code_point =
            utf8_to_codepoint((*emitter).buffer.pointer, seq_len);
        (*emitter).buffer.pointer =
            (*emitter).buffer.pointer.add(seq_len);

        if code_point > MAX_UNICODE {
            return yaml_emitter_set_writer_error(
                emitter,
                INVALID_UTF8.as_ptr().cast::<libc::c_char>(),
            );
        }

        if code_point < 0x10000 {
            if let Ok(code_point_u16) = u16::try_from(code_point) {
                if write_utf16_unit(emitter, code_point_u16, low, high)
                    == FAIL
                {
                    return FAIL;
                }
            } else {
                return yaml_emitter_set_writer_error(
                    emitter,
                    INVALID_UTF8.as_ptr().cast::<libc::c_char>(),
                );
            }
        } else {
            // Write surrogate pair
            let code_point = code_point - 0x10000;
            let high_surrogate = 0xD800 | ((code_point >> 10) & 0x3FF);
            let low_surrogate = 0xDC00 | (code_point & 0x3FF);

            // Safely convert high_surrogate to u16
            if let Ok(high_u16) = u16::try_from(high_surrogate) {
                if write_utf16_unit(emitter, high_u16, low, high)
                    == FAIL
                {
                    return FAIL;
                }
            } else {
                return yaml_emitter_set_writer_error(
                    emitter,
                    INVALID_UTF8.as_ptr().cast::<libc::c_char>(),
                );
            }

            // Safely convert low_surrogate to u16
            if let Ok(low_u16) = u16::try_from(low_surrogate) {
                if write_utf16_unit(emitter, low_u16, low, high) == FAIL
                {
                    return FAIL;
                }
            } else {
                return yaml_emitter_set_writer_error(
                    emitter,
                    INVALID_UTF8.as_ptr().cast::<libc::c_char>(),
                );
            }
        }
    }

    // Write accumulated UTF-16 data
    let offset = (*emitter)
        .raw_buffer
        .last
        .c_offset_from((*emitter).raw_buffer.start);

    let write_result = offset.try_into().map_or_else(|_| {
            panic!("Offset calculation resulted in a negative value, which is invalid for size_t");
        }, |offset_unsigned| (*emitter).write_handler.expect("non-null function pointer")(
                (*emitter).write_handler_data,
                (*emitter).raw_buffer.start,
                offset_unsigned,
            ));

    if write_result != 0 {
        // Reset all buffer pointers
        *addr_of_mut!((*emitter).buffer.last) = (*emitter).buffer.start;
        *addr_of_mut!((*emitter).buffer.pointer) =
            (*emitter).buffer.start;
        *addr_of_mut!((*emitter).raw_buffer.last) =
            (*emitter).raw_buffer.start;
        *addr_of_mut!((*emitter).raw_buffer.pointer) =
            (*emitter).raw_buffer.start;
        OK
    } else {
        yaml_emitter_set_writer_error(
            emitter,
            WRITE_ERROR.as_ptr().cast::<libc::c_char>(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::YamlEncodingT::YamlUtf16beEncoding;
    use alloc::boxed::Box;
    use alloc::vec;
    use alloc::vec::Vec;
    use core::mem::MaybeUninit;

    #[test]
    fn utf8_to_codepoint_test() {
        let data = [(b"\x41", 0x41)];

        for (input, expected) in data.iter() {
            let ptr = input.as_ptr();
            let len = input.len();
            let code_point = unsafe { utf8_to_codepoint(ptr, len) };
            assert_eq!(code_point, *expected);
        }
    }

    #[test]
    fn test_check_utf8_sequence() {
        unsafe {
            // Test valid sequences
            let ascii = b"A";
            assert_eq!(check_utf8_sequence(ascii.as_ptr(), 1), Some(1));

            let two_byte = "é".as_bytes();
            assert_eq!(
                check_utf8_sequence(two_byte.as_ptr(), 2),
                Some(2)
            );

            let three_byte = "€".as_bytes();
            assert_eq!(
                check_utf8_sequence(three_byte.as_ptr(), 3),
                Some(3)
            );

            let four_byte = "🦀".as_bytes();
            assert_eq!(
                check_utf8_sequence(four_byte.as_ptr(), 4),
                Some(4)
            );

            // Test invalid sequences
            let overlong = &[0xC0, 0x80]; // Overlong encoding of ASCII NUL
            assert_eq!(check_utf8_sequence(overlong.as_ptr(), 2), None);

            let surrogate = &[0xED, 0xA0, 0x80]; // UTF-16 surrogate
            assert_eq!(
                check_utf8_sequence(surrogate.as_ptr(), 3),
                None
            );

            let too_high = &[0xF5, 0x80, 0x80, 0x80]; // Beyond valid Unicode range
            assert_eq!(check_utf8_sequence(too_high.as_ptr(), 4), None);
        }
    }

    #[test]
    fn test_write_utf16_unit() {
        unsafe {
            // Set up a minimal emitter with a buffer
            let mut emitter = MaybeUninit::uninit();
            let mut raw_buffer = [0u8; 16];

            let emitter_ptr: *mut YamlEmitterT = emitter.as_mut_ptr();
            (*emitter_ptr).raw_buffer.start = raw_buffer.as_mut_ptr();
            (*emitter_ptr).raw_buffer.end =
                raw_buffer.as_mut_ptr().add(raw_buffer.len());
            (*emitter_ptr).raw_buffer.last = raw_buffer.as_mut_ptr();

            // Test little-endian write
            assert_eq!(write_utf16_unit(emitter_ptr, 0x1234, 0, 1), OK);
            assert_eq!(raw_buffer[0], 0x34); // low byte
            assert_eq!(raw_buffer[1], 0x12); // high byte

            // Test big-endian write
            (*emitter_ptr).raw_buffer.last =
                raw_buffer.as_mut_ptr().add(2);
            assert_eq!(write_utf16_unit(emitter_ptr, 0x5678, 1, 0), OK);
            assert_eq!(raw_buffer[2], 0x56); // high byte
            assert_eq!(raw_buffer[3], 0x78); // low byte

            // Test buffer overflow
            (*emitter_ptr).raw_buffer.end =
                (*emitter_ptr).raw_buffer.last;
            assert_eq!(
                write_utf16_unit(emitter_ptr, 0x0000, 0, 1),
                FAIL
            );
            assert_eq!((*emitter_ptr).error, YamlWriterError);
        }
    }

    #[test]
    fn test_yaml_emitter_set_writer_error() {
        unsafe {
            let mut emitter = MaybeUninit::uninit();
            let emitter_ptr = emitter.as_mut_ptr();

            let error_message = b"test error\0";
            assert_eq!(
                yaml_emitter_set_writer_error(
                    emitter_ptr,
                    error_message.as_ptr() as *const libc::c_char
                ),
                FAIL
            );
            assert_eq!((*emitter_ptr).error, YamlWriterError);
            assert_eq!(
                (*emitter_ptr).problem,
                error_message.as_ptr() as *const libc::c_char
            );
        }
    }

    #[test]
    fn test_utf8_to_codepoint_extended() {
        // Create test cases using Vec<u8> to avoid array size mismatches
        let test_cases = vec![
            // ASCII
            (vec![0x41], 1, 0x41), // 'A'
            // 2-byte UTF-8
            (vec![0xC2, 0xA9], 2, 0xA9), // '©'
            // 3-byte UTF-8
            (vec![0xE2, 0x82, 0xAC], 3, 0x20AC), // '€'
            // 4-byte UTF-8
            (vec![0xF0, 0x90, 0x90, 0xB7], 4, 0x10437), // '𐐷'
        ];

        for (input, len, expected) in test_cases {
            unsafe {
                let code_point = utf8_to_codepoint(input.as_ptr(), len);
                assert_eq!(code_point, expected);
            }
        }
    }

    #[test]
    fn check_utf8_sequence_valid_ascii() {
        // "A" = 0x41
        let input = b"A";
        let ptr = input.as_ptr();
        let remaining = input.len() as isize;

        unsafe {
            let seq_len = check_utf8_sequence(ptr, remaining);
            assert_eq!(
                seq_len,
                Some(1),
                "Expected 1-byte sequence for ASCII"
            );
        }
    }

    #[test]
    fn check_utf8_sequence_valid_2bytes() {
        // "©" (U+00A9) in UTF-8 is 0xC2 0xA9
        let input = &[0xC2, 0xA9];
        let ptr = input.as_ptr();
        let remaining = input.len() as isize;

        unsafe {
            let seq_len = check_utf8_sequence(ptr, remaining);
            assert_eq!(seq_len, Some(2), "Expected 2-byte sequence");
        }
    }

    #[test]
    fn check_utf8_sequence_invalid_overlong() {
        // 0xC0 0x80 is an overlong encoding for NULL (U+0000)
        let input = &[0xC0, 0x80];
        let ptr = input.as_ptr();
        let remaining = input.len() as isize;

        unsafe {
            let seq_len = check_utf8_sequence(ptr, remaining);
            assert_eq!(
                seq_len, None,
                "Overlong sequence should be invalid"
            );
        }
    }

    #[test]
    fn write_utf16_unit_ok() {
        // Create a mock emitter with a small, but sufficient raw buffer.
        let mut emitter = YamlEmitterT::default(); // or custom init in your real code
        let raw_storage = &mut [0u8; 4]; // Enough space for 2 bytes

        unsafe {
            // Use a single pointer and avoid multiple `as_mut_ptr()` calls
            let ptr = raw_storage.as_mut_ptr();

            // Initialize the emitter's raw buffer fields using the same pointer
            emitter.raw_buffer.start = ptr;
            emitter.raw_buffer.end = ptr.add(raw_storage.len());
            emitter.raw_buffer.pointer = ptr;
            emitter.raw_buffer.last = ptr;

            // Write 0x1234 in little-endian
            let res = write_utf16_unit(&mut emitter, 0x1234, 0, 1);
            assert_eq!(res, OK, "Expected successful write");

            // Verify contents: little-endian => [0x34, 0x12]
            assert_eq!(raw_storage[0], 0x34, "Low byte mismatch");
            assert_eq!(raw_storage[1], 0x12, "High byte mismatch");
        }
    }

    #[test]
    fn yaml_emitter_set_writer_error_test() {
        let mut emitter = YamlEmitterT::default(); // or custom init
        let msg = b"Custom error\0";

        unsafe {
            let res = yaml_emitter_set_writer_error(
                &mut emitter,
                msg.as_ptr() as *const libc::c_char,
            );
            assert_eq!(res, FAIL, "Expected FAIL on writer error");
            assert_eq!(
                emitter.error, YamlWriterError,
                "Emitter error not set"
            );
            assert_eq!(
                emitter.problem,
                msg.as_ptr() as *const libc::c_char,
                "Problem message not set"
            );
        }
    }

    #[test]
    fn check_utf8_sequence_valid_4bytes() {
        // "🦀" (U+1F980) in UTF-8 is 0xF0 0x9F 0xA6 0x80
        let input = &[0xF0, 0x9F, 0xA6, 0x80];
        let ptr = input.as_ptr();
        let remaining = input.len() as isize;

        unsafe {
            let seq_len = check_utf8_sequence(ptr, remaining);
            assert_eq!(seq_len, Some(4), "Expected 4-byte sequence");
        }
    }

    #[test]
    fn write_utf16_unit_surrogate_pair() {
        // Test writing a surrogate pair for code points > U+10000
        let mut emitter = YamlEmitterT::default();
        let raw_storage = &mut [0u8; 8]; // Enough space for 4 bytes

        unsafe {
            let ptr = raw_storage.as_mut_ptr();
            emitter.raw_buffer.start = ptr;
            emitter.raw_buffer.end = ptr.add(raw_storage.len());
            emitter.raw_buffer.pointer = ptr;
            emitter.raw_buffer.last = ptr;

            // U+1F980 (🦀) requires a surrogate pair
            let code_point = 0x1F980;
            let high_surrogate =
                0xD800 | ((code_point - 0x10000) >> 10) as u16;
            let low_surrogate =
                0xDC00 | ((code_point - 0x10000) & 0x3FF) as u16;

            // Write high surrogate
            let res_high =
                write_utf16_unit(&mut emitter, high_surrogate, 0, 1);
            assert_eq!(
                res_high, OK,
                "Expected successful write for high surrogate"
            );

            // Write low surrogate
            let res_low =
                write_utf16_unit(&mut emitter, low_surrogate, 0, 1);
            assert_eq!(
                res_low, OK,
                "Expected successful write for low surrogate"
            );

            // Verify buffer contents
            assert_eq!(raw_storage[0], (high_surrogate & 0xFF) as u8); // Low byte of high surrogate
            assert_eq!(raw_storage[1], (high_surrogate >> 8) as u8); // High byte of high surrogate
            assert_eq!(raw_storage[2], (low_surrogate & 0xFF) as u8); // Low byte of low surrogate
            assert_eq!(raw_storage[3], (low_surrogate >> 8) as u8); // High byte of low surrogate
        }
    }

    #[test]
    fn utf8_to_codepoint_edge_cases() {
        unsafe {
            // Edge case: smallest valid 4-byte UTF-8 (U+010000)
            let input = &[0xF0, 0x90, 0x80, 0x80]; // U+010000
            let code_point =
                utf8_to_codepoint(input.as_ptr(), input.len());
            assert_eq!(code_point, 0x10000, "Expected U+010000");

            // Edge case: largest valid 4-byte UTF-8 (U+10FFFF)
            let input = &[0xF4, 0x8F, 0xBF, 0xBF]; // U+10FFFF
            let code_point =
                utf8_to_codepoint(input.as_ptr(), input.len());
            assert_eq!(code_point, 0x10FFFF, "Expected U+10FFFF");
        }
    }

    struct TestData {
        output: Vec<u8>,
        buffer: Box<[u8; 1024]>,
    }

    unsafe fn mock_write_handler(
        data: *mut libc::c_void,
        buffer: *mut u8,
        size: size_t,
    ) -> i32 {
        let test_data = &mut *(data as *mut TestData);
        test_data.output.clear();
        test_data.output.extend_from_slice(
            core::slice::from_raw_parts(buffer, size as usize),
        );
        1
    }

    #[test]
    fn test_yaml_emitter_flush_empty_buffer() {
        let mut test_data = TestData {
            output: Vec::new(),
            buffer: Box::new([0u8; 1024]),
        };

        let mut emitter = YamlEmitterT::default();

        unsafe {
            let write_handler: unsafe fn(
                *mut libc::c_void,
                *mut u8,
                size_t,
            ) -> i32 = mock_write_handler;
            emitter.write_handler = Some(write_handler);

            // Use proper coercion instead of casting
            let data_ptr: *mut TestData = &mut test_data;
            emitter.write_handler_data = data_ptr as *mut libc::c_void;

            emitter.encoding = YamlUtf8Encoding;

            let buffer_ptr = test_data.buffer.as_mut_ptr();
            emitter.buffer.start = buffer_ptr;
            emitter.buffer.end = buffer_ptr.add(test_data.buffer.len());
            emitter.buffer.pointer = buffer_ptr;
            emitter.buffer.last = buffer_ptr;

            let result = yaml_emitter_flush(&mut emitter);
            assert_eq!(result, OK, "Empty buffer flush should succeed");
            assert!(
                test_data.output.is_empty(),
                "Output should be empty"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_flush_utf16le_surrogate_pairs() {
        let mut test_data = TestData {
            output: Vec::new(),
            buffer: Box::new([0u8; 1024]),
        };

        let mut emitter = YamlEmitterT::default();

        unsafe {
            let write_handler: unsafe fn(
                *mut libc::c_void,
                *mut u8,
                size_t,
            ) -> i32 = mock_write_handler;
            emitter.write_handler = Some(write_handler);

            // Use proper coercion
            let data_ptr: *mut TestData = &mut test_data;
            emitter.write_handler_data = data_ptr as *mut libc::c_void;

            emitter.encoding = YamlUtf16leEncoding;

            let buffer_ptr = test_data.buffer.as_mut_ptr();
            emitter.buffer.start = buffer_ptr;
            emitter.buffer.end = buffer_ptr.add(test_data.buffer.len());
            emitter.buffer.pointer = buffer_ptr;
            emitter.buffer.last = buffer_ptr;

            let mut raw_buffer = Box::new([0u8; 2048]);
            let raw_ptr = raw_buffer.as_mut_ptr();
            emitter.raw_buffer.start = raw_ptr;
            emitter.raw_buffer.end = raw_ptr.add(raw_buffer.len());
            emitter.raw_buffer.pointer = raw_ptr;
            emitter.raw_buffer.last = raw_ptr;

            let content = "🦀".as_bytes();
            core::ptr::copy(
                content.as_ptr(),
                buffer_ptr,
                content.len(),
            );
            emitter.buffer.pointer = buffer_ptr.add(content.len());

            let result = yaml_emitter_flush(&mut emitter);
            assert_eq!(
                result, OK,
                "UTF-16 surrogate pair flush should succeed"
            );

            assert_eq!(
                test_data.output.len(),
                4,
                "Expected 4 bytes for surrogate pair"
            );
            let high_surrogate = u16::from_le_bytes([
                test_data.output[0],
                test_data.output[1],
            ]);
            let low_surrogate = u16::from_le_bytes([
                test_data.output[2],
                test_data.output[3],
            ]);

            assert!(
                (0xD800..=0xDBFF).contains(&high_surrogate),
                "Invalid high surrogate"
            );
            assert!(
                (0xDC00..=0xDFFF).contains(&low_surrogate),
                "Invalid low surrogate"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_flush_buffer_overflow() {
        let mut test_data = TestData {
            output: Vec::new(),
            buffer: Box::new([0u8; 1024]),
        };

        let mut emitter = YamlEmitterT::default();

        unsafe {
            let write_handler: unsafe fn(
                *mut libc::c_void,
                *mut u8,
                size_t,
            ) -> i32 = mock_write_handler;
            emitter.write_handler = Some(write_handler);

            let data_ptr: *mut TestData = &mut test_data;
            emitter.write_handler_data = data_ptr as *mut libc::c_void;

            emitter.encoding = YamlUtf16leEncoding;

            // Set up input buffer with a surrogate pair that requires 4 bytes in UTF-16
            let buffer_ptr = test_data.buffer.as_mut_ptr();
            emitter.buffer.start = buffer_ptr;
            emitter.buffer.end = buffer_ptr.add(test_data.buffer.len());
            emitter.buffer.pointer = buffer_ptr;

            // Create a raw buffer that's just big enough for one UTF-16 unit (2 bytes)
            // but we'll need 4 bytes for the surrogate pair
            let mut raw_buffer = Box::new([0u8; 8]); // Actual buffer size bigger to avoid UB
            let raw_ptr = raw_buffer.as_mut_ptr();
            emitter.raw_buffer.start = raw_ptr;
            emitter.raw_buffer.end = raw_ptr.add(2); // Only allow 2 bytes
            emitter.raw_buffer.pointer = raw_ptr;
            emitter.raw_buffer.last = raw_ptr;

            // Write a character that requires surrogate pair encoding (4 bytes in UTF-16)
            let content = "🦀".as_bytes(); // Crab emoji - requires surrogate pair
            core::ptr::copy(
                content.as_ptr(),
                buffer_ptr,
                content.len(),
            );
            emitter.buffer.pointer = buffer_ptr.add(content.len());
            emitter.buffer.last = emitter.buffer.pointer;

            let result = yaml_emitter_flush(&mut emitter);
            assert_eq!(
                result, FAIL,
                "Expected failure due to buffer overflow"
            );
            assert_eq!(emitter.error, YamlWriterError);
        }
    }

    #[test]
    fn test_yaml_emitter_flush_write_handler_failure() {
        unsafe fn mock_fail_handler(
            _: *mut libc::c_void,
            _: *mut u8,
            _: size_t,
        ) -> i32 {
            0 // Simulate write failure
        }

        let mut test_data = TestData {
            output: Vec::new(),
            buffer: Box::new([0u8; 1024]),
        };

        let mut emitter = YamlEmitterT::default();

        unsafe {
            let write_handler: unsafe fn(
                *mut libc::c_void,
                *mut u8,
                size_t,
            ) -> i32 = mock_fail_handler;
            emitter.write_handler = Some(write_handler);

            // Use proper coercion
            let data_ptr: *mut TestData = &mut test_data;
            emitter.write_handler_data = data_ptr as *mut libc::c_void;

            emitter.encoding = YamlUtf8Encoding;

            let buffer_ptr = test_data.buffer.as_mut_ptr();
            emitter.buffer.start = buffer_ptr;
            emitter.buffer.end = buffer_ptr.add(test_data.buffer.len());
            emitter.buffer.pointer = buffer_ptr;
            emitter.buffer.last = buffer_ptr;

            let content = b"Test content";
            core::ptr::copy(
                content.as_ptr(),
                buffer_ptr,
                content.len(),
            );
            emitter.buffer.pointer = buffer_ptr.add(content.len());

            let result = yaml_emitter_flush(&mut emitter);
            assert_eq!(
                result, FAIL,
                "Expected failure from write handler"
            );
            assert_eq!(emitter.error, YamlWriterError);
        }
    }

    #[test]
    fn test_yaml_emitter_flush_invalid_utf8() {
        let mut test_data = TestData {
            output: Vec::new(),
            buffer: Box::new([0u8; 1024]),
        };

        let mut emitter = YamlEmitterT::default();

        unsafe {
            let write_handler: unsafe fn(
                *mut libc::c_void,
                *mut u8,
                size_t,
            ) -> i32 = mock_write_handler;
            emitter.write_handler = Some(write_handler);

            // Use proper coercion
            let data_ptr: *mut TestData = &mut test_data;
            emitter.write_handler_data = data_ptr as *mut libc::c_void;

            emitter.encoding = YamlUtf16leEncoding;

            let buffer_ptr = test_data.buffer.as_mut_ptr();
            emitter.buffer.start = buffer_ptr;
            emitter.buffer.end = buffer_ptr.add(test_data.buffer.len());
            emitter.buffer.pointer = buffer_ptr;
            emitter.buffer.last = buffer_ptr;

            let mut raw_buffer = Box::new([0u8; 1024]);
            let raw_ptr = raw_buffer.as_mut_ptr();
            emitter.raw_buffer.start = raw_ptr;
            emitter.raw_buffer.end = raw_ptr.add(raw_buffer.len());
            emitter.raw_buffer.pointer = raw_ptr;
            emitter.raw_buffer.last = raw_ptr;

            // Write invalid UTF-8 sequence
            let invalid_utf8 = &[0xFF, 0xFF];
            core::ptr::copy(
                invalid_utf8.as_ptr(),
                buffer_ptr,
                invalid_utf8.len(),
            );
            emitter.buffer.pointer = buffer_ptr.add(invalid_utf8.len());

            let result = yaml_emitter_flush(&mut emitter);
            assert_eq!(
                result, FAIL,
                "Expected failure for invalid UTF-8"
            );
            assert_eq!(emitter.error, YamlWriterError);
        }
    }

    #[test]
    fn test_yaml_emitter_flush_utf16be() {
        let mut test_data = TestData {
            output: Vec::new(),
            buffer: Box::new([0u8; 1024]),
        };

        let mut emitter = YamlEmitterT::default();

        unsafe {
            let write_handler: unsafe fn(
                *mut libc::c_void,
                *mut u8,
                size_t,
            ) -> i32 = mock_write_handler;
            emitter.write_handler = Some(write_handler);

            // Use proper coercion
            let data_ptr: *mut TestData = &mut test_data;
            emitter.write_handler_data = data_ptr as *mut libc::c_void;

            // Set UTF-16BE encoding
            emitter.encoding = YamlUtf16beEncoding;

            let buffer_ptr = test_data.buffer.as_mut_ptr();
            emitter.buffer.start = buffer_ptr;
            emitter.buffer.end = buffer_ptr.add(test_data.buffer.len());
            emitter.buffer.pointer = buffer_ptr;
            emitter.buffer.last = buffer_ptr;

            let mut raw_buffer = Box::new([0u8; 1024]);
            let raw_ptr = raw_buffer.as_mut_ptr();
            emitter.raw_buffer.start = raw_ptr;
            emitter.raw_buffer.end = raw_ptr.add(raw_buffer.len());
            emitter.raw_buffer.pointer = raw_ptr;
            emitter.raw_buffer.last = raw_ptr;

            // Write simple ASCII content
            let content = b"A";
            core::ptr::copy(
                content.as_ptr(),
                buffer_ptr,
                content.len(),
            );
            emitter.buffer.pointer = buffer_ptr.add(content.len());

            let result = yaml_emitter_flush(&mut emitter);
            assert_eq!(
                result, OK,
                "UTF-16BE conversion should succeed"
            );
            assert_eq!(
                test_data.output.len(),
                2,
                "Expected 2 bytes for UTF-16BE"
            );
            assert_eq!(
                test_data.output[0], 0x00,
                "Expected high byte first in BE"
            );
            assert_eq!(
                test_data.output[1], 0x41,
                "Expected 'A' as low byte in BE"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_flush_incomplete_utf8() {
        let mut test_data = TestData {
            output: Vec::new(),
            buffer: Box::new([0u8; 1024]),
        };

        let mut emitter = YamlEmitterT::default();

        unsafe {
            let write_handler: unsafe fn(
                *mut libc::c_void,
                *mut u8,
                size_t,
            ) -> i32 = mock_write_handler;
            emitter.write_handler = Some(write_handler);

            let data_ptr: *mut TestData = &mut test_data;
            emitter.write_handler_data = data_ptr as *mut libc::c_void;

            emitter.encoding = YamlUtf16leEncoding;

            let buffer_ptr = test_data.buffer.as_mut_ptr();
            emitter.buffer.start = buffer_ptr;
            emitter.buffer.end = buffer_ptr.add(test_data.buffer.len());
            emitter.buffer.pointer = buffer_ptr;

            let mut raw_buffer = Box::new([0u8; 1024]);
            let raw_ptr = raw_buffer.as_mut_ptr();
            emitter.raw_buffer.start = raw_ptr;
            emitter.raw_buffer.end = raw_ptr.add(raw_buffer.len());
            emitter.raw_buffer.pointer = raw_ptr;
            emitter.raw_buffer.last = raw_ptr;

            // Write incomplete UTF-8 sequence (only first byte of a 2-byte sequence)
            let incomplete_utf8 = &[0xC2];
            core::ptr::copy(
                incomplete_utf8.as_ptr(),
                buffer_ptr,
                incomplete_utf8.len(),
            );
            emitter.buffer.pointer =
                buffer_ptr.add(incomplete_utf8.len());
            emitter.buffer.last = emitter.buffer.pointer;

            let result = yaml_emitter_flush(&mut emitter);
            assert_eq!(
                result, FAIL,
                "Expected failure for incomplete UTF-8"
            );
            assert_eq!(emitter.error, YamlWriterError);
        }
    }

    #[test]
    fn test_yaml_emitter_flush_boundary_unicode() {
        let mut test_data = TestData {
            output: Vec::new(),
            buffer: Box::new([0u8; 1024]),
        };

        let mut emitter = YamlEmitterT::default();

        unsafe {
            let write_handler: unsafe fn(
                *mut libc::c_void,
                *mut u8,
                size_t,
            ) -> i32 = mock_write_handler;
            emitter.write_handler = Some(write_handler);

            let data_ptr: *mut TestData = &mut test_data;
            emitter.write_handler_data = data_ptr as *mut libc::c_void;

            emitter.encoding = YamlUtf16leEncoding;

            let buffer_ptr = test_data.buffer.as_mut_ptr();
            emitter.buffer.start = buffer_ptr;
            emitter.buffer.end = buffer_ptr.add(test_data.buffer.len());
            emitter.buffer.pointer = buffer_ptr;

            let mut raw_buffer = Box::new([0u8; 1024]);
            let raw_ptr = raw_buffer.as_mut_ptr();
            emitter.raw_buffer.start = raw_ptr;
            emitter.raw_buffer.end = raw_ptr.add(raw_buffer.len());
            emitter.raw_buffer.pointer = raw_ptr;
            emitter.raw_buffer.last = raw_ptr;

            // U+10FFFF - Maximum valid Unicode code point
            let max_unicode = &[0xF4, 0x8F, 0xBF, 0xBF];
            core::ptr::copy(
                max_unicode.as_ptr(),
                buffer_ptr,
                max_unicode.len(),
            );
            emitter.buffer.pointer = buffer_ptr.add(max_unicode.len());
            emitter.buffer.last = emitter.buffer.pointer;

            let result = yaml_emitter_flush(&mut emitter);
            assert_eq!(
                result, OK,
                "Expected success for maximum valid Unicode"
            );
            assert_eq!(
                test_data.output.len(),
                4,
                "Expected 2 UTF-16 units (4 bytes)"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_flush_utf16_mixed_content() {
        let mut test_data = TestData {
            output: Vec::new(),
            buffer: Box::new([0u8; 1024]),
        };

        let mut emitter = YamlEmitterT::default();

        unsafe {
            let write_handler: unsafe fn(
                *mut libc::c_void,
                *mut u8,
                size_t,
            ) -> i32 = mock_write_handler;
            emitter.write_handler = Some(write_handler);

            let data_ptr: *mut TestData = &mut test_data;
            emitter.write_handler_data = data_ptr as *mut libc::c_void;

            emitter.encoding = YamlUtf16leEncoding;

            let buffer_ptr = test_data.buffer.as_mut_ptr();
            emitter.buffer.start = buffer_ptr;
            emitter.buffer.end = buffer_ptr.add(test_data.buffer.len());
            emitter.buffer.pointer = buffer_ptr;

            let mut raw_buffer = Box::new([0u8; 1024]);
            let raw_ptr = raw_buffer.as_mut_ptr();
            emitter.raw_buffer.start = raw_ptr;
            emitter.raw_buffer.end = raw_ptr.add(raw_buffer.len());
            emitter.raw_buffer.pointer = raw_ptr;
            emitter.raw_buffer.last = raw_ptr;

            // Mix of ASCII, 2-byte UTF-8, and surrogate pairs
            let content = "A€🦀";
            let content_bytes = content.as_bytes();
            core::ptr::copy(
                content_bytes.as_ptr(),
                buffer_ptr,
                content_bytes.len(),
            );
            emitter.buffer.pointer =
                buffer_ptr.add(content_bytes.len());
            emitter.buffer.last = emitter.buffer.pointer;

            let result = yaml_emitter_flush(&mut emitter);
            assert_eq!(
                result, OK,
                "Expected success for mixed content"
            );
            assert_eq!(
                test_data.output.len(),
                8,
                "Expected 4 UTF-16 units (8 bytes total)"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_flush_consecutive_surrogates() {
        let mut test_data = TestData {
            output: Vec::new(),
            buffer: Box::new([0u8; 1024]),
        };

        let mut emitter = YamlEmitterT::default();

        unsafe {
            let write_handler: unsafe fn(
                *mut libc::c_void,
                *mut u8,
                size_t,
            ) -> i32 = mock_write_handler;
            emitter.write_handler = Some(write_handler);

            let data_ptr: *mut TestData = &mut test_data;
            emitter.write_handler_data = data_ptr as *mut libc::c_void;

            emitter.encoding = YamlUtf16leEncoding;

            let buffer_ptr = test_data.buffer.as_mut_ptr();
            emitter.buffer.start = buffer_ptr;
            emitter.buffer.end = buffer_ptr.add(test_data.buffer.len());
            emitter.buffer.pointer = buffer_ptr;

            let mut raw_buffer = Box::new([0u8; 1024]);
            let raw_ptr = raw_buffer.as_mut_ptr();
            emitter.raw_buffer.start = raw_ptr;
            emitter.raw_buffer.end = raw_ptr.add(raw_buffer.len());
            emitter.raw_buffer.pointer = raw_ptr;
            emitter.raw_buffer.last = raw_ptr;

            // Two emojis that require surrogate pairs
            let content = "🦀🌟";
            let content_bytes = content.as_bytes();
            core::ptr::copy(
                content_bytes.as_ptr(),
                buffer_ptr,
                content_bytes.len(),
            );
            emitter.buffer.pointer =
                buffer_ptr.add(content_bytes.len());
            emitter.buffer.last = emitter.buffer.pointer;

            let result = yaml_emitter_flush(&mut emitter);
            assert_eq!(
                result, OK,
                "Expected success for consecutive surrogate pairs"
            );
            assert_eq!(
                test_data.output.len(),
                8,
                "Expected 4 UTF-16 units (8 bytes)"
            );
        }
    }

    #[test]
    fn test_yaml_emitter_flush_overlong_sequence() {
        let mut test_data = TestData {
            output: Vec::new(),
            buffer: Box::new([0u8; 1024]),
        };

        let mut emitter = YamlEmitterT::default();

        unsafe {
            let write_handler: unsafe fn(
                *mut libc::c_void,
                *mut u8,
                size_t,
            ) -> i32 = mock_write_handler;
            emitter.write_handler = Some(write_handler);

            let data_ptr: *mut TestData = &mut test_data;
            emitter.write_handler_data = data_ptr as *mut libc::c_void;

            emitter.encoding = YamlUtf16leEncoding;

            let buffer_ptr = test_data.buffer.as_mut_ptr();
            emitter.buffer.start = buffer_ptr;
            emitter.buffer.end = buffer_ptr.add(test_data.buffer.len());
            emitter.buffer.pointer = buffer_ptr;

            let mut raw_buffer = Box::new([0u8; 1024]);
            let raw_ptr = raw_buffer.as_mut_ptr();
            emitter.raw_buffer.start = raw_ptr;
            emitter.raw_buffer.end = raw_ptr.add(raw_buffer.len());
            emitter.raw_buffer.pointer = raw_ptr;
            emitter.raw_buffer.last = raw_ptr;

            // Overlong sequence - using 2 bytes to encode ASCII 'A'
            let overlong = &[0xC1, 0x81];
            core::ptr::copy(
                overlong.as_ptr(),
                buffer_ptr,
                overlong.len(),
            );
            emitter.buffer.pointer = buffer_ptr.add(overlong.len());
            emitter.buffer.last = emitter.buffer.pointer;

            let result = yaml_emitter_flush(&mut emitter);
            assert_eq!(
                result, FAIL,
                "Expected failure for overlong sequence"
            );
            assert_eq!(emitter.error, YamlWriterError);
        }
    }

    #[test]
    fn test_yaml_emitter_flush_partial_surrogate() {
        let mut test_data = TestData {
            output: Vec::new(),
            buffer: Box::new([0u8; 1024]),
        };

        let mut emitter = YamlEmitterT::default();

        unsafe {
            let write_handler: unsafe fn(
                *mut libc::c_void,
                *mut u8,
                size_t,
            ) -> i32 = mock_write_handler;
            emitter.write_handler = Some(write_handler);

            let data_ptr: *mut TestData = &mut test_data;
            emitter.write_handler_data = data_ptr as *mut libc::c_void;

            emitter.encoding = YamlUtf16leEncoding;

            let buffer_ptr = test_data.buffer.as_mut_ptr();
            emitter.buffer.start = buffer_ptr;
            emitter.buffer.end = buffer_ptr.add(test_data.buffer.len());
            emitter.buffer.pointer = buffer_ptr;

            let mut raw_buffer = Box::new([0u8; 1024]);
            let raw_ptr = raw_buffer.as_mut_ptr();
            emitter.raw_buffer.start = raw_ptr;
            emitter.raw_buffer.end = raw_ptr.add(2); // Only allow space for one UTF-16 unit
            emitter.raw_buffer.pointer = raw_ptr;
            emitter.raw_buffer.last = raw_ptr;

            // Character requiring surrogate pair
            let content = "🌟";
            let content_bytes = content.as_bytes();
            core::ptr::copy(
                content_bytes.as_ptr(),
                buffer_ptr,
                content_bytes.len(),
            );
            emitter.buffer.pointer =
                buffer_ptr.add(content_bytes.len());
            emitter.buffer.last = emitter.buffer.pointer;

            let result = yaml_emitter_flush(&mut emitter);
            assert_eq!(result, FAIL, "Expected failure when there's only space for half a surrogate pair");
            assert_eq!(emitter.error, YamlWriterError);
        }
    }

    #[test]
    fn test_yaml_emitter_flush_boundary_conditions() {
        let mut test_data = TestData {
            output: Vec::new(),
            buffer: Box::new([0u8; 1024]),
        };

        let mut emitter = YamlEmitterT::default();

        unsafe {
            let write_handler: unsafe fn(
                *mut libc::c_void,
                *mut u8,
                size_t,
            ) -> i32 = mock_write_handler;
            emitter.write_handler = Some(write_handler);

            let data_ptr: *mut TestData = &mut test_data;
            emitter.write_handler_data = data_ptr as *mut libc::c_void;

            emitter.encoding = YamlUtf16leEncoding;

            let buffer_ptr = test_data.buffer.as_mut_ptr();
            emitter.buffer.start = buffer_ptr;
            emitter.buffer.end = buffer_ptr.add(test_data.buffer.len());
            emitter.buffer.pointer = buffer_ptr;

            let mut raw_buffer = Box::new([0u8; 1024]);
            let raw_ptr = raw_buffer.as_mut_ptr();
            emitter.raw_buffer.start = raw_ptr;
            emitter.raw_buffer.end = raw_ptr.add(raw_buffer.len());
            emitter.raw_buffer.pointer = raw_ptr;
            emitter.raw_buffer.last = raw_ptr;

            // Test boundary conditions:
            // 1. Last valid two-byte sequence (0xDF, 0xBF)
            // 2. First valid three-byte sequence (0xE0, 0xA0, 0x80)
            let content = &[0xDF, 0xBF, 0xE0, 0xA0, 0x80];
            core::ptr::copy(
                content.as_ptr(),
                buffer_ptr,
                content.len(),
            );
            emitter.buffer.pointer = buffer_ptr.add(content.len());
            emitter.buffer.last = emitter.buffer.pointer;

            let result = yaml_emitter_flush(&mut emitter);
            assert_eq!(
                result, OK,
                "Expected success for boundary UTF-8 sequences"
            );
            assert!(
                !test_data.output.is_empty(),
                "Expected non-empty output"
            );
        }
    }
}
