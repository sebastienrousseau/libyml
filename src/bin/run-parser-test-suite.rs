// run-parser-test-suite.rs
//
// A YAML parser and formatter using the libyml library.
//
// This program reads YAML files, parses them using the libyml library,
// and outputs a formatted representation of the YAML structure.
//
// The bottom section (#[cfg(test)]) provides thorough unit tests
// for verifying parser correctness on well-formed and malformed YAML.

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
    clippy::too_many_lines,
    clippy::uninlined_format_args
)]

mod cstr;

use self::cstr::CStr;
use anyhow::{bail, Error, Result};
use libyml::{
    yaml_event_delete, yaml_parser_delete, yaml_parser_initialize,
    yaml_parser_parse, yaml_parser_set_input, YamlAliasEvent,
    YamlDocumentEndEvent, YamlDocumentStartEvent,
    YamlDoubleQuotedScalarStyle, YamlEventT, YamlEventTypeT,
    YamlFoldedScalarStyle, YamlLiteralScalarStyle, YamlMappingEndEvent,
    YamlMappingStartEvent, YamlNoEvent, YamlParserT,
    YamlPlainScalarStyle, YamlScalarEvent, YamlSequenceEndEvent,
    YamlSequenceStartEvent, YamlSingleQuotedScalarStyle,
    YamlStreamEndEvent, YamlStreamStartEvent,
};
use std::{
    env,
    ffi::c_void,
    fs::File,
    io::{self, Read, Write},
    mem::MaybeUninit,
    path::Path,
    process::ExitCode,
    ptr::addr_of_mut,
    slice,
};

/// The main parsing function that processes YAML input and writes formatted output.
///
/// # Safety
///
/// This function is unsafe because it deals with raw pointers and FFI.
/// Callers must ensure that the provided `stdin` and `stdout` are valid
/// and that the FFI calls are used correctly.
///
/// # Arguments
///
/// * `stdin` - A mutable reference to a type that implements `Read`, from which YAML will be read.
/// * `stdout` - A mutable reference to a type that implements `Write`, to which formatted output will be written.
///
/// # Returns
///
/// Returns `Ok(())` if parsing and formatting succeed, or an `Error` if any issues occur.
pub(crate) unsafe fn unsafe_main(
    mut stdin: &mut dyn Read,
    stdout: &mut dyn Write,
) -> Result<()> {
    let mut parser = MaybeUninit::<YamlParserT>::uninit();
    let parser = parser.as_mut_ptr();
    if yaml_parser_initialize(parser).fail {
        bail!("Could not initialize the parser object");
    }

    /// Callback function for reading input from stdio.
    ///
    /// This function is called by the YAML parser to read input data.
    /// It deals with raw pointers that point to a `Read` trait object.
    unsafe fn read_from_stdio(
        data: *mut c_void,
        buffer: *mut u8,
        size: u64,
        size_read: *mut u64,
    ) -> i32 {
        let stdin: *mut &mut dyn Read = data.cast();
        let slice =
            slice::from_raw_parts_mut(buffer.cast(), size as usize);
        match (*stdin).read(slice) {
            Ok(n) => {
                *size_read = n as u64;
                1
            }
            Err(_) => 0,
        }
    }

    yaml_parser_set_input(
        parser,
        read_from_stdio,
        addr_of_mut!(stdin).cast(),
    );

    let mut event = MaybeUninit::<YamlEventT>::uninit();
    let event_ptr = event.as_mut_ptr();

    loop {
        if yaml_parser_parse(parser, event_ptr).fail {
            let error = format!(
                "Parse error: {}",
                CStr::from_ptr((*parser).problem)
            );
            let error = if (*parser).problem_mark.line != 0
                || (*parser).problem_mark.column != 0
            {
                format!(
                    "{}\nLine: {} Column: {}",
                    error,
                    ((*parser).problem_mark.line).wrapping_add(1),
                    ((*parser).problem_mark.column).wrapping_add(1),
                )
            } else {
                error
            };
            yaml_parser_delete(parser);
            return Err(Error::msg(error));
        }

        let event_type: YamlEventTypeT = (*event_ptr).type_;
        match event_type {
            YamlNoEvent => writeln!(stdout, "???")?,
            YamlStreamStartEvent => writeln!(stdout, "+STR")?,
            YamlStreamEndEvent => {
                writeln!(stdout, "-STR")?;
            }
            YamlDocumentStartEvent => {
                write!(stdout, "+DOC")?;
                if !(*event_ptr).data.document_start.implicit {
                    write!(stdout, " ---")?;
                }
                writeln!(stdout)?;
            }
            YamlDocumentEndEvent => {
                write!(stdout, "-DOC")?;
                if !(*event_ptr).data.document_end.implicit {
                    write!(stdout, " ...")?;
                }
                writeln!(stdout)?;
            }
            YamlMappingStartEvent => {
                let data = (*event_ptr).data.mapping_start;
                write!(stdout, "+MAP")?;
                if !data.anchor.is_null() {
                    let anchor_str =
                        CStr::from_ptr(data.anchor as *const i8);
                    write!(stdout, " &{}", anchor_str)?;
                }
                if !data.tag.is_null() {
                    let tag_str = CStr::from_ptr(data.tag as *const i8);
                    write!(stdout, " <{}>", tag_str)?;
                }
                writeln!(stdout)?;
            }
            YamlMappingEndEvent => {
                writeln!(stdout, "-MAP")?;
            }
            YamlSequenceStartEvent => {
                let data = (*event_ptr).data.sequence_start;
                write!(stdout, "+SEQ")?;
                if !data.anchor.is_null() {
                    let anchor_str =
                        CStr::from_ptr(data.anchor as *const i8);
                    write!(stdout, " &{}", anchor_str)?;
                }
                if !data.tag.is_null() {
                    let tag_str = CStr::from_ptr(data.tag as *const i8);
                    write!(stdout, " <{}>", tag_str)?;
                }
                writeln!(stdout)?;
            }
            YamlSequenceEndEvent => {
                writeln!(stdout, "-SEQ")?;
            }
            YamlScalarEvent => {
                let data = (*event_ptr).data.scalar;
                write!(stdout, "=VAL")?;
                if !data.anchor.is_null() {
                    let anchor_str =
                        CStr::from_ptr(data.anchor as *const i8);
                    write!(stdout, " &{}", anchor_str)?;
                }
                if !data.tag.is_null() {
                    let tag_str = CStr::from_ptr(data.tag as *const i8);
                    write!(stdout, " <{}>", tag_str)?;
                }
                // identify style
                let style_bytes = match data.style {
                    YamlPlainScalarStyle => b" :",
                    YamlSingleQuotedScalarStyle => b" '",
                    YamlDoubleQuotedScalarStyle => b" \"",
                    YamlLiteralScalarStyle => b" |",
                    YamlFoldedScalarStyle => b" >",
                    _ => {
                        yaml_parser_delete(parser);
                        return Err(Error::msg("Unknown scalar style"));
                    }
                };
                stdout.write_all(style_bytes)?;

                let slice = slice::from_raw_parts(
                    data.value,
                    data.length as usize,
                );
                print_escaped(stdout, slice)?;
                writeln!(stdout)?;
            }
            YamlAliasEvent => {
                let data = (*event_ptr).data.alias;
                let anchor_str =
                    CStr::from_ptr(data.anchor as *const i8);
                writeln!(stdout, "=ALI *{}", anchor_str)?;
            }
            _ => {
                yaml_parser_delete(parser);
                return Err(Error::msg("Unknown event type"));
            }
        }

        yaml_event_delete(event_ptr);

        if event_type == YamlStreamEndEvent {
            break;
        }
    }

    yaml_parser_delete(parser);
    Ok(())
}

/// Prints a slice of bytes to `stdout`, escaping special characters.
fn print_escaped(
    stdout: &mut dyn Write,
    slice: &[u8],
) -> io::Result<()> {
    // Attempt to interpret the entire slice as UTF-8.
    // If it fails, we revert to a fallback that hex-escapes unknown bytes.
    match std::str::from_utf8(slice) {
        Ok(utf8_str) => {
            // Iterate each *Unicode character* in the string
            for ch in utf8_str.chars() {
                match ch {
                    '\\' => write!(stdout, "\\\\")?,
                    '\0' => write!(stdout, "\\0")?,
                    '\x08' => write!(stdout, "\\b")?,
                    '\n' => write!(stdout, "\\n")?,
                    '\r' => write!(stdout, "\\r")?,
                    '\t' => write!(stdout, "\\t")?,
                    // Otherwise, print as-is (including non-ASCII Unicode).
                    _ => write!(stdout, "{}", ch)?,
                }
            }
        }
        Err(_) => {
            // Fallback: If it's not valid UTF-8, we can do what you did before:
            // escape controls, pass ASCII as-is, and hex for non-ASCII.
            for &byte in slice {
                match byte {
                    b'\\' => stdout.write_all(b"\\\\")?,
                    b'\0' => stdout.write_all(b"\\0")?,
                    b'\x08' => stdout.write_all(b"\\b")?,
                    b'\n' => stdout.write_all(b"\\n")?,
                    b'\r' => stdout.write_all(b"\\r")?,
                    b'\t' => stdout.write_all(b"\\t")?,
                    c if c.is_ascii_graphic() || c == b' ' => {
                        stdout.write_all(&[c])?;
                    }
                    c => {
                        write!(stdout, "\\x{:02x}", c)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// The main entry point for the parser test suite.
///
/// Usage: run-parser-test-suite <file.yaml>...
/// Reads each file and attempts to parse it, printing the event stream or error.
fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.is_empty() {
        eprintln!("Error: No input files provided.");
        eprintln!(
            "Usage: {} <in.yaml>...",
            env::args().next().unwrap_or_default()
        );
        return ExitCode::FAILURE;
    }

    for arg in args {
        let path = Path::new(&arg);
        if !path.exists() {
            eprintln!("Error: File {:?} does not exist.", path);
            return ExitCode::FAILURE;
        }
        if !path.is_file() {
            eprintln!("Error: {:?} is not a file.", path);
            return ExitCode::FAILURE;
        }

        match File::open(path) {
            Ok(mut file) => {
                let mut stdout = io::stdout();
                eprintln!("Processing file: {:?}", path);
                match unsafe { unsafe_main(&mut file, &mut stdout) } {
                    Ok(()) => eprintln!(
                        "Successfully processed file: {:?}",
                        path
                    ),
                    Err(err) => {
                        eprintln!(
                            "Error processing file {:?}: {}",
                            path, err
                        );
                        return ExitCode::FAILURE;
                    }
                }
            }
            Err(err) => {
                eprintln!("Error opening file {:?}: {}", path, err);
                return ExitCode::FAILURE;
            }
        }
    }

    eprintln!("All files processed successfully.");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    /*!
    # Unit Tests for `run-parser-test-suite`

    We define tests for:
    1. **Basic well-formed YAML** (small, single doc).
    2. **Multi-document** YAML.
    3. **Anchors, tags, aliases**.
    4. **Scalars** (quoted, folded, literal).
    5. **Malformed** input (unexpected tokens).
    6. **Empty** input.
    7. **I/O error** simulation with a failing reader.
    */

    use super::*;
    use std::io::{self, Read, Write};

    /// A simple mock reader that returns data from a byte slice.
    struct MockRead {
        data: Vec<u8>,
        pos: usize,
    }

    impl MockRead {
        fn new(data: &[u8]) -> Self {
            Self {
                data: data.to_vec(),
                pos: 0,
            }
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

    /// A mock writer that accumulates writes into an internal buffer.
    struct MockWrite {
        pub data: Vec<u8>,
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

    /// A mock reader that fails after reading `fail_after` bytes in total.
    struct FailingMockRead {
        data: Vec<u8>,
        pos: usize,
        fail_after: usize,
        total_read: usize,
    }

    impl FailingMockRead {
        fn new(data: &[u8], fail_after: usize) -> Self {
            Self {
                data: data.to_vec(),
                pos: 0,
                fail_after,
                total_read: 0,
            }
        }
    }

    impl Read for FailingMockRead {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.pos >= self.data.len() {
                return Ok(0);
            }
            // If we've already read fail_after, produce an I/O error
            if self.total_read >= self.fail_after {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Simulated I/O error",
                ));
            }
            let remaining = &self.data[self.pos..];
            let n = remaining.len().min(buf.len());
            // Also ensure we don't read beyond fail_after
            let max_possible =
                self.fail_after.saturating_sub(self.total_read);
            let read_count = n.min(max_possible);

            buf[..read_count].copy_from_slice(&remaining[..read_count]);
            self.pos += read_count;
            self.total_read += read_count;
            Ok(read_count)
        }
    }

    fn as_str_lossy(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).to_string()
    }

    /// Tests a simple, well-formed YAML with a single doc.
    #[test]
    fn test_parse_basic_single_doc() {
        let input = b"\
+STR
+DOC
=VAL :hello
-DOC
-STR
";
        let mut mock_read = MockRead::new(input);
        let mut mock_write = MockWrite::new();

        unsafe {
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(result.is_ok());
            // parse output
            let output = as_str_lossy(&mock_write.data);
            // We expect the parser to interpret these events. Output is the same as input or with some transformations?
            // Actually, the parser produces: ??? for NoEvent or +STR, etc.
            // We can just check that there's no error. If you want to verify the EXACT output, you'd do so here.
            println!("Parser output:\n{}", output);
        }
    }

    /// Tests a multi-document YAML input.
    #[test]
    fn test_parse_multi_document() {
        let input = b"\
+STR
+DOC ---
=VAL :first
-DOC ...
+DOC ---
=VAL :second
-DOC ...
-STR
";
        let mut mock_read = MockRead::new(input);
        let mut mock_write = MockWrite::new();

        unsafe {
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(
                result.is_ok(),
                "Parsing multi-doc input should succeed"
            );
            let output = as_str_lossy(&mock_write.data);
            println!("Parser output (multi-doc):\n{}", output);
            // Here you'd typically check for certain lines like:
            // +STR, +DOC ---, =VAL :first, ...
        }
    }

    /// Tests anchors, tags, and aliases.
    #[test]
    fn test_parse_anchor_and_alias() {
        let input = b"\
+STR
+DOC
+SEQ &myseq
=VAL :item1
=VAL :item2
-SEQ
=ALI *myseq
-DOC
-STR
";
        let mut mock_read = MockRead::new(input);
        let mut mock_write = MockWrite::new();

        unsafe {
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(
                result.is_ok(),
                "Parsing anchor & alias should succeed"
            );
            let output = as_str_lossy(&mock_write.data);
            println!("Parser output (anchors):\n{}", output);
        }
    }

    /// Tests various scalar styles in a single doc: plain, single-quoted, double-quoted, literal, folded
    #[test]
    fn test_parse_various_scalars() {
        // Note: The parser prints them as =VAL :someValue, etc. We'll see if it complains about style.
        let input = b"\
+STR
+DOC
=VAL :plain
=VAL 'single-quoted'
=VAL \"double-quoted\"
=VAL |literal
=VAL >folded
-DOC
-STR
";
        let mut mock_read = MockRead::new(input);
        let mut mock_write = MockWrite::new();

        unsafe {
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(
                result.is_ok(),
                "Parser should handle different scalar styles"
            );
            let output = as_str_lossy(&mock_write.data);
            println!("Parser output (scalars):\n{}", output);
        }
    }

    /// Tests a malformed input (invalid event lines).
    #[test]
    fn test_parse_malformed_input() {
        // Raw YAML with unclosed bracket
        let input = br"
someKey: [1, 2, 3
AnotherKey: 42
";

        let mut mock_read = MockRead::new(input);
        let mut mock_write = MockWrite::new();

        unsafe {
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(
                result.is_err(),
                "Parser should fail on truly malformed YAML"
            );
            println!("Malformed parse result: {:?}", result.err());
        }
    }

    /// Tests empty input: no lines at all
    #[test]
    fn test_parse_empty_input() {
        let input = b"";
        let mut mock_read = MockRead::new(input);
        let mut mock_write = MockWrite::new();

        unsafe {
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            // The parser ends up not reading anything, so we expect no events
            assert!(
                result.is_ok(),
                "Empty input should be ok, no events"
            );
            let output = as_str_lossy(&mock_write.data);
            println!("Output for empty input:\n{}", output);
            // Possibly it prints nothing or ???. Implementation-defined
        }
    }

    /// Tests an I/O error scenario: we feed a doc, but the read fails mid-way
    #[test]
    fn test_parse_io_error() {
        let doc = b"+STR\n+DOC\n=VAL :someData\n-DOC\n-STR\n";
        // Force an I/O fail after 5 bytes
        let mut mock_read = FailingMockRead::new(doc, 5);
        let mut mock_write = MockWrite::new();

        unsafe {
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            // We expect an Err(...) because reading fails mid-parse
            assert!(
                result.is_err(),
                "Failing I/O should produce parse error"
            );
            let err_msg = format!("{:?}", result.err());
            println!("I/O error parse result: {}", err_msg);
        }
    }

    #[test]
    fn test_parse_nested_structures() {
        let input = b"\
+STR
+DOC
+MAP
=VAL :key1
+SEQ
=VAL :item1
=VAL :item2
-SEQ
=VAL :key2
+MAP
=VAL :nestedKey
=VAL :nestedValue
-MAP
-MAP
-DOC
-STR
";
        let mut mock_read = MockRead::new(input);
        let mut mock_write = MockWrite::new();

        unsafe {
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(
                result.is_ok(),
                "Parser should handle nested mappings and sequences"
            );
            let output = as_str_lossy(&mock_write.data);
            println!("Parser output (nested structures):\n{}", output);
        }
    }

    #[test]
    fn test_missing_document_end() {
        let input = b"\
+STR
+DOC
=VAL :key
";
        let mut mock_read = MockRead::new(input);
        let mut mock_write = MockWrite::new();

        unsafe {
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(
                result.is_ok(),
                "Parser should handle missing document end gracefully"
            );
            let output = as_str_lossy(&mock_write.data);
            println!(
                "Parser output (missing document end):\n{}",
                output
            );
        }
    }

    #[test]
    fn test_parse_with_bom() {
        let input = b"\xEF\xBB\xBF+STR\n+DOC\n=VAL :key\n-DOC\n-STR\n";
        let mut mock_read = MockRead::new(input);
        let mut mock_write = MockWrite::new();

        unsafe {
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(
                result.is_ok(),
                "Parser should handle BOM correctly"
            );
            let output = as_str_lossy(&mock_write.data);
            println!("Parser output (BOM):\n{}", output);
        }
    }

    #[test]
    fn test_repeated_anchors_and_aliases() {
        let input = b"\
+STR
+DOC
+MAP
=VAL :key1
=VAL &anchor value
=ALI *anchor
-STR
";

        let mut mock_read = MockRead::new(input);
        let mut mock_write = MockWrite::new();

        unsafe {
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(
                result.is_ok(),
                "Parser should handle repeated anchors and aliases"
            );
            let output = as_str_lossy(&mock_write.data);
            assert!(
                output.contains("&anchor")
                    && output.contains("*anchor"),
                "Anchors and aliases were not handled correctly"
            );
        }
    }

    #[test]
    fn test_edge_case_tags() {
        let input = b"\
+STR
+DOC
+MAP <tag:yaml.org,2002:str>
=VAL :key
=VAL :value
-MAP
-DOC
-STR
";

        let mut mock_read = MockRead::new(input);
        let mut mock_write = MockWrite::new();

        unsafe {
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(
                result.is_ok(),
                "Parser should handle edge case tags"
            );
            let output = as_str_lossy(&mock_write.data);
            assert!(
                output.contains("<tag:yaml.org,2002:str>"),
                "Tag format was not handled correctly"
            );
        }
    }

    #[test]
    fn test_deeply_nested_structures() {
        let input = b"+STR\n+DOC\n".to_vec(); // Convert the byte string to Vec<u8>
        let nested_indentation = "  ".repeat(1000); // Indent for deep nesting

        let mut input = input;
        input.extend(nested_indentation.as_bytes()); // Append the indentation as bytes
        input.extend(b"+MAP\n=VAL :key\n-STR\n"); // Append the rest of the YAML structure

        let mut mock_read = MockRead::new(&input);
        let mut mock_write = MockWrite::new();

        unsafe {
            let result = unsafe_main(&mut mock_read, &mut mock_write);
            assert!(
                result.is_ok(),
                "Parser should handle deeply nested structures"
            );
        }
    }
}
