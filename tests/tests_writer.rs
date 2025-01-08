#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    mod writer {
        use libyml::success::FAIL;
        use libyml::yaml::size_t;
        use libyml::yaml_emitter_delete;
        use libyml::yaml_emitter_initialize;
        use libyml::yaml_emitter_set_output;
        use libyml::{
            api::yaml_emitter_set_encoding,
            libc::{c_int, c_uchar, c_void},
            success,
            writer::yaml_emitter_flush,
            YamlEmitterT, YamlUtf16beEncoding, YamlUtf16leEncoding,
            YamlUtf8Encoding, YamlWriterError,
        };

        // Define the mock write handler with the correct signature
        unsafe fn mock_write_handler(
            _data: *mut c_void,
            _buffer: *mut c_uchar,
            _size: u64,
        ) -> c_int {
            1 // Simulate a successful write
        }

        /// Tests basic UTF-8 writing with no errors
        #[test]
        fn test_yaml_emitter_flush_utf8_no_error() {
            let mut emitter = YamlEmitterT::new();

            unsafe {
                yaml_emitter_set_encoding(
                    &mut emitter,
                    YamlUtf8Encoding,
                );
            }

            // Set up the buffer
            emitter.buffer.start =
                [b'a', b'b', b'c'].as_ptr() as *mut u8;
            emitter.buffer.pointer = emitter.buffer.start;
            emitter.buffer.last =
                unsafe { emitter.buffer.start.add(3) };

            // Set the write handler directly without casting
            emitter.write_handler = Some(mock_write_handler);

            let result = unsafe { yaml_emitter_flush(&mut emitter) };
            assert_eq!(result, success::OK);
            assert_eq!(emitter.buffer.pointer, emitter.buffer.start);
            assert_eq!(emitter.buffer.last, emitter.buffer.start);
        }

        /// Tests UTF-16LE write failure handling
        #[test]
        fn test_yaml_emitter_flush_utf16le_write_error() {
            unsafe fn fail_write_handler(
                _data: *mut c_void,
                _buffer: *mut u8,
                _size: size_t,
            ) -> c_int {
                0 // Return 0 to simulate failure
            }

            let mut emitter = YamlEmitterT::new();

            unsafe {
                // Initialize the emitter
                let _ = yaml_emitter_initialize(&mut emitter);

                // Set UTF-16LE encoding
                yaml_emitter_set_encoding(
                    &mut emitter,
                    YamlUtf16leEncoding,
                );

                // Set the write handler
                yaml_emitter_set_output(
                    &mut emitter,
                    fail_write_handler,
                    std::ptr::null_mut(),
                );

                // Put content in both main and raw buffers
                // Set last pointer after current pointer to ensure there's content to write
                emitter.buffer.pointer = emitter.buffer.start.add(1);
                emitter.buffer.last = emitter.buffer.pointer;

                // Make sure buffer has content different from start position
                *emitter.buffer.start = b'A';

                let result = yaml_emitter_flush(&mut emitter);

                assert_eq!(
                    result, FAIL,
                    "Expected flush to fail on write error"
                );
                assert_eq!(
                    emitter.error, YamlWriterError,
                    "Expected emitter.error to be YamlWriterError"
                );

                // Clean up
                yaml_emitter_delete(&mut emitter);
            }
        }

        /// Tests empty buffer handling in yaml_emitter_flush
        #[test]
        fn test_yaml_emitter_flush_empty_buffer() {
            unsafe fn mock_write_handler(
                _data: *mut c_void,
                _buffer: *mut c_uchar,
                _size: u64,
            ) -> c_int {
                1 // Would succeed, but hopefully we don't call it for empty buffer
            }

            let mut emitter = YamlEmitterT::new();

            unsafe {
                yaml_emitter_set_encoding(
                    &mut emitter,
                    YamlUtf8Encoding,
                );
            }

            // buffer.start == buffer.last => empty
            emitter.buffer.start = [0u8; 0].as_ptr() as *mut u8;
            emitter.buffer.pointer = emitter.buffer.start;
            emitter.buffer.last = emitter.buffer.start;

            emitter.write_handler = Some(mock_write_handler);

            let result = unsafe { yaml_emitter_flush(&mut emitter) };
            assert_eq!(result, success::OK);
            // No data => pointers remain
            assert_eq!(emitter.buffer.pointer, emitter.buffer.start);
            assert_eq!(emitter.buffer.last, emitter.buffer.start);
        }

        ///  Tests multibyte UTF-8 character handling in flush
        #[test]
        fn test_yaml_emitter_flush_multibyte_utf8() {
            use libyml::api::yaml_emitter_set_encoding;
            use libyml::libc::{c_int, c_uchar, c_void};
            use libyml::writer::yaml_emitter_flush;
            use libyml::{success, YamlEmitterT, YamlUtf8Encoding};

            let data = [0xC3u8, 0xA9]; // UTF-8 for "é"

            // Mock write handler
            unsafe fn mock_write_handler(
                data_ptr: *mut c_void,
                buffer: *mut c_uchar,
                size: u64,
            ) -> c_int {
                let expected_data = data_ptr as *const [u8; 2];
                let slice =
                    std::slice::from_raw_parts(buffer, size as usize);
                assert_eq!(slice, unsafe { &*expected_data });
                1 // success
            }

            let mut emitter = YamlEmitterT::new();
            unsafe {
                yaml_emitter_set_encoding(
                    &mut emitter,
                    YamlUtf8Encoding,
                );
            }

            // Set up the buffer
            emitter.buffer.start = data.as_ptr() as *mut u8;
            emitter.buffer.pointer = emitter.buffer.start;
            emitter.buffer.last =
                unsafe { emitter.buffer.start.add(data.len()) };

            // Set the write handler and its data
            emitter.write_handler = Some(mock_write_handler);
            emitter
                .set_write_handler_data(data.as_ptr() as *mut c_void);

            let result = unsafe { yaml_emitter_flush(&mut emitter) };
            assert_eq!(result, success::OK);
            assert_eq!(emitter.buffer.pointer, emitter.buffer.start);
            assert_eq!(emitter.buffer.last, emitter.buffer.start);
        }

        /// Tests UTF-16LE surrogate pair encoding and writing
        #[test]
        fn test_yaml_emitter_flush_utf16_surrogate_pair() {
            unsafe fn mock_write_handler(
                _data: *mut c_void,
                _buffer: *mut c_uchar,
                size: u64,
            ) -> c_int {
                // For a character requiring surrogate pairs (e.g. 🦀),
                // we expect 4 bytes in UTF-16
                assert_eq!(size, 4);
                1 // success
            }

            let mut emitter = YamlEmitterT::new();

            unsafe {
                let _ = yaml_emitter_initialize(&mut emitter);
                yaml_emitter_set_encoding(
                    &mut emitter,
                    YamlUtf16leEncoding,
                );
                yaml_emitter_set_output(
                    &mut emitter,
                    mock_write_handler,
                    std::ptr::null_mut(),
                );

                // 🦀 in UTF-8: F0 9F A6 80
                let crab = [0xF0, 0x9F, 0xA6, 0x80];

                // Set up the buffer with the UTF-8 bytes
                emitter.buffer.pointer = emitter.buffer.start;
                for &byte in &crab {
                    *emitter.buffer.pointer = byte;
                    emitter.buffer.pointer =
                        emitter.buffer.pointer.add(1);
                }
                emitter.buffer.last = emitter.buffer.pointer;
                emitter.buffer.pointer = emitter.buffer.start;

                let result = yaml_emitter_flush(&mut emitter);
                assert_eq!(result, success::OK);

                yaml_emitter_delete(&mut emitter);
            }
        }

        /// Tests UTF-16BE encoding and writing
        #[test]
        fn test_yaml_emitter_flush_utf16be() {
            unsafe fn mock_write_handler(
                _data: *mut c_void,
                buffer: *mut c_uchar,
                size: u64,
            ) -> c_int {
                // For 'A', we expect 2 bytes in UTF-16BE: 0x00 0x41
                assert_eq!(size, 2);
                let bytes =
                    std::slice::from_raw_parts(buffer, size as usize);
                assert_eq!(bytes[0], 0x00); // high byte first in BE
                assert_eq!(bytes[1], 0x41); // 'A'
                1 // success
            }

            let mut emitter = YamlEmitterT::new();

            unsafe {
                let _ = yaml_emitter_initialize(&mut emitter);
                yaml_emitter_set_encoding(
                    &mut emitter,
                    YamlUtf16beEncoding,
                );
                yaml_emitter_set_output(
                    &mut emitter,
                    mock_write_handler,
                    std::ptr::null_mut(),
                );

                // Set up buffer with 'A'
                *emitter.buffer.start = b'A';
                emitter.buffer.pointer = emitter.buffer.start.add(1);
                emitter.buffer.last = emitter.buffer.pointer;
                emitter.buffer.pointer = emitter.buffer.start;

                let result = yaml_emitter_flush(&mut emitter);
                assert_eq!(result, success::OK);

                yaml_emitter_delete(&mut emitter);
            }
        }

        /// Tests various UTF-8 sequence lengths (2, 3, and 4 bytes)
        #[test]
        fn test_yaml_emitter_flush_utf8_sequences() {
            unsafe fn mock_write_handler(
                _data: *mut c_void,
                buffer: *mut c_uchar,
                size: u64,
            ) -> c_int {
                let bytes =
                    std::slice::from_raw_parts(buffer, size as usize);
                // Verify we have all our test characters
                assert_eq!(bytes.len(), 7); // é (2 bytes) + ⭐ (3 bytes) + 🦀 (4 bytes)
                1 // success
            }

            let mut emitter = YamlEmitterT::new();

            unsafe {
                let _ = yaml_emitter_initialize(&mut emitter);
                yaml_emitter_set_encoding(
                    &mut emitter,
                    YamlUtf8Encoding,
                );
                yaml_emitter_set_output(
                    &mut emitter,
                    mock_write_handler,
                    std::ptr::null_mut(),
                );

                // Test data: é (2 bytes) + ⭐ (3 bytes) + 🦀 (4 bytes)
                let test_data = [
                    0xC3, 0xA9, // é
                    0xE2, 0xAD, 0x90, // ⭐
                    0xF0, 0x9F, 0xA6, 0x80, // 🦀
                ];

                // Set up buffer
                for &byte in &test_data {
                    *emitter.buffer.pointer = byte;
                    emitter.buffer.pointer =
                        emitter.buffer.pointer.add(1);
                }
                emitter.buffer.last = emitter.buffer.pointer;
                emitter.buffer.pointer = emitter.buffer.start;

                let result = yaml_emitter_flush(&mut emitter);
                assert_eq!(result, success::OK);

                yaml_emitter_delete(&mut emitter);
            }
        }

        /// Tests UTF-8 write error handling
        #[test]
        fn test_yaml_emitter_flush_utf8_write_error() {
            use std::sync::atomic::{AtomicUsize, Ordering};

            static WRITE_COUNT: AtomicUsize = AtomicUsize::new(0);

            unsafe fn fail_write_handler(
                _data: *mut c_void,
                _buffer: *mut u8,
                _size: size_t,
            ) -> c_int {
                WRITE_COUNT.fetch_add(1, Ordering::SeqCst);
                0 // Always return failure
            }

            let mut emitter = YamlEmitterT::new();

            unsafe {
                let _ = yaml_emitter_initialize(&mut emitter);

                yaml_emitter_set_encoding(
                    &mut emitter,
                    YamlUtf8Encoding,
                );

                yaml_emitter_set_output(
                    &mut emitter,
                    fail_write_handler,
                    std::ptr::null_mut(),
                );

                // First, write to the buffer
                let data = b"test data";
                for (i, &byte) in data.iter().enumerate() {
                    *emitter.buffer.start.add(i) = byte;
                }

                // Set the pointer first, then set last
                emitter.buffer.pointer =
                    emitter.buffer.start.add(data.len());
                emitter.buffer.last = emitter.buffer.pointer.add(1);

                println!("Before flush:");
                println!("buffer.start: {:?}", emitter.buffer.start);
                println!(
                    "buffer.pointer: {:?}",
                    emitter.buffer.pointer
                );
                println!("buffer.last: {:?}", emitter.buffer.last);

                let result = yaml_emitter_flush(&mut emitter);

                println!("After flush:");
                println!("buffer.start: {:?}", emitter.buffer.start);
                println!(
                    "buffer.pointer: {:?}",
                    emitter.buffer.pointer
                );
                println!("buffer.last: {:?}", emitter.buffer.last);
                println!(
                    "Write count: {}",
                    WRITE_COUNT.load(Ordering::SeqCst)
                );

                assert_eq!(
                    WRITE_COUNT.load(Ordering::SeqCst),
                    1,
                    "Write handler should be called exactly once"
                );
                assert_eq!(
                    result, FAIL,
                    "Expected flush to fail on write error"
                );
                assert_eq!(
                    emitter.error, YamlWriterError,
                    "Expected emitter.error to be YamlWriterError"
                );

                yaml_emitter_delete(&mut emitter);
            }
        }
    }
}
