#![feature(prelude_import)]
#![allow(missing_docs)]
#![allow(clippy::type_complexity, clippy::uninlined_format_args)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
mod bin {
    use std::error::Error;
    use std::fmt;
    use std::io::{Read, Write};
    use std::path::Path;
    use std::process::{Command, Stdio};
    pub(crate) enum MyError {
        IoError(std::io::Error),
        Other(String),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for MyError {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                MyError::IoError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "IoError",
                        &__self_0,
                    )
                }
                MyError::Other(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Other",
                        &__self_0,
                    )
                }
            }
        }
    }
    impl fmt::Display for MyError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                MyError::IoError(e) => f.write_fmt(format_args!("I/O Error: {0}", e)),
                MyError::Other(s) => f.write_fmt(format_args!("Error: {0}", s)),
            }
        }
    }
    impl Error for MyError {}
    impl From<std::io::Error> for MyError {
        fn from(error: std::io::Error) -> Self {
            MyError::IoError(error)
        }
    }
    pub(crate) struct Output {
        pub(crate) success: bool,
        pub(crate) stdout: Vec<u8>,
        pub(crate) stderr: Vec<u8>,
    }
    pub(crate) fn run(
        compiled: &str,
        unsafe_main: unsafe fn(
            stdin: &mut dyn Read,
            stdout: &mut dyn Write,
        ) -> Result<(), MyError>,
        input: &Path,
    ) -> Output {
        if false {
            let input_data = std::fs::read(input).unwrap();
            let mut stdin = &input_data[..];
            let mut stdout = Vec::new();
            let result = unsafe { unsafe_main(&mut stdin, &mut stdout) };
            Output {
                success: result.is_ok(),
                stdout,
                stderr: match result {
                    Ok(_) => Vec::new(),
                    Err(e) => e.to_string().into_bytes(),
                },
            }
        } else {
            let output = Command::new(compiled)
                .arg(input)
                .stdin(Stdio::null())
                .output()
                .unwrap();
            Output {
                success: output.status.success(),
                stdout: output.stdout,
                stderr: output.stderr,
            }
        }
    }
}
#[path = "../src/bin/run-emitter-test-suite.rs"]
#[allow(dead_code)]
mod run_emitter_test_suite {
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
    mod cstr {
        use core::ffi::c_char;
        use std::fmt::{self, Display, Write as _};
        use std::slice;
        use std::str;
        pub(crate) struct CStr {
            pub(crate) ptr: *const u8,
        }
        impl CStr {
            pub(crate) unsafe fn from_ptr(ptr: *const c_char) -> Self {
                CStr { ptr: ptr.cast() }
            }
        }
        impl Display for CStr {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                let len = unsafe { strlen(self.ptr) };
                let mut bytes = unsafe { slice::from_raw_parts(self.ptr, len) };
                loop {
                    match str::from_utf8(bytes) {
                        Ok(valid) => return formatter.write_str(valid),
                        Err(utf8_error) => {
                            let valid_up_to = utf8_error.valid_up_to();
                            let valid = unsafe {
                                str::from_utf8_unchecked(&bytes[..valid_up_to])
                            };
                            formatter.write_str(valid)?;
                            formatter.write_char(char::REPLACEMENT_CHARACTER)?;
                            if let Some(error_len) = utf8_error.error_len() {
                                bytes = &bytes[valid_up_to + error_len..];
                            } else {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
        unsafe fn strlen(s: *const u8) -> usize {
            let mut end = s;
            while *end != 0 {
                end = end.add(1);
            }
            end.offset_from(s) as usize
        }
    }
    use self::cstr::CStr;
    use core::fmt;
    pub(crate) use core::primitive::u8 as yaml_char_t;
    use libyml::api::ScalarEventData;
    use libyml::document::{
        yaml_document_end_event_initialize, yaml_document_start_event_initialize,
    };
    use libyml::{
        yaml_alias_event_initialize, yaml_emitter_delete, yaml_emitter_emit,
        yaml_emitter_initialize, yaml_emitter_set_canonical, yaml_emitter_set_output,
        yaml_emitter_set_unicode, yaml_mapping_end_event_initialize,
        yaml_mapping_start_event_initialize, yaml_scalar_event_initialize,
        yaml_sequence_end_event_initialize, yaml_sequence_start_event_initialize,
        yaml_stream_end_event_initialize, yaml_stream_start_event_initialize,
        YamlAnyScalarStyle, YamlBlockMappingStyle, YamlBlockSequenceStyle,
        YamlDoubleQuotedScalarStyle, YamlEmitterError, YamlEmitterT, YamlEventT,
        YamlFoldedScalarStyle, YamlLiteralScalarStyle, YamlMemoryError,
        YamlPlainScalarStyle, YamlScalarStyleT, YamlSingleQuotedScalarStyle,
        YamlTagDirectiveT, YamlUtf8Encoding, YamlVersionDirectiveT, YamlWriterError,
    };
    use std::env;
    use std::error::Error;
    use std::ffi::c_void;
    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::mem::MaybeUninit;
    use std::process::{self, ExitCode};
    use std::ptr::{self, addr_of_mut};
    pub(crate) enum EmitterError {
        InitializationError(String),
        MemoryError(String),
        UnknownEvent(String),
        EmittingError(String),
        InternalError,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for EmitterError {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                EmitterError::InitializationError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "InitializationError",
                        &__self_0,
                    )
                }
                EmitterError::MemoryError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "MemoryError",
                        &__self_0,
                    )
                }
                EmitterError::UnknownEvent(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "UnknownEvent",
                        &__self_0,
                    )
                }
                EmitterError::EmittingError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "EmittingError",
                        &__self_0,
                    )
                }
                EmitterError::InternalError => {
                    ::core::fmt::Formatter::write_str(f, "InternalError")
                }
            }
        }
    }
    impl fmt::Display for EmitterError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                EmitterError::InitializationError(msg)
                | EmitterError::MemoryError(msg)
                | EmitterError::UnknownEvent(msg)
                | EmitterError::EmittingError(msg) => {
                    f.write_fmt(format_args!("{0}", msg))
                }
                EmitterError::InternalError => {
                    f.write_fmt(format_args!("Internal error"))
                }
            }
        }
    }
    impl Error for EmitterError {}
    pub(crate) unsafe fn unsafe_main(
        stdin: &mut dyn Read,
        mut stdout: &mut dyn Write,
    ) -> Result<(), EmitterError> {
        let mut emitter = MaybeUninit::<YamlEmitterT>::uninit();
        let emitter = emitter.as_mut_ptr();
        if yaml_emitter_initialize(emitter).fail {
            return Err(
                EmitterError::InitializationError(
                    "Could not initialize the emitter object".into(),
                ),
            );
        }
        unsafe fn write_to_stdio(data: *mut c_void, buffer: *mut u8, size: u64) -> i32 {
            let stdout: *mut &mut dyn Write = data as _;
            let bytes = std::slice::from_raw_parts(buffer, size as usize);
            match (*stdout).write(bytes) {
                Ok(n) => n as i32,
                Err(_) => 0,
            }
        }
        yaml_emitter_set_output(emitter, write_to_stdio, (&raw mut stdout).cast());
        yaml_emitter_set_canonical(emitter, false);
        yaml_emitter_set_unicode(emitter, false);
        let mut buf = ReadBuf::new();
        let mut event = MaybeUninit::<YamlEventT>::uninit();
        let event = event.as_mut_ptr();
        let result = loop {
            let line = match buf.get_line(stdin) {
                Some(line) => line,
                None => break Ok(()),
            };
            let mut anchor = [0u8; 256];
            let mut tag = [0u8; 256];
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
                yaml_alias_event_initialize(
                    event,
                    get_anchor(b'*', line, anchor.as_mut_ptr()),
                )
            } else {
                let line = line.as_mut_ptr() as *mut i8;
                break Err(
                    EmitterError::UnknownEvent(
                        ::alloc::__export::must_use({
                            let res = ::alloc::fmt::format(
                                format_args!("Unknown event: \'{0}\'", CStr::from_ptr(line)),
                            );
                            res
                        }),
                    ),
                );
            };
            if result.fail {
                break Err(
                    EmitterError::MemoryError(
                        "Memory error: Not enough memory for creating an event".into(),
                    ),
                );
            }
            if yaml_emitter_emit(emitter, event).fail {
                break Err(
                    match (*emitter).error {
                        YamlMemoryError => {
                            EmitterError::MemoryError(
                                "Memory error: Not enough memory for emitting".into(),
                            )
                        }
                        YamlWriterError => {
                            EmitterError::EmittingError(
                                ::alloc::__export::must_use({
                                    let res = ::alloc::fmt::format(
                                        format_args!(
                                            "Writer error: {0}",
                                            CStr::from_ptr((*emitter).problem),
                                        ),
                                    );
                                    res
                                }),
                            )
                        }
                        YamlEmitterError => {
                            EmitterError::EmittingError(
                                ::alloc::__export::must_use({
                                    let res = ::alloc::fmt::format(
                                        format_args!(
                                            "Emitter error: {0}",
                                            CStr::from_ptr((*emitter).problem),
                                        ),
                                    );
                                    res
                                }),
                            )
                        }
                        _ => EmitterError::InternalError,
                    },
                );
            }
        };
        yaml_emitter_delete(emitter);
        result.map_err(Into::into)
    }
    struct ReadBuf {
        buf: [u8; 1024],
        offset: usize,
        filled: usize,
    }
    impl ReadBuf {
        fn new() -> Self {
            ReadBuf {
                buf: [0; 1024],
                offset: 0,
                filled: 0,
            }
        }
        fn get_line(&mut self, input: &mut dyn Read) -> Option<&mut [u8]> {
            loop {
                for i in self.offset..self.offset + self.filled {
                    if self.buf[i] == b'\n' {
                        self.buf[i] = b'\0';
                        let line = &mut self.buf[self.offset..=i];
                        self.offset = i + 1;
                        self.filled -= line.len();
                        return Some(line);
                    }
                }
                let mut remainder = &mut self.buf[self.offset + self.filled..];
                if remainder.is_empty() {
                    if self.offset == 0 {
                        let _ = io::stderr()
                            .write_fmt(
                                format_args!(
                                    "Line too long: \'{0}\'\n",
                                    String::from_utf8_lossy(&self.buf),
                                ),
                            );
                        process::abort();
                    }
                    self.buf.copy_within(self.offset.., 0);
                    self.offset = 0;
                    remainder = &mut self.buf;
                }
                let n = input.read(remainder).ok()?;
                self.filled += n;
                if n == 0 {
                    return None;
                }
            }
        }
    }
    unsafe fn get_anchor(sigil: u8, line: &[u8], anchor: *mut u8) -> *mut u8 {
        let start = match line.iter().position(|ch| *ch == sigil) {
            Some(offset) => offset + 1,
            None => return ptr::null_mut(),
        };
        let end = match line[start..].iter().position(|ch| *ch == b' ') {
            Some(offset) => start + offset,
            None => line.len(),
        };
        anchor.copy_from_nonoverlapping(line[start..end].as_ptr(), end - start);
        *anchor.add(end - start) = b'\0';
        anchor
    }
    unsafe fn get_tag(line: &[u8], tag: *mut u8) -> *mut u8 {
        let start = match line.iter().position(|ch| *ch == b'<') {
            Some(offset) => offset + 1,
            None => return ptr::null_mut(),
        };
        let end = match line[start..].iter().position(|ch| *ch == b'>') {
            Some(offset) => start + offset,
            None => return ptr::null_mut(),
        };
        tag.copy_from_nonoverlapping(line[start..end].as_ptr(), end - start);
        *tag.add(end - start) = b'\0';
        tag
    }
    unsafe fn get_value(line: &[u8], value: *mut i8, style: *mut YamlScalarStyleT) {
        let line_len = line.len();
        let line = line.as_ptr() as *mut i8;
        let mut start = ptr::null_mut::<i8>();
        let end = line.add(line_len);
        let mut c = line.offset(4);
        while c < end {
            if *c as u8 == b' ' {
                start = c.offset(1);
                *style = match *start as u8 {
                    b':' => YamlPlainScalarStyle,
                    b'\'' => YamlSingleQuotedScalarStyle,
                    b'"' => YamlDoubleQuotedScalarStyle,
                    b'|' => YamlLiteralScalarStyle,
                    b'>' => YamlFoldedScalarStyle,
                    _ => {
                        start = ptr::null_mut::<i8>();
                        c = c.offset(1);
                        continue;
                    }
                };
                start = start.offset(1);
                break;
            }
            c = c.offset(1);
        }
        if start.is_null() {
            process::abort();
        }
        let mut i = 0;
        c = start;
        while c < end {
            *value.offset(i) = if *c as u8 == b'\\' {
                c = c.offset(1);
                match *c as u8 {
                    b'\\' => b'\\' as i8,
                    b'0' => b'\0' as i8,
                    b'b' => b'\x08' as i8,
                    b'n' => b'\n' as i8,
                    b'r' => b'\r' as i8,
                    b't' => b'\t' as i8,
                    _ => process::abort(),
                }
            } else {
                *c
            };
            i += 1;
            c = c.offset(1);
        }
        *value.offset(i) = b'\0' as i8;
    }
    fn main() -> ExitCode {
        let args = env::args_os().skip(1);
        if args.len() == 0 {
            let _ = io::stderr()
                .write_fmt(
                    format_args!("Usage: run-emitter-test-suite <test.event>...\n"),
                );
            return ExitCode::FAILURE;
        }
        for arg in args {
            let mut stdin = File::open(arg).unwrap();
            let mut stdout = io::stdout();
            let result = unsafe { unsafe_main(&mut stdin, &mut stdout) };
            if let Err(err) = result {
                let _ = io::stderr().write_fmt(format_args!("{0}\n", err));
                return ExitCode::FAILURE;
            }
        }
        ExitCode::SUCCESS
    }
}
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
unsafe fn unsafe_main_wrapper(
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
) -> Result<(), bin::MyError> {
    run_emitter_test_suite::unsafe_main(stdin, stdout)
        .map_err(|e| bin::MyError::Other(e.to_string()))
}
fn test(id: &str) {
    let dir = Path::new("tests").join("data").join("yaml-test-suite").join(id);
    let output = bin::run(
        "/Users/seb/Team Rousseau Dropbox/Sebastien Rousseau/System/Library/Code/Rust/libyml/target/debug/run-emitter-test-suite",
        unsafe_main_wrapper,
        &dir.join("test.event"),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    {
        ::std::io::_eprint(format_args!("{0}", stderr));
    };
    let out = if dir.join("out.yaml").exists() {
        dir.join("out.yaml")
    } else {
        dir.join("in.yaml")
    };
    let expected = fs::read_to_string(out).unwrap();
    {
        {
            match (&(expected), &(stdout)) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "assertion failed: `(left == right)`{0}{1}\n\n{2}\n",
                                    "",
                                    format_args!(""),
                                    ::pretty_assertions::StrComparison::new(left_val, right_val),
                                ),
                            );
                        }
                    }
                }
            }
        };
    };
    if !output.success {
        ::core::panicking::panic("assertion failed: output.success")
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_229Q_spec_example_2_4_sequence_of_mappings"]
#[doc(hidden)]
pub const _229Q_spec_example_2_4_sequence_of_mappings: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_229Q_spec_example_2_4_sequence_of_mappings"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_229Q_spec_example_2_4_sequence_of_mappings()),
    ),
};
#[allow(non_snake_case)]
fn _229Q_spec_example_2_4_sequence_of_mappings() {
    test("229Q");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_26DV_whitespace_around_colon_in_mappings"]
#[doc(hidden)]
pub const _26DV_whitespace_around_colon_in_mappings: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_26DV_whitespace_around_colon_in_mappings"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_26DV_whitespace_around_colon_in_mappings()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _26DV_whitespace_around_colon_in_mappings() {
    test("26DV");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_27NA_spec_example_5_9_directive_indicator"]
#[doc(hidden)]
pub const _27NA_spec_example_5_9_directive_indicator: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_27NA_spec_example_5_9_directive_indicator"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_27NA_spec_example_5_9_directive_indicator()),
    ),
};
#[allow(non_snake_case)]
fn _27NA_spec_example_5_9_directive_indicator() {
    test("27NA");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_2AUY_tags_in_block_sequence"]
#[doc(hidden)]
pub const _2AUY_tags_in_block_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_2AUY_tags_in_block_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_2AUY_tags_in_block_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _2AUY_tags_in_block_sequence() {
    test("2AUY");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_2EBW_allowed_characters_in_keys"]
#[doc(hidden)]
pub const _2EBW_allowed_characters_in_keys: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_2EBW_allowed_characters_in_keys"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_2EBW_allowed_characters_in_keys()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _2EBW_allowed_characters_in_keys() {
    test("2EBW");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_2JQS_block_mapping_with_missing_keys"]
#[doc(hidden)]
pub const _2JQS_block_mapping_with_missing_keys: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_2JQS_block_mapping_with_missing_keys"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_2JQS_block_mapping_with_missing_keys()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _2JQS_block_mapping_with_missing_keys() {
    test("2JQS");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_2LFX_spec_example_6_13_reserved_directives_1_3"]
#[doc(hidden)]
pub const _2LFX_spec_example_6_13_reserved_directives_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_2LFX_spec_example_6_13_reserved_directives_1_3"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_2LFX_spec_example_6_13_reserved_directives_1_3()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _2LFX_spec_example_6_13_reserved_directives_1_3() {
    test("2LFX");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_2SXE_anchors_with_colon_in_name"]
#[doc(hidden)]
pub const _2SXE_anchors_with_colon_in_name: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_2SXE_anchors_with_colon_in_name"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_2SXE_anchors_with_colon_in_name()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _2SXE_anchors_with_colon_in_name() {
    test("2SXE");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_2XXW_spec_example_2_25_unordered_sets"]
#[doc(hidden)]
pub const _2XXW_spec_example_2_25_unordered_sets: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_2XXW_spec_example_2_25_unordered_sets"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_2XXW_spec_example_2_25_unordered_sets()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _2XXW_spec_example_2_25_unordered_sets() {
    test("2XXW");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_33X3_three_explicit_integers_in_a_block_sequence"]
#[doc(hidden)]
pub const _33X3_three_explicit_integers_in_a_block_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_33X3_three_explicit_integers_in_a_block_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_33X3_three_explicit_integers_in_a_block_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _33X3_three_explicit_integers_in_a_block_sequence() {
    test("33X3");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_35KP_tags_for_root_objects"]
#[doc(hidden)]
pub const _35KP_tags_for_root_objects: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_35KP_tags_for_root_objects"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_35KP_tags_for_root_objects()),
    ),
};
#[allow(non_snake_case)]
fn _35KP_tags_for_root_objects() {
    test("35KP");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_36F6_multiline_plain_scalar_with_empty_line"]
#[doc(hidden)]
pub const _36F6_multiline_plain_scalar_with_empty_line: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_36F6_multiline_plain_scalar_with_empty_line"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_36F6_multiline_plain_scalar_with_empty_line()),
    ),
};
#[allow(non_snake_case)]
fn _36F6_multiline_plain_scalar_with_empty_line() {
    test("36F6");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_3ALJ_block_sequence_in_block_sequence"]
#[doc(hidden)]
pub const _3ALJ_block_sequence_in_block_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_3ALJ_block_sequence_in_block_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_3ALJ_block_sequence_in_block_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _3ALJ_block_sequence_in_block_sequence() {
    test("3ALJ");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_3GZX_spec_example_7_1_alias_nodes"]
#[doc(hidden)]
pub const _3GZX_spec_example_7_1_alias_nodes: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_3GZX_spec_example_7_1_alias_nodes"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_3GZX_spec_example_7_1_alias_nodes()),
    ),
};
#[allow(non_snake_case)]
fn _3GZX_spec_example_7_1_alias_nodes() {
    test("3GZX");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_3MYT_plain_scalar_looking_like_key_comment_anchor_and_tag"]
#[doc(hidden)]
pub const _3MYT_plain_scalar_looking_like_key_comment_anchor_and_tag: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_3MYT_plain_scalar_looking_like_key_comment_anchor_and_tag",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _3MYT_plain_scalar_looking_like_key_comment_anchor_and_tag(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _3MYT_plain_scalar_looking_like_key_comment_anchor_and_tag() {
    test("3MYT");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_3R3P_single_block_sequence_with_anchor"]
#[doc(hidden)]
pub const _3R3P_single_block_sequence_with_anchor: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_3R3P_single_block_sequence_with_anchor"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_3R3P_single_block_sequence_with_anchor()),
    ),
};
#[allow(non_snake_case)]
fn _3R3P_single_block_sequence_with_anchor() {
    test("3R3P");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_3UYS_escaped_slash_in_double_quotes"]
#[doc(hidden)]
pub const _3UYS_escaped_slash_in_double_quotes: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_3UYS_escaped_slash_in_double_quotes"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_3UYS_escaped_slash_in_double_quotes()),
    ),
};
#[allow(non_snake_case)]
fn _3UYS_escaped_slash_in_double_quotes() {
    test("3UYS");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_4ABK_spec_example_7_17_flow_mapping_separate_values"]
#[doc(hidden)]
pub const _4ABK_spec_example_7_17_flow_mapping_separate_values: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_4ABK_spec_example_7_17_flow_mapping_separate_values",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _4ABK_spec_example_7_17_flow_mapping_separate_values(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _4ABK_spec_example_7_17_flow_mapping_separate_values() {
    test("4ABK");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_4CQQ_spec_example_2_18_multi_line_flow_scalars"]
#[doc(hidden)]
pub const _4CQQ_spec_example_2_18_multi_line_flow_scalars: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_4CQQ_spec_example_2_18_multi_line_flow_scalars"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_4CQQ_spec_example_2_18_multi_line_flow_scalars()),
    ),
};
#[allow(non_snake_case)]
fn _4CQQ_spec_example_2_18_multi_line_flow_scalars() {
    test("4CQQ");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_4FJ6_nested_implicit_complex_keys"]
#[doc(hidden)]
pub const _4FJ6_nested_implicit_complex_keys: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_4FJ6_nested_implicit_complex_keys"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_4FJ6_nested_implicit_complex_keys()),
    ),
};
#[allow(non_snake_case)]
fn _4FJ6_nested_implicit_complex_keys() {
    test("4FJ6");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_4GC6_spec_example_7_7_single_quoted_characters"]
#[doc(hidden)]
pub const _4GC6_spec_example_7_7_single_quoted_characters: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_4GC6_spec_example_7_7_single_quoted_characters"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_4GC6_spec_example_7_7_single_quoted_characters()),
    ),
};
#[allow(non_snake_case)]
fn _4GC6_spec_example_7_7_single_quoted_characters() {
    test("4GC6");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_4MUZ_flow_mapping_colon_on_line_after_key"]
#[doc(hidden)]
pub const _4MUZ_flow_mapping_colon_on_line_after_key: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_4MUZ_flow_mapping_colon_on_line_after_key"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_4MUZ_flow_mapping_colon_on_line_after_key()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _4MUZ_flow_mapping_colon_on_line_after_key() {
    test("4MUZ");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_4Q9F_folded_block_scalar_1_3"]
#[doc(hidden)]
pub const _4Q9F_folded_block_scalar_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_4Q9F_folded_block_scalar_1_3"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_4Q9F_folded_block_scalar_1_3()),
    ),
};
#[allow(non_snake_case)]
fn _4Q9F_folded_block_scalar_1_3() {
    test("4Q9F");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_4QFQ_spec_example_8_2_block_indentation_indicator_1_3"]
#[doc(hidden)]
pub const _4QFQ_spec_example_8_2_block_indentation_indicator_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_4QFQ_spec_example_8_2_block_indentation_indicator_1_3",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _4QFQ_spec_example_8_2_block_indentation_indicator_1_3(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _4QFQ_spec_example_8_2_block_indentation_indicator_1_3() {
    test("4QFQ");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_4UYU_colon_in_double_quoted_string"]
#[doc(hidden)]
pub const _4UYU_colon_in_double_quoted_string: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_4UYU_colon_in_double_quoted_string"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_4UYU_colon_in_double_quoted_string()),
    ),
};
#[allow(non_snake_case)]
fn _4UYU_colon_in_double_quoted_string() {
    test("4UYU");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_4V8U_plain_scalar_with_backslashes"]
#[doc(hidden)]
pub const _4V8U_plain_scalar_with_backslashes: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_4V8U_plain_scalar_with_backslashes"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_4V8U_plain_scalar_with_backslashes()),
    ),
};
#[allow(non_snake_case)]
fn _4V8U_plain_scalar_with_backslashes() {
    test("4V8U");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_4ZYM_spec_example_6_4_line_prefixes"]
#[doc(hidden)]
pub const _4ZYM_spec_example_6_4_line_prefixes: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_4ZYM_spec_example_6_4_line_prefixes"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_4ZYM_spec_example_6_4_line_prefixes()),
    ),
};
#[allow(non_snake_case)]
fn _4ZYM_spec_example_6_4_line_prefixes() {
    test("4ZYM");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_52DL_explicit_non_specific_tag_1_3"]
#[doc(hidden)]
pub const _52DL_explicit_non_specific_tag_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_52DL_explicit_non_specific_tag_1_3"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_52DL_explicit_non_specific_tag_1_3()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _52DL_explicit_non_specific_tag_1_3() {
    test("52DL");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_54T7_flow_mapping"]
#[doc(hidden)]
pub const _54T7_flow_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_54T7_flow_mapping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_54T7_flow_mapping()),
    ),
};
#[allow(non_snake_case)]
fn _54T7_flow_mapping() {
    test("54T7");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_565N_construct_binary"]
#[doc(hidden)]
pub const _565N_construct_binary: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_565N_construct_binary"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_565N_construct_binary()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _565N_construct_binary() {
    test("565N");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_57H4_spec_example_8_22_block_collection_nodes"]
#[doc(hidden)]
pub const _57H4_spec_example_8_22_block_collection_nodes: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_57H4_spec_example_8_22_block_collection_nodes"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_57H4_spec_example_8_22_block_collection_nodes()),
    ),
};
#[allow(non_snake_case)]
fn _57H4_spec_example_8_22_block_collection_nodes() {
    test("57H4");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_5BVJ_spec_example_5_7_block_scalar_indicators"]
#[doc(hidden)]
pub const _5BVJ_spec_example_5_7_block_scalar_indicators: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_5BVJ_spec_example_5_7_block_scalar_indicators"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_5BVJ_spec_example_5_7_block_scalar_indicators()),
    ),
};
#[allow(non_snake_case)]
fn _5BVJ_spec_example_5_7_block_scalar_indicators() {
    test("5BVJ");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_5C5M_spec_example_7_15_flow_mappings"]
#[doc(hidden)]
pub const _5C5M_spec_example_7_15_flow_mappings: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_5C5M_spec_example_7_15_flow_mappings"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_5C5M_spec_example_7_15_flow_mappings()),
    ),
};
#[allow(non_snake_case)]
fn _5C5M_spec_example_7_15_flow_mappings() {
    test("5C5M");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_5GBF_spec_example_6_5_empty_lines"]
#[doc(hidden)]
pub const _5GBF_spec_example_6_5_empty_lines: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_5GBF_spec_example_6_5_empty_lines"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_5GBF_spec_example_6_5_empty_lines()),
    ),
};
#[allow(non_snake_case)]
fn _5GBF_spec_example_6_5_empty_lines() {
    test("5GBF");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_5KJE_spec_example_7_13_flow_sequence"]
#[doc(hidden)]
pub const _5KJE_spec_example_7_13_flow_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_5KJE_spec_example_7_13_flow_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_5KJE_spec_example_7_13_flow_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _5KJE_spec_example_7_13_flow_sequence() {
    test("5KJE");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_5MUD_colon_and_adjacent_value_on_next_line"]
#[doc(hidden)]
pub const _5MUD_colon_and_adjacent_value_on_next_line: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_5MUD_colon_and_adjacent_value_on_next_line"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_5MUD_colon_and_adjacent_value_on_next_line()),
    ),
};
#[allow(non_snake_case)]
fn _5MUD_colon_and_adjacent_value_on_next_line() {
    test("5MUD");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_5NYZ_spec_example_6_9_separated_comment"]
#[doc(hidden)]
pub const _5NYZ_spec_example_6_9_separated_comment: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_5NYZ_spec_example_6_9_separated_comment"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_5NYZ_spec_example_6_9_separated_comment()),
    ),
};
#[allow(non_snake_case)]
fn _5NYZ_spec_example_6_9_separated_comment() {
    test("5NYZ");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_5TYM_spec_example_6_21_local_tag_prefix"]
#[doc(hidden)]
pub const _5TYM_spec_example_6_21_local_tag_prefix: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_5TYM_spec_example_6_21_local_tag_prefix"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_5TYM_spec_example_6_21_local_tag_prefix()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _5TYM_spec_example_6_21_local_tag_prefix() {
    test("5TYM");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_5WE3_spec_example_8_17_explicit_block_mapping_entries"]
#[doc(hidden)]
pub const _5WE3_spec_example_8_17_explicit_block_mapping_entries: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_5WE3_spec_example_8_17_explicit_block_mapping_entries",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _5WE3_spec_example_8_17_explicit_block_mapping_entries(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _5WE3_spec_example_8_17_explicit_block_mapping_entries() {
    test("5WE3");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_65WH_single_entry_block_sequence"]
#[doc(hidden)]
pub const _65WH_single_entry_block_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_65WH_single_entry_block_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_65WH_single_entry_block_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _65WH_single_entry_block_sequence() {
    test("65WH");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6BCT_spec_example_6_3_separation_spaces"]
#[doc(hidden)]
pub const _6BCT_spec_example_6_3_separation_spaces: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6BCT_spec_example_6_3_separation_spaces"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6BCT_spec_example_6_3_separation_spaces()),
    ),
};
#[allow(non_snake_case)]
fn _6BCT_spec_example_6_3_separation_spaces() {
    test("6BCT");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6BFJ_mapping_key_and_flow_sequence_item_anchors"]
#[doc(hidden)]
pub const _6BFJ_mapping_key_and_flow_sequence_item_anchors: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6BFJ_mapping_key_and_flow_sequence_item_anchors"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6BFJ_mapping_key_and_flow_sequence_item_anchors()),
    ),
};
#[allow(non_snake_case)]
fn _6BFJ_mapping_key_and_flow_sequence_item_anchors() {
    test("6BFJ");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6CK3_spec_example_6_26_tag_shorthands"]
#[doc(hidden)]
pub const _6CK3_spec_example_6_26_tag_shorthands: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6CK3_spec_example_6_26_tag_shorthands"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6CK3_spec_example_6_26_tag_shorthands()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _6CK3_spec_example_6_26_tag_shorthands() {
    test("6CK3");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6FWR_block_scalar_keep"]
#[doc(hidden)]
pub const _6FWR_block_scalar_keep: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6FWR_block_scalar_keep"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6FWR_block_scalar_keep()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _6FWR_block_scalar_keep() {
    test("6FWR");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6H3V_backslashes_in_singlequotes"]
#[doc(hidden)]
pub const _6H3V_backslashes_in_singlequotes: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6H3V_backslashes_in_singlequotes"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6H3V_backslashes_in_singlequotes()),
    ),
};
#[allow(non_snake_case)]
fn _6H3V_backslashes_in_singlequotes() {
    test("6H3V");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6HB6_spec_example_6_1_indentation_spaces"]
#[doc(hidden)]
pub const _6HB6_spec_example_6_1_indentation_spaces: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6HB6_spec_example_6_1_indentation_spaces"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6HB6_spec_example_6_1_indentation_spaces()),
    ),
};
#[allow(non_snake_case)]
fn _6HB6_spec_example_6_1_indentation_spaces() {
    test("6HB6");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6JQW_spec_example_2_13_in_literals_newlines_are_preserved"]
#[doc(hidden)]
pub const _6JQW_spec_example_2_13_in_literals_newlines_are_preserved: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_6JQW_spec_example_2_13_in_literals_newlines_are_preserved",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _6JQW_spec_example_2_13_in_literals_newlines_are_preserved(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _6JQW_spec_example_2_13_in_literals_newlines_are_preserved() {
    test("6JQW");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6JWB_tags_for_block_objects"]
#[doc(hidden)]
pub const _6JWB_tags_for_block_objects: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6JWB_tags_for_block_objects"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6JWB_tags_for_block_objects()),
    ),
};
#[allow(non_snake_case)]
fn _6JWB_tags_for_block_objects() {
    test("6JWB");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6KGN_anchor_for_empty_node"]
#[doc(hidden)]
pub const _6KGN_anchor_for_empty_node: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6KGN_anchor_for_empty_node"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6KGN_anchor_for_empty_node()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _6KGN_anchor_for_empty_node() {
    test("6KGN");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6LVF_spec_example_6_13_reserved_directives"]
#[doc(hidden)]
pub const _6LVF_spec_example_6_13_reserved_directives: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6LVF_spec_example_6_13_reserved_directives"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6LVF_spec_example_6_13_reserved_directives()),
    ),
};
#[allow(non_snake_case)]
fn _6LVF_spec_example_6_13_reserved_directives() {
    test("6LVF");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6M2F_aliases_in_explicit_block_mapping"]
#[doc(hidden)]
pub const _6M2F_aliases_in_explicit_block_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6M2F_aliases_in_explicit_block_mapping"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6M2F_aliases_in_explicit_block_mapping()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _6M2F_aliases_in_explicit_block_mapping() {
    test("6M2F");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6PBE_zero_indented_sequences_in_explicit_mapping_keys"]
#[doc(hidden)]
pub const _6PBE_zero_indented_sequences_in_explicit_mapping_keys: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_6PBE_zero_indented_sequences_in_explicit_mapping_keys",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _6PBE_zero_indented_sequences_in_explicit_mapping_keys(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _6PBE_zero_indented_sequences_in_explicit_mapping_keys() {
    test("6PBE");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6SLA_allowed_characters_in_quoted_mapping_key"]
#[doc(hidden)]
pub const _6SLA_allowed_characters_in_quoted_mapping_key: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6SLA_allowed_characters_in_quoted_mapping_key"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6SLA_allowed_characters_in_quoted_mapping_key()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _6SLA_allowed_characters_in_quoted_mapping_key() {
    test("6SLA");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6VJK_spec_example_2_15_folded_newlines_are_preserved_for_more_indented_and_blank_lines"]
#[doc(hidden)]
pub const _6VJK_spec_example_2_15_folded_newlines_are_preserved_for_more_indented_and_blank_lines: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_6VJK_spec_example_2_15_folded_newlines_are_preserved_for_more_indented_and_blank_lines",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _6VJK_spec_example_2_15_folded_newlines_are_preserved_for_more_indented_and_blank_lines(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _6VJK_spec_example_2_15_folded_newlines_are_preserved_for_more_indented_and_blank_lines() {
    test("6VJK");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6WLZ_spec_example_6_18_primary_tag_handle_1_3"]
#[doc(hidden)]
pub const _6WLZ_spec_example_6_18_primary_tag_handle_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6WLZ_spec_example_6_18_primary_tag_handle_1_3"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6WLZ_spec_example_6_18_primary_tag_handle_1_3()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _6WLZ_spec_example_6_18_primary_tag_handle_1_3() {
    test("6WLZ");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6WPF_spec_example_6_8_flow_folding_1_3"]
#[doc(hidden)]
pub const _6WPF_spec_example_6_8_flow_folding_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6WPF_spec_example_6_8_flow_folding_1_3"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6WPF_spec_example_6_8_flow_folding_1_3()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _6WPF_spec_example_6_8_flow_folding_1_3() {
    test("6WPF");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6XDY_two_document_start_markers"]
#[doc(hidden)]
pub const _6XDY_two_document_start_markers: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6XDY_two_document_start_markers"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6XDY_two_document_start_markers()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _6XDY_two_document_start_markers() {
    test("6XDY");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_6ZKB_spec_example_9_6_stream"]
#[doc(hidden)]
pub const _6ZKB_spec_example_9_6_stream: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_6ZKB_spec_example_9_6_stream"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_6ZKB_spec_example_9_6_stream()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _6ZKB_spec_example_9_6_stream() {
    test("6ZKB");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_735Y_spec_example_8_20_block_node_types"]
#[doc(hidden)]
pub const _735Y_spec_example_8_20_block_node_types: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_735Y_spec_example_8_20_block_node_types"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_735Y_spec_example_8_20_block_node_types()),
    ),
};
#[allow(non_snake_case)]
fn _735Y_spec_example_8_20_block_node_types() {
    test("735Y");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_74H7_tags_in_implicit_mapping"]
#[doc(hidden)]
pub const _74H7_tags_in_implicit_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_74H7_tags_in_implicit_mapping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_74H7_tags_in_implicit_mapping()),
    ),
};
#[allow(non_snake_case)]
fn _74H7_tags_in_implicit_mapping() {
    test("74H7");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_753E_block_scalar_strip_1_3"]
#[doc(hidden)]
pub const _753E_block_scalar_strip_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_753E_block_scalar_strip_1_3"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_753E_block_scalar_strip_1_3()),
    ),
};
#[allow(non_snake_case)]
fn _753E_block_scalar_strip_1_3() {
    test("753E");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_77H8_spec_example_2_23_various_explicit_tags"]
#[doc(hidden)]
pub const _77H8_spec_example_2_23_various_explicit_tags: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_77H8_spec_example_2_23_various_explicit_tags"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_77H8_spec_example_2_23_various_explicit_tags()),
    ),
};
#[allow(non_snake_case)]
fn _77H8_spec_example_2_23_various_explicit_tags() {
    test("77H8");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_7A4E_spec_example_7_6_double_quoted_lines"]
#[doc(hidden)]
pub const _7A4E_spec_example_7_6_double_quoted_lines: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_7A4E_spec_example_7_6_double_quoted_lines"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_7A4E_spec_example_7_6_double_quoted_lines()),
    ),
};
#[allow(non_snake_case)]
fn _7A4E_spec_example_7_6_double_quoted_lines() {
    test("7A4E");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_7BMT_node_and_mapping_key_anchors_1_3"]
#[doc(hidden)]
pub const _7BMT_node_and_mapping_key_anchors_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_7BMT_node_and_mapping_key_anchors_1_3"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_7BMT_node_and_mapping_key_anchors_1_3()),
    ),
};
#[allow(non_snake_case)]
fn _7BMT_node_and_mapping_key_anchors_1_3() {
    test("7BMT");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_7BUB_spec_example_2_10_node_for_sammy_sosa_appears_twice_in_this_document"]
#[doc(hidden)]
pub const _7BUB_spec_example_2_10_node_for_sammy_sosa_appears_twice_in_this_document: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_7BUB_spec_example_2_10_node_for_sammy_sosa_appears_twice_in_this_document",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _7BUB_spec_example_2_10_node_for_sammy_sosa_appears_twice_in_this_document(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _7BUB_spec_example_2_10_node_for_sammy_sosa_appears_twice_in_this_document() {
    test("7BUB");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_7FWL_spec_example_6_24_verbatim_tags"]
#[doc(hidden)]
pub const _7FWL_spec_example_6_24_verbatim_tags: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_7FWL_spec_example_6_24_verbatim_tags"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_7FWL_spec_example_6_24_verbatim_tags()),
    ),
};
#[allow(non_snake_case)]
fn _7FWL_spec_example_6_24_verbatim_tags() {
    test("7FWL");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_7T8X_spec_example_8_10_folded_lines_8_13_final_empty_lines"]
#[doc(hidden)]
pub const _7T8X_spec_example_8_10_folded_lines_8_13_final_empty_lines: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_7T8X_spec_example_8_10_folded_lines_8_13_final_empty_lines",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _7T8X_spec_example_8_10_folded_lines_8_13_final_empty_lines(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _7T8X_spec_example_8_10_folded_lines_8_13_final_empty_lines() {
    test("7T8X");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_7TMG_comment_in_flow_sequence_before_comma"]
#[doc(hidden)]
pub const _7TMG_comment_in_flow_sequence_before_comma: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_7TMG_comment_in_flow_sequence_before_comma"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_7TMG_comment_in_flow_sequence_before_comma()),
    ),
};
#[allow(non_snake_case)]
fn _7TMG_comment_in_flow_sequence_before_comma() {
    test("7TMG");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_7W2P_block_mapping_with_missing_values"]
#[doc(hidden)]
pub const _7W2P_block_mapping_with_missing_values: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_7W2P_block_mapping_with_missing_values"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_7W2P_block_mapping_with_missing_values()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _7W2P_block_mapping_with_missing_values() {
    test("7W2P");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_7Z25_bare_document_after_document_end_marker"]
#[doc(hidden)]
pub const _7Z25_bare_document_after_document_end_marker: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_7Z25_bare_document_after_document_end_marker"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_7Z25_bare_document_after_document_end_marker()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _7Z25_bare_document_after_document_end_marker() {
    test("7Z25");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_7ZZ5_empty_flow_collections"]
#[doc(hidden)]
pub const _7ZZ5_empty_flow_collections: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_7ZZ5_empty_flow_collections"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_7ZZ5_empty_flow_collections()),
    ),
};
#[allow(non_snake_case)]
fn _7ZZ5_empty_flow_collections() {
    test("7ZZ5");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_82AN_three_dashes_and_content_without_space"]
#[doc(hidden)]
pub const _82AN_three_dashes_and_content_without_space: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_82AN_three_dashes_and_content_without_space"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_82AN_three_dashes_and_content_without_space()),
    ),
};
#[allow(non_snake_case)]
fn _82AN_three_dashes_and_content_without_space() {
    test("82AN");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_87E4_spec_example_7_8_single_quoted_implicit_keys"]
#[doc(hidden)]
pub const _87E4_spec_example_7_8_single_quoted_implicit_keys: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_87E4_spec_example_7_8_single_quoted_implicit_keys"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_87E4_spec_example_7_8_single_quoted_implicit_keys()),
    ),
};
#[allow(non_snake_case)]
fn _87E4_spec_example_7_8_single_quoted_implicit_keys() {
    test("87E4");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_8CWC_plain_mapping_key_ending_with_colon"]
#[doc(hidden)]
pub const _8CWC_plain_mapping_key_ending_with_colon: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_8CWC_plain_mapping_key_ending_with_colon"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_8CWC_plain_mapping_key_ending_with_colon()),
    ),
};
#[allow(non_snake_case)]
fn _8CWC_plain_mapping_key_ending_with_colon() {
    test("8CWC");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_8G76_spec_example_6_10_comment_lines"]
#[doc(hidden)]
pub const _8G76_spec_example_6_10_comment_lines: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_8G76_spec_example_6_10_comment_lines"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_8G76_spec_example_6_10_comment_lines()),
    ),
};
#[allow(non_snake_case)]
fn _8G76_spec_example_6_10_comment_lines() {
    test("8G76");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_8KB6_multiline_plain_flow_mapping_key_without_value"]
#[doc(hidden)]
pub const _8KB6_multiline_plain_flow_mapping_key_without_value: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_8KB6_multiline_plain_flow_mapping_key_without_value",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _8KB6_multiline_plain_flow_mapping_key_without_value(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _8KB6_multiline_plain_flow_mapping_key_without_value() {
    test("8KB6");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_8MK2_explicit_non_specific_tag"]
#[doc(hidden)]
pub const _8MK2_explicit_non_specific_tag: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_8MK2_explicit_non_specific_tag"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_8MK2_explicit_non_specific_tag()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _8MK2_explicit_non_specific_tag() {
    test("8MK2");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_8QBE_block_sequence_in_block_mapping"]
#[doc(hidden)]
pub const _8QBE_block_sequence_in_block_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_8QBE_block_sequence_in_block_mapping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_8QBE_block_sequence_in_block_mapping()),
    ),
};
#[allow(non_snake_case)]
fn _8QBE_block_sequence_in_block_mapping() {
    test("8QBE");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_8UDB_spec_example_7_14_flow_sequence_entries"]
#[doc(hidden)]
pub const _8UDB_spec_example_7_14_flow_sequence_entries: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_8UDB_spec_example_7_14_flow_sequence_entries"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_8UDB_spec_example_7_14_flow_sequence_entries()),
    ),
};
#[allow(non_snake_case)]
fn _8UDB_spec_example_7_14_flow_sequence_entries() {
    test("8UDB");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_8XYN_anchor_with_unicode_character"]
#[doc(hidden)]
pub const _8XYN_anchor_with_unicode_character: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_8XYN_anchor_with_unicode_character"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_8XYN_anchor_with_unicode_character()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _8XYN_anchor_with_unicode_character() {
    test("8XYN");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_93JH_block_mappings_in_block_sequence"]
#[doc(hidden)]
pub const _93JH_block_mappings_in_block_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_93JH_block_mappings_in_block_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_93JH_block_mappings_in_block_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _93JH_block_mappings_in_block_sequence() {
    test("93JH");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_93WF_spec_example_6_6_line_folding_1_3"]
#[doc(hidden)]
pub const _93WF_spec_example_6_6_line_folding_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_93WF_spec_example_6_6_line_folding_1_3"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_93WF_spec_example_6_6_line_folding_1_3()),
    ),
};
#[allow(non_snake_case)]
fn _93WF_spec_example_6_6_line_folding_1_3() {
    test("93WF");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_96L6_spec_example_2_14_in_the_folded_scalars_newlines_become_spaces"]
#[doc(hidden)]
pub const _96L6_spec_example_2_14_in_the_folded_scalars_newlines_become_spaces: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_96L6_spec_example_2_14_in_the_folded_scalars_newlines_become_spaces",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _96L6_spec_example_2_14_in_the_folded_scalars_newlines_become_spaces(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _96L6_spec_example_2_14_in_the_folded_scalars_newlines_become_spaces() {
    test("96L6");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_98YD_spec_example_5_5_comment_indicator"]
#[doc(hidden)]
pub const _98YD_spec_example_5_5_comment_indicator: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_98YD_spec_example_5_5_comment_indicator"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_98YD_spec_example_5_5_comment_indicator()),
    ),
};
#[allow(non_snake_case)]
fn _98YD_spec_example_5_5_comment_indicator() {
    test("98YD");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_9BXH_multiline_doublequoted_flow_mapping_key_without_value"]
#[doc(hidden)]
pub const _9BXH_multiline_doublequoted_flow_mapping_key_without_value: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_9BXH_multiline_doublequoted_flow_mapping_key_without_value",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _9BXH_multiline_doublequoted_flow_mapping_key_without_value(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _9BXH_multiline_doublequoted_flow_mapping_key_without_value() {
    test("9BXH");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_9DXL_spec_example_9_6_stream_1_3"]
#[doc(hidden)]
pub const _9DXL_spec_example_9_6_stream_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_9DXL_spec_example_9_6_stream_1_3"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_9DXL_spec_example_9_6_stream_1_3()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _9DXL_spec_example_9_6_stream_1_3() {
    test("9DXL");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_9FMG_multi_level_mapping_indent"]
#[doc(hidden)]
pub const _9FMG_multi_level_mapping_indent: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_9FMG_multi_level_mapping_indent"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_9FMG_multi_level_mapping_indent()),
    ),
};
#[allow(non_snake_case)]
fn _9FMG_multi_level_mapping_indent() {
    test("9FMG");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_9J7A_simple_mapping_indent"]
#[doc(hidden)]
pub const _9J7A_simple_mapping_indent: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_9J7A_simple_mapping_indent"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_9J7A_simple_mapping_indent()),
    ),
};
#[allow(non_snake_case)]
fn _9J7A_simple_mapping_indent() {
    test("9J7A");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_9KAX_various_combinations_of_tags_and_anchors"]
#[doc(hidden)]
pub const _9KAX_various_combinations_of_tags_and_anchors: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_9KAX_various_combinations_of_tags_and_anchors"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_9KAX_various_combinations_of_tags_and_anchors()),
    ),
};
#[allow(non_snake_case)]
fn _9KAX_various_combinations_of_tags_and_anchors() {
    test("9KAX");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_9MMW_spec_example_7_21_single_pair_implicit_entries_1_3"]
#[doc(hidden)]
pub const _9MMW_spec_example_7_21_single_pair_implicit_entries_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_9MMW_spec_example_7_21_single_pair_implicit_entries_1_3",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _9MMW_spec_example_7_21_single_pair_implicit_entries_1_3(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _9MMW_spec_example_7_21_single_pair_implicit_entries_1_3() {
    test("9MMW");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_9SA2_multiline_double_quoted_flow_mapping_key"]
#[doc(hidden)]
pub const _9SA2_multiline_double_quoted_flow_mapping_key: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_9SA2_multiline_double_quoted_flow_mapping_key"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_9SA2_multiline_double_quoted_flow_mapping_key()),
    ),
};
#[allow(non_snake_case)]
fn _9SA2_multiline_double_quoted_flow_mapping_key() {
    test("9SA2");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_9SHH_spec_example_5_8_quoted_scalar_indicators"]
#[doc(hidden)]
pub const _9SHH_spec_example_5_8_quoted_scalar_indicators: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_9SHH_spec_example_5_8_quoted_scalar_indicators"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_9SHH_spec_example_5_8_quoted_scalar_indicators()),
    ),
};
#[allow(non_snake_case)]
fn _9SHH_spec_example_5_8_quoted_scalar_indicators() {
    test("9SHH");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_9TFX_spec_example_7_6_double_quoted_lines_1_3"]
#[doc(hidden)]
pub const _9TFX_spec_example_7_6_double_quoted_lines_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_9TFX_spec_example_7_6_double_quoted_lines_1_3"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_9TFX_spec_example_7_6_double_quoted_lines_1_3()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _9TFX_spec_example_7_6_double_quoted_lines_1_3() {
    test("9TFX");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_9U5K_spec_example_2_12_compact_nested_mapping"]
#[doc(hidden)]
pub const _9U5K_spec_example_2_12_compact_nested_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_9U5K_spec_example_2_12_compact_nested_mapping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_9U5K_spec_example_2_12_compact_nested_mapping()),
    ),
};
#[allow(non_snake_case)]
fn _9U5K_spec_example_2_12_compact_nested_mapping() {
    test("9U5K");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_9WXW_spec_example_6_18_primary_tag_handle"]
#[doc(hidden)]
pub const _9WXW_spec_example_6_18_primary_tag_handle: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_9WXW_spec_example_6_18_primary_tag_handle"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_9WXW_spec_example_6_18_primary_tag_handle()),
    ),
};
#[allow(non_snake_case)]
fn _9WXW_spec_example_6_18_primary_tag_handle() {
    test("9WXW");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_9YRD_multiline_scalar_at_top_level"]
#[doc(hidden)]
pub const _9YRD_multiline_scalar_at_top_level: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_9YRD_multiline_scalar_at_top_level"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_9YRD_multiline_scalar_at_top_level()),
    ),
};
#[allow(non_snake_case)]
fn _9YRD_multiline_scalar_at_top_level() {
    test("9YRD");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_A2M4_spec_example_6_2_indentation_indicators"]
#[doc(hidden)]
pub const _A2M4_spec_example_6_2_indentation_indicators: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_A2M4_spec_example_6_2_indentation_indicators"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_A2M4_spec_example_6_2_indentation_indicators()),
    ),
};
#[allow(non_snake_case)]
fn _A2M4_spec_example_6_2_indentation_indicators() {
    test("A2M4");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_A6F9_spec_example_8_4_chomping_final_line_break"]
#[doc(hidden)]
pub const _A6F9_spec_example_8_4_chomping_final_line_break: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_A6F9_spec_example_8_4_chomping_final_line_break"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_A6F9_spec_example_8_4_chomping_final_line_break()),
    ),
};
#[allow(non_snake_case)]
fn _A6F9_spec_example_8_4_chomping_final_line_break() {
    test("A6F9");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_A984_multiline_scalar_in_mapping"]
#[doc(hidden)]
pub const _A984_multiline_scalar_in_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_A984_multiline_scalar_in_mapping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_A984_multiline_scalar_in_mapping()),
    ),
};
#[allow(non_snake_case)]
fn _A984_multiline_scalar_in_mapping() {
    test("A984");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_AB8U_sequence_entry_that_looks_like_two_with_wrong_indentation"]
#[doc(hidden)]
pub const _AB8U_sequence_entry_that_looks_like_two_with_wrong_indentation: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_AB8U_sequence_entry_that_looks_like_two_with_wrong_indentation",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _AB8U_sequence_entry_that_looks_like_two_with_wrong_indentation(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _AB8U_sequence_entry_that_looks_like_two_with_wrong_indentation() {
    test("AB8U");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_AVM7_empty_stream"]
#[doc(hidden)]
pub const _AVM7_empty_stream: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_AVM7_empty_stream"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_AVM7_empty_stream()),
    ),
};
#[allow(non_snake_case)]
fn _AVM7_empty_stream() {
    test("AVM7");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_AZ63_sequence_with_same_indentation_as_parent_mapping"]
#[doc(hidden)]
pub const _AZ63_sequence_with_same_indentation_as_parent_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_AZ63_sequence_with_same_indentation_as_parent_mapping",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _AZ63_sequence_with_same_indentation_as_parent_mapping(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _AZ63_sequence_with_same_indentation_as_parent_mapping() {
    test("AZ63");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_AZW3_lookahead_test_cases"]
#[doc(hidden)]
pub const _AZW3_lookahead_test_cases: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_AZW3_lookahead_test_cases"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_AZW3_lookahead_test_cases()),
    ),
};
#[allow(non_snake_case)]
fn _AZW3_lookahead_test_cases() {
    test("AZW3");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_B3HG_spec_example_8_9_folded_scalar_1_3"]
#[doc(hidden)]
pub const _B3HG_spec_example_8_9_folded_scalar_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_B3HG_spec_example_8_9_folded_scalar_1_3"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_B3HG_spec_example_8_9_folded_scalar_1_3()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _B3HG_spec_example_8_9_folded_scalar_1_3() {
    test("B3HG");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_BEC7_spec_example_6_14_yaml_directive"]
#[doc(hidden)]
pub const _BEC7_spec_example_6_14_yaml_directive: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_BEC7_spec_example_6_14_yaml_directive"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_BEC7_spec_example_6_14_yaml_directive()),
    ),
};
#[allow(non_snake_case)]
fn _BEC7_spec_example_6_14_yaml_directive() {
    test("BEC7");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_BU8L_node_anchor_and_tag_on_seperate_lines"]
#[doc(hidden)]
pub const _BU8L_node_anchor_and_tag_on_seperate_lines: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_BU8L_node_anchor_and_tag_on_seperate_lines"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_BU8L_node_anchor_and_tag_on_seperate_lines()),
    ),
};
#[allow(non_snake_case)]
fn _BU8L_node_anchor_and_tag_on_seperate_lines() {
    test("BU8L");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_C2DT_spec_example_7_18_flow_mapping_adjacent_values"]
#[doc(hidden)]
pub const _C2DT_spec_example_7_18_flow_mapping_adjacent_values: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_C2DT_spec_example_7_18_flow_mapping_adjacent_values",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _C2DT_spec_example_7_18_flow_mapping_adjacent_values(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _C2DT_spec_example_7_18_flow_mapping_adjacent_values() {
    test("C2DT");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_C4HZ_spec_example_2_24_global_tags"]
#[doc(hidden)]
pub const _C4HZ_spec_example_2_24_global_tags: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_C4HZ_spec_example_2_24_global_tags"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_C4HZ_spec_example_2_24_global_tags()),
    ),
};
#[allow(non_snake_case)]
fn _C4HZ_spec_example_2_24_global_tags() {
    test("C4HZ");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_CC74_spec_example_6_20_tag_handles"]
#[doc(hidden)]
pub const _CC74_spec_example_6_20_tag_handles: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_CC74_spec_example_6_20_tag_handles"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_CC74_spec_example_6_20_tag_handles()),
    ),
};
#[allow(non_snake_case)]
fn _CC74_spec_example_6_20_tag_handles() {
    test("CC74");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_CN3R_various_location_of_anchors_in_flow_sequence"]
#[doc(hidden)]
pub const _CN3R_various_location_of_anchors_in_flow_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_CN3R_various_location_of_anchors_in_flow_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_CN3R_various_location_of_anchors_in_flow_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _CN3R_various_location_of_anchors_in_flow_sequence() {
    test("CN3R");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_CPZ3_doublequoted_scalar_starting_with_a_tab"]
#[doc(hidden)]
pub const _CPZ3_doublequoted_scalar_starting_with_a_tab: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_CPZ3_doublequoted_scalar_starting_with_a_tab"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_CPZ3_doublequoted_scalar_starting_with_a_tab()),
    ),
};
#[allow(non_snake_case)]
fn _CPZ3_doublequoted_scalar_starting_with_a_tab() {
    test("CPZ3");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_CT4Q_spec_example_7_20_single_pair_explicit_entry"]
#[doc(hidden)]
pub const _CT4Q_spec_example_7_20_single_pair_explicit_entry: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_CT4Q_spec_example_7_20_single_pair_explicit_entry"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_CT4Q_spec_example_7_20_single_pair_explicit_entry()),
    ),
};
#[allow(non_snake_case)]
fn _CT4Q_spec_example_7_20_single_pair_explicit_entry() {
    test("CT4Q");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_CUP7_spec_example_5_6_node_property_indicators"]
#[doc(hidden)]
pub const _CUP7_spec_example_5_6_node_property_indicators: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_CUP7_spec_example_5_6_node_property_indicators"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_CUP7_spec_example_5_6_node_property_indicators()),
    ),
};
#[allow(non_snake_case)]
fn _CUP7_spec_example_5_6_node_property_indicators() {
    test("CUP7");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_D83L_block_scalar_indicator_order"]
#[doc(hidden)]
pub const _D83L_block_scalar_indicator_order: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_D83L_block_scalar_indicator_order"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_D83L_block_scalar_indicator_order()),
    ),
};
#[allow(non_snake_case)]
fn _D83L_block_scalar_indicator_order() {
    test("D83L");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_D88J_flow_sequence_in_block_mapping"]
#[doc(hidden)]
pub const _D88J_flow_sequence_in_block_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_D88J_flow_sequence_in_block_mapping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_D88J_flow_sequence_in_block_mapping()),
    ),
};
#[allow(non_snake_case)]
fn _D88J_flow_sequence_in_block_mapping() {
    test("D88J");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_D9TU_single_pair_block_mapping"]
#[doc(hidden)]
pub const _D9TU_single_pair_block_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_D9TU_single_pair_block_mapping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_D9TU_single_pair_block_mapping()),
    ),
};
#[allow(non_snake_case)]
fn _D9TU_single_pair_block_mapping() {
    test("D9TU");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_DBG4_spec_example_7_10_plain_characters"]
#[doc(hidden)]
pub const _DBG4_spec_example_7_10_plain_characters: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_DBG4_spec_example_7_10_plain_characters"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_DBG4_spec_example_7_10_plain_characters()),
    ),
};
#[allow(non_snake_case)]
fn _DBG4_spec_example_7_10_plain_characters() {
    test("DBG4");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_DC7X_various_trailing_tabs"]
#[doc(hidden)]
pub const _DC7X_various_trailing_tabs: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_DC7X_various_trailing_tabs"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_DC7X_various_trailing_tabs()),
    ),
};
#[allow(non_snake_case)]
fn _DC7X_various_trailing_tabs() {
    test("DC7X");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_DFF7_spec_example_7_16_flow_mapping_entries"]
#[doc(hidden)]
pub const _DFF7_spec_example_7_16_flow_mapping_entries: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_DFF7_spec_example_7_16_flow_mapping_entries"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_DFF7_spec_example_7_16_flow_mapping_entries()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _DFF7_spec_example_7_16_flow_mapping_entries() {
    test("DFF7");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_DHP8_flow_sequence"]
#[doc(hidden)]
pub const _DHP8_flow_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_DHP8_flow_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_DHP8_flow_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _DHP8_flow_sequence() {
    test("DHP8");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_DK3J_zero_indented_block_scalar_with_line_that_looks_like_a_comment"]
#[doc(hidden)]
pub const _DK3J_zero_indented_block_scalar_with_line_that_looks_like_a_comment: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_DK3J_zero_indented_block_scalar_with_line_that_looks_like_a_comment",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _DK3J_zero_indented_block_scalar_with_line_that_looks_like_a_comment(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _DK3J_zero_indented_block_scalar_with_line_that_looks_like_a_comment() {
    test("DK3J");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_DWX9_spec_example_8_8_literal_content"]
#[doc(hidden)]
pub const _DWX9_spec_example_8_8_literal_content: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_DWX9_spec_example_8_8_literal_content"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_DWX9_spec_example_8_8_literal_content()),
    ),
};
#[allow(non_snake_case)]
fn _DWX9_spec_example_8_8_literal_content() {
    test("DWX9");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_E76Z_aliases_in_implicit_block_mapping"]
#[doc(hidden)]
pub const _E76Z_aliases_in_implicit_block_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_E76Z_aliases_in_implicit_block_mapping"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_E76Z_aliases_in_implicit_block_mapping()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _E76Z_aliases_in_implicit_block_mapping() {
    test("E76Z");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_EHF6_tags_for_flow_objects"]
#[doc(hidden)]
pub const _EHF6_tags_for_flow_objects: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_EHF6_tags_for_flow_objects"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_EHF6_tags_for_flow_objects()),
    ),
};
#[allow(non_snake_case)]
fn _EHF6_tags_for_flow_objects() {
    test("EHF6");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_EX5H_multiline_scalar_at_top_level_1_3"]
#[doc(hidden)]
pub const _EX5H_multiline_scalar_at_top_level_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_EX5H_multiline_scalar_at_top_level_1_3"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_EX5H_multiline_scalar_at_top_level_1_3()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _EX5H_multiline_scalar_at_top_level_1_3() {
    test("EX5H");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_EXG3_three_dashes_and_content_without_space_1_3"]
#[doc(hidden)]
pub const _EXG3_three_dashes_and_content_without_space_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_EXG3_three_dashes_and_content_without_space_1_3"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_EXG3_three_dashes_and_content_without_space_1_3()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _EXG3_three_dashes_and_content_without_space_1_3() {
    test("EXG3");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_F2C7_anchors_and_tags"]
#[doc(hidden)]
pub const _F2C7_anchors_and_tags: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_F2C7_anchors_and_tags"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_F2C7_anchors_and_tags()),
    ),
};
#[allow(non_snake_case)]
fn _F2C7_anchors_and_tags() {
    test("F2C7");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_F3CP_nested_flow_collections_on_one_line"]
#[doc(hidden)]
pub const _F3CP_nested_flow_collections_on_one_line: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_F3CP_nested_flow_collections_on_one_line"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_F3CP_nested_flow_collections_on_one_line()),
    ),
};
#[allow(non_snake_case)]
fn _F3CP_nested_flow_collections_on_one_line() {
    test("F3CP");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_F6MC_more_indented_lines_at_the_beginning_of_folded_block_scalars"]
#[doc(hidden)]
pub const _F6MC_more_indented_lines_at_the_beginning_of_folded_block_scalars: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_F6MC_more_indented_lines_at_the_beginning_of_folded_block_scalars",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _F6MC_more_indented_lines_at_the_beginning_of_folded_block_scalars(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _F6MC_more_indented_lines_at_the_beginning_of_folded_block_scalars() {
    test("F6MC");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_F8F9_spec_example_8_5_chomping_trailing_lines"]
#[doc(hidden)]
pub const _F8F9_spec_example_8_5_chomping_trailing_lines: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_F8F9_spec_example_8_5_chomping_trailing_lines"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_F8F9_spec_example_8_5_chomping_trailing_lines()),
    ),
};
#[allow(non_snake_case)]
fn _F8F9_spec_example_8_5_chomping_trailing_lines() {
    test("F8F9");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_FBC9_allowed_characters_in_plain_scalars"]
#[doc(hidden)]
pub const _FBC9_allowed_characters_in_plain_scalars: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_FBC9_allowed_characters_in_plain_scalars"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_FBC9_allowed_characters_in_plain_scalars()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _FBC9_allowed_characters_in_plain_scalars() {
    test("FBC9");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_FH7J_tags_on_empty_scalars"]
#[doc(hidden)]
pub const _FH7J_tags_on_empty_scalars: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_FH7J_tags_on_empty_scalars"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_FH7J_tags_on_empty_scalars()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _FH7J_tags_on_empty_scalars() {
    test("FH7J");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_FP8R_zero_indented_block_scalar"]
#[doc(hidden)]
pub const _FP8R_zero_indented_block_scalar: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_FP8R_zero_indented_block_scalar"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_FP8R_zero_indented_block_scalar()),
    ),
};
#[allow(non_snake_case)]
fn _FP8R_zero_indented_block_scalar() {
    test("FP8R");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_FQ7F_spec_example_2_1_sequence_of_scalars"]
#[doc(hidden)]
pub const _FQ7F_spec_example_2_1_sequence_of_scalars: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_FQ7F_spec_example_2_1_sequence_of_scalars"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_FQ7F_spec_example_2_1_sequence_of_scalars()),
    ),
};
#[allow(non_snake_case)]
fn _FQ7F_spec_example_2_1_sequence_of_scalars() {
    test("FQ7F");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_FRK4_spec_example_7_3_completely_empty_flow_nodes"]
#[doc(hidden)]
pub const _FRK4_spec_example_7_3_completely_empty_flow_nodes: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_FRK4_spec_example_7_3_completely_empty_flow_nodes"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_FRK4_spec_example_7_3_completely_empty_flow_nodes()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _FRK4_spec_example_7_3_completely_empty_flow_nodes() {
    test("FRK4");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_FTA2_single_block_sequence_with_anchor_and_explicit_document_start"]
#[doc(hidden)]
pub const _FTA2_single_block_sequence_with_anchor_and_explicit_document_start: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_FTA2_single_block_sequence_with_anchor_and_explicit_document_start",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _FTA2_single_block_sequence_with_anchor_and_explicit_document_start(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _FTA2_single_block_sequence_with_anchor_and_explicit_document_start() {
    test("FTA2");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_FUP4_flow_sequence_in_flow_sequence"]
#[doc(hidden)]
pub const _FUP4_flow_sequence_in_flow_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_FUP4_flow_sequence_in_flow_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_FUP4_flow_sequence_in_flow_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _FUP4_flow_sequence_in_flow_sequence() {
    test("FUP4");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_G4RS_spec_example_2_17_quoted_scalars"]
#[doc(hidden)]
pub const _G4RS_spec_example_2_17_quoted_scalars: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_G4RS_spec_example_2_17_quoted_scalars"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_G4RS_spec_example_2_17_quoted_scalars()),
    ),
};
#[allow(non_snake_case)]
fn _G4RS_spec_example_2_17_quoted_scalars() {
    test("G4RS");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_G5U8_plain_dashes_in_flow_sequence"]
#[doc(hidden)]
pub const _G5U8_plain_dashes_in_flow_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_G5U8_plain_dashes_in_flow_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_G5U8_plain_dashes_in_flow_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _G5U8_plain_dashes_in_flow_sequence() {
    test("G5U8");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_G992_spec_example_8_9_folded_scalar"]
#[doc(hidden)]
pub const _G992_spec_example_8_9_folded_scalar: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_G992_spec_example_8_9_folded_scalar"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_G992_spec_example_8_9_folded_scalar()),
    ),
};
#[allow(non_snake_case)]
fn _G992_spec_example_8_9_folded_scalar() {
    test("G992");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_GH63_mixed_block_mapping_explicit_to_implicit"]
#[doc(hidden)]
pub const _GH63_mixed_block_mapping_explicit_to_implicit: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_GH63_mixed_block_mapping_explicit_to_implicit"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_GH63_mixed_block_mapping_explicit_to_implicit()),
    ),
};
#[allow(non_snake_case)]
fn _GH63_mixed_block_mapping_explicit_to_implicit() {
    test("GH63");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_H2RW_blank_lines"]
#[doc(hidden)]
pub const _H2RW_blank_lines: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_H2RW_blank_lines"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_H2RW_blank_lines()),
    ),
};
#[allow(non_snake_case)]
fn _H2RW_blank_lines() {
    test("H2RW");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_H3Z8_literal_unicode"]
#[doc(hidden)]
pub const _H3Z8_literal_unicode: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_H3Z8_literal_unicode"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_H3Z8_literal_unicode()),
    ),
};
#[allow(non_snake_case)]
fn _H3Z8_literal_unicode() {
    test("H3Z8");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_HMK4_spec_example_2_16_indentation_determines_scope"]
#[doc(hidden)]
pub const _HMK4_spec_example_2_16_indentation_determines_scope: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_HMK4_spec_example_2_16_indentation_determines_scope",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _HMK4_spec_example_2_16_indentation_determines_scope(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _HMK4_spec_example_2_16_indentation_determines_scope() {
    test("HMK4");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_HMQ5_spec_example_6_23_node_properties"]
#[doc(hidden)]
pub const _HMQ5_spec_example_6_23_node_properties: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_HMQ5_spec_example_6_23_node_properties"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_HMQ5_spec_example_6_23_node_properties()),
    ),
};
#[allow(non_snake_case)]
fn _HMQ5_spec_example_6_23_node_properties() {
    test("HMQ5");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_HS5T_spec_example_7_12_plain_lines"]
#[doc(hidden)]
pub const _HS5T_spec_example_7_12_plain_lines: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_HS5T_spec_example_7_12_plain_lines"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_HS5T_spec_example_7_12_plain_lines()),
    ),
};
#[allow(non_snake_case)]
fn _HS5T_spec_example_7_12_plain_lines() {
    test("HS5T");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_HWV9_document_end_marker"]
#[doc(hidden)]
pub const _HWV9_document_end_marker: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_HWV9_document_end_marker"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_HWV9_document_end_marker()),
    ),
};
#[allow(non_snake_case)]
fn _HWV9_document_end_marker() {
    test("HWV9");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_J3BT_spec_example_5_12_tabs_and_spaces"]
#[doc(hidden)]
pub const _J3BT_spec_example_5_12_tabs_and_spaces: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_J3BT_spec_example_5_12_tabs_and_spaces"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_J3BT_spec_example_5_12_tabs_and_spaces()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _J3BT_spec_example_5_12_tabs_and_spaces() {
    test("J3BT");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_J5UC_multiple_pair_block_mapping"]
#[doc(hidden)]
pub const _J5UC_multiple_pair_block_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_J5UC_multiple_pair_block_mapping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_J5UC_multiple_pair_block_mapping()),
    ),
};
#[allow(non_snake_case)]
fn _J5UC_multiple_pair_block_mapping() {
    test("J5UC");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_J7PZ_spec_example_2_26_ordered_mappings"]
#[doc(hidden)]
pub const _J7PZ_spec_example_2_26_ordered_mappings: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_J7PZ_spec_example_2_26_ordered_mappings"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_J7PZ_spec_example_2_26_ordered_mappings()),
    ),
};
#[allow(non_snake_case)]
fn _J7PZ_spec_example_2_26_ordered_mappings() {
    test("J7PZ");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_J7VC_empty_lines_between_mapping_elements"]
#[doc(hidden)]
pub const _J7VC_empty_lines_between_mapping_elements: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_J7VC_empty_lines_between_mapping_elements"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_J7VC_empty_lines_between_mapping_elements()),
    ),
};
#[allow(non_snake_case)]
fn _J7VC_empty_lines_between_mapping_elements() {
    test("J7VC");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_J9HZ_spec_example_2_9_single_document_with_two_comments"]
#[doc(hidden)]
pub const _J9HZ_spec_example_2_9_single_document_with_two_comments: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_J9HZ_spec_example_2_9_single_document_with_two_comments",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _J9HZ_spec_example_2_9_single_document_with_two_comments(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _J9HZ_spec_example_2_9_single_document_with_two_comments() {
    test("J9HZ");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_JDH8_plain_scalar_looking_like_key_comment_anchor_and_tag_1_3"]
#[doc(hidden)]
pub const _JDH8_plain_scalar_looking_like_key_comment_anchor_and_tag_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_JDH8_plain_scalar_looking_like_key_comment_anchor_and_tag_1_3",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _JDH8_plain_scalar_looking_like_key_comment_anchor_and_tag_1_3(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _JDH8_plain_scalar_looking_like_key_comment_anchor_and_tag_1_3() {
    test("JDH8");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_JHB9_spec_example_2_7_two_documents_in_a_stream"]
#[doc(hidden)]
pub const _JHB9_spec_example_2_7_two_documents_in_a_stream: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_JHB9_spec_example_2_7_two_documents_in_a_stream"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_JHB9_spec_example_2_7_two_documents_in_a_stream()),
    ),
};
#[allow(non_snake_case)]
fn _JHB9_spec_example_2_7_two_documents_in_a_stream() {
    test("JHB9");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_JQ4R_spec_example_8_14_block_sequence"]
#[doc(hidden)]
pub const _JQ4R_spec_example_8_14_block_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_JQ4R_spec_example_8_14_block_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_JQ4R_spec_example_8_14_block_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _JQ4R_spec_example_8_14_block_sequence() {
    test("JQ4R");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_JS2J_spec_example_6_29_node_anchors"]
#[doc(hidden)]
pub const _JS2J_spec_example_6_29_node_anchors: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_JS2J_spec_example_6_29_node_anchors"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_JS2J_spec_example_6_29_node_anchors()),
    ),
};
#[allow(non_snake_case)]
fn _JS2J_spec_example_6_29_node_anchors() {
    test("JS2J");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_JTV5_block_mapping_with_multiline_scalars"]
#[doc(hidden)]
pub const _JTV5_block_mapping_with_multiline_scalars: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_JTV5_block_mapping_with_multiline_scalars"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_JTV5_block_mapping_with_multiline_scalars()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _JTV5_block_mapping_with_multiline_scalars() {
    test("JTV5");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_K3WX_colon_and_adjacent_value_after_comment_on_next_line"]
#[doc(hidden)]
pub const _K3WX_colon_and_adjacent_value_after_comment_on_next_line: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_K3WX_colon_and_adjacent_value_after_comment_on_next_line",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _K3WX_colon_and_adjacent_value_after_comment_on_next_line(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _K3WX_colon_and_adjacent_value_after_comment_on_next_line() {
    test("K3WX");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_K4SU_multiple_entry_block_sequence"]
#[doc(hidden)]
pub const _K4SU_multiple_entry_block_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_K4SU_multiple_entry_block_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_K4SU_multiple_entry_block_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _K4SU_multiple_entry_block_sequence() {
    test("K4SU");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_K527_spec_example_6_6_line_folding"]
#[doc(hidden)]
pub const _K527_spec_example_6_6_line_folding: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_K527_spec_example_6_6_line_folding"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_K527_spec_example_6_6_line_folding()),
    ),
};
#[allow(non_snake_case)]
fn _K527_spec_example_6_6_line_folding() {
    test("K527");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_K54U_tab_after_document_header"]
#[doc(hidden)]
pub const _K54U_tab_after_document_header: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_K54U_tab_after_document_header"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_K54U_tab_after_document_header()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _K54U_tab_after_document_header() {
    test("K54U");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_K858_spec_example_8_6_empty_scalar_chomping"]
#[doc(hidden)]
pub const _K858_spec_example_8_6_empty_scalar_chomping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_K858_spec_example_8_6_empty_scalar_chomping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_K858_spec_example_8_6_empty_scalar_chomping()),
    ),
};
#[allow(non_snake_case)]
fn _K858_spec_example_8_6_empty_scalar_chomping() {
    test("K858");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_KK5P_various_combinations_of_explicit_block_mappings"]
#[doc(hidden)]
pub const _KK5P_various_combinations_of_explicit_block_mappings: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_KK5P_various_combinations_of_explicit_block_mappings",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _KK5P_various_combinations_of_explicit_block_mappings(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _KK5P_various_combinations_of_explicit_block_mappings() {
    test("KK5P");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_KMK3_block_submapping"]
#[doc(hidden)]
pub const _KMK3_block_submapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_KMK3_block_submapping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_KMK3_block_submapping()),
    ),
};
#[allow(non_snake_case)]
fn _KMK3_block_submapping() {
    test("KMK3");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_KSS4_scalars_on_line"]
#[doc(hidden)]
pub const _KSS4_scalars_on_line: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_KSS4_scalars_on_line"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_KSS4_scalars_on_line()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _KSS4_scalars_on_line() {
    test("KSS4");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_KZN9_spec_example_7_21_single_pair_implicit_entries"]
#[doc(hidden)]
pub const _KZN9_spec_example_7_21_single_pair_implicit_entries: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_KZN9_spec_example_7_21_single_pair_implicit_entries",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _KZN9_spec_example_7_21_single_pair_implicit_entries(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _KZN9_spec_example_7_21_single_pair_implicit_entries() {
    test("KZN9");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_L94M_tags_in_explicit_mapping"]
#[doc(hidden)]
pub const _L94M_tags_in_explicit_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_L94M_tags_in_explicit_mapping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_L94M_tags_in_explicit_mapping()),
    ),
};
#[allow(non_snake_case)]
fn _L94M_tags_in_explicit_mapping() {
    test("L94M");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_L9U5_spec_example_7_11_plain_implicit_keys"]
#[doc(hidden)]
pub const _L9U5_spec_example_7_11_plain_implicit_keys: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_L9U5_spec_example_7_11_plain_implicit_keys"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_L9U5_spec_example_7_11_plain_implicit_keys()),
    ),
};
#[allow(non_snake_case)]
fn _L9U5_spec_example_7_11_plain_implicit_keys() {
    test("L9U5");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_LE5A_spec_example_7_24_flow_nodes"]
#[doc(hidden)]
pub const _LE5A_spec_example_7_24_flow_nodes: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_LE5A_spec_example_7_24_flow_nodes"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_LE5A_spec_example_7_24_flow_nodes()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _LE5A_spec_example_7_24_flow_nodes() {
    test("LE5A");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_LP6E_whitespace_after_scalars_in_flow"]
#[doc(hidden)]
pub const _LP6E_whitespace_after_scalars_in_flow: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_LP6E_whitespace_after_scalars_in_flow"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_LP6E_whitespace_after_scalars_in_flow()),
    ),
};
#[allow(non_snake_case)]
fn _LP6E_whitespace_after_scalars_in_flow() {
    test("LP6E");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_LQZ7_spec_example_7_4_double_quoted_implicit_keys"]
#[doc(hidden)]
pub const _LQZ7_spec_example_7_4_double_quoted_implicit_keys: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_LQZ7_spec_example_7_4_double_quoted_implicit_keys"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_LQZ7_spec_example_7_4_double_quoted_implicit_keys()),
    ),
};
#[allow(non_snake_case)]
fn _LQZ7_spec_example_7_4_double_quoted_implicit_keys() {
    test("LQZ7");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_LX3P_implicit_flow_mapping_key_on_one_line"]
#[doc(hidden)]
pub const _LX3P_implicit_flow_mapping_key_on_one_line: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_LX3P_implicit_flow_mapping_key_on_one_line"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_LX3P_implicit_flow_mapping_key_on_one_line()),
    ),
};
#[allow(non_snake_case)]
fn _LX3P_implicit_flow_mapping_key_on_one_line() {
    test("LX3P");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_M29M_literal_block_scalar"]
#[doc(hidden)]
pub const _M29M_literal_block_scalar: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_M29M_literal_block_scalar"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_M29M_literal_block_scalar()),
    ),
};
#[allow(non_snake_case)]
fn _M29M_literal_block_scalar() {
    test("M29M");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_M5C3_spec_example_8_21_block_scalar_nodes"]
#[doc(hidden)]
pub const _M5C3_spec_example_8_21_block_scalar_nodes: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_M5C3_spec_example_8_21_block_scalar_nodes"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_M5C3_spec_example_8_21_block_scalar_nodes()),
    ),
};
#[allow(non_snake_case)]
fn _M5C3_spec_example_8_21_block_scalar_nodes() {
    test("M5C3");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_M5DY_spec_example_2_11_mapping_between_sequences"]
#[doc(hidden)]
pub const _M5DY_spec_example_2_11_mapping_between_sequences: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_M5DY_spec_example_2_11_mapping_between_sequences"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_M5DY_spec_example_2_11_mapping_between_sequences()),
    ),
};
#[allow(non_snake_case)]
fn _M5DY_spec_example_2_11_mapping_between_sequences() {
    test("M5DY");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_M7A3_spec_example_9_3_bare_documents"]
#[doc(hidden)]
pub const _M7A3_spec_example_9_3_bare_documents: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_M7A3_spec_example_9_3_bare_documents"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_M7A3_spec_example_9_3_bare_documents()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _M7A3_spec_example_9_3_bare_documents() {
    test("M7A3");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_M7NX_nested_flow_collections"]
#[doc(hidden)]
pub const _M7NX_nested_flow_collections: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_M7NX_nested_flow_collections"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_M7NX_nested_flow_collections()),
    ),
};
#[allow(non_snake_case)]
fn _M7NX_nested_flow_collections() {
    test("M7NX");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_M9B4_spec_example_8_7_literal_scalar"]
#[doc(hidden)]
pub const _M9B4_spec_example_8_7_literal_scalar: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_M9B4_spec_example_8_7_literal_scalar"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_M9B4_spec_example_8_7_literal_scalar()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _M9B4_spec_example_8_7_literal_scalar() {
    test("M9B4");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_MJS9_spec_example_6_7_block_folding"]
#[doc(hidden)]
pub const _MJS9_spec_example_6_7_block_folding: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_MJS9_spec_example_6_7_block_folding"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_MJS9_spec_example_6_7_block_folding()),
    ),
};
#[allow(non_snake_case)]
fn _MJS9_spec_example_6_7_block_folding() {
    test("MJS9");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_MXS3_flow_mapping_in_block_sequence"]
#[doc(hidden)]
pub const _MXS3_flow_mapping_in_block_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_MXS3_flow_mapping_in_block_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_MXS3_flow_mapping_in_block_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _MXS3_flow_mapping_in_block_sequence() {
    test("MXS3");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_MYW6_block_scalar_strip"]
#[doc(hidden)]
pub const _MYW6_block_scalar_strip: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_MYW6_block_scalar_strip"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_MYW6_block_scalar_strip()),
    ),
};
#[allow(non_snake_case)]
fn _MYW6_block_scalar_strip() {
    test("MYW6");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_MZX3_non_specific_tags_on_scalars"]
#[doc(hidden)]
pub const _MZX3_non_specific_tags_on_scalars: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_MZX3_non_specific_tags_on_scalars"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_MZX3_non_specific_tags_on_scalars()),
    ),
};
#[allow(non_snake_case)]
fn _MZX3_non_specific_tags_on_scalars() {
    test("MZX3");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_NAT4_various_empty_or_newline_only_quoted_strings"]
#[doc(hidden)]
pub const _NAT4_various_empty_or_newline_only_quoted_strings: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_NAT4_various_empty_or_newline_only_quoted_strings"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_NAT4_various_empty_or_newline_only_quoted_strings()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _NAT4_various_empty_or_newline_only_quoted_strings() {
    test("NAT4");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_NB6Z_multiline_plain_value_with_tabs_on_empty_lines"]
#[doc(hidden)]
pub const _NB6Z_multiline_plain_value_with_tabs_on_empty_lines: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_NB6Z_multiline_plain_value_with_tabs_on_empty_lines",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _NB6Z_multiline_plain_value_with_tabs_on_empty_lines(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _NB6Z_multiline_plain_value_with_tabs_on_empty_lines() {
    test("NB6Z");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_NHX8_empty_lines_at_end_of_document"]
#[doc(hidden)]
pub const _NHX8_empty_lines_at_end_of_document: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_NHX8_empty_lines_at_end_of_document"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_NHX8_empty_lines_at_end_of_document()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _NHX8_empty_lines_at_end_of_document() {
    test("NHX8");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_NJ66_multiline_plain_flow_mapping_key"]
#[doc(hidden)]
pub const _NJ66_multiline_plain_flow_mapping_key: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_NJ66_multiline_plain_flow_mapping_key"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_NJ66_multiline_plain_flow_mapping_key()),
    ),
};
#[allow(non_snake_case)]
fn _NJ66_multiline_plain_flow_mapping_key() {
    test("NJ66");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_NP9H_spec_example_7_5_double_quoted_line_breaks"]
#[doc(hidden)]
pub const _NP9H_spec_example_7_5_double_quoted_line_breaks: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_NP9H_spec_example_7_5_double_quoted_line_breaks"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_NP9H_spec_example_7_5_double_quoted_line_breaks()),
    ),
};
#[allow(non_snake_case)]
fn _NP9H_spec_example_7_5_double_quoted_line_breaks() {
    test("NP9H");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_P2AD_spec_example_8_1_block_scalar_header"]
#[doc(hidden)]
pub const _P2AD_spec_example_8_1_block_scalar_header: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_P2AD_spec_example_8_1_block_scalar_header"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_P2AD_spec_example_8_1_block_scalar_header()),
    ),
};
#[allow(non_snake_case)]
fn _P2AD_spec_example_8_1_block_scalar_header() {
    test("P2AD");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_P76L_spec_example_6_19_secondary_tag_handle"]
#[doc(hidden)]
pub const _P76L_spec_example_6_19_secondary_tag_handle: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_P76L_spec_example_6_19_secondary_tag_handle"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_P76L_spec_example_6_19_secondary_tag_handle()),
    ),
};
#[allow(non_snake_case)]
fn _P76L_spec_example_6_19_secondary_tag_handle() {
    test("P76L");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_P94K_spec_example_6_11_multi_line_comments"]
#[doc(hidden)]
pub const _P94K_spec_example_6_11_multi_line_comments: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_P94K_spec_example_6_11_multi_line_comments"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_P94K_spec_example_6_11_multi_line_comments()),
    ),
};
#[allow(non_snake_case)]
fn _P94K_spec_example_6_11_multi_line_comments() {
    test("P94K");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_PBJ2_spec_example_2_3_mapping_scalars_to_sequences"]
#[doc(hidden)]
pub const _PBJ2_spec_example_2_3_mapping_scalars_to_sequences: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_PBJ2_spec_example_2_3_mapping_scalars_to_sequences",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _PBJ2_spec_example_2_3_mapping_scalars_to_sequences(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _PBJ2_spec_example_2_3_mapping_scalars_to_sequences() {
    test("PBJ2");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_PRH3_spec_example_7_9_single_quoted_lines"]
#[doc(hidden)]
pub const _PRH3_spec_example_7_9_single_quoted_lines: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_PRH3_spec_example_7_9_single_quoted_lines"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_PRH3_spec_example_7_9_single_quoted_lines()),
    ),
};
#[allow(non_snake_case)]
fn _PRH3_spec_example_7_9_single_quoted_lines() {
    test("PRH3");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_PUW8_document_start_on_last_line"]
#[doc(hidden)]
pub const _PUW8_document_start_on_last_line: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_PUW8_document_start_on_last_line"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_PUW8_document_start_on_last_line()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _PUW8_document_start_on_last_line() {
    test("PUW8");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_PW8X_anchors_on_empty_scalars"]
#[doc(hidden)]
pub const _PW8X_anchors_on_empty_scalars: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_PW8X_anchors_on_empty_scalars"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_PW8X_anchors_on_empty_scalars()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _PW8X_anchors_on_empty_scalars() {
    test("PW8X");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_Q5MG_tab_at_beginning_of_line_followed_by_a_flow_mapping"]
#[doc(hidden)]
pub const _Q5MG_tab_at_beginning_of_line_followed_by_a_flow_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_Q5MG_tab_at_beginning_of_line_followed_by_a_flow_mapping",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _Q5MG_tab_at_beginning_of_line_followed_by_a_flow_mapping(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _Q5MG_tab_at_beginning_of_line_followed_by_a_flow_mapping() {
    test("Q5MG");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_Q88A_spec_example_7_23_flow_content"]
#[doc(hidden)]
pub const _Q88A_spec_example_7_23_flow_content: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_Q88A_spec_example_7_23_flow_content"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_Q88A_spec_example_7_23_flow_content()),
    ),
};
#[allow(non_snake_case)]
fn _Q88A_spec_example_7_23_flow_content() {
    test("Q88A");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_Q8AD_spec_example_7_5_double_quoted_line_breaks_1_3"]
#[doc(hidden)]
pub const _Q8AD_spec_example_7_5_double_quoted_line_breaks_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_Q8AD_spec_example_7_5_double_quoted_line_breaks_1_3",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _Q8AD_spec_example_7_5_double_quoted_line_breaks_1_3(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _Q8AD_spec_example_7_5_double_quoted_line_breaks_1_3() {
    test("Q8AD");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_Q9WF_spec_example_6_12_separation_spaces"]
#[doc(hidden)]
pub const _Q9WF_spec_example_6_12_separation_spaces: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_Q9WF_spec_example_6_12_separation_spaces"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_Q9WF_spec_example_6_12_separation_spaces()),
    ),
};
#[allow(non_snake_case)]
fn _Q9WF_spec_example_6_12_separation_spaces() {
    test("Q9WF");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_QF4Y_spec_example_7_19_single_pair_flow_mappings"]
#[doc(hidden)]
pub const _QF4Y_spec_example_7_19_single_pair_flow_mappings: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_QF4Y_spec_example_7_19_single_pair_flow_mappings"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_QF4Y_spec_example_7_19_single_pair_flow_mappings()),
    ),
};
#[allow(non_snake_case)]
fn _QF4Y_spec_example_7_19_single_pair_flow_mappings() {
    test("QF4Y");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_QT73_comment_and_document_end_marker"]
#[doc(hidden)]
pub const _QT73_comment_and_document_end_marker: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_QT73_comment_and_document_end_marker"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_QT73_comment_and_document_end_marker()),
    ),
};
#[allow(non_snake_case)]
fn _QT73_comment_and_document_end_marker() {
    test("QT73");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_R4YG_spec_example_8_2_block_indentation_indicator"]
#[doc(hidden)]
pub const _R4YG_spec_example_8_2_block_indentation_indicator: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_R4YG_spec_example_8_2_block_indentation_indicator"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_R4YG_spec_example_8_2_block_indentation_indicator()),
    ),
};
#[allow(non_snake_case)]
fn _R4YG_spec_example_8_2_block_indentation_indicator() {
    test("R4YG");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_R52L_nested_flow_mapping_sequence_and_mappings"]
#[doc(hidden)]
pub const _R52L_nested_flow_mapping_sequence_and_mappings: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_R52L_nested_flow_mapping_sequence_and_mappings"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_R52L_nested_flow_mapping_sequence_and_mappings()),
    ),
};
#[allow(non_snake_case)]
fn _R52L_nested_flow_mapping_sequence_and_mappings() {
    test("R52L");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_RLU9_sequence_indent"]
#[doc(hidden)]
pub const _RLU9_sequence_indent: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_RLU9_sequence_indent"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_RLU9_sequence_indent()),
    ),
};
#[allow(non_snake_case)]
fn _RLU9_sequence_indent() {
    test("RLU9");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_RR7F_mixed_block_mapping_implicit_to_explicit"]
#[doc(hidden)]
pub const _RR7F_mixed_block_mapping_implicit_to_explicit: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_RR7F_mixed_block_mapping_implicit_to_explicit"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_RR7F_mixed_block_mapping_implicit_to_explicit()),
    ),
};
#[allow(non_snake_case)]
fn _RR7F_mixed_block_mapping_implicit_to_explicit() {
    test("RR7F");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_RTP8_spec_example_9_2_document_markers"]
#[doc(hidden)]
pub const _RTP8_spec_example_9_2_document_markers: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_RTP8_spec_example_9_2_document_markers"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_RTP8_spec_example_9_2_document_markers()),
    ),
};
#[allow(non_snake_case)]
fn _RTP8_spec_example_9_2_document_markers() {
    test("RTP8");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_RZP5_various_trailing_comments_1_3"]
#[doc(hidden)]
pub const _RZP5_various_trailing_comments_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_RZP5_various_trailing_comments_1_3"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_RZP5_various_trailing_comments_1_3()),
    ),
};
#[allow(non_snake_case)]
fn _RZP5_various_trailing_comments_1_3() {
    test("RZP5");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_RZT7_spec_example_2_28_log_file"]
#[doc(hidden)]
pub const _RZT7_spec_example_2_28_log_file: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_RZT7_spec_example_2_28_log_file"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_RZT7_spec_example_2_28_log_file()),
    ),
};
#[allow(non_snake_case)]
fn _RZT7_spec_example_2_28_log_file() {
    test("RZT7");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_S3PD_spec_example_8_18_implicit_block_mapping_entries"]
#[doc(hidden)]
pub const _S3PD_spec_example_8_18_implicit_block_mapping_entries: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_S3PD_spec_example_8_18_implicit_block_mapping_entries",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _S3PD_spec_example_8_18_implicit_block_mapping_entries(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _S3PD_spec_example_8_18_implicit_block_mapping_entries() {
    test("S3PD");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_S4JQ_spec_example_6_28_non_specific_tags"]
#[doc(hidden)]
pub const _S4JQ_spec_example_6_28_non_specific_tags: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_S4JQ_spec_example_6_28_non_specific_tags"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_S4JQ_spec_example_6_28_non_specific_tags()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _S4JQ_spec_example_6_28_non_specific_tags() {
    test("S4JQ");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_S4T7_document_with_footer"]
#[doc(hidden)]
pub const _S4T7_document_with_footer: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_S4T7_document_with_footer"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_S4T7_document_with_footer()),
    ),
};
#[allow(non_snake_case)]
fn _S4T7_document_with_footer() {
    test("S4T7");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_S7BG_colon_followed_by_comma"]
#[doc(hidden)]
pub const _S7BG_colon_followed_by_comma: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_S7BG_colon_followed_by_comma"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_S7BG_colon_followed_by_comma()),
    ),
};
#[allow(non_snake_case)]
fn _S7BG_colon_followed_by_comma() {
    test("S7BG");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_S9E8_spec_example_5_3_block_structure_indicators"]
#[doc(hidden)]
pub const _S9E8_spec_example_5_3_block_structure_indicators: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_S9E8_spec_example_5_3_block_structure_indicators"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_S9E8_spec_example_5_3_block_structure_indicators()),
    ),
};
#[allow(non_snake_case)]
fn _S9E8_spec_example_5_3_block_structure_indicators() {
    test("S9E8");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_SBG9_flow_sequence_in_flow_mapping"]
#[doc(hidden)]
pub const _SBG9_flow_sequence_in_flow_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_SBG9_flow_sequence_in_flow_mapping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_SBG9_flow_sequence_in_flow_mapping()),
    ),
};
#[allow(non_snake_case)]
fn _SBG9_flow_sequence_in_flow_mapping() {
    test("SBG9");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_SKE5_anchor_before_zero_indented_sequence"]
#[doc(hidden)]
pub const _SKE5_anchor_before_zero_indented_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_SKE5_anchor_before_zero_indented_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_SKE5_anchor_before_zero_indented_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _SKE5_anchor_before_zero_indented_sequence() {
    test("SKE5");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_SSW6_spec_example_7_7_single_quoted_characters_1_3"]
#[doc(hidden)]
pub const _SSW6_spec_example_7_7_single_quoted_characters_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_SSW6_spec_example_7_7_single_quoted_characters_1_3",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _SSW6_spec_example_7_7_single_quoted_characters_1_3(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _SSW6_spec_example_7_7_single_quoted_characters_1_3() {
    test("SSW6");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_SYW4_spec_example_2_2_mapping_scalars_to_scalars"]
#[doc(hidden)]
pub const _SYW4_spec_example_2_2_mapping_scalars_to_scalars: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_SYW4_spec_example_2_2_mapping_scalars_to_scalars"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_SYW4_spec_example_2_2_mapping_scalars_to_scalars()),
    ),
};
#[allow(non_snake_case)]
fn _SYW4_spec_example_2_2_mapping_scalars_to_scalars() {
    test("SYW4");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_T26H_spec_example_8_8_literal_content_1_3"]
#[doc(hidden)]
pub const _T26H_spec_example_8_8_literal_content_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_T26H_spec_example_8_8_literal_content_1_3"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_T26H_spec_example_8_8_literal_content_1_3()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _T26H_spec_example_8_8_literal_content_1_3() {
    test("T26H");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_T4YY_spec_example_7_9_single_quoted_lines_1_3"]
#[doc(hidden)]
pub const _T4YY_spec_example_7_9_single_quoted_lines_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_T4YY_spec_example_7_9_single_quoted_lines_1_3"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_T4YY_spec_example_7_9_single_quoted_lines_1_3()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _T4YY_spec_example_7_9_single_quoted_lines_1_3() {
    test("T4YY");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_T5N4_spec_example_8_7_literal_scalar_1_3"]
#[doc(hidden)]
pub const _T5N4_spec_example_8_7_literal_scalar_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_T5N4_spec_example_8_7_literal_scalar_1_3"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_T5N4_spec_example_8_7_literal_scalar_1_3()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _T5N4_spec_example_8_7_literal_scalar_1_3() {
    test("T5N4");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_TE2A_spec_example_8_16_block_mappings"]
#[doc(hidden)]
pub const _TE2A_spec_example_8_16_block_mappings: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_TE2A_spec_example_8_16_block_mappings"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_TE2A_spec_example_8_16_block_mappings()),
    ),
};
#[allow(non_snake_case)]
fn _TE2A_spec_example_8_16_block_mappings() {
    test("TE2A");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_TL85_spec_example_6_8_flow_folding"]
#[doc(hidden)]
pub const _TL85_spec_example_6_8_flow_folding: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_TL85_spec_example_6_8_flow_folding"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_TL85_spec_example_6_8_flow_folding()),
    ),
};
#[allow(non_snake_case)]
fn _TL85_spec_example_6_8_flow_folding() {
    test("TL85");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_TS54_folded_block_scalar"]
#[doc(hidden)]
pub const _TS54_folded_block_scalar: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_TS54_folded_block_scalar"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_TS54_folded_block_scalar()),
    ),
};
#[allow(non_snake_case)]
fn _TS54_folded_block_scalar() {
    test("TS54");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_U3C3_spec_example_6_16_tag_directive"]
#[doc(hidden)]
pub const _U3C3_spec_example_6_16_tag_directive: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_U3C3_spec_example_6_16_tag_directive"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_U3C3_spec_example_6_16_tag_directive()),
    ),
};
#[allow(non_snake_case)]
fn _U3C3_spec_example_6_16_tag_directive() {
    test("U3C3");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_U3XV_node_and_mapping_key_anchors"]
#[doc(hidden)]
pub const _U3XV_node_and_mapping_key_anchors: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_U3XV_node_and_mapping_key_anchors"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_U3XV_node_and_mapping_key_anchors()),
    ),
};
#[allow(non_snake_case)]
fn _U3XV_node_and_mapping_key_anchors() {
    test("U3XV");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_U9NS_spec_example_2_8_play_by_play_feed_from_a_game"]
#[doc(hidden)]
pub const _U9NS_spec_example_2_8_play_by_play_feed_from_a_game: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_U9NS_spec_example_2_8_play_by_play_feed_from_a_game",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _U9NS_spec_example_2_8_play_by_play_feed_from_a_game(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _U9NS_spec_example_2_8_play_by_play_feed_from_a_game() {
    test("U9NS");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_UDM2_plain_url_in_flow_mapping"]
#[doc(hidden)]
pub const _UDM2_plain_url_in_flow_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_UDM2_plain_url_in_flow_mapping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_UDM2_plain_url_in_flow_mapping()),
    ),
};
#[allow(non_snake_case)]
fn _UDM2_plain_url_in_flow_mapping() {
    test("UDM2");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_UDR7_spec_example_5_4_flow_collection_indicators"]
#[doc(hidden)]
pub const _UDR7_spec_example_5_4_flow_collection_indicators: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_UDR7_spec_example_5_4_flow_collection_indicators"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_UDR7_spec_example_5_4_flow_collection_indicators()),
    ),
};
#[allow(non_snake_case)]
fn _UDR7_spec_example_5_4_flow_collection_indicators() {
    test("UDR7");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_UGM3_spec_example_2_27_invoice"]
#[doc(hidden)]
pub const _UGM3_spec_example_2_27_invoice: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_UGM3_spec_example_2_27_invoice"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_UGM3_spec_example_2_27_invoice()),
    ),
};
#[allow(non_snake_case)]
fn _UGM3_spec_example_2_27_invoice() {
    test("UGM3");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_UT92_spec_example_9_4_explicit_documents"]
#[doc(hidden)]
pub const _UT92_spec_example_9_4_explicit_documents: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_UT92_spec_example_9_4_explicit_documents"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_UT92_spec_example_9_4_explicit_documents()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _UT92_spec_example_9_4_explicit_documents() {
    test("UT92");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_V55R_aliases_in_block_sequence"]
#[doc(hidden)]
pub const _V55R_aliases_in_block_sequence: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_V55R_aliases_in_block_sequence"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_V55R_aliases_in_block_sequence()),
    ),
};
#[allow(non_snake_case)]
fn _V55R_aliases_in_block_sequence() {
    test("V55R");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_V9D5_spec_example_8_19_compact_block_mappings"]
#[doc(hidden)]
pub const _V9D5_spec_example_8_19_compact_block_mappings: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_V9D5_spec_example_8_19_compact_block_mappings"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_V9D5_spec_example_8_19_compact_block_mappings()),
    ),
};
#[allow(non_snake_case)]
fn _V9D5_spec_example_8_19_compact_block_mappings() {
    test("V9D5");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_W42U_spec_example_8_15_block_sequence_entry_types"]
#[doc(hidden)]
pub const _W42U_spec_example_8_15_block_sequence_entry_types: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_W42U_spec_example_8_15_block_sequence_entry_types"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_W42U_spec_example_8_15_block_sequence_entry_types()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _W42U_spec_example_8_15_block_sequence_entry_types() {
    test("W42U");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_W4TN_spec_example_9_5_directives_documents"]
#[doc(hidden)]
pub const _W4TN_spec_example_9_5_directives_documents: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_W4TN_spec_example_9_5_directives_documents"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_W4TN_spec_example_9_5_directives_documents()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _W4TN_spec_example_9_5_directives_documents() {
    test("W4TN");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_W5VH_allowed_characters_in_alias"]
#[doc(hidden)]
pub const _W5VH_allowed_characters_in_alias: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_W5VH_allowed_characters_in_alias"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_W5VH_allowed_characters_in_alias()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _W5VH_allowed_characters_in_alias() {
    test("W5VH");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_WZ62_spec_example_7_2_empty_content"]
#[doc(hidden)]
pub const _WZ62_spec_example_7_2_empty_content: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_WZ62_spec_example_7_2_empty_content"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_WZ62_spec_example_7_2_empty_content()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _WZ62_spec_example_7_2_empty_content() {
    test("WZ62");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_X38W_aliases_in_flow_objects"]
#[doc(hidden)]
pub const _X38W_aliases_in_flow_objects: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_X38W_aliases_in_flow_objects"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_X38W_aliases_in_flow_objects()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _X38W_aliases_in_flow_objects() {
    test("X38W");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_X8DW_explicit_key_and_value_seperated_by_comment"]
#[doc(hidden)]
pub const _X8DW_explicit_key_and_value_seperated_by_comment: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_X8DW_explicit_key_and_value_seperated_by_comment"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_X8DW_explicit_key_and_value_seperated_by_comment()),
    ),
};
#[allow(non_snake_case)]
fn _X8DW_explicit_key_and_value_seperated_by_comment() {
    test("X8DW");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_XLQ9_multiline_scalar_that_looks_like_a_yaml_directive"]
#[doc(hidden)]
pub const _XLQ9_multiline_scalar_that_looks_like_a_yaml_directive: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_XLQ9_multiline_scalar_that_looks_like_a_yaml_directive",
        ),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _XLQ9_multiline_scalar_that_looks_like_a_yaml_directive(),
        ),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _XLQ9_multiline_scalar_that_looks_like_a_yaml_directive() {
    test("XLQ9");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_XV9V_spec_example_6_5_empty_lines_1_3"]
#[doc(hidden)]
pub const _XV9V_spec_example_6_5_empty_lines_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_XV9V_spec_example_6_5_empty_lines_1_3"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_XV9V_spec_example_6_5_empty_lines_1_3()),
    ),
};
#[allow(non_snake_case)]
fn _XV9V_spec_example_6_5_empty_lines_1_3() {
    test("XV9V");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_XW4D_various_trailing_comments"]
#[doc(hidden)]
pub const _XW4D_various_trailing_comments: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_XW4D_various_trailing_comments"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_XW4D_various_trailing_comments()),
    ),
};
#[allow(non_snake_case)]
fn _XW4D_various_trailing_comments() {
    test("XW4D");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_Y2GN_anchor_with_colon_in_the_middle"]
#[doc(hidden)]
pub const _Y2GN_anchor_with_colon_in_the_middle: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_Y2GN_anchor_with_colon_in_the_middle"),
        ignore: true,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_Y2GN_anchor_with_colon_in_the_middle()),
    ),
};
#[ignore]
#[allow(non_snake_case)]
fn _Y2GN_anchor_with_colon_in_the_middle() {
    test("Y2GN");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_YD5X_spec_example_2_5_sequence_of_sequences"]
#[doc(hidden)]
pub const _YD5X_spec_example_2_5_sequence_of_sequences: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_YD5X_spec_example_2_5_sequence_of_sequences"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_YD5X_spec_example_2_5_sequence_of_sequences()),
    ),
};
#[allow(non_snake_case)]
fn _YD5X_spec_example_2_5_sequence_of_sequences() {
    test("YD5X");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_Z67P_spec_example_8_21_block_scalar_nodes_1_3"]
#[doc(hidden)]
pub const _Z67P_spec_example_8_21_block_scalar_nodes_1_3: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_Z67P_spec_example_8_21_block_scalar_nodes_1_3"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_Z67P_spec_example_8_21_block_scalar_nodes_1_3()),
    ),
};
#[allow(non_snake_case)]
fn _Z67P_spec_example_8_21_block_scalar_nodes_1_3() {
    test("Z67P");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_Z9M4_spec_example_6_22_global_tag_prefix"]
#[doc(hidden)]
pub const _Z9M4_spec_example_6_22_global_tag_prefix: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_Z9M4_spec_example_6_22_global_tag_prefix"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_Z9M4_spec_example_6_22_global_tag_prefix()),
    ),
};
#[allow(non_snake_case)]
fn _Z9M4_spec_example_6_22_global_tag_prefix() {
    test("Z9M4");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_ZF4X_spec_example_2_6_mapping_of_mappings"]
#[doc(hidden)]
pub const _ZF4X_spec_example_2_6_mapping_of_mappings: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_ZF4X_spec_example_2_6_mapping_of_mappings"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_ZF4X_spec_example_2_6_mapping_of_mappings()),
    ),
};
#[allow(non_snake_case)]
fn _ZF4X_spec_example_2_6_mapping_of_mappings() {
    test("ZF4X");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_ZH7C_anchors_in_mapping"]
#[doc(hidden)]
pub const _ZH7C_anchors_in_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_ZH7C_anchors_in_mapping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_ZH7C_anchors_in_mapping()),
    ),
};
#[allow(non_snake_case)]
fn _ZH7C_anchors_in_mapping() {
    test("ZH7C");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_ZK9H_nested_top_level_flow_mapping"]
#[doc(hidden)]
pub const _ZK9H_nested_top_level_flow_mapping: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("_ZK9H_nested_top_level_flow_mapping"),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(_ZK9H_nested_top_level_flow_mapping()),
    ),
};
#[allow(non_snake_case)]
fn _ZK9H_nested_top_level_flow_mapping() {
    test("ZK9H");
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker = "_ZWK4_key_with_anchor_after_missing_explicit_mapping_value"]
#[doc(hidden)]
pub const _ZWK4_key_with_anchor_after_missing_explicit_mapping_value: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName(
            "_ZWK4_key_with_anchor_after_missing_explicit_mapping_value",
        ),
        ignore: false,
        ignore_message: ::core::option::Option::None,
        source_file: "tests/test_emitter.rs",
        start_line: 48usize,
        start_col: 1usize,
        end_line: 48usize,
        end_col: 35usize,
        compile_fail: false,
        no_run: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(
        #[coverage(off)]
        || test::assert_test_result(
            _ZWK4_key_with_anchor_after_missing_explicit_mapping_value(),
        ),
    ),
};
#[allow(non_snake_case)]
fn _ZWK4_key_with_anchor_after_missing_explicit_mapping_value() {
    test("ZWK4");
}
#[rustc_main]
#[coverage(off)]
#[doc(hidden)]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(
        &[
            &_229Q_spec_example_2_4_sequence_of_mappings,
            &_26DV_whitespace_around_colon_in_mappings,
            &_27NA_spec_example_5_9_directive_indicator,
            &_2AUY_tags_in_block_sequence,
            &_2EBW_allowed_characters_in_keys,
            &_2JQS_block_mapping_with_missing_keys,
            &_2LFX_spec_example_6_13_reserved_directives_1_3,
            &_2SXE_anchors_with_colon_in_name,
            &_2XXW_spec_example_2_25_unordered_sets,
            &_33X3_three_explicit_integers_in_a_block_sequence,
            &_35KP_tags_for_root_objects,
            &_36F6_multiline_plain_scalar_with_empty_line,
            &_3ALJ_block_sequence_in_block_sequence,
            &_3GZX_spec_example_7_1_alias_nodes,
            &_3MYT_plain_scalar_looking_like_key_comment_anchor_and_tag,
            &_3R3P_single_block_sequence_with_anchor,
            &_3UYS_escaped_slash_in_double_quotes,
            &_4ABK_spec_example_7_17_flow_mapping_separate_values,
            &_4CQQ_spec_example_2_18_multi_line_flow_scalars,
            &_4FJ6_nested_implicit_complex_keys,
            &_4GC6_spec_example_7_7_single_quoted_characters,
            &_4MUZ_flow_mapping_colon_on_line_after_key,
            &_4Q9F_folded_block_scalar_1_3,
            &_4QFQ_spec_example_8_2_block_indentation_indicator_1_3,
            &_4UYU_colon_in_double_quoted_string,
            &_4V8U_plain_scalar_with_backslashes,
            &_4ZYM_spec_example_6_4_line_prefixes,
            &_52DL_explicit_non_specific_tag_1_3,
            &_54T7_flow_mapping,
            &_565N_construct_binary,
            &_57H4_spec_example_8_22_block_collection_nodes,
            &_5BVJ_spec_example_5_7_block_scalar_indicators,
            &_5C5M_spec_example_7_15_flow_mappings,
            &_5GBF_spec_example_6_5_empty_lines,
            &_5KJE_spec_example_7_13_flow_sequence,
            &_5MUD_colon_and_adjacent_value_on_next_line,
            &_5NYZ_spec_example_6_9_separated_comment,
            &_5TYM_spec_example_6_21_local_tag_prefix,
            &_5WE3_spec_example_8_17_explicit_block_mapping_entries,
            &_65WH_single_entry_block_sequence,
            &_6BCT_spec_example_6_3_separation_spaces,
            &_6BFJ_mapping_key_and_flow_sequence_item_anchors,
            &_6CK3_spec_example_6_26_tag_shorthands,
            &_6FWR_block_scalar_keep,
            &_6H3V_backslashes_in_singlequotes,
            &_6HB6_spec_example_6_1_indentation_spaces,
            &_6JQW_spec_example_2_13_in_literals_newlines_are_preserved,
            &_6JWB_tags_for_block_objects,
            &_6KGN_anchor_for_empty_node,
            &_6LVF_spec_example_6_13_reserved_directives,
            &_6M2F_aliases_in_explicit_block_mapping,
            &_6PBE_zero_indented_sequences_in_explicit_mapping_keys,
            &_6SLA_allowed_characters_in_quoted_mapping_key,
            &_6VJK_spec_example_2_15_folded_newlines_are_preserved_for_more_indented_and_blank_lines,
            &_6WLZ_spec_example_6_18_primary_tag_handle_1_3,
            &_6WPF_spec_example_6_8_flow_folding_1_3,
            &_6XDY_two_document_start_markers,
            &_6ZKB_spec_example_9_6_stream,
            &_735Y_spec_example_8_20_block_node_types,
            &_74H7_tags_in_implicit_mapping,
            &_753E_block_scalar_strip_1_3,
            &_77H8_spec_example_2_23_various_explicit_tags,
            &_7A4E_spec_example_7_6_double_quoted_lines,
            &_7BMT_node_and_mapping_key_anchors_1_3,
            &_7BUB_spec_example_2_10_node_for_sammy_sosa_appears_twice_in_this_document,
            &_7FWL_spec_example_6_24_verbatim_tags,
            &_7T8X_spec_example_8_10_folded_lines_8_13_final_empty_lines,
            &_7TMG_comment_in_flow_sequence_before_comma,
            &_7W2P_block_mapping_with_missing_values,
            &_7Z25_bare_document_after_document_end_marker,
            &_7ZZ5_empty_flow_collections,
            &_82AN_three_dashes_and_content_without_space,
            &_87E4_spec_example_7_8_single_quoted_implicit_keys,
            &_8CWC_plain_mapping_key_ending_with_colon,
            &_8G76_spec_example_6_10_comment_lines,
            &_8KB6_multiline_plain_flow_mapping_key_without_value,
            &_8MK2_explicit_non_specific_tag,
            &_8QBE_block_sequence_in_block_mapping,
            &_8UDB_spec_example_7_14_flow_sequence_entries,
            &_8XYN_anchor_with_unicode_character,
            &_93JH_block_mappings_in_block_sequence,
            &_93WF_spec_example_6_6_line_folding_1_3,
            &_96L6_spec_example_2_14_in_the_folded_scalars_newlines_become_spaces,
            &_98YD_spec_example_5_5_comment_indicator,
            &_9BXH_multiline_doublequoted_flow_mapping_key_without_value,
            &_9DXL_spec_example_9_6_stream_1_3,
            &_9FMG_multi_level_mapping_indent,
            &_9J7A_simple_mapping_indent,
            &_9KAX_various_combinations_of_tags_and_anchors,
            &_9MMW_spec_example_7_21_single_pair_implicit_entries_1_3,
            &_9SA2_multiline_double_quoted_flow_mapping_key,
            &_9SHH_spec_example_5_8_quoted_scalar_indicators,
            &_9TFX_spec_example_7_6_double_quoted_lines_1_3,
            &_9U5K_spec_example_2_12_compact_nested_mapping,
            &_9WXW_spec_example_6_18_primary_tag_handle,
            &_9YRD_multiline_scalar_at_top_level,
            &_A2M4_spec_example_6_2_indentation_indicators,
            &_A6F9_spec_example_8_4_chomping_final_line_break,
            &_A984_multiline_scalar_in_mapping,
            &_AB8U_sequence_entry_that_looks_like_two_with_wrong_indentation,
            &_AVM7_empty_stream,
            &_AZ63_sequence_with_same_indentation_as_parent_mapping,
            &_AZW3_lookahead_test_cases,
            &_B3HG_spec_example_8_9_folded_scalar_1_3,
            &_BEC7_spec_example_6_14_yaml_directive,
            &_BU8L_node_anchor_and_tag_on_seperate_lines,
            &_C2DT_spec_example_7_18_flow_mapping_adjacent_values,
            &_C4HZ_spec_example_2_24_global_tags,
            &_CC74_spec_example_6_20_tag_handles,
            &_CN3R_various_location_of_anchors_in_flow_sequence,
            &_CPZ3_doublequoted_scalar_starting_with_a_tab,
            &_CT4Q_spec_example_7_20_single_pair_explicit_entry,
            &_CUP7_spec_example_5_6_node_property_indicators,
            &_D83L_block_scalar_indicator_order,
            &_D88J_flow_sequence_in_block_mapping,
            &_D9TU_single_pair_block_mapping,
            &_DBG4_spec_example_7_10_plain_characters,
            &_DC7X_various_trailing_tabs,
            &_DFF7_spec_example_7_16_flow_mapping_entries,
            &_DHP8_flow_sequence,
            &_DK3J_zero_indented_block_scalar_with_line_that_looks_like_a_comment,
            &_DWX9_spec_example_8_8_literal_content,
            &_E76Z_aliases_in_implicit_block_mapping,
            &_EHF6_tags_for_flow_objects,
            &_EX5H_multiline_scalar_at_top_level_1_3,
            &_EXG3_three_dashes_and_content_without_space_1_3,
            &_F2C7_anchors_and_tags,
            &_F3CP_nested_flow_collections_on_one_line,
            &_F6MC_more_indented_lines_at_the_beginning_of_folded_block_scalars,
            &_F8F9_spec_example_8_5_chomping_trailing_lines,
            &_FBC9_allowed_characters_in_plain_scalars,
            &_FH7J_tags_on_empty_scalars,
            &_FP8R_zero_indented_block_scalar,
            &_FQ7F_spec_example_2_1_sequence_of_scalars,
            &_FRK4_spec_example_7_3_completely_empty_flow_nodes,
            &_FTA2_single_block_sequence_with_anchor_and_explicit_document_start,
            &_FUP4_flow_sequence_in_flow_sequence,
            &_G4RS_spec_example_2_17_quoted_scalars,
            &_G5U8_plain_dashes_in_flow_sequence,
            &_G992_spec_example_8_9_folded_scalar,
            &_GH63_mixed_block_mapping_explicit_to_implicit,
            &_H2RW_blank_lines,
            &_H3Z8_literal_unicode,
            &_HMK4_spec_example_2_16_indentation_determines_scope,
            &_HMQ5_spec_example_6_23_node_properties,
            &_HS5T_spec_example_7_12_plain_lines,
            &_HWV9_document_end_marker,
            &_J3BT_spec_example_5_12_tabs_and_spaces,
            &_J5UC_multiple_pair_block_mapping,
            &_J7PZ_spec_example_2_26_ordered_mappings,
            &_J7VC_empty_lines_between_mapping_elements,
            &_J9HZ_spec_example_2_9_single_document_with_two_comments,
            &_JDH8_plain_scalar_looking_like_key_comment_anchor_and_tag_1_3,
            &_JHB9_spec_example_2_7_two_documents_in_a_stream,
            &_JQ4R_spec_example_8_14_block_sequence,
            &_JS2J_spec_example_6_29_node_anchors,
            &_JTV5_block_mapping_with_multiline_scalars,
            &_K3WX_colon_and_adjacent_value_after_comment_on_next_line,
            &_K4SU_multiple_entry_block_sequence,
            &_K527_spec_example_6_6_line_folding,
            &_K54U_tab_after_document_header,
            &_K858_spec_example_8_6_empty_scalar_chomping,
            &_KK5P_various_combinations_of_explicit_block_mappings,
            &_KMK3_block_submapping,
            &_KSS4_scalars_on_line,
            &_KZN9_spec_example_7_21_single_pair_implicit_entries,
            &_L94M_tags_in_explicit_mapping,
            &_L9U5_spec_example_7_11_plain_implicit_keys,
            &_LE5A_spec_example_7_24_flow_nodes,
            &_LP6E_whitespace_after_scalars_in_flow,
            &_LQZ7_spec_example_7_4_double_quoted_implicit_keys,
            &_LX3P_implicit_flow_mapping_key_on_one_line,
            &_M29M_literal_block_scalar,
            &_M5C3_spec_example_8_21_block_scalar_nodes,
            &_M5DY_spec_example_2_11_mapping_between_sequences,
            &_M7A3_spec_example_9_3_bare_documents,
            &_M7NX_nested_flow_collections,
            &_M9B4_spec_example_8_7_literal_scalar,
            &_MJS9_spec_example_6_7_block_folding,
            &_MXS3_flow_mapping_in_block_sequence,
            &_MYW6_block_scalar_strip,
            &_MZX3_non_specific_tags_on_scalars,
            &_NAT4_various_empty_or_newline_only_quoted_strings,
            &_NB6Z_multiline_plain_value_with_tabs_on_empty_lines,
            &_NHX8_empty_lines_at_end_of_document,
            &_NJ66_multiline_plain_flow_mapping_key,
            &_NP9H_spec_example_7_5_double_quoted_line_breaks,
            &_P2AD_spec_example_8_1_block_scalar_header,
            &_P76L_spec_example_6_19_secondary_tag_handle,
            &_P94K_spec_example_6_11_multi_line_comments,
            &_PBJ2_spec_example_2_3_mapping_scalars_to_sequences,
            &_PRH3_spec_example_7_9_single_quoted_lines,
            &_PUW8_document_start_on_last_line,
            &_PW8X_anchors_on_empty_scalars,
            &_Q5MG_tab_at_beginning_of_line_followed_by_a_flow_mapping,
            &_Q88A_spec_example_7_23_flow_content,
            &_Q8AD_spec_example_7_5_double_quoted_line_breaks_1_3,
            &_Q9WF_spec_example_6_12_separation_spaces,
            &_QF4Y_spec_example_7_19_single_pair_flow_mappings,
            &_QT73_comment_and_document_end_marker,
            &_R4YG_spec_example_8_2_block_indentation_indicator,
            &_R52L_nested_flow_mapping_sequence_and_mappings,
            &_RLU9_sequence_indent,
            &_RR7F_mixed_block_mapping_implicit_to_explicit,
            &_RTP8_spec_example_9_2_document_markers,
            &_RZP5_various_trailing_comments_1_3,
            &_RZT7_spec_example_2_28_log_file,
            &_S3PD_spec_example_8_18_implicit_block_mapping_entries,
            &_S4JQ_spec_example_6_28_non_specific_tags,
            &_S4T7_document_with_footer,
            &_S7BG_colon_followed_by_comma,
            &_S9E8_spec_example_5_3_block_structure_indicators,
            &_SBG9_flow_sequence_in_flow_mapping,
            &_SKE5_anchor_before_zero_indented_sequence,
            &_SSW6_spec_example_7_7_single_quoted_characters_1_3,
            &_SYW4_spec_example_2_2_mapping_scalars_to_scalars,
            &_T26H_spec_example_8_8_literal_content_1_3,
            &_T4YY_spec_example_7_9_single_quoted_lines_1_3,
            &_T5N4_spec_example_8_7_literal_scalar_1_3,
            &_TE2A_spec_example_8_16_block_mappings,
            &_TL85_spec_example_6_8_flow_folding,
            &_TS54_folded_block_scalar,
            &_U3C3_spec_example_6_16_tag_directive,
            &_U3XV_node_and_mapping_key_anchors,
            &_U9NS_spec_example_2_8_play_by_play_feed_from_a_game,
            &_UDM2_plain_url_in_flow_mapping,
            &_UDR7_spec_example_5_4_flow_collection_indicators,
            &_UGM3_spec_example_2_27_invoice,
            &_UT92_spec_example_9_4_explicit_documents,
            &_V55R_aliases_in_block_sequence,
            &_V9D5_spec_example_8_19_compact_block_mappings,
            &_W42U_spec_example_8_15_block_sequence_entry_types,
            &_W4TN_spec_example_9_5_directives_documents,
            &_W5VH_allowed_characters_in_alias,
            &_WZ62_spec_example_7_2_empty_content,
            &_X38W_aliases_in_flow_objects,
            &_X8DW_explicit_key_and_value_seperated_by_comment,
            &_XLQ9_multiline_scalar_that_looks_like_a_yaml_directive,
            &_XV9V_spec_example_6_5_empty_lines_1_3,
            &_XW4D_various_trailing_comments,
            &_Y2GN_anchor_with_colon_in_the_middle,
            &_YD5X_spec_example_2_5_sequence_of_sequences,
            &_Z67P_spec_example_8_21_block_scalar_nodes_1_3,
            &_Z9M4_spec_example_6_22_global_tag_prefix,
            &_ZF4X_spec_example_2_6_mapping_of_mappings,
            &_ZH7C_anchors_in_mapping,
            &_ZK9H_nested_top_level_flow_mapping,
            &_ZWK4_key_with_anchor_after_missing_explicit_mapping_value,
        ],
    )
}
