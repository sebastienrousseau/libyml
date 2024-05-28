
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
pub(crate) use core::primitive::u8 as yaml_char_t;
use libyml::api::ScalarEventData;
use std::env;
use std::error::Error;
use std::ffi::c_void;
use std::fs::File;
use std::io::{self, Read, Write};
use std::mem::MaybeUninit;
use std::process::{self, ExitCode};
use std::ptr::{self, addr_of_mut};
// use std::slice;
use libyml::{
    yaml_alias_event_initialize, yaml_document_end_event_initialize,
    yaml_document_start_event_initialize, yaml_emitter_delete,
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

pub(crate) unsafe fn unsafe_main(
    stdin: &mut dyn Read,
    mut stdout: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    let mut emitter = MaybeUninit::<YamlEmitterT>::uninit();
    let emitter = emitter.as_mut_ptr();
    if yaml_emitter_initialize(emitter).fail {
        return Err("Could not initialize the emitter object".into());
    }

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

    yaml_emitter_set_output(
        emitter,
        write_to_stdio,
        addr_of_mut!(stdout).cast(),
    );
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
            // yaml_scalar_event_initialize(
            //     event,
            //     get_anchor(b'&', line, anchor.as_mut_ptr()),
            //     get_tag(line, tag.as_mut_ptr()),
            //     value.as_mut_ptr() as *mut u8,
            //     -1,
            //     implicit,
            //     implicit,
            //     style,
            // )
        } else if line.starts_with(b"=ALI") {
            yaml_alias_event_initialize(
                event,
                get_anchor(b'*', line, anchor.as_mut_ptr()),
            )
        } else {
            let line = line.as_mut_ptr() as *mut i8;
            break Err(format!(
                "Unknown event: '{}'",
                CStr::from_ptr(line)
            )
            .into());
        };

        if result.fail {
            break Err(
                "Memory error: Not enough memory for creating an event"
                    .into(),
            );
        }
        if yaml_emitter_emit(emitter, event).fail {
            break Err(match (*emitter).error {
                YamlMemoryError => {
                    "Memory error: Not enough memory for emitting"
                        .into()
                }
                YamlWriterError => format!(
                    "Writer error: {}",
                    CStr::from_ptr((*emitter).problem)
                )
                .into(),
                YamlEmitterError => format!(
                    "Emitter error: {}",
                    CStr::from_ptr((*emitter).problem)
                )
                .into(),
                // Couldn't happen.
                _ => "Internal error".into(),
            });
        }
    };

    yaml_emitter_delete(emitter);
    result
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
            let mut remainder =
                &mut self.buf[self.offset + self.filled..];
            if remainder.is_empty() {
                if self.offset == 0 {
                    let _ = writeln!(
                        io::stderr(),
                        "Line too long: '{}'",
                        String::from_utf8_lossy(&self.buf),
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

unsafe fn get_value(
    line: &[u8],
    value: *mut i8,
    style: *mut YamlScalarStyleT,
) {
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
        let _ = writeln!(
            io::stderr(),
            "Usage: run-emitter-test-suite <test.event>...",
        );
        return ExitCode::FAILURE;
    }
    for arg in args {
        let mut stdin = File::open(arg).unwrap();
        let mut stdout = io::stdout();
        let result = unsafe { unsafe_main(&mut stdin, &mut stdout) };
        if let Err(err) = result {
            let _ = writeln!(io::stderr(), "{}", err);
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
