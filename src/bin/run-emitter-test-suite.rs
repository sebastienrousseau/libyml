// run-emitter-test-suite.rs - Run the emitter test suite
//

//! A simple YAML emitter test suite that reads a series of `.event` files and emits YAML events.
//!

#![allow(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::items_after_statements,
    clippy::let_underscore_untyped,
    clippy::missing_errors_doc,
    clippy::missing_safety_doc,
    clippy::ptr_as_ptr,
    clippy::single_match_else,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unreadable_literal
)]

mod cstr;

use self::cstr::CStr;
use core::fmt;
pub(crate) use core::primitive::u8 as yaml_char_t;
use libyml::api::ScalarEventData;
use libyml::document::{
    yaml_document_end_event_initialize,
    yaml_document_start_event_initialize,
};
use libyml::{
    yaml_alias_event_initialize, yaml_emitter_delete,
    yaml_emitter_emit, yaml_emitter_initialize,
    yaml_emitter_set_canonical, yaml_emitter_set_output,
    yaml_emitter_set_unicode, yaml_mapping_end_event_initialize,
    yaml_mapping_start_event_initialize, yaml_scalar_event_initialize,
    yaml_sequence_end_event_initialize,
    yaml_sequence_start_event_initialize,
    yaml_stream_end_event_initialize,
    yaml_stream_start_event_initialize, YamlAnyScalarStyle,
    YamlBlockMappingStyle, YamlBlockSequenceStyle,
    YamlDoubleQuotedScalarStyle, YamlEmitterError, YamlEmitterT,
    YamlEventT, YamlFoldedScalarStyle, YamlLiteralScalarStyle,
    YamlMemoryError, YamlPlainScalarStyle, YamlScalarStyleT,
    YamlSingleQuotedScalarStyle, YamlTagDirectiveT, YamlUtf8Encoding,
    YamlVersionDirectiveT, YamlWriterError,
};
use std::env;
use std::error::Error;
use std::ffi::c_void;
use std::fs::File;
use std::io::{self, Read, Write};
use std::mem::MaybeUninit;
use std::process::{self, ExitCode};
use std::ptr::{self, addr_of_mut};

/// A high-level error type for the YAML emitter wrapper.
/// Each variant corresponds to a possible failure mode (memory, unknown event, etc.).
#[derive(Debug)]
pub(crate) enum EmitterError {
    InitializationError(String),
    MemoryError(String),
    UnknownEvent(String),
    EmittingError(String),
    InternalError,
}

impl fmt::Display for EmitterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmitterError::InitializationError(msg)
            | EmitterError::MemoryError(msg)
            | EmitterError::UnknownEvent(msg)
            | EmitterError::EmittingError(msg) => {
                write!(f, "{}", msg)
            }
            EmitterError::InternalError => write!(f, "Internal error"),
        }
    }
}

impl Error for EmitterError {}

/// A static counter that forces a memory error once it hits 0.
/// If `None`, no memory errors are forced; if `Some(n)`, we decrement
/// on each internal "allocation" attempt and simulate `YamlMemoryError` when it hits zero.
static mut FORCE_MEMORY_FAIL_AFTER: Option<i32> = None;

/// Decrements the static memory counter. Returns `true` if a memory failure should occur now.
unsafe fn maybe_fail_memory() -> bool {
    if let Some(ref mut count) = FORCE_MEMORY_FAIL_AFTER {
        if *count == 0 {
            return true;
        }
        *count -= 1;
    }
    false
}

/// Main unsafe function that initializes a `YamlEmitter`, reads events from `stdin`,
/// then emits those events to `stdout`. Cleans up on success or error.
pub(crate) unsafe fn unsafe_main(
    stdin: &mut dyn Read,
    mut stdout: &mut dyn Write,
) -> Result<(), EmitterError> {
    // Simulate memory failure if the counter hits 0 (before init).
    if maybe_fail_memory() {
        return Err(EmitterError::MemoryError(
            "Simulated memory failure (before emitter init)".into(),
        ));
    }

    let mut emitter = MaybeUninit::<YamlEmitterT>::uninit();
    let emitter = emitter.as_mut_ptr();

    // 1. Initialize the emitter
    if yaml_emitter_initialize(emitter).fail {
        return Err(EmitterError::InitializationError(
            "Could not initialize the emitter object".into(),
        ));
    }

    /// A small helper function that integrates the Rust `Write` trait with the
    /// libyml C library's callback signature. This is used by `yaml_emitter_set_output`.
    unsafe fn write_to_stdio(
        data: *mut c_void,
        buffer: *mut u8,
        size: u64,
    ) -> i32 {
        let stdout: *mut &mut dyn Write = data as _;
        let bytes = std::slice::from_raw_parts(buffer, size as usize);
        match (*stdout).write(bytes) {
            Ok(n) => n as i32,
            Err(_) => 0,
        }
    }

    // 2. Configure the emitter (output callback, canonical mode, unicode, etc.)
    yaml_emitter_set_output(
        emitter,
        write_to_stdio,
        addr_of_mut!(stdout).cast(),
    );
    yaml_emitter_set_canonical(emitter, false);
    yaml_emitter_set_unicode(emitter, false);

    // 3. Read from `stdin` line by line, parse events, and emit them
    let mut buf = ReadBuf::new();
    let mut event = MaybeUninit::<YamlEventT>::uninit();
    let event = event.as_mut_ptr();

    let result = loop {
        let line = match buf.get_line(stdin) {
            Some(line) => line,
            None => break Ok(()), // no more lines => success
        };

        // Simulate memory failure on each loop
        if maybe_fail_memory() {
            break Err(EmitterError::MemoryError(
                "Simulated memory failure (event init)".into(),
            ));
        }

        // Temporary buffers for anchors and tags
        let mut anchor = [0u8; 256];
        let mut tag = [0u8; 256];

        // 3a. Identify which YAML event to produce based on the line prefix
        let result = if line.starts_with(b"+STR") {
            yaml_stream_start_event_initialize(event, YamlUtf8Encoding)
        } else if line.starts_with(b"-STR") {
            yaml_stream_end_event_initialize(event)
        } else if line.starts_with(b"+DOC") {
            let implicit = !line[4..].starts_with(b" ---");
            yaml_document_start_event_initialize(
                event,
                ptr::null_mut::<YamlVersionDirectiveT>(),
                ptr::null_mut::<YamlTagDirectiveT>(),
                ptr::null_mut::<YamlTagDirectiveT>(),
                implicit,
            )
        } else if line.starts_with(b"-DOC") {
            let implicit = !line[4..].starts_with(b" ...");
            yaml_document_end_event_initialize(event, implicit)
        } else if line.starts_with(b"+MAP") {
            yaml_mapping_start_event_initialize(
                event,
                get_anchor(b'&', line, anchor.as_mut_ptr()),
                get_tag(line, tag.as_mut_ptr()),
                false,
                YamlBlockMappingStyle,
            )
        } else if line.starts_with(b"-MAP") {
            yaml_mapping_end_event_initialize(event)
        } else if line.starts_with(b"+SEQ") {
            yaml_sequence_start_event_initialize(
                event,
                get_anchor(b'&', line, anchor.as_mut_ptr()),
                get_tag(line, tag.as_mut_ptr()),
                false,
                YamlBlockSequenceStyle,
            )
        } else if line.starts_with(b"-SEQ") {
            yaml_sequence_end_event_initialize(event)
        } else if line.starts_with(b"=VAL") {
            // Scalar
            let mut value = [0i8; 1024];
            let mut style = YamlAnyScalarStyle;
            get_value(line, value.as_mut_ptr(), &mut style);

            let implicit = get_tag(line, tag.as_mut_ptr()).is_null();
            let scalar_event_data = ScalarEventData {
                anchor: get_anchor(b'&', line, anchor.as_mut_ptr()),
                tag: get_tag(line, tag.as_mut_ptr()),
                value: value.as_mut_ptr() as *const yaml_char_t,
                length: -1,
                plain_implicit: implicit,
                quoted_implicit: implicit,
                style,
                _marker: core::marker::PhantomData,
            };
            yaml_scalar_event_initialize(event, scalar_event_data)
        } else if line.starts_with(b"=ALI") {
            // Alias
            yaml_alias_event_initialize(
                event,
                get_anchor(b'*', line, anchor.as_mut_ptr()),
            )
        } else {
            let line = line.as_mut_ptr() as *mut i8;
            break Err(EmitterError::UnknownEvent(format!(
                "Unknown event: '{}'",
                CStr::from_ptr(line)
            )));
        };

        // If event creation failed => memory error
        if result.fail {
            break Err(EmitterError::MemoryError(
                "Memory error: Not enough memory for creating an event"
                    .into(),
            ));
        }

        // 3b. Emit the event
        if yaml_emitter_emit(emitter, event).fail {
            break Err(match (*emitter).error {
                YamlMemoryError => EmitterError::MemoryError(
                    "Memory error: Not enough memory for emitting"
                        .into(),
                ),
                YamlWriterError => {
                    EmitterError::EmittingError(format!(
                        "Writer error: {}",
                        CStr::from_ptr((*emitter).problem)
                    ))
                }
                YamlEmitterError => {
                    EmitterError::EmittingError(format!(
                        "Emitter error: {}",
                        CStr::from_ptr((*emitter).problem)
                    ))
                }
                _ => EmitterError::InternalError,
            });
        }
    };

    // 4. Clean up
    yaml_emitter_delete(emitter);
    result
}

/// A helper struct for buffering input lines from `stdin` or another `Read`.
/// Internally keeps a 1024-byte buffer and attempts to return one line at a time.
struct ReadBuf {
    buf: [u8; 1024],
    offset: usize,
    filled: usize,
}

impl ReadBuf {
    /// Create a new, empty `ReadBuf`.
    fn new() -> Self {
        ReadBuf {
            buf: [0; 1024],
            offset: 0,
            filled: 0,
        }
    }

    /// Attempts to return the next null-terminated line from the input.
    /// Returns `None` on EOF or if reading fails.
    fn get_line(&mut self, input: &mut dyn Read) -> Option<&mut [u8]> {
        loop {
            // Search the existing buffer contents for a newline
            for i in self.offset..(self.offset + self.filled) {
                if self.buf[i] == b'\n' {
                    self.buf[i] = b'\0';
                    let line = &mut self.buf[self.offset..=i];
                    self.offset = i + 1;
                    self.filled -= line.len();
                    return Some(line);
                }
            }
            // Need more data
            let mut remainder =
                &mut self.buf[self.offset + self.filled..];
            if remainder.is_empty() {
                // No more space => line too long for the buffer
                if self.offset == 0 {
                    let _ = writeln!(
                        io::stderr(),
                        "Line too long: '{}'",
                        String::from_utf8_lossy(&self.buf),
                    );
                    process::abort();
                }
                // Otherwise, shift existing data down
                self.buf.copy_within(self.offset.., 0);
                self.offset = 0;
                remainder = &mut self.buf;
            }
            let n = input.read(remainder).ok()?;
            self.filled += n;
            if n == 0 {
                // EOF
                return None;
            }
        }
    }
}

/// Extracts an anchor substring after a given sigil (`&` or `*`).
/// Returns a pointer to the `anchor` buffer if found, or null if not.
unsafe fn get_anchor(
    sigil: u8,
    line: &[u8],
    anchor: *mut u8,
) -> *mut u8 {
    let start = match line.iter().position(|ch| *ch == sigil) {
        Some(offset) => offset + 1,
        None => return ptr::null_mut(),
    };
    let end = match line[start..].iter().position(|ch| *ch == b' ') {
        Some(offset) => start + offset,
        None => line.len(),
    };
    anchor.copy_from_nonoverlapping(
        line[start..end].as_ptr(),
        end - start,
    );
    *anchor.add(end - start) = b'\0';
    anchor
}

/// Extracts a `<tag:...>` substring from `line`, if any.
/// Returns a pointer into the `tag` buffer or null if no tag found.
unsafe fn get_tag(line: &[u8], tag: *mut u8) -> *mut u8 {
    let start = match line.iter().position(|ch| *ch == b'<') {
        Some(offset) => offset + 1,
        None => return ptr::null_mut(),
    };
    let end = match line[start..].iter().position(|ch| *ch == b'>') {
        Some(offset) => start + offset,
        None => return ptr::null_mut(),
    };
    tag.copy_from_nonoverlapping(
        line[start..end].as_ptr(),
        end - start,
    );
    *tag.add(end - start) = b'\0';
    tag
}

/// Parses a line that starts with "`=VAL`" to identify:
///  - The scalar style character (`:`, `'`, `"`, `|`, `>`),
///  - Then copies out the rest of the scalar, decoding escape sequences (like `\n`, `\r`, etc.).
///
/// `value` is the output buffer, `style` is where we store the resulting `YamlScalarStyleT`.
unsafe fn get_value(
    line: &[u8],
    value: *mut i8,
    style: *mut YamlScalarStyleT,
) {
    let line_len = line.len();
    let line_ptr = line.as_ptr() as *mut i8;
    let end = line_ptr.add(line_len);

    // skip "=VAL"
    let mut c = line_ptr.add(4);
    let mut start = ptr::null_mut::<i8>();

    // Identify the style character and the start of the actual text
    while c < end {
        if *c as u8 == b' ' {
            start = c.add(1);
            *style = match *start as u8 {
                b':' => YamlPlainScalarStyle,
                b'\'' => YamlSingleQuotedScalarStyle,
                b'"' => YamlDoubleQuotedScalarStyle,
                b'|' => YamlLiteralScalarStyle,
                b'>' => YamlFoldedScalarStyle,
                _ => {
                    // We might keep scanning, but typically this means malformed input
                    start = ptr::null_mut();
                    c = c.add(1);
                    continue;
                }
            };
            // Move past the style sigil
            start = start.add(1);
            break;
        }
        c = c.add(1);
    }

    if start.is_null() {
        // Malformed => abort or handle differently
        process::abort();
    }

    // Copy characters, interpreting backslash escapes
    let mut i = 0;
    c = start;
    while c < end {
        if *c as u8 == b'\\' {
            c = c.add(1);
            if c >= end {
                break;
            }
            *value.add(i) = match *c as u8 {
                b'\\' => b'\\' as i8,
                b'0' => b'\0' as i8,
                b'b' => b'\x08' as i8,
                b'n' => b'\n' as i8,
                b'r' => b'\r' as i8,
                b't' => b'\t' as i8,
                _ => process::abort(),
            };
        } else {
            *value.add(i) = *c;
        }
        i += 1;
        c = c.add(1);
    }
    *value.add(i) = b'\0' as i8;
}

/// The main entry point for command-line usage: `run-emitter-test-suite <test.event>...`
/// Reads each file in turn, passing it to `unsafe_main`.
fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.is_empty() {
        let _ = writeln!(
            io::stderr(),
            "Usage: run-emitter-test-suite <test.event>..."
        );
        return ExitCode::FAILURE;
    }

    for arg in args {
        let mut stdin = match File::open(&arg) {
            Ok(f) => f,
            Err(e) => {
                let _ = writeln!(
                    io::stderr(),
                    "Error opening file {:?}: {}",
                    arg,
                    e
                );
                return ExitCode::FAILURE;
            }
        };
        let mut stdout = io::stdout();
        let result = unsafe { unsafe_main(&mut stdin, &mut stdout) };
        if let Err(err) = result {
            let _ = writeln!(io::stderr(), "{}", err);
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::{self, Read, Write};

    /// A simple in-memory mock for reading. Consumes data from `self.data`.
    struct MockRead {
        data: Vec<u8>,
        pos: usize,
    }

    impl MockRead {
        fn new(data: Vec<u8>) -> Self {
            Self { data, pos: 0 }
        }
    }

    impl Read for MockRead {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.pos >= self.data.len() {
                return Ok(0);
            }
            let remaining = &self.data[self.pos..];
            let n = remaining.len().min(buf.len());
            buf[..n].copy_from_slice(&remaining[..n]);
            self.pos += n;
            Ok(n)
        }
    }
    /// Call this before `unsafe_main` to force memory failure after N attempts.
    pub(crate) fn set_memory_fail_after(n: i32) {
        unsafe {
            FORCE_MEMORY_FAIL_AFTER = Some(n);
        }
    }
    struct MockWrite {
        data: Vec<u8>,
    }

    impl MockWrite {
        fn new() -> Self {
            Self { data: Vec::new() }
        }
    }

    impl Write for MockWrite {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.data.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    //-----------------------------------------------------------------------// 1. ReadBuf tests
    //-----------------------------------------------------------------------//

    #[test]
    fn test_read_buf_basic() {
        let input = b"line1\nline2\nline3\n";
        let mut mock_read = MockRead::new(input.to_vec());
        let mut buf = ReadBuf::new();

        let line1 = buf.get_line(&mut mock_read).unwrap();
        assert_eq!(line1, b"line1\0");

        let line2 = buf.get_line(&mut mock_read).unwrap();
        assert_eq!(line2, b"line2\0");

        let line3 = buf.get_line(&mut mock_read).unwrap();
        assert_eq!(line3, b"line3\0");

        assert!(buf.get_line(&mut mock_read).is_none());
    }

    #[test]
    fn test_read_buf_no_final_newline() {
        let input = b"line1\nline2";
        let mut mock_read = MockRead::new(input.to_vec());
        let mut buf = ReadBuf::new();

        let line1 = buf.get_line(&mut mock_read).unwrap();
        assert_eq!(line1, b"line1\0");

        let line2 = buf.get_line(&mut mock_read);
        assert!(line2.is_none());
    }

    //-----------------------------------------------------------------------//
    // 2. Anchor/Tag parsing tests
    //-----------------------------------------------------------------------//
    unsafe fn print_debug_str(
        ptr: *const i8,
        max_len: usize,
    ) -> String {
        let mut result = String::new();
        for i in 0..max_len {
            let c = *ptr.add(i);
            if c == 0 {
                break;
            }
            match c as u8 {
                b'\n' => result.push_str("\\n"),
                b'\r' => result.push_str("\\r"),
                b'\t' => result.push_str("\\t"),
                _ => result.push(c as u8 as char),
            }
        }
        result
    }

    unsafe fn compare_raw_str(ptr: *const i8, expected: &[u8]) -> bool {
        let actual = print_debug_str(ptr, 1024);
        let expected_str = std::str::from_utf8(expected).unwrap();

        if actual != expected_str {
            eprintln!("String mismatch:");
            eprintln!("  Expected: {:?}", expected_str);
            eprintln!("  Actual:   {:?}", actual);
            return false;
        }

        // Also check for proper null termination
        let null_pos = expected.len();
        let null_char = *ptr.add(null_pos);
        if null_char != 0 {
            eprintln!(
                "Missing null terminator after '{}', got: {}",
                actual, null_char
            );
            return false;
        }
        true
    }

    #[test]
    fn test_get_anchor() {
        let mut anchor_buf = [0u8; 256];

        unsafe {
            // Test normal anchor
            let line = b"+SEQ &anchor123 ";
            let anchor =
                get_anchor(b'&', line, anchor_buf.as_mut_ptr());
            assert!(!anchor.is_null());
            assert!(compare_raw_str(anchor as *const i8, b"anchor123"));

            // Test no anchor
            let line2 = b"+SEQ ";
            let anchor2 =
                get_anchor(b'&', line2, anchor_buf.as_mut_ptr());
            assert!(anchor2.is_null());
        }
    }

    #[test]
    fn test_get_tag() {
        let mut tag_buf = [0u8; 256];

        unsafe {
            // Test normal tag
            let line = b"=VAL <tag:yaml.org,2002:str>";
            let tag = get_tag(line, tag_buf.as_mut_ptr());
            assert!(!tag.is_null());
            assert!(compare_raw_str(
                tag as *const i8,
                b"tag:yaml.org,2002:str"
            ));

            // Test no tag
            let line2 = b"=VAL plain";
            let tag2 = get_tag(line2, tag_buf.as_mut_ptr());
            assert!(tag2.is_null());
        }
    }

    //-----------------------------------------------------------------------//
    // 3. Value parsing tests
    //-----------------------------------------------------------------------//
    #[test]
    fn test_get_value() {
        unsafe {
            let mut value_buf = [0i8; 1024];
            let mut style = YamlAnyScalarStyle;

            // Plain scalar
            let line = b"=VAL :plain";
            get_value(line, value_buf.as_mut_ptr(), &mut style);
            assert_eq!(style, YamlPlainScalarStyle);
            assert!(compare_raw_str(value_buf.as_ptr(), b"plain"));

            // Single-quoted
            let line = b"=VAL 'here''s to \"quotes\"'";
            get_value(line, value_buf.as_mut_ptr(), &mut style);
            assert_eq!(style, YamlSingleQuotedScalarStyle);
            assert!(compare_raw_str(
                value_buf.as_ptr(),
                b"here''s to \"quotes\"'"
            ));

            // Double-quoted
            let line = b"=VAL \"foo: bar\": baz\"";
            get_value(line, value_buf.as_mut_ptr(), &mut style);
            assert_eq!(style, YamlDoubleQuotedScalarStyle);
            assert!(compare_raw_str(
                value_buf.as_ptr(),
                b"foo: bar\": baz\""
            ));

            // Escaped chars
            let line = b"=VAL :test\\n\\t\\r";
            get_value(line, value_buf.as_mut_ptr(), &mut style);
            assert_eq!(style, YamlPlainScalarStyle);
            // e.g. "test\n\t\r"
            let debug_str = print_debug_str(value_buf.as_ptr(), 1024);
            println!("Escaped debug: {:?}", debug_str);
        }
    }

    #[test]
    fn test_special_chars_in_identifiers() {
        let mut anchor_buf = [0u8; 256];
        let mut tag_buf = [0u8; 256];

        unsafe {
            let line1 = b"+SEQ &anchor-with_special-chars ";
            let anchor =
                get_anchor(b'&', line1, anchor_buf.as_mut_ptr());
            assert!(!anchor.is_null());
            assert!(compare_raw_str(
                anchor as *const i8,
                b"anchor-with_special-chars"
            ));

            let line2 = b"=VAL <tag:yaml.org,2002:str>";
            let tag = get_tag(line2, tag_buf.as_mut_ptr());
            assert!(!tag.is_null());
            assert!(compare_raw_str(
                tag as *const i8,
                b"tag:yaml.org,2002:str"
            ));
        }
    }

    #[test]
    fn test_boundary_conditions() {
        unsafe {
            let mut value_buf = [0i8; 1024];
            let mut style = YamlAnyScalarStyle;

            // Empty
            let line = b"=VAL :";
            get_value(line, value_buf.as_mut_ptr(), &mut style);
            assert_eq!(style, YamlPlainScalarStyle);
            assert!(compare_raw_str(value_buf.as_ptr(), b""));

            // Very long
            let long_line = format!("=VAL :{}", "a".repeat(1000));
            get_value(
                long_line.as_bytes(),
                value_buf.as_mut_ptr(),
                &mut style,
            );
            assert_eq!(style, YamlPlainScalarStyle);
            assert!(compare_raw_str(
                value_buf.as_ptr(),
                "a".repeat(1000).as_bytes()
            ));
        }
    }

    #[test]
    fn test_block_style_values() {
        unsafe {
            let mut value_buf = [0i8; 1024];
            let mut style = YamlAnyScalarStyle;

            // Literal
            let line = b"=VAL |line1\\nline2\\n";
            get_value(line, value_buf.as_mut_ptr(), &mut style);
            assert_eq!(style, YamlLiteralScalarStyle);
            assert!(compare_raw_str(
                value_buf.as_ptr(),
                b"line1\\nline2\\n"
            ));

            // Folded
            let line2 = b"=VAL >folded line\\nnext line\\n";
            get_value(line2, value_buf.as_mut_ptr(), &mut style);
            assert_eq!(style, YamlFoldedScalarStyle);
            assert!(compare_raw_str(
                value_buf.as_ptr(),
                b"folded line\\nnext line\\n"
            ));
        }
    }

    #[test]
    fn test_escape_sequences() {
        unsafe {
            let mut value_buf = [0i8; 1024];
            let mut style = YamlAnyScalarStyle;

            let line = b"=VAL \"\\0\\b\\n\\r\\t\\\\\"";
            get_value(line, value_buf.as_mut_ptr(), &mut style);
            assert_eq!(style, YamlDoubleQuotedScalarStyle);

            let expected = [
                (0, b'\0' as i8, "null"),
                (1, b'\x08' as i8, "backspace"),
                (2, b'\n' as i8, "newline"),
                (3, b'\r' as i8, "carriage return"),
                (4, b'\t' as i8, "tab"),
                (5, b'\\' as i8, "backslash"),
                (6, b'"' as i8, "quote"),
                (7, 0, "terminator"),
            ];

            for (pos, expected_byte, desc) in &expected {
                assert_eq!(
                    value_buf[*pos], *expected_byte,
                    "Mismatch at position {} (expected {}): got {:?}",
                    pos, desc, value_buf[*pos] as u8 as char
                );
            }
        }
    }

    #[test]
    fn test_maximum_length_values() {
        unsafe {
            let mut value_buf = [0i8; 1024];
            let mut style = YamlAnyScalarStyle;

            let max_line = format!("=VAL :{}", "a".repeat(1023));
            get_value(
                max_line.as_bytes(),
                value_buf.as_mut_ptr(),
                &mut style,
            );
            assert_eq!(style, YamlPlainScalarStyle);
            assert!(compare_raw_str(
                value_buf.as_ptr(),
                "a".repeat(1023).as_bytes()
            ));
        }
    }

    #[test]
    fn test_minimum_length_values() {
        unsafe {
            let mut value_buf = [0i8; 1024];
            let mut style = YamlAnyScalarStyle;
            let min_line = b"=VAL :a";
            get_value(min_line, value_buf.as_mut_ptr(), &mut style);
            assert_eq!(style, YamlPlainScalarStyle);
            assert!(compare_raw_str(value_buf.as_ptr(), b"a"));
        }
    }

    //-----------------------------------------------------------------------//
    // 4. Basic Emitter Tests (Successful / Standard Paths)
    //-----------------------------------------------------------------------//
    #[test]
    fn test_emitter_basic_flow() {
        let mut mock_write = MockWrite::new();
        let input = b"+STR\n+DOC\n=VAL :test\n-DOC\n-STR\n";
        let mut mock_read = MockRead::new(input.to_vec());

        unsafe {
            // Ensure no forced memory failure for this test
            set_memory_fail_after(-1);
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(result.is_ok());
            assert!(!mock_write.data.is_empty());
        }
    }

    #[test]
    fn test_emitter_sequence() {
        let mut mock_write = MockWrite::new();
        let input = b"+STR\n+DOC\n+SEQ\n=VAL :item1\n=VAL :item2\n-SEQ\n-DOC\n-STR\n";
        let mut mock_read = MockRead::new(input.to_vec());

        unsafe {
            set_memory_fail_after(-1);
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(result.is_ok());
            assert!(!mock_write.data.is_empty());
        }
    }

    #[test]
    fn test_emitter_mapping() {
        let mut mock_write = MockWrite::new();
        let input = b"+STR\n+DOC\n+MAP\n=VAL :key1\n=VAL :value1\n-MAP\n-DOC\n-STR\n";
        let mut mock_read = MockRead::new(input.to_vec());

        unsafe {
            set_memory_fail_after(-1);
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(result.is_ok());
            assert!(!mock_write.data.is_empty());
        }
    }

    //-----------------------------------------------------------------------//
    // 5. Advanced Emitter Tests (Large input, malformed input, injection, etc.)
    //-----------------------------------------------------------------------//
    #[test]
    fn test_complex_document() {
        let mut mock_write = MockWrite::new();
        let input = b"+STR\n\
                  +DOC ---\n\
                  +MAP\n\
                  =VAL :key1\n\
                  +SEQ\n\
                  =VAL :item1\n\
                  =VAL :item2\n\
                  -SEQ\n\
                  -MAP\n\
                  -DOC\n\
                  -STR\n";
        let mut mock_read = MockRead::new(input.to_vec());

        unsafe {
            set_memory_fail_after(-1);
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(result.is_ok());
            assert!(!mock_write.data.is_empty());
        }
    }

    #[test]
    fn test_empty_input() {
        let mut mock_write = MockWrite::new();
        let input = b"";
        let mut mock_read = MockRead::new(input.to_vec());

        unsafe {
            set_memory_fail_after(-1);
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(result.is_ok());
            assert!(mock_write.data.is_empty());
        }
    }

    #[test]
    fn test_end_to_end_flow() {
        let mut mock_write = MockWrite::new();
        let input = b"+STR\n+DOC\n=VAL :test\n-DOC\n-STR\n";
        let mut mock_read = MockRead::new(input.to_vec());

        unsafe {
            set_memory_fail_after(-1);
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(result.is_ok());
            assert!(!mock_write.data.is_empty());
        }
    }

    //-----------------------------------------------------------------------//
    // 6. Emitter Error Simulation Tests (Memory, I/O, etc.)
    //-----------------------------------------------------------------------//

    #[test]
    fn test_memory_allocation_failure() {
        let mut mock_write = MockWrite::new();
        let input = b"+STR\n+DOC\n=VAL :test\n-DOC\n-STR\n";
        let mut mock_read = MockRead::new(input.to_vec());

        unsafe {
            // Force memory to fail after 2 "allocations."
            set_memory_fail_after(2);
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(
                matches!(result, Err(EmitterError::MemoryError(_))),
                "Expected MemoryError, got: {result:?}"
            );
        }
    }

    #[test]
    fn test_emitter_error_handling() {
        let mut mock_write = MockWrite::new();
        let input = b"INVALID\n";
        let mut mock_read = MockRead::new(input.to_vec());

        unsafe {
            set_memory_fail_after(-1);
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(matches!(
                result,
                Err(EmitterError::UnknownEvent(_))
            ));
        }
    }
}
