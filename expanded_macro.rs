#![feature(prelude_import)]
//! # LibYML (a fork of unsafe-libyaml)
//!
//! [![Made With Love][made-with-rust]][10]
//! [![Crates.io][crates-badge]][06]
//! [![lib.rs][libs-badge]][11]
//! [![Docs.rs][docs-badge]][07]
//! [![Codecov][codecov-badge]][08]
//! [![Build Status][build-badge]][09]
//! [![GitHub][github-badge]][05]
//!
//! LibYML is a Rust library for working with YAML data, forked from [unsafe-libyaml][01]. It offers a safe and efficient interface for parsing, emitting, and manipulating YAML data.
//!
//! ## Features
//!
//! - **Serialization and Deserialization**: Easy-to-use APIs for serializing Rust structs and enums to YAML and vice versa.
//! - **Custom Struct and Enum Support**: Seamless serialization and deserialization of custom data types.
//! - **Comprehensive Error Handling**: Detailed error messages and recovery mechanisms.
//! - **Streaming Support**: Efficient processing of large YAML documents.
//! - **Alias and Anchor Support**: Handling of complex YAML structures with references.
//! - **Tag Handling**: Support for custom tags and type-specific serialization.
//! - **Configurable Emitter**: Customizable YAML output generation.
//! - **Extensive Documentation**: Detailed docs and examples for easy onboarding.
//! - **Safety and Efficiency**: Minimized unsafe code with an interface designed to prevent common pitfalls.
//!
//! ## Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! libyml = "0.0.6"
//! ```
//!
//! ## Documentation
//!
//! For full API documentation, please visit [https://doc.libyml.com/libyml/][03] or [https://docs.rs/libyml][07].
//!
//! ## Rust Version Compatibility
//!
//! Compiler support: requires rustc 1.56.0+
//!
//! ## Contributing
//!
//! Contributions are welcome! If you'd like to contribute, please feel free to submit a Pull Request on [GitHub][05].
//!
//! ## Credits and Acknowledgements
//!
//! LibYML is a fork of the work done by [David Tolnay][04] and the maintainers of [unsafe-libyaml][01]. While it has evolved into a separate library, we express our sincere gratitude to them as well as the [libyaml][02] maintainers for their contributions to the Rust and C programming communities.
//!
//! ## License
//!
//! [MIT license](https://opensource.org/license/MIT), same as libyaml.
//!
//! [00]: https://libyml.com
//! [01]: https://github.com/dtolnay/unsafe-libyaml
//! [02]: https://github.com/yaml/libyaml/tree/2c891fc7a770e8ba2fec34fc6b545c672beb37e6
//! [03]: https://doc.libyml.com/libyml/
//! [04]: https://github.com/dtolnay
//! [05]: https://github.com/sebastienrousseau/libyml
//! [06]: https://crates.io/crates/libyml
//! [07]: https://docs.rs/libyml
//! [08]: https://codecov.io/gh/sebastienrousseau/libyml
//! [09]: https://github.com/sebastienrousseau/libyml/actions?query=branch%3Amaster
//! [10]: https://www.rust-lang.org/
//! [11]: https://lib.rs/crates/libyml
//!
//! [build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/libyml/release.yml?branch=master&style=for-the-badge&logo=github "Build Status"
//! [codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/libyml?style=for-the-badge&logo=codecov&token=yc9s578xIk "Code Coverage"
//! [crates-badge]: https://img.shields.io/crates/v/libyml.svg?style=for-the-badge&color=fc8d62&logo=rust "View on Crates.io"
//! [libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.6-orange.svg?style=for-the-badge "View on lib.rs"
//! [docs-badge]: https://img.shields.io/badge/docs.rs-libyml-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs "View Documentation"
//! [github-badge]: https://img.shields.io/badge/github-sebastienrousseau/libyml-8da0cb?style=for-the-badge&labelColor=555555&logo=github "View on GitHub"
//! [made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust'
//!
#![no_std]
#![doc(
    html_favicon_url = "https://kura.pro/libyml/images/favicon.ico",
    html_logo_url = "https://kura.pro/libyml/images/logos/libyml.svg",
    html_root_url = "https://docs.rs/libyml"
)]
#![crate_name = "libyml"]
#![crate_type = "lib"]
#[prelude_import]
use core::prelude::rust_2021::*;
#[macro_use]
extern crate core;
extern crate compiler_builtins as _;
extern crate alloc;
use core::mem::size_of;
/// Declarations for C library functions used within the LibYML library.
///
/// This module contains the necessary types and constants required for
/// interfacing with C libraries, particularly in the context of memory management
/// and low-level operations within LibYML.
pub mod libc {
    pub use core::ffi::c_char;
    pub use core::ffi::c_void;
    pub use core::primitive::{
        i32 as c_int, i64 as c_long, u32 as c_uint, u64 as c_ulong, u8 as c_uchar,
    };
}
/// Extern functions and macros for interacting with the underlying libyaml C library.
///
/// This module includes low-level functions for memory allocation, comparison, and
/// movement that bridge Rust and C. It also contains some internal macros for
/// asserting conditions in a way that integrates with the C code.
#[macro_use]
pub mod externs {
    use crate::libc;
    use crate::ops::{die, ForceAdd as _, ForceInto as _};
    use alloc::alloc::{self as rust, Layout};
    use core::mem::MaybeUninit;
    use core::mem::{align_of, size_of};
    use core::ptr;
    use core::slice;
    const HEADER: usize = {
        let need_len = size_of::<usize>();
        (need_len + MALLOC_ALIGN - 1) & !(MALLOC_ALIGN - 1)
    };
    const MALLOC_ALIGN: usize = {
        let int_align = align_of::<libc::c_ulong>();
        let ptr_align = align_of::<usize>();
        if int_align >= ptr_align { int_align } else { ptr_align }
    };
    /// Allocates memory.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it directly manipulates raw memory. The caller must ensure that
    /// the allocated memory is properly managed and freed when no longer needed.
    pub unsafe fn malloc(size: libc::c_ulong) -> *mut libc::c_void {
        let size = HEADER.force_add(size.force_into());
        let layout = Layout::from_size_align(size, MALLOC_ALIGN)
            .ok()
            .unwrap_or_else(die);
        let memory = rust::alloc(layout);
        if memory.is_null() {
            return die();
        }
        memory.cast::<usize>().write(size);
        memory.add(HEADER).cast()
    }
    /// Reallocates memory.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it directly manipulates raw memory.
    /// The caller must ensure that the original memory was allocated by `malloc`.
    pub(crate) unsafe fn realloc(
        ptr: *mut libc::c_void,
        new_size: libc::c_ulong,
    ) -> *mut libc::c_void {
        let mut memory = ptr.cast::<u8>().sub(HEADER);
        let size = memory.cast::<usize>().read();
        let layout = Layout::from_size_align_unchecked(size, MALLOC_ALIGN);
        let new_size = HEADER.force_add(new_size.force_into());
        memory = rust::realloc(memory, layout, new_size);
        if memory.is_null() {
            return die();
        }
        memory.cast::<usize>().write(new_size);
        memory.add(HEADER).cast()
    }
    /// Frees allocated memory.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it deallocates memory pointed to by `ptr`.
    /// The caller must ensure that `ptr` was previously allocated by `malloc` or `realloc`.
    pub unsafe fn free(ptr: *mut libc::c_void) {
        let memory = ptr.cast::<u8>().sub(HEADER);
        let size = memory.cast::<usize>().read();
        let layout = Layout::from_size_align_unchecked(size, MALLOC_ALIGN);
        rust::dealloc(memory, layout);
    }
    /// Compares two memory blocks.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it compares raw memory. The caller must ensure the pointers
    /// and size are correct.
    pub(crate) unsafe fn memcmp(
        lhs: *const libc::c_void,
        rhs: *const libc::c_void,
        count: libc::c_ulong,
    ) -> libc::c_int {
        let lhs = slice::from_raw_parts(lhs.cast::<u8>(), count as usize);
        let rhs = slice::from_raw_parts(rhs.cast::<u8>(), count as usize);
        lhs.cmp(rhs) as libc::c_int
    }
    /// Copies memory from `src` to `dest`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the memory areas do not overlap and that the destination is large
    /// enough to hold the data.
    pub(crate) unsafe fn memcpy(
        dest: *mut libc::c_void,
        src: *const libc::c_void,
        count: libc::c_ulong,
    ) -> *mut libc::c_void {
        if dest.is_null() || src.is_null() {
            return die();
        }
        ptr::copy_nonoverlapping(
            src.cast::<MaybeUninit<u8>>(),
            dest.cast::<MaybeUninit<u8>>(),
            count as usize,
        );
        dest
    }
    /// Moves memory from `src` to `dest`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the memory areas do not overlap or are correctly managed.
    pub unsafe fn memmove(
        dest: *mut libc::c_void,
        src: *const libc::c_void,
        count: libc::c_ulong,
    ) -> *mut libc::c_void {
        if dest.is_null() || src.is_null() {
            return die();
        }
        ptr::copy(
            src.cast::<MaybeUninit<u8>>(),
            dest.cast::<MaybeUninit<u8>>(),
            count as usize,
        );
        dest
    }
    /// Fills memory with a constant byte.
    ///
    /// # Safety
    ///
    /// The caller must ensure the memory is valid and the length is correct.
    pub unsafe fn memset(
        dest: *mut libc::c_void,
        ch: libc::c_int,
        count: libc::c_ulong,
    ) -> *mut libc::c_void {
        ptr::write_bytes(dest.cast::<u8>(), ch as u8, count as usize);
        dest
    }
    /// Compares two strings.
    ///
    /// # Safety
    ///
    /// The caller must ensure the strings are null-terminated and valid.
    pub(crate) unsafe fn strcmp(
        lhs: *const libc::c_char,
        rhs: *const libc::c_char,
    ) -> libc::c_int {
        if lhs.is_null() || rhs.is_null() {
            return die();
        }
        let lhs = slice::from_raw_parts(lhs.cast::<u8>(), strlen(lhs) as usize);
        let rhs = slice::from_raw_parts(rhs.cast::<u8>(), strlen(rhs) as usize);
        lhs.cmp(rhs) as libc::c_int
    }
    /// Returns the length of a string.
    ///
    /// # Safety
    ///
    /// The caller must ensure the string is null-terminated and valid.
    pub(crate) unsafe fn strlen(str: *const libc::c_char) -> libc::c_ulong {
        let mut end = str;
        while *end != 0 {
            end = end.add(1);
        }
        end.offset_from(str) as libc::c_ulong
    }
    /// Compares up to `count` characters of two strings.
    ///
    /// # Safety
    ///
    /// The caller must ensure the strings are null-terminated and valid.
    pub(crate) unsafe fn strncmp(
        lhs: *const libc::c_char,
        rhs: *const libc::c_char,
        mut count: libc::c_ulong,
    ) -> libc::c_int {
        if lhs.is_null() || rhs.is_null() {
            return die();
        }
        let mut lhs = lhs.cast::<u8>();
        let mut rhs = rhs.cast::<u8>();
        while count > 0 && *lhs != 0 && *lhs == *rhs {
            lhs = lhs.add(1);
            rhs = rhs.add(1);
            count -= 1;
        }
        if count == 0 { 0 } else { (*lhs).cmp(&*rhs) as libc::c_int }
    }
    /// Internal function for handling assertion failures.
    ///
    /// # Safety
    ///
    /// This function is called when an assertion fails, and it triggers a panic.
    pub(crate) unsafe fn __assert_fail(
        __assertion: &'static str,
        __file: &'static str,
        __line: u32,
    ) -> ! {
        struct Abort;
        impl Drop for Abort {
            fn drop(&mut self) {
                {
                    ::core::panicking::panic_fmt(format_args!("arithmetic overflow"));
                };
            }
        }
        let _abort_on_panic = Abort;
        {
            ::core::panicking::panic_fmt(
                format_args!(
                    "{0}:{1}: Assertion `{2}` failed.",
                    __file,
                    __line,
                    __assertion,
                ),
            );
        };
    }
}
/// Module for formatting utilities.
///
/// This module provides utilities for formatting data,
/// particularly for writing formatted strings to pointers.
mod fmt {
    use crate::yaml::yaml_char_t;
    use core::fmt::{self, Write};
    use core::ptr;
    /// Struct for writing formatted data to a pointer.
    pub(crate) struct WriteToPtr {
        ptr: *mut yaml_char_t,
    }
    impl WriteToPtr {
        /// Creates a new `WriteToPtr`.
        ///
        /// # Safety
        ///
        /// This function is unsafe because it directly manipulates raw pointers.
        pub(crate) unsafe fn new(ptr: *mut yaml_char_t) -> Self {
            WriteToPtr { ptr }
        }
        /// Writes formatted data to the pointer.
        pub(crate) fn write_fmt(&mut self, args: fmt::Arguments<'_>) {
            let _ = Write::write_fmt(self, args);
        }
    }
    impl Write for WriteToPtr {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            unsafe {
                ptr::copy_nonoverlapping(s.as_ptr(), self.ptr, s.len());
                self.ptr = self.ptr.add(s.len());
            }
            Ok(())
        }
    }
}
/// Trait extension for pointers.
///
/// This trait provides methods for working with pointers,
/// particularly for calculating the offset between two pointers.
trait PointerExt: Sized {
    fn c_offset_from(self, origin: Self) -> isize;
}
impl<T> PointerExt for *const T {
    fn c_offset_from(self, origin: *const T) -> isize {
        (self as isize - origin as isize) / size_of::<T>() as isize
    }
}
impl<T> PointerExt for *mut T {
    fn c_offset_from(self, origin: *mut T) -> isize {
        (self as isize - origin as isize) / size_of::<T>() as isize
    }
}
/// Macros module for LibYML.
///
/// This module contains various macros used throughout the LibYML library.
#[macro_use]
pub mod macros {}
/// Utility functions for LibYML.
///
/// This module contains utility functions and macros that are used throughout the LibYML library.
#[macro_use]
pub mod utils {
    /// A macro for `memory_macros` module.
    pub mod memory_macros {
        //! Macros for memory management operations.
        //!
        //! This module provides a set of macros that wrap the unsafe memory management
        //! functions, providing a slightly safer interface while still allowing
        //! for low-level memory operations in a no_std environment.
    }
}
/// API module for LibYML.
///
/// This module provides the public API functions for working with YAML data.
pub mod api {
    use crate::{
        externs::{memcpy, memset, strlen},
        internal::yaml_check_utf8, libc, memory::{yaml_free, yaml_malloc, yaml_strdup},
        ops::ForceAdd as _, success::{Success, FAIL, OK},
        yaml::{size_t, yaml_char_t},
        PointerExt, YamlAliasEvent, YamlAliasToken, YamlAnchorToken, YamlAnyEncoding,
        YamlBreakT, YamlEmitterStateT, YamlEmitterT, YamlEncodingT, YamlEventT,
        YamlEventTypeT::{YamlDocumentStartEvent, YamlScalarEvent, YamlStreamEndEvent},
        YamlMappingEndEvent, YamlMappingStartEvent, YamlMappingStyleT, YamlMarkT,
        YamlParserT, YamlReadHandlerT, YamlScalarStyleT, YamlScalarToken,
        YamlSequenceEndEvent, YamlSequenceStartEvent, YamlSequenceStyleT,
        YamlStreamStartEvent, YamlTagDirectiveT, YamlTagDirectiveToken, YamlTagToken,
        YamlTokenT, YamlWriteHandlerT,
    };
    use core::{mem::size_of, ptr::{self, addr_of_mut}};
    const OUTPUT_BUFFER_SIZE: usize = 16384;
    const OUTPUT_RAW_BUFFER_SIZE: usize = OUTPUT_BUFFER_SIZE * 2 + 2;
    unsafe fn yaml_string_read_handler(
        data: *mut libc::c_void,
        buffer: *mut libc::c_uchar,
        mut size: size_t,
        size_read: *mut size_t,
    ) -> libc::c_int {
        let parser: *mut YamlParserT = data as *mut YamlParserT;
        if (*parser).input.string.current == (*parser).input.string.end {
            *size_read = 0_u64;
            return 1;
        }
        if size
            > (*parser).input.string.end.c_offset_from((*parser).input.string.current)
                as size_t
        {
            size = (*parser)
                .input
                .string
                .end
                .c_offset_from((*parser).input.string.current) as size_t;
        }
        let _ = memcpy(
            buffer as *mut libc::c_void,
            (*parser).input.string.current as *const libc::c_void,
            size,
        );
        let fresh80 = &raw mut (*parser).input.string.current;
        *fresh80 = (*fresh80).wrapping_offset(size as isize);
        *size_read = size;
        1
    }
    /// Set a string input.
    ///
    /// This function sets the input source for the parser to a string buffer.
    /// Note that the `input` pointer must be valid while the `parser` object
    /// exists. The application is responsible for destroying `input` after
    /// destroying the `parser`.
    ///
    /// # Safety
    ///
    /// - `parser` must be a valid, non-null pointer to a properly initialized `YamlParserT` struct.
    /// - The `YamlParserT` struct must not have an input handler already set.
    /// - `input` must be a valid, non-null pointer to a null-terminated string buffer.
    /// - The `input` string buffer must remain valid and unmodified until the `parser` object is destroyed.
    /// - The `YamlParserT` struct and its associated data structures must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_parser_set_input_string(
        parser: *mut YamlParserT,
        input: *const libc::c_uchar,
        size: size_t,
    ) {
        if !!parser.is_null() {
            ::core::panicking::panic("assertion failed: !parser.is_null()")
        }
        if !(*parser).read_handler.is_none() {
            ::core::panicking::panic(
                "assertion failed: (*parser).read_handler.is_none()",
            )
        }
        if !!input.is_null() {
            ::core::panicking::panic("assertion failed: !input.is_null()")
        }
        (*parser).read_handler = Some(yaml_string_read_handler);
        (*parser).read_handler_data = parser as *mut libc::c_void;
        (*parser).input.string.start = input;
        (*parser).input.string.current = input;
        (*parser).input.string.end = input.wrapping_offset(size as isize);
    }
    /// Set a generic input handler.
    ///
    /// This function sets a custom input handler for the parser.
    ///
    /// # Safety
    ///
    /// - `parser` must be a valid, non-null pointer to a properly initialized `YamlParserT` struct.
    /// - The `YamlParserT` struct must not have an input handler already set.
    /// - `handler` must be a valid function pointer that follows the signature of `YamlReadHandlerT`.
    /// - `data` must be a valid pointer that will be passed to the `handler` function.
    /// - The `YamlParserT` struct and its associated data structures must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_parser_set_input(
        parser: *mut YamlParserT,
        handler: YamlReadHandlerT,
        data: *mut libc::c_void,
    ) {
        if !!parser.is_null() {
            crate::externs::__assert_fail("!parser.is_null()", "src/api.rs", 114u32);
        }
        if !((*parser).read_handler).is_none() {
            crate::externs::__assert_fail(
                "((*parser).read_handler).is_none()",
                "src/api.rs",
                115u32,
            );
        }
        let fresh89 = &raw mut (*parser).read_handler;
        *fresh89 = Some(handler);
        let fresh90 = &raw mut (*parser).read_handler_data;
        *fresh90 = data;
    }
    /// Set the source encoding.
    ///
    /// This function sets the expected encoding of the input source for the parser.
    ///
    /// # Safety
    ///
    /// - `parser` must be a valid, non-null pointer to a properly initialized `YamlParserT` struct.
    /// - The `YamlParserT` struct must not have an encoding already set, or the encoding must be `YamlAnyEncoding`.
    /// - The `YamlParserT` struct and its associated data structures must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_parser_set_encoding(
        parser: *mut YamlParserT,
        encoding: YamlEncodingT,
    ) {
        if !!parser.is_null() {
            crate::externs::__assert_fail("!parser.is_null()", "src/api.rs", 136u32);
        }
        if !((*parser).encoding == YamlAnyEncoding) {
            crate::externs::__assert_fail(
                "(*parser).encoding == YamlAnyEncoding",
                "src/api.rs",
                137u32,
            );
        }
        (*parser).encoding = encoding;
    }
    /// Initialize an emitter.
    ///
    /// This function creates a new emitter object. An application is responsible
    /// for destroying the object using the yaml_emitter_delete() function.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to an uninitialized `YamlEmitterT` struct.
    /// - The `YamlEmitterT` struct must be properly aligned and have the expected memory layout.
    /// - The caller is responsible for properly destroying the emitter object using `yaml_emitter_delete`.
    ///
    pub unsafe fn yaml_emitter_initialize(emitter: *mut YamlEmitterT) -> Success {
        if !!emitter.is_null() {
            crate::externs::__assert_fail("!emitter.is_null()", "src/api.rs", 155u32);
        }
        let _ = memset(
            emitter as *mut libc::c_void,
            0,
            size_of::<YamlEmitterT>() as libc::c_ulong,
        );
        {
            let start = &raw mut (*emitter).buffer.start;
            *start = yaml_malloc(OUTPUT_BUFFER_SIZE as size_t) as *mut yaml_char_t;
            if !start.is_null() {
                let _ = memset(
                    *start as *mut libc::c_void,
                    0,
                    OUTPUT_BUFFER_SIZE as u64,
                );
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for buffer"),
                    );
                };
            }
            let pointer = &raw mut (*emitter).buffer.pointer;
            *pointer = *start;
            let last = &raw mut (*emitter).buffer.last;
            *last = *pointer;
            let end = &raw mut (*emitter).buffer.end;
            *end = (*start).wrapping_add(OUTPUT_BUFFER_SIZE);
        };
        {
            let start = &raw mut (*emitter).raw_buffer.start;
            *start = yaml_malloc(OUTPUT_RAW_BUFFER_SIZE as size_t) as *mut yaml_char_t;
            if !start.is_null() {
                let _ = memset(
                    *start as *mut libc::c_void,
                    0,
                    OUTPUT_RAW_BUFFER_SIZE as u64,
                );
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for buffer"),
                    );
                };
            }
            let pointer = &raw mut (*emitter).raw_buffer.pointer;
            *pointer = *start;
            let last = &raw mut (*emitter).raw_buffer.last;
            *last = *pointer;
            let end = &raw mut (*emitter).raw_buffer.end;
            *end = (*start).wrapping_add(OUTPUT_RAW_BUFFER_SIZE);
        };
        {
            (*emitter).states.start = yaml_malloc(
                16 * size_of::<YamlEmitterStateT>() as libc::c_ulong,
            ) as *mut YamlEmitterStateT;
            (*emitter).states.top = (*emitter).states.start;
            (*emitter).states.end = (*emitter).states.start.offset(16_isize);
        };
        {
            (*emitter).events.start = yaml_malloc(
                16 * size_of::<YamlEventT>() as libc::c_ulong,
            ) as *mut YamlEventT;
            (*emitter).events.tail = (*emitter).events.start;
            (*emitter).events.head = (*emitter).events.tail;
            (*emitter).events.end = (*emitter).events.start.offset(16_isize);
        };
        {
            (*emitter).indents.start = yaml_malloc(
                16 * size_of::<libc::c_int>() as libc::c_ulong,
            ) as *mut libc::c_int;
            (*emitter).indents.top = (*emitter).indents.start;
            (*emitter).indents.end = (*emitter).indents.start.offset(16_isize);
        };
        {
            (*emitter).tag_directives.start = yaml_malloc(
                16 * size_of::<YamlTagDirectiveT>() as libc::c_ulong,
            ) as *mut YamlTagDirectiveT;
            (*emitter).tag_directives.top = (*emitter).tag_directives.start;
            (*emitter).tag_directives.end = (*emitter)
                .tag_directives
                .start
                .offset(16_isize);
        };
        OK
    }
    /// Destroy an emitter.
    ///
    /// This function frees all memory associated with an emitter object, including
    /// any dynamically allocated buffers, events, and other data structures.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT` struct.
    /// - The `YamlEmitterT` struct and its associated data structures must have been properly initialized and their memory allocated correctly.
    /// - The `YamlEmitterT` struct and its associated data structures must be properly aligned and have the expected memory layout.
    /// - After calling this function, the `emitter` pointer should be considered invalid and should not be used again.
    ///
    pub unsafe fn yaml_emitter_delete(emitter: *mut YamlEmitterT) {
        if !!emitter.is_null() {
            crate::externs::__assert_fail("!emitter.is_null()", "src/api.rs", 183u32);
        }
        {
            yaml_free((*emitter).buffer.start as *mut libc::c_void);
            (*emitter).buffer.start = ptr::null_mut::<yaml_char_t>();
            (*emitter).buffer.pointer = ptr::null_mut::<yaml_char_t>();
            (*emitter).buffer.last = ptr::null_mut::<yaml_char_t>();
            (*emitter).buffer.end = ptr::null_mut::<yaml_char_t>();
        };
        {
            yaml_free((*emitter).raw_buffer.start as *mut libc::c_void);
            (*emitter).raw_buffer.start = ptr::null_mut::<yaml_char_t>();
            (*emitter).raw_buffer.pointer = ptr::null_mut::<yaml_char_t>();
            (*emitter).raw_buffer.last = ptr::null_mut::<yaml_char_t>();
            (*emitter).raw_buffer.end = ptr::null_mut::<yaml_char_t>();
        };
        yaml_free((*emitter).states.start as *mut libc::c_void);
        (*emitter).states.end = ptr::null_mut();
        (*emitter).states.top = ptr::null_mut();
        (*emitter).states.start = ptr::null_mut();
        while !((*emitter).events.head == (*emitter).events.tail) {
            yaml_event_delete(
                &raw mut *{
                    let head = (*emitter).events.head;
                    (*emitter).events.head = (*emitter).events.head.wrapping_offset(1);
                    head
                },
            );
        }
        yaml_free((*emitter).events.start as *mut libc::c_void);
        (*emitter).events.end = ptr::null_mut();
        (*emitter).events.tail = ptr::null_mut();
        (*emitter).events.head = ptr::null_mut();
        (*emitter).events.start = ptr::null_mut();
        yaml_free((*emitter).indents.start as *mut libc::c_void);
        (*emitter).indents.end = ptr::null_mut();
        (*emitter).indents.top = ptr::null_mut();
        (*emitter).indents.start = ptr::null_mut();
        while !((*emitter).tag_directives.start == (*emitter).tag_directives.top) {
            let tag_directive = *{
                (*emitter).tag_directives.top = (*emitter).tag_directives.top.offset(-1);
                (*emitter).tag_directives.top
            };
            yaml_free(tag_directive.handle as *mut libc::c_void);
            yaml_free(tag_directive.prefix as *mut libc::c_void);
        }
        yaml_free((*emitter).tag_directives.start as *mut libc::c_void);
        (*emitter).tag_directives.end = ptr::null_mut();
        (*emitter).tag_directives.top = ptr::null_mut();
        (*emitter).tag_directives.start = ptr::null_mut();
        yaml_free((*emitter).anchors as *mut libc::c_void);
        let _ = memset(
            emitter as *mut libc::c_void,
            0,
            size_of::<YamlEmitterT>() as libc::c_ulong,
        );
    }
    unsafe fn yaml_string_write_handler(
        data: *mut libc::c_void,
        buffer: *mut libc::c_uchar,
        size: size_t,
    ) -> libc::c_int {
        let emitter: *mut YamlEmitterT = data as *mut YamlEmitterT;
        if (*emitter)
            .output
            .string
            .size
            .wrapping_sub(*(*emitter).output.string.size_written) < size
        {
            let _ = memcpy(
                (*emitter)
                    .output
                    .string
                    .buffer
                    .wrapping_offset(*(*emitter).output.string.size_written as isize)
                    as *mut libc::c_void,
                buffer as *const libc::c_void,
                (*emitter)
                    .output
                    .string
                    .size
                    .wrapping_sub(*(*emitter).output.string.size_written),
            );
            *(*emitter).output.string.size_written = (*emitter).output.string.size;
            return 0;
        }
        let _ = memcpy(
            (*emitter)
                .output
                .string
                .buffer
                .wrapping_offset(*(*emitter).output.string.size_written as isize)
                as *mut libc::c_void,
            buffer as *const libc::c_void,
            size,
        );
        let fresh153 = &raw mut (*(*emitter).output.string.size_written);
        *fresh153 = (*fresh153).wrapping_add(size);
        1
    }
    /// Set a string output.
    ///
    /// This function sets the output destination for the emitter to a string buffer.
    /// The emitter will write the output characters to the `output` buffer of the
    /// specified `size`. The emitter will set `size_written` to the number of written
    /// bytes. If the buffer is smaller than required, the emitter produces the
    /// YAML_write_ERROR error.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT` struct.
    /// - The `YamlEmitterT` struct must not have an output handler already set.
    /// - `output` must be a valid, non-null pointer to a writeable buffer of size `size`.
    /// - `size_written` must be a valid, non-null pointer to a `size_t` variable.
    /// - The `output` buffer must remain valid and unmodified until the emitter is destroyed or the output is reset.
    /// - The `YamlEmitterT` struct and its associated data structures must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_emitter_set_output_string(
        emitter: *mut YamlEmitterT,
        output: *mut libc::c_uchar,
        size: size_t,
        size_written: *mut size_t,
    ) {
        if !!emitter.is_null() {
            ::core::panicking::panic("assertion failed: !emitter.is_null()")
        }
        if !(*emitter).write_handler.is_none() {
            ::core::panicking::panic(
                "assertion failed: (*emitter).write_handler.is_none()",
            )
        }
        if !!output.is_null() {
            ::core::panicking::panic("assertion failed: !output.is_null()")
        }
        (*emitter).write_handler = Some(yaml_string_write_handler);
        (*emitter).write_handler_data = emitter as *mut libc::c_void;
        (*emitter).output.string.buffer = output;
        (*emitter).output.string.size = size;
        *size_written = 0;
    }
    /// Set a generic output handler.
    ///
    /// This function sets a custom output handler for the emitter.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT` struct.
    /// - The `YamlEmitterT` struct must not have an output handler already set.
    /// - `handler` must be a valid function pointer that follows the signature of `YamlWriteHandlerT`.
    /// - `data` must be a valid pointer that will be passed to the `handler` function.
    /// - The `YamlEmitterT` struct and its associated data structures must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_emitter_set_output(
        emitter: *mut YamlEmitterT,
        handler: YamlWriteHandlerT,
        data: *mut libc::c_void,
    ) {
        if !!emitter.is_null() {
            crate::externs::__assert_fail("!emitter.is_null()", "src/api.rs", 297u32);
        }
        if !((*emitter).write_handler).is_none() {
            crate::externs::__assert_fail(
                "((*emitter).write_handler).is_none()",
                "src/api.rs",
                298u32,
            );
        }
        let fresh161 = &raw mut (*emitter).write_handler;
        *fresh161 = Some(handler);
        let fresh162 = &raw mut (*emitter).write_handler_data;
        *fresh162 = data;
    }
    /// Set the output encoding.
    ///
    /// This function sets the encoding to be used for the output by the emitter.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT` struct.
    /// - The `YamlEmitterT` struct must not have an encoding already set, or the encoding must be `YamlAnyEncoding`.
    /// - The `YamlEmitterT` struct and its associated data structures must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_emitter_set_encoding(
        emitter: *mut YamlEmitterT,
        encoding: YamlEncodingT,
    ) {
        if !!emitter.is_null() {
            crate::externs::__assert_fail("!emitter.is_null()", "src/api.rs", 319u32);
        }
        if !((*emitter).encoding == YamlAnyEncoding) {
            crate::externs::__assert_fail(
                "(*emitter).encoding == YamlAnyEncoding",
                "src/api.rs",
                320u32,
            );
        }
        (*emitter).encoding = encoding;
    }
    /// Set if the output should be in the "canonical" format as in the YAML
    /// specification.
    ///
    /// This function sets whether the emitter should produce output in the canonical
    /// format, as defined by the YAML specification.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT` struct.
    /// - The `YamlEmitterT` struct and its associated data structures must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_emitter_set_canonical(
        emitter: *mut YamlEmitterT,
        canonical: bool,
    ) {
        if !!emitter.is_null() {
            crate::externs::__assert_fail("!emitter.is_null()", "src/api.rs", 339u32);
        }
        (*emitter).canonical = canonical;
    }
    /// Set the indentation increment.
    ///
    /// This function sets the indentation increment to be used by the emitter when
    /// emitting indented content.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT` struct.
    /// - The `YamlEmitterT` struct and its associated data structures must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_emitter_set_indent(
        emitter: *mut YamlEmitterT,
        indent: libc::c_int,
    ) {
        if !!emitter.is_null() {
            crate::externs::__assert_fail("!emitter.is_null()", "src/api.rs", 357u32);
        }
        (*emitter).best_indent = if 1 < indent && indent < 10 { indent } else { 2 };
    }
    /// Set the preferred line width. -1 means unlimited.
    ///
    /// This function sets the preferred line width for the emitter's output.
    /// A value of -1 means that the line width is unlimited.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT` struct.
    /// - The `YamlEmitterT` struct and its associated data structures must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_emitter_set_width(
        emitter: *mut YamlEmitterT,
        width: libc::c_int,
    ) {
        if !!emitter.is_null() {
            crate::externs::__assert_fail("!emitter.is_null()", "src/api.rs", 376u32);
        }
        (*emitter).best_width = if width >= 0 { width } else { -1 };
    }
    /// Set if unescaped non-ASCII characters are allowed.
    ///
    /// This function sets whether the emitter should allow unescaped non-ASCII
    /// characters in its output.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT` struct.
    /// - The `YamlEmitterT` struct and its associated data structures must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_emitter_set_unicode(emitter: *mut YamlEmitterT, unicode: bool) {
        if !!emitter.is_null() {
            crate::externs::__assert_fail("!emitter.is_null()", "src/api.rs", 394u32);
        }
        (*emitter).unicode = unicode;
    }
    /// Set the preferred line break.
    ///
    /// This function sets the preferred line break character to be used by the emitter.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT` struct.
    /// - The `YamlEmitterT` struct and its associated data structures must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_emitter_set_break(
        emitter: *mut YamlEmitterT,
        line_break: YamlBreakT,
    ) {
        if !!emitter.is_null() {
            crate::externs::__assert_fail("!emitter.is_null()", "src/api.rs", 411u32);
        }
        (*emitter).line_break = line_break;
    }
    /// Free any memory allocated for a token object.
    ///
    /// This function frees the dynamically allocated memory associated with a `YamlTokenT` struct,
    /// such as strings for tag directives, aliases, anchors, tags, and scalar values.
    ///
    /// # Safety
    ///
    /// - `token` must be a valid, non-null pointer to a `YamlTokenT` struct.
    /// - The `YamlTokenT` struct must have been properly initialized and its memory allocated correctly.
    /// - The `YamlTokenT` struct must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_token_delete(token: *mut YamlTokenT) {
        if !!token.is_null() {
            crate::externs::__assert_fail("!token.is_null()", "src/api.rs", 427u32);
        }
        match (*token).type_ {
            YamlTagDirectiveToken => {
                yaml_free((*token).data.tag_directive.handle as *mut libc::c_void);
                yaml_free((*token).data.tag_directive.prefix as *mut libc::c_void);
            }
            YamlAliasToken => {
                yaml_free((*token).data.alias.value as *mut libc::c_void);
            }
            YamlAnchorToken => {
                yaml_free((*token).data.anchor.value as *mut libc::c_void);
            }
            YamlTagToken => {
                yaml_free((*token).data.tag.handle as *mut libc::c_void);
                yaml_free((*token).data.tag.suffix as *mut libc::c_void);
            }
            YamlScalarToken => {
                yaml_free((*token).data.scalar.value as *mut libc::c_void);
            }
            _ => {}
        }
        let _ = memset(
            token as *mut libc::c_void,
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
    }
    /// Create the STREAM-START event.
    ///
    /// This function initializes a `YamlEventT` struct with the type `YamlStreamStartEvent`.
    /// It is used to signal the start of a YAML stream being emitted.
    ///
    /// # Safety
    ///
    /// - `event` must be a valid, non-null pointer to a `YamlEventT` struct that can be safely written to.
    /// - The `YamlEventT` struct must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_stream_start_event_initialize(
        event: *mut YamlEventT,
        encoding: YamlEncodingT,
    ) -> Success {
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        if !!event.is_null() {
            crate::externs::__assert_fail("!event.is_null()", "src/api.rs", 478u32);
        }
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlStreamStartEvent;
        (*event).start_mark = mark;
        (*event).end_mark = mark;
        (*event).data.stream_start.encoding = encoding;
        OK
    }
    /// Create the STREAM-END event.
    ///
    /// This function initializes a `YamlEventT` struct with the type `YamlStreamEndEvent`.
    /// It is used to signal the end of a YAML stream being emitted.
    ///
    /// # Safety
    ///
    /// - `event` must be a valid, non-null pointer to a `YamlEventT` struct that can be safely written to.
    /// - The `YamlEventT` struct must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_stream_end_event_initialize(event: *mut YamlEventT) -> Success {
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        if !!event.is_null() {
            crate::externs::__assert_fail("!event.is_null()", "src/api.rs", 509u32);
        }
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlStreamEndEvent;
        (*event).start_mark = mark;
        (*event).end_mark = mark;
        OK
    }
    /// Create an ALIAS event.
    ///
    /// # Safety
    ///
    /// - `event` must be a valid, non-null pointer to a `YamlEventT` struct that can be safely written to.
    /// - `anchor` must be a valid, non-null pointer to a null-terminated UTF-8 string.
    /// - The `YamlEventT` struct must be properly aligned and have the expected memory layout.
    /// - The caller is responsible for freeing any dynamically allocated memory associated with the event using `yaml_event_delete`.
    ///
    pub unsafe fn yaml_alias_event_initialize(
        event: *mut YamlEventT,
        anchor: *const yaml_char_t,
    ) -> Success {
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        if !!event.is_null() {
            crate::externs::__assert_fail("!event.is_null()", "src/api.rs", 539u32);
        }
        if !!anchor.is_null() {
            crate::externs::__assert_fail("!anchor.is_null()", "src/api.rs", 540u32);
        }
        if yaml_check_utf8(anchor, strlen(anchor as *mut libc::c_char)).fail {
            return FAIL;
        }
        let anchor_copy: *mut yaml_char_t = yaml_strdup(anchor);
        if anchor_copy.is_null() {
            return FAIL;
        }
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlAliasEvent;
        (*event).start_mark = mark;
        (*event).end_mark = mark;
        let fresh167 = &raw mut (*event).data.alias.anchor;
        *fresh167 = anchor_copy;
        OK
    }
    /// Create a SCALAR event.
    ///
    /// The `style` argument may be ignored by the emitter.
    ///
    /// Either the `tag` attribute or one of the `plain_implicit` and
    /// `quoted_implicit` flags must be set.
    ///
    /// # Safety
    ///
    /// - `event` must be a valid, non-null pointer to a `YamlEventT` struct that can be safely written to.
    /// - `data.value` must be a valid, non-null pointer to a null-terminated UTF-8 string.
    /// - `data.anchor`, if not null, must be a valid pointer to a null-terminated UTF-8 string.
    /// - `data.tag`, if not null, must be a valid pointer to a null-terminated UTF-8 string.
    /// - The `YamlEventT` struct must be properly aligned and have the expected memory layout.
    /// - The caller is responsible for freeing any dynamically allocated memory associated with the event using `yaml_event_delete`.
    ///
    #[repr(C)]
    pub struct ScalarEventData<'a> {
        /// Anchor name or null.
        pub anchor: *const yaml_char_t,
        /// Tag or null.
        pub tag: *const yaml_char_t,
        /// Value.
        pub value: *const yaml_char_t,
        /// Value length.
        pub length: libc::c_int,
        /// Is the tag optional for the plain style?
        pub plain_implicit: bool,
        /// Is the tag optional for any non-plain style?
        pub quoted_implicit: bool,
        /// Scalar style.
        pub style: YamlScalarStyleT,
        /// Lifetime marker.
        pub _marker: core::marker::PhantomData<&'a ()>,
    }
    #[automatically_derived]
    impl<'a> ::core::marker::Copy for ScalarEventData<'a> {}
    #[automatically_derived]
    impl<'a> ::core::clone::Clone for ScalarEventData<'a> {
        #[inline]
        fn clone(&self) -> ScalarEventData<'a> {
            let _: ::core::clone::AssertParamIsClone<*const yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<*const yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<*const yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<libc::c_int>;
            let _: ::core::clone::AssertParamIsClone<bool>;
            let _: ::core::clone::AssertParamIsClone<YamlScalarStyleT>;
            let _: ::core::clone::AssertParamIsClone<core::marker::PhantomData<&'a ()>>;
            *self
        }
    }
    #[automatically_derived]
    impl<'a> ::core::fmt::Debug for ScalarEventData<'a> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "anchor",
                "tag",
                "value",
                "length",
                "plain_implicit",
                "quoted_implicit",
                "style",
                "_marker",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.anchor,
                &self.tag,
                &self.value,
                &self.length,
                &self.plain_implicit,
                &self.quoted_implicit,
                &self.style,
                &&self._marker,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "ScalarEventData",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl<'a> ::core::marker::StructuralPartialEq for ScalarEventData<'a> {}
    #[automatically_derived]
    impl<'a> ::core::cmp::PartialEq for ScalarEventData<'a> {
        #[inline]
        fn eq(&self, other: &ScalarEventData<'a>) -> bool {
            self.anchor == other.anchor && self.tag == other.tag
                && self.value == other.value && self.length == other.length
                && self.plain_implicit == other.plain_implicit
                && self.quoted_implicit == other.quoted_implicit
                && self.style == other.style && self._marker == other._marker
        }
    }
    #[automatically_derived]
    impl<'a> ::core::cmp::Eq for ScalarEventData<'a> {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<*const yaml_char_t>;
            let _: ::core::cmp::AssertParamIsEq<*const yaml_char_t>;
            let _: ::core::cmp::AssertParamIsEq<*const yaml_char_t>;
            let _: ::core::cmp::AssertParamIsEq<libc::c_int>;
            let _: ::core::cmp::AssertParamIsEq<bool>;
            let _: ::core::cmp::AssertParamIsEq<YamlScalarStyleT>;
            let _: ::core::cmp::AssertParamIsEq<core::marker::PhantomData<&'a ()>>;
        }
    }
    /// Create a SCALAR event.
    ///
    /// The `style` argument may be ignored by the emitter.
    ///
    /// Either the `tag` attribute or one of the `plain_implicit` and
    /// `quoted_implicit` flags must be set.
    ///
    /// # Safety
    ///
    /// - `event` must be a valid, non-null pointer to a `YamlEventT` struct that can be safely written to.
    /// - `value` must be a valid, non-null pointer to a null-terminated UTF-8 string.
    /// - `anchor`, if not null, must be a valid pointer to a null-terminated UTF-8 string.
    /// - `tag`, if not null, must be a valid pointer to a null-terminated UTF-8 string.
    /// - The `YamlEventT` struct must be properly aligned and have the expected memory layout.
    /// - The caller is responsible for freeing any dynamically allocated memory associated with the event using `yaml_event_delete`.
    ///
    pub unsafe fn yaml_scalar_event_initialize(
        event: *mut YamlEventT,
        mut data: ScalarEventData<'_>,
    ) -> Success {
        let mut current_block: u64;
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        let mut anchor_copy: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let mut tag_copy: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let mut value_copy: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        if !!event.is_null() {
            crate::externs::__assert_fail("!event.is_null()", "src/api.rs", 631u32);
        }
        if !!data.value.is_null() {
            crate::externs::__assert_fail("!data.value.is_null()", "src/api.rs", 632u32);
        }
        if !data.anchor.is_null() {
            if yaml_check_utf8(data.anchor, strlen(data.anchor as *mut libc::c_char))
                .fail
            {
                current_block = 16285396129609901221;
            } else {
                anchor_copy = yaml_strdup(data.anchor);
                if anchor_copy.is_null() {
                    current_block = 16285396129609901221;
                } else {
                    current_block = 8515828400728868193;
                }
            }
        } else {
            current_block = 8515828400728868193;
        }
        if current_block == 8515828400728868193 {
            if !data.tag.is_null() {
                if yaml_check_utf8(data.tag, strlen(data.tag as *mut libc::c_char)).fail
                {
                    current_block = 16285396129609901221;
                } else {
                    tag_copy = yaml_strdup(data.tag);
                    if tag_copy.is_null() {
                        current_block = 16285396129609901221;
                    } else {
                        current_block = 12800627514080957624;
                    }
                }
            } else {
                current_block = 12800627514080957624;
            }
            if current_block != 16285396129609901221 {
                if data.length < 0 {
                    data.length = strlen(data.value as *mut libc::c_char) as libc::c_int;
                }
                if yaml_check_utf8(data.value, data.length as size_t).ok {
                    value_copy = yaml_malloc(data.length.force_add(1) as size_t)
                        as *mut yaml_char_t;
                    let _ = memcpy(
                        value_copy as *mut libc::c_void,
                        data.value as *const libc::c_void,
                        data.length as libc::c_ulong,
                    );
                    *value_copy.wrapping_offset(data.length as isize) = b'\0';
                    let _ = memset(
                        event as *mut libc::c_void,
                        0,
                        size_of::<YamlEventT>() as libc::c_ulong,
                    );
                    (*event).type_ = YamlScalarEvent;
                    (*event).start_mark = mark;
                    (*event).end_mark = mark;
                    let fresh168 = &raw mut (*event).data.scalar.anchor;
                    *fresh168 = anchor_copy;
                    let fresh169 = &raw mut (*event).data.scalar.tag;
                    *fresh169 = tag_copy;
                    let fresh170 = &raw mut (*event).data.scalar.value;
                    *fresh170 = value_copy;
                    (*event).data.scalar.length = data.length as size_t;
                    (*event).data.scalar.plain_implicit = data.plain_implicit;
                    (*event).data.scalar.quoted_implicit = data.quoted_implicit;
                    (*event).data.scalar.style = data.style;
                    return OK;
                }
            }
        }
        yaml_free(anchor_copy as *mut libc::c_void);
        yaml_free(tag_copy as *mut libc::c_void);
        yaml_free(value_copy as *mut libc::c_void);
        FAIL
    }
    /// Create a SEQUENCE-START event.
    ///
    /// The `style` argument may be ignored by the emitter.
    ///
    /// Either the `tag` attribute or the `implicit` flag must be set.
    ///
    /// # Safety
    ///
    /// - `event` must be a valid, non-null pointer to a `YamlEventT` struct that can be safely written to.
    /// - `anchor`, if not null, must be a valid pointer to a null-terminated UTF-8 string.
    /// - `tag`, if not null, must be a valid pointer to a null-terminated UTF-8 string.
    /// - The `YamlEventT` struct must be properly aligned and have the expected memory layout.
    /// - The caller is responsible for freeing any dynamically allocated memory associated with the event using `yaml_event_delete`.
    ///
    pub unsafe fn yaml_sequence_start_event_initialize(
        event: *mut YamlEventT,
        anchor: *const yaml_char_t,
        tag: *const yaml_char_t,
        implicit: bool,
        style: YamlSequenceStyleT,
    ) -> Success {
        let mut current_block: u64;
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        let mut anchor_copy: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let mut tag_copy: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        if !!event.is_null() {
            crate::externs::__assert_fail("!event.is_null()", "src/api.rs", 754u32);
        }
        if !anchor.is_null() {
            if yaml_check_utf8(anchor, strlen(anchor as *mut libc::c_char)).fail {
                current_block = 8817775685815971442;
            } else {
                anchor_copy = yaml_strdup(anchor);
                if anchor_copy.is_null() {
                    current_block = 8817775685815971442;
                } else {
                    current_block = 11006700562992250127;
                }
            }
        } else {
            current_block = 11006700562992250127;
        }
        if current_block == 11006700562992250127 {
            if !tag.is_null() {
                if yaml_check_utf8(tag, strlen(tag as *mut libc::c_char)).fail {
                    current_block = 8817775685815971442;
                } else {
                    tag_copy = yaml_strdup(tag);
                    if tag_copy.is_null() {
                        current_block = 8817775685815971442;
                    } else {
                        current_block = 7651349459974463963;
                    }
                }
            } else {
                current_block = 7651349459974463963;
            }
            if current_block != 8817775685815971442 {
                let _ = memset(
                    event as *mut libc::c_void,
                    0,
                    size_of::<YamlEventT>() as libc::c_ulong,
                );
                (*event).type_ = YamlSequenceStartEvent;
                (*event).start_mark = mark;
                (*event).end_mark = mark;
                let fresh171 = &raw mut (*event).data.sequence_start.anchor;
                *fresh171 = anchor_copy;
                let fresh172 = &raw mut (*event).data.sequence_start.tag;
                *fresh172 = tag_copy;
                (*event).data.sequence_start.implicit = implicit;
                (*event).data.sequence_start.style = style;
                return OK;
            }
        }
        yaml_free(anchor_copy as *mut libc::c_void);
        yaml_free(tag_copy as *mut libc::c_void);
        FAIL
    }
    /// Create a SEQUENCE-END event.
    ///
    /// This function initializes a `YamlEventT` struct with the type `YamlSequenceEndEvent`.
    /// It is used to signal the end of a sequence in the YAML document being emitted.
    ///
    /// # Safety
    ///
    /// - `event` must be a valid, non-null pointer to a `YamlEventT` struct that can be safely written to.
    /// - The `YamlEventT` struct must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_sequence_end_event_initialize(event: *mut YamlEventT) -> Success {
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        if !!event.is_null() {
            crate::externs::__assert_fail("!event.is_null()", "src/api.rs", 831u32);
        }
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlSequenceEndEvent;
        (*event).start_mark = mark;
        (*event).end_mark = mark;
        OK
    }
    /// Create a MAPPING-START event.
    ///
    /// This function initializes a `YamlEventT` struct with the type `YamlMappingStartEvent`.
    /// It is used to signal the start of a mapping (key-value pairs) in the YAML document being emitted.
    ///
    /// The `style` argument may be ignored by the emitter.
    ///
    /// Either the `tag` attribute or the `implicit` flag must be set.
    ///
    /// # Safety
    ///
    /// - `event` must be a valid, non-null pointer to a `YamlEventT` struct that can be safely written to.
    /// - `anchor`, if not null, must be a valid pointer to a null-terminated UTF-8 string.
    /// - `tag`, if not null, must be a valid pointer to a null-terminated UTF-8 string.
    /// - The `YamlEventT` struct must be properly aligned and have the expected memory layout.
    /// - The caller is responsible for freeing any dynamically allocated memory associated with the event using `yaml_event_delete`.
    ///
    pub unsafe fn yaml_mapping_start_event_initialize(
        event: *mut YamlEventT,
        anchor: *const yaml_char_t,
        tag: *const yaml_char_t,
        implicit: bool,
        style: YamlMappingStyleT,
    ) -> Success {
        let mut current_block: u64;
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        let mut anchor_copy: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let mut tag_copy: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        if !!event.is_null() {
            crate::externs::__assert_fail("!event.is_null()", "src/api.rs", 876u32);
        }
        if !anchor.is_null() {
            if yaml_check_utf8(anchor, strlen(anchor as *mut libc::c_char)).fail {
                current_block = 14748279734549812740;
            } else {
                anchor_copy = yaml_strdup(anchor);
                if anchor_copy.is_null() {
                    current_block = 14748279734549812740;
                } else {
                    current_block = 11006700562992250127;
                }
            }
        } else {
            current_block = 11006700562992250127;
        }
        if current_block == 11006700562992250127 {
            if !tag.is_null() {
                if yaml_check_utf8(tag, strlen(tag as *mut libc::c_char)).fail {
                    current_block = 14748279734549812740;
                } else {
                    tag_copy = yaml_strdup(tag);
                    if tag_copy.is_null() {
                        current_block = 14748279734549812740;
                    } else {
                        current_block = 7651349459974463963;
                    }
                }
            } else {
                current_block = 7651349459974463963;
            }
            if current_block != 14748279734549812740 {
                let _ = memset(
                    event as *mut libc::c_void,
                    0,
                    size_of::<YamlEventT>() as libc::c_ulong,
                );
                (*event).type_ = YamlMappingStartEvent;
                (*event).start_mark = mark;
                (*event).end_mark = mark;
                let fresh173 = &raw mut (*event).data.mapping_start.anchor;
                *fresh173 = anchor_copy;
                let fresh174 = &raw mut (*event).data.mapping_start.tag;
                *fresh174 = tag_copy;
                (*event).data.mapping_start.implicit = implicit;
                (*event).data.mapping_start.style = style;
                return OK;
            }
        }
        yaml_free(anchor_copy as *mut libc::c_void);
        yaml_free(tag_copy as *mut libc::c_void);
        FAIL
    }
    /// Create a MAPPING-END event.
    ///
    /// This function initializes a `YamlEventT` struct with the type `YamlMappingEndEvent`.
    /// It is used to signal the end of a mapping (key-value pairs) in the YAML document being emitted.
    ///
    /// # Safety
    ///
    /// - `event` must be a valid, non-null pointer to a `YamlEventT` struct that can be safely written to.
    /// - The `YamlEventT` struct must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_mapping_end_event_initialize(event: *mut YamlEventT) -> Success {
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        if !!event.is_null() {
            crate::externs::__assert_fail("!event.is_null()", "src/api.rs", 953u32);
        }
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlMappingEndEvent;
        (*event).start_mark = mark;
        (*event).end_mark = mark;
        OK
    }
    /// Free any memory allocated for an event object.
    ///
    /// This function frees the dynamically allocated memory associated with a `YamlEventT` struct,
    /// such as strings for anchors, tags, and scalar values.
    ///
    /// # Safety
    ///
    /// - `event` must be a valid, non-null pointer to a `YamlEventT` struct.
    /// - The `YamlEventT` struct must have been properly initialized and its memory allocated correctly.
    /// - The `YamlEventT` struct must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_event_delete(event: *mut YamlEventT) {
        let mut tag_directive: *mut YamlTagDirectiveT;
        if !!event.is_null() {
            crate::externs::__assert_fail("!event.is_null()", "src/api.rs", 978u32);
        }
        match (*event).type_ {
            YamlDocumentStartEvent => {
                yaml_free(
                    (*event).data.document_start.version_directive as *mut libc::c_void,
                );
                tag_directive = (*event).data.document_start.tag_directives.start;
                while tag_directive != (*event).data.document_start.tag_directives.end {
                    yaml_free((*tag_directive).handle as *mut libc::c_void);
                    yaml_free((*tag_directive).prefix as *mut libc::c_void);
                    tag_directive = tag_directive.wrapping_offset(1);
                }
                yaml_free(
                    (*event).data.document_start.tag_directives.start
                        as *mut libc::c_void,
                );
            }
            YamlAliasEvent => {
                yaml_free((*event).data.alias.anchor as *mut libc::c_void);
            }
            YamlScalarEvent => {
                yaml_free((*event).data.scalar.anchor as *mut libc::c_void);
                yaml_free((*event).data.scalar.tag as *mut libc::c_void);
                yaml_free((*event).data.scalar.value as *mut libc::c_void);
            }
            YamlSequenceStartEvent => {
                yaml_free((*event).data.sequence_start.anchor as *mut libc::c_void);
                yaml_free((*event).data.sequence_start.tag as *mut libc::c_void);
            }
            YamlMappingStartEvent => {
                yaml_free((*event).data.mapping_start.anchor as *mut libc::c_void);
                yaml_free((*event).data.mapping_start.tag as *mut libc::c_void);
            }
            _ => {}
        }
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
    }
}
/// String utilities for LibYML.
///
/// This module provides utilities for working with YAML strings.
pub mod string {
    use crate::{
        externs::memset, libc, memory::{yaml_realloc, yaml_strdup},
        yaml::{size_t, yaml_char_t},
    };
    /// Extend a string buffer by reallocating and copying the existing data.
    ///
    /// # Safety
    ///
    /// - This function is unsafe because it directly calls the system's `realloc` and
    ///   `memset` functions, which can lead to undefined behaviour if misused.
    /// - The caller must ensure that `start`, `pointer`, and `end` are valid pointers
    ///   into the same allocated memory block.
    /// - The caller must ensure that the memory block being extended is large enough
    ///   to accommodate the new size.
    /// - The caller is responsible for properly freeing the extended memory block using
    ///   the corresponding `yaml_free` function when it is no longer needed.
    ///
    pub unsafe fn yaml_string_extend(
        start: *mut *mut yaml_char_t,
        pointer: *mut *mut yaml_char_t,
        end: *mut *mut yaml_char_t,
    ) {
        let current_size: size_t = (*end).offset_from(*start) as size_t;
        let old_offset: size_t = (*pointer).offset_from(*start) as size_t;
        let new_size: size_t = current_size * 2;
        let new_start = yaml_realloc((*start).cast::<libc::c_void>(), new_size)
            as *mut yaml_char_t;
        if new_start.is_null() {
            {
                ::core::panicking::panic_fmt(
                    format_args!("yaml_string_extend: reallocation failed"),
                );
            };
        }
        memset(
            new_start.add(current_size as usize).cast::<libc::c_void>(),
            0,
            current_size,
        );
        *start = new_start;
        *pointer = new_start.add(old_offset as usize);
        *end = new_start.add(new_size as usize);
    }
    /// Duplicate a null-terminated string.
    /// # Safety
    /// - This function is unsafe because it involves memory allocation.
    pub unsafe fn yaml_string_duplicate(str: *const yaml_char_t) -> *mut yaml_char_t {
        yaml_strdup(str)
    }
    /// Join two string buffers by copying data from one to the other.
    ///
    /// This function is used to concatenate two string buffers.
    ///
    /// # Safety
    ///
    /// - This function is unsafe because it directly calls the system's `memcpy` function,
    ///   which can lead to undefined behaviour if misused.
    /// - The caller must ensure that `a_start`, `a_pointer`, `a_end`, `b_start`, `b_pointer`,
    ///   and `b_end` are valid pointers into their respective allocated memory blocks.
    /// - The caller must ensure that the memory blocks being joined are large enough to
    ///   accommodate the combined data.
    /// - The caller is responsible for properly freeing the joined memory block using
    ///   the corresponding `yaml_free` function when it is no longer needed.
    ///
    pub unsafe fn yaml_string_join(
        a_start: *mut *mut yaml_char_t,
        a_pointer: *mut *mut yaml_char_t,
        a_end: *mut *mut yaml_char_t,
        b_start: *mut *mut yaml_char_t,
        b_pointer: *mut *mut yaml_char_t,
        b_end: *mut *mut yaml_char_t,
    ) {
        if *b_start == *b_pointer {
            return;
        }
        let b_length = ((*b_pointer).offset_from(*b_start))
            .min((*b_end).offset_from(*b_start)) as usize;
        if b_length == 0 {
            return;
        }
        while ((*a_end).offset_from(*a_pointer) as usize) < b_length {
            yaml_string_extend(a_start, a_pointer, a_end);
        }
        core::ptr::copy_nonoverlapping(*b_start, *a_pointer, b_length);
        *a_pointer = (*a_pointer).add(b_length);
    }
}
/// Dumper module for LibYML.
///
/// This module contains functions related to dumping YAML data.
pub mod dumper {
    use crate::externs::{memset, strcmp};
    use crate::fmt::WriteToPtr;
    use crate::memory::yaml_free;
    use crate::memory::yaml_malloc;
    use crate::ops::ForceMul as _;
    use crate::success::{Success, FAIL, OK};
    use crate::yaml::{
        yaml_char_t, YamlAliasEvent, YamlAnchorsT, YamlAnyEncoding, YamlDocumentEndEvent,
        YamlDocumentStartEvent, YamlDocumentT, YamlEmitterT, YamlEventT,
        YamlMappingEndEvent, YamlMappingNode, YamlMappingStartEvent, YamlMarkT,
        YamlNodeItemT, YamlNodePairT, YamlNodeT, YamlScalarEvent, YamlScalarNode,
        YamlSequenceEndEvent, YamlSequenceNode, YamlSequenceStartEvent,
        YamlStreamEndEvent, YamlStreamStartEvent,
    };
    use crate::{libc, yaml_document_delete, yaml_emitter_emit, PointerExt};
    use core::mem::{size_of, MaybeUninit};
    use core::ptr::{self, addr_of_mut};
    /// Start a YAML stream.
    ///
    /// This function should be used before yaml_emitter_dump() is called.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT` struct.
    /// - The `YamlEmitterT` struct must not be in an opened state.
    /// - The `YamlEmitterT` struct must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_emitter_open(emitter: *mut YamlEmitterT) -> Success {
        if emitter.is_null() {
            return FAIL;
        }
        if (*emitter).opened {
            return FAIL;
        }
        if (*emitter).closed {
            (*emitter).closed = false;
        }
        let mut event = MaybeUninit::<YamlEventT>::uninit();
        let event_ptr = event.as_mut_ptr();
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        let _ = memset(
            event_ptr as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event_ptr).type_ = YamlStreamStartEvent;
        (*event_ptr).start_mark = mark;
        (*event_ptr).end_mark = mark;
        (*event_ptr).data.stream_start.encoding = YamlAnyEncoding;
        if yaml_emitter_emit(emitter, event_ptr).fail {
            return FAIL;
        }
        (*emitter).opened = true;
        (*emitter).closed = false;
        OK
    }
    /// Finish a YAML stream.
    ///
    /// This function should be used after yaml_emitter_dump() is called.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT` struct.
    /// - The `YamlEmitterT` struct must be in an opened state and not already closed.
    /// - The `YamlEmitterT` struct must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_emitter_close(emitter: *mut YamlEmitterT) -> Success {
        if emitter.is_null() {
            return FAIL;
        }
        if !(*emitter).opened {
            return OK;
        }
        if (*emitter).closed {
            return OK;
        }
        let mut event = MaybeUninit::<YamlEventT>::uninit();
        let event = event.as_mut_ptr();
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlStreamEndEvent;
        (*event).start_mark = mark;
        (*event).end_mark = mark;
        if yaml_emitter_emit(emitter, event).fail {
            return FAIL;
        }
        (*emitter).closed = true;
        (*emitter).opened = false;
        OK
    }
    /// Emit a YAML document.
    ///
    /// The document object may be generated using the yaml_parser_load() function or
    /// the yaml_document_initialize() function. The emitter takes the
    /// responsibility for the document object and destroys its content after it is
    /// emitted. The document object is destroyed even if the function fails.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT` struct.
    /// - `document` must be a valid, non-null pointer to a `YamlDocumentT` struct that can be safely read from and will be destroyed by the function.
    /// - The `YamlEmitterT` and `YamlDocumentT` structs must be properly aligned and have the expected memory layout.
    /// - The `YamlEmitterT` struct must be in a valid state to emit the provided document.
    ///
    pub unsafe fn yaml_emitter_dump(
        emitter: *mut YamlEmitterT,
        document: *mut YamlDocumentT,
    ) -> Success {
        let mut event = MaybeUninit::<YamlEventT>::uninit();
        let event = event.as_mut_ptr();
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        if !!emitter.is_null() {
            crate::externs::__assert_fail("!emitter.is_null()", "src/dumper.rs", 150u32);
        }
        if !!document.is_null() {
            crate::externs::__assert_fail(
                "!document.is_null()",
                "src/dumper.rs",
                151u32,
            );
        }
        let fresh0 = &raw mut (*emitter).document;
        *fresh0 = document;
        if !(*emitter).opened && yaml_emitter_open(emitter).fail {
            return FAIL;
        }
        if (*document).nodes.start == (*document).nodes.top {
            if yaml_emitter_close(emitter).ok {
                yaml_emitter_delete_document_and_anchors(emitter);
                return OK;
            }
        } else {
            if !(*emitter).opened {
                crate::externs::__assert_fail(
                    "(*emitter).opened",
                    "src/dumper.rs",
                    166u32,
                );
            }
            let fresh1 = &raw mut (*emitter).anchors;
            *fresh1 = yaml_malloc(
                (size_of::<YamlAnchorsT>() as libc::c_ulong)
                    .force_mul(
                        (*document).nodes.top.c_offset_from((*document).nodes.start)
                            as libc::c_ulong,
                    ),
            ) as *mut YamlAnchorsT;
            let _ = memset(
                (*emitter).anchors as *mut libc::c_void,
                0,
                (size_of::<YamlAnchorsT>() as libc::c_ulong)
                    .force_mul(
                        (*document).nodes.top.c_offset_from((*document).nodes.start)
                            as libc::c_ulong,
                    ),
            );
            let _ = memset(
                event as *mut libc::c_void,
                0,
                size_of::<YamlEventT>() as libc::c_ulong,
            );
            (*event).type_ = YamlDocumentStartEvent;
            (*event).start_mark = mark;
            (*event).end_mark = mark;
            (*event).data.document_start.version_directive = (*document)
                .version_directive;
            (*event).data.document_start.tag_directives.start = (*document)
                .tag_directives
                .start;
            (*event).data.document_start.tag_directives.end = (*document)
                .tag_directives
                .end;
            (*event).data.document_start.implicit = (*document).start_implicit;
            if yaml_emitter_emit(emitter, event).ok {
                yaml_emitter_anchor_node(emitter, 1);
                if yaml_emitter_dump_node(emitter, 1).ok {
                    let _ = memset(
                        event as *mut libc::c_void,
                        0,
                        size_of::<YamlEventT>() as libc::c_ulong,
                    );
                    (*event).type_ = YamlDocumentEndEvent;
                    (*event).start_mark = mark;
                    (*event).end_mark = mark;
                    (*event).data.document_end.implicit = (*document).end_implicit;
                    if yaml_emitter_emit(emitter, event).ok {
                        yaml_emitter_delete_document_and_anchors(emitter);
                        return OK;
                    }
                }
            }
        }
        yaml_emitter_delete_document_and_anchors(emitter);
        FAIL
    }
    unsafe fn yaml_emitter_delete_document_and_anchors(emitter: *mut YamlEmitterT) {
        let mut index: libc::c_int;
        if (*emitter).anchors.is_null() {
            yaml_document_delete((*emitter).document);
            let fresh2 = &raw mut (*emitter).document;
            *fresh2 = ptr::null_mut::<YamlDocumentT>();
            return;
        }
        index = 0;
        while (*(*emitter).document).nodes.start.wrapping_offset(index as isize)
            < (*(*emitter).document).nodes.top
        {
            let mut node: YamlNodeT = *(*(*emitter).document)
                .nodes
                .start
                .wrapping_offset(index as isize);
            if !(*(*emitter).anchors.wrapping_offset(index as isize)).serialized {
                yaml_free(node.tag as *mut libc::c_void);
                if node.type_ == YamlScalarNode {
                    yaml_free(node.data.scalar.value as *mut libc::c_void);
                }
            }
            if node.type_ == YamlSequenceNode {
                yaml_free(node.data.sequence.items.start as *mut libc::c_void);
                node.data.sequence.items.end = ptr::null_mut();
                node.data.sequence.items.top = ptr::null_mut();
                node.data.sequence.items.start = ptr::null_mut();
            }
            if node.type_ == YamlMappingNode {
                yaml_free(node.data.mapping.pairs.start as *mut libc::c_void);
                node.data.mapping.pairs.end = ptr::null_mut();
                node.data.mapping.pairs.top = ptr::null_mut();
                node.data.mapping.pairs.start = ptr::null_mut();
            }
            index += 1;
        }
        yaml_free((*(*emitter).document).nodes.start as *mut libc::c_void);
        (*(*emitter).document).nodes.end = ptr::null_mut();
        (*(*emitter).document).nodes.top = ptr::null_mut();
        (*(*emitter).document).nodes.start = ptr::null_mut();
        yaml_free((*emitter).anchors as *mut libc::c_void);
        let fresh6 = &raw mut (*emitter).anchors;
        *fresh6 = ptr::null_mut::<YamlAnchorsT>();
        (*emitter).last_anchor_id = 0;
        let fresh7 = &raw mut (*emitter).document;
        *fresh7 = ptr::null_mut::<YamlDocumentT>();
    }
    unsafe fn yaml_emitter_anchor_node_sub(
        emitter: *mut YamlEmitterT,
        index: libc::c_int,
    ) {
        (*((*emitter).anchors).offset((index - 1) as isize)).references += 1;
        if (*(*emitter).anchors.offset((index - 1) as isize)).references == 2 {
            (*emitter).last_anchor_id += 1;
            (*(*emitter).anchors.offset((index - 1) as isize)).anchor = (*emitter)
                .last_anchor_id;
        }
    }
    unsafe fn yaml_emitter_anchor_node(emitter: *mut YamlEmitterT, index: libc::c_int) {
        let node: *mut YamlNodeT = (*(*emitter).document)
            .nodes
            .start
            .wrapping_offset(index as isize)
            .wrapping_offset(-1_isize);
        let mut item: *mut YamlNodeItemT;
        let mut pair: *mut YamlNodePairT;
        let fresh8 = &raw mut (*((*emitter).anchors)
            .wrapping_offset((index - 1) as isize))
            .references;
        *fresh8 += 1;
        if (*(*emitter).anchors.wrapping_offset((index - 1) as isize)).references == 1 {
            match (*node).type_ {
                YamlSequenceNode => {
                    item = (*node).data.sequence.items.start;
                    while item < (*node).data.sequence.items.top {
                        yaml_emitter_anchor_node_sub(emitter, *item);
                        item = item.wrapping_offset(1);
                    }
                }
                YamlMappingNode => {
                    pair = (*node).data.mapping.pairs.start;
                    while pair < (*node).data.mapping.pairs.top {
                        yaml_emitter_anchor_node_sub(emitter, (*pair).key);
                        yaml_emitter_anchor_node_sub(emitter, (*pair).value);
                        pair = pair.wrapping_offset(1);
                    }
                }
                _ => {}
            }
        } else if (*(*emitter).anchors.wrapping_offset((index - 1) as isize)).references
            == 2
        {
            let fresh9 = &raw mut (*emitter).last_anchor_id;
            *fresh9 += 1;
            (*(*emitter).anchors.wrapping_offset((index - 1) as isize)).anchor = *fresh9;
        }
    }
    unsafe fn yaml_emitter_generate_anchor(
        _emitter: *mut YamlEmitterT,
        anchor_id: libc::c_int,
    ) -> *mut yaml_char_t {
        let anchor: *mut yaml_char_t = yaml_malloc(16_u64) as *mut yaml_char_t;
        WriteToPtr::new(anchor).write_fmt(format_args!("id{0:03}\0", anchor_id));
        anchor
    }
    /// Dumps a YAML node to the emitter.
    ///
    /// This function is responsible for emitting a single YAML node from a document.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to an initialized `YamlEmitterT` struct.
    /// - `index` must be a valid index within the YAML document associated with the emitter.
    /// - The caller must ensure that the node at `index` can be safely emitted without causing memory issues.
    pub unsafe fn yaml_emitter_dump_node(
        emitter: *mut YamlEmitterT,
        index: libc::c_int,
    ) -> Success {
        let node: *mut YamlNodeT = (*(*emitter).document)
            .nodes
            .start
            .wrapping_offset(index as isize)
            .wrapping_offset(-1_isize);
        let anchor_id: libc::c_int = (*(*emitter)
            .anchors
            .wrapping_offset((index - 1) as isize))
            .anchor;
        let mut anchor: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        if anchor_id != 0 {
            anchor = yaml_emitter_generate_anchor(emitter, anchor_id);
        }
        if (*(*emitter).anchors.wrapping_offset((index - 1) as isize)).serialized {
            return yaml_emitter_dump_alias(emitter, anchor);
        }
        (*(*emitter).anchors.wrapping_offset((index - 1) as isize)).serialized = true;
        match (*node).type_ {
            YamlScalarNode => yaml_emitter_dump_scalar(emitter, node, anchor),
            YamlSequenceNode => yaml_emitter_dump_sequence(emitter, node, anchor),
            YamlMappingNode => yaml_emitter_dump_mapping(emitter, node, anchor),
            _ => crate::externs::__assert_fail("false", "src/dumper.rs", 400u32),
        }
    }
    unsafe fn yaml_emitter_dump_alias(
        emitter: *mut YamlEmitterT,
        anchor: *mut yaml_char_t,
    ) -> Success {
        let mut event = MaybeUninit::<YamlEventT>::uninit();
        let event = event.as_mut_ptr();
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlAliasEvent;
        (*event).start_mark = mark;
        (*event).end_mark = mark;
        (*event).data.alias.anchor = anchor;
        yaml_emitter_emit(emitter, event)
    }
    /// Dumps a YAML scalar node to the emitter.
    ///
    /// This function handles emitting a scalar node, which is a single key-value pair.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to an initialized `YamlEmitterT` struct.
    /// - `node` must be a valid, non-null pointer to a `YamlNodeT` struct representing the scalar node.
    /// - `anchor` must be a valid, non-null pointer to a `yaml_char_t` if provided, or null if no anchor is used.
    /// - The caller must ensure that the node and anchor pointers are valid and properly aligned.
    pub unsafe fn yaml_emitter_dump_scalar(
        emitter: *mut YamlEmitterT,
        node: *mut YamlNodeT,
        anchor: *mut yaml_char_t,
    ) -> Success {
        let mut event = MaybeUninit::<YamlEventT>::uninit();
        let event = event.as_mut_ptr();
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        let plain_implicit = strcmp(
            (*node).tag as *mut libc::c_char,
            b"tag:yaml.org,2002:str\0" as *const u8 as *const libc::c_char,
        ) == 0;
        let quoted_implicit = strcmp(
            (*node).tag as *mut libc::c_char,
            b"tag:yaml.org,2002:str\0" as *const u8 as *const libc::c_char,
        ) == 0;
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlScalarEvent;
        (*event).start_mark = mark;
        (*event).end_mark = mark;
        (*event).data.scalar.anchor = anchor;
        (*event).data.scalar.tag = (*node).tag;
        (*event).data.scalar.value = (*node).data.scalar.value;
        (*event).data.scalar.length = (*node).data.scalar.length;
        (*event).data.scalar.plain_implicit = plain_implicit;
        (*event).data.scalar.quoted_implicit = quoted_implicit;
        (*event).data.scalar.style = (*node).data.scalar.style;
        yaml_emitter_emit(emitter, event)
    }
    /// Dumps a YAML sequence node to the emitter.
    ///
    /// This function handles emitting a sequence node, which is a list of items.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to an initialized `YamlEmitterT` struct.
    /// - `node` must be a valid, non-null pointer to a `YamlNodeT` struct representing the sequence node.
    /// - `anchor` must be a valid, non-null pointer to a `yaml_char_t` if provided, or null if no anchor is used.
    /// - The caller must ensure that the node and anchor pointers are valid and properly aligned.
    /// - The sequence node must contain a valid list of items that can be safely iterated and emitted.
    pub unsafe fn yaml_emitter_dump_sequence(
        emitter: *mut YamlEmitterT,
        node: *mut YamlNodeT,
        anchor: *mut yaml_char_t,
    ) -> Success {
        let mut event = MaybeUninit::<YamlEventT>::uninit();
        let event = event.as_mut_ptr();
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        let implicit = strcmp(
            (*node).tag as *mut libc::c_char,
            b"tag:yaml.org,2002:seq\0" as *const u8 as *const libc::c_char,
        ) == 0;
        let mut item: *mut YamlNodeItemT;
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlSequenceStartEvent;
        (*event).start_mark = mark;
        (*event).end_mark = mark;
        (*event).data.sequence_start.anchor = anchor;
        (*event).data.sequence_start.tag = (*node).tag;
        (*event).data.sequence_start.implicit = implicit;
        (*event).data.sequence_start.style = (*node).data.sequence.style;
        if yaml_emitter_emit(emitter, event).fail {
            return FAIL;
        }
        item = (*node).data.sequence.items.start;
        while item < (*node).data.sequence.items.top {
            if yaml_emitter_dump_node(emitter, *item).fail {
                return FAIL;
            }
            item = item.wrapping_offset(1);
        }
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlSequenceEndEvent;
        (*event).start_mark = mark;
        (*event).end_mark = mark;
        yaml_emitter_emit(emitter, event)
    }
    /// Dumps a YAML mapping node to the emitter.
    ///
    /// This function handles emitting a mapping node, which is a set of key-value pairs.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to an initialized `YamlEmitterT` struct.
    /// - `node` must be a valid, non-null pointer to a `YamlNodeT` struct representing the mapping node.
    /// - `anchor` must be a valid, non-null pointer to a `yaml_char_t` if provided, or null if no anchor is used.
    /// - The caller must ensure that the node and anchor pointers are valid and properly aligned.
    /// - The mapping node must contain a valid set of key-value pairs that can be safely iterated and emitted.
    pub unsafe fn yaml_emitter_dump_mapping(
        emitter: *mut YamlEmitterT,
        node: *mut YamlNodeT,
        anchor: *mut yaml_char_t,
    ) -> Success {
        let mut event = MaybeUninit::<YamlEventT>::uninit();
        let event = event.as_mut_ptr();
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        let implicit = strcmp(
            (*node).tag as *mut libc::c_char,
            b"tag:yaml.org,2002:map\0" as *const u8 as *const libc::c_char,
        ) == 0;
        let mut pair: *mut YamlNodePairT;
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlMappingStartEvent;
        (*event).start_mark = mark;
        (*event).end_mark = mark;
        (*event).data.mapping_start.anchor = anchor;
        (*event).data.mapping_start.tag = (*node).tag;
        (*event).data.mapping_start.implicit = implicit;
        (*event).data.mapping_start.style = (*node).data.mapping.style;
        if yaml_emitter_emit(emitter, event).fail {
            return FAIL;
        }
        pair = (*node).data.mapping.pairs.start;
        while pair < (*node).data.mapping.pairs.top {
            if yaml_emitter_dump_node(emitter, (*pair).key).fail {
                return FAIL;
            }
            if yaml_emitter_dump_node(emitter, (*pair).value).fail {
                return FAIL;
            }
            pair = pair.wrapping_offset(1);
        }
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlMappingEndEvent;
        (*event).start_mark = mark;
        (*event).end_mark = mark;
        yaml_emitter_emit(emitter, event)
    }
}
/// Emitter module for LibYML.
///
/// This module provides functions for emitting YAML data.
mod emitter {
    use crate::externs::{strcmp, strlen, strncmp};
    use crate::internal::{yaml_queue_extend, yaml_stack_extend};
    use crate::memory::{yaml_free, yaml_strdup};
    use crate::ops::{ForceAdd as _, ForceMul as _};
    use crate::success::{Success, FAIL, OK};
    use crate::yaml::{size_t, yaml_char_t, YamlStringT};
    use crate::{
        libc, yaml_emitter_flush, yaml_event_delete, PointerExt, YamlAliasEvent,
        YamlAnyBreak, YamlAnyEncoding, YamlAnyScalarStyle, YamlCrBreak, YamlCrlnBreak,
        YamlDocumentEndEvent, YamlDocumentStartEvent, YamlDoubleQuotedScalarStyle,
        YamlEmitBlockMappingFirstKeyState, YamlEmitBlockMappingKeyState,
        YamlEmitBlockMappingSimpleValueState, YamlEmitBlockMappingValueState,
        YamlEmitBlockSequenceFirstItemState, YamlEmitBlockSequenceItemState,
        YamlEmitDocumentContentState, YamlEmitDocumentEndState,
        YamlEmitDocumentStartState, YamlEmitEndState, YamlEmitFirstDocumentStartState,
        YamlEmitFlowMappingFirstKeyState, YamlEmitFlowMappingKeyState,
        YamlEmitFlowMappingSimpleValueState, YamlEmitFlowMappingValueState,
        YamlEmitFlowSequenceFirstItemState, YamlEmitFlowSequenceItemState,
        YamlEmitStreamStartState, YamlEmitterError, YamlEmitterT, YamlEventT,
        YamlFlowMappingStyle, YamlFlowSequenceStyle, YamlFoldedScalarStyle,
        YamlLiteralScalarStyle, YamlLnBreak, YamlMappingEndEvent, YamlMappingStartEvent,
        YamlPlainScalarStyle, YamlScalarEvent, YamlScalarStyleT, YamlSequenceEndEvent,
        YamlSequenceStartEvent, YamlSingleQuotedScalarStyle, YamlStreamEndEvent,
        YamlStreamStartEvent, YamlTagDirectiveT, YamlUtf8Encoding, YamlVersionDirectiveT,
    };
    use core::ptr::{self, addr_of_mut};
    unsafe fn flush(emitter: *mut YamlEmitterT) -> Success {
        if (*emitter).buffer.pointer.wrapping_offset(5_isize) < (*emitter).buffer.end {
            OK
        } else {
            yaml_emitter_flush(emitter)
        }
    }
    unsafe fn put(emitter: *mut YamlEmitterT, value: u8) -> Success {
        if flush(emitter).fail {
            return FAIL;
        }
        let fresh40 = &raw mut (*emitter).buffer.pointer;
        let fresh41 = *fresh40;
        *fresh40 = (*fresh40).wrapping_offset(1);
        *fresh41 = value;
        let fresh42 = &raw mut (*emitter).column;
        *fresh42 += 1;
        OK
    }
    unsafe fn put_break(emitter: *mut YamlEmitterT) -> Success {
        if flush(emitter).fail {
            return FAIL;
        }
        if (*emitter).line_break == YamlCrBreak {
            let fresh62 = &raw mut (*emitter).buffer.pointer;
            let fresh63 = *fresh62;
            *fresh62 = (*fresh62).wrapping_offset(1);
            *fresh63 = b'\r';
        } else if (*emitter).line_break == YamlLnBreak {
            let fresh64 = &raw mut (*emitter).buffer.pointer;
            let fresh65 = *fresh64;
            *fresh64 = (*fresh64).wrapping_offset(1);
            *fresh65 = b'\n';
        } else if (*emitter).line_break == YamlCrlnBreak {
            let fresh66 = &raw mut (*emitter).buffer.pointer;
            let fresh67 = *fresh66;
            *fresh66 = (*fresh66).wrapping_offset(1);
            *fresh67 = b'\r';
            let fresh68 = &raw mut (*emitter).buffer.pointer;
            let fresh69 = *fresh68;
            *fresh68 = (*fresh68).wrapping_offset(1);
            *fresh69 = b'\n';
        }
        (*emitter).column = 0;
        let fresh70 = &raw mut (*emitter).line;
        *fresh70 += 1;
        OK
    }
    unsafe fn write(emitter: *mut YamlEmitterT, string: *mut YamlStringT) -> Success {
        if flush(emitter).fail {
            return FAIL;
        }
        if *(*string).pointer & 0x80 == 0x00 {
            *(*emitter).buffer.pointer = *(*string).pointer;
            (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
            (*string).pointer = (*string).pointer.wrapping_offset(1);
        } else if *(*string).pointer & 0xE0 == 0xC0 {
            *(*emitter).buffer.pointer = *(*string).pointer;
            (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            *(*emitter).buffer.pointer = *(*string).pointer;
            (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
            (*string).pointer = (*string).pointer.wrapping_offset(1);
        } else if *(*string).pointer & 0xF0 == 0xE0 {
            *(*emitter).buffer.pointer = *(*string).pointer;
            (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            *(*emitter).buffer.pointer = *(*string).pointer;
            (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            *(*emitter).buffer.pointer = *(*string).pointer;
            (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
            (*string).pointer = (*string).pointer.wrapping_offset(1);
        } else if *(*string).pointer & 0xF8 == 0xF0 {
            *(*emitter).buffer.pointer = *(*string).pointer;
            (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            *(*emitter).buffer.pointer = *(*string).pointer;
            (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            *(*emitter).buffer.pointer = *(*string).pointer;
            (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            *(*emitter).buffer.pointer = *(*string).pointer;
            (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
            (*string).pointer = (*string).pointer.wrapping_offset(1);
        }
        let fresh107 = &raw mut (*emitter).column;
        *fresh107 += 1;
        OK
    }
    unsafe fn write_break(
        emitter: *mut YamlEmitterT,
        string: *mut YamlStringT,
    ) -> Success {
        if flush(emitter).fail {
            return FAIL;
        }
        if *(*string).pointer == b'\n' {
            let _ = put_break(emitter);
            (*string).pointer = (*string).pointer.wrapping_offset(1);
        } else {
            if *(*string).pointer & 0x80 == 0x00 {
                *(*emitter).buffer.pointer = *(*string).pointer;
                (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
                (*string).pointer = (*string).pointer.wrapping_offset(1);
            } else if *(*string).pointer & 0xE0 == 0xC0 {
                *(*emitter).buffer.pointer = *(*string).pointer;
                (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
                (*string).pointer = (*string).pointer.wrapping_offset(1);
                *(*emitter).buffer.pointer = *(*string).pointer;
                (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
                (*string).pointer = (*string).pointer.wrapping_offset(1);
            } else if *(*string).pointer & 0xF0 == 0xE0 {
                *(*emitter).buffer.pointer = *(*string).pointer;
                (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
                (*string).pointer = (*string).pointer.wrapping_offset(1);
                *(*emitter).buffer.pointer = *(*string).pointer;
                (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
                (*string).pointer = (*string).pointer.wrapping_offset(1);
                *(*emitter).buffer.pointer = *(*string).pointer;
                (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
                (*string).pointer = (*string).pointer.wrapping_offset(1);
            } else if *(*string).pointer & 0xF8 == 0xF0 {
                *(*emitter).buffer.pointer = *(*string).pointer;
                (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
                (*string).pointer = (*string).pointer.wrapping_offset(1);
                *(*emitter).buffer.pointer = *(*string).pointer;
                (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
                (*string).pointer = (*string).pointer.wrapping_offset(1);
                *(*emitter).buffer.pointer = *(*string).pointer;
                (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
                (*string).pointer = (*string).pointer.wrapping_offset(1);
                *(*emitter).buffer.pointer = *(*string).pointer;
                (*emitter).buffer.pointer = (*emitter).buffer.pointer.wrapping_offset(1);
                (*string).pointer = (*string).pointer.wrapping_offset(1);
            }
            (*emitter).column = 0;
            let fresh300 = &raw mut (*emitter).line;
            *fresh300 += 1;
        }
        OK
    }
    unsafe fn yaml_emitter_set_emitter_error(
        emitter: *mut YamlEmitterT,
        problem: *const libc::c_char,
    ) -> Success {
        (*emitter).error = YamlEmitterError;
        let fresh0 = &raw mut (*emitter).problem;
        *fresh0 = problem;
        FAIL
    }
    /// Emit an event.
    ///
    /// The event object may be generated using the yaml_parser_parse() function.
    /// The emitter takes the responsibility for the event object and destroys its
    /// content after it is emitted. The event object is destroyed even if the
    /// function fails.
    ///
    /// # Safety
    ///
    /// - `emitter` must be a valid, non-null pointer to a properly initialized `YamlEmitterT` struct.
    /// - `event` must be a valid, non-null pointer to a `YamlEventT` struct that can be safely read from and will be destroyed by the function.
    /// - The `YamlEmitterT` and `YamlEventT` structs must be properly aligned and have the expected memory layout.
    /// - The `YamlEmitterT` struct must be in a valid state to emit the provided event.
    pub unsafe fn yaml_emitter_emit(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
    ) -> Success {
        {
            if (*emitter).events.tail == (*emitter).events.end {
                yaml_queue_extend(
                    &raw mut (*emitter).events.start as *mut *mut libc::c_void,
                    &raw mut (*emitter).events.head as *mut *mut libc::c_void,
                    &raw mut (*emitter).events.tail as *mut *mut libc::c_void,
                    &raw mut (*emitter).events.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(event, (*emitter).events.tail, 1);
            (*emitter).events.tail = (*emitter).events.tail.wrapping_offset(1);
        };
        while yaml_emitter_need_more_events(emitter).fail {
            if yaml_emitter_analyze_event(emitter, (*emitter).events.head).fail {
                return FAIL;
            }
            if yaml_emitter_state_machine(emitter, (*emitter).events.head).fail {
                return FAIL;
            }
            yaml_event_delete(
                &raw mut *{
                    let head = (*emitter).events.head;
                    (*emitter).events.head = (*emitter).events.head.wrapping_offset(1);
                    head
                },
            );
        }
        OK
    }
    unsafe fn yaml_emitter_need_more_events(emitter: *mut YamlEmitterT) -> Success {
        let mut level: libc::c_int = 0;
        let mut event: *mut YamlEventT;
        if (*emitter).events.head == (*emitter).events.tail {
            return OK;
        }
        let accumulate = match (*(*emitter).events.head).type_ {
            YamlDocumentStartEvent => 1,
            YamlSequenceStartEvent => 2,
            YamlMappingStartEvent => 3,
            _ => return FAIL,
        };
        if (*emitter).events.tail.c_offset_from((*emitter).events.head) as libc::c_long
            > accumulate as libc::c_long
        {
            return FAIL;
        }
        event = (*emitter).events.head;
        while event != (*emitter).events.tail {
            match (*event).type_ {
                YamlStreamStartEvent
                | YamlDocumentStartEvent
                | YamlSequenceStartEvent
                | YamlMappingStartEvent => {
                    level += 1;
                }
                YamlStreamEndEvent
                | YamlDocumentEndEvent
                | YamlSequenceEndEvent
                | YamlMappingEndEvent => {
                    level -= 1;
                }
                _ => {}
            }
            if level == 0 {
                return FAIL;
            }
            event = event.wrapping_offset(1);
        }
        OK
    }
    unsafe fn yaml_emitter_append_tag_directive(
        emitter: *mut YamlEmitterT,
        value: YamlTagDirectiveT,
        allow_duplicates: bool,
    ) -> Success {
        let mut tag_directive: *mut YamlTagDirectiveT;
        let mut copy = YamlTagDirectiveT {
            handle: ptr::null_mut::<yaml_char_t>(),
            prefix: ptr::null_mut::<yaml_char_t>(),
        };
        tag_directive = (*emitter).tag_directives.start;
        while tag_directive != (*emitter).tag_directives.top {
            if strcmp(
                value.handle as *mut libc::c_char,
                (*tag_directive).handle as *mut libc::c_char,
            ) == 0
            {
                if allow_duplicates {
                    return OK;
                }
                return yaml_emitter_set_emitter_error(
                    emitter,
                    b"duplicate %TAG directive\0" as *const u8 as *const libc::c_char,
                );
            }
            tag_directive = tag_directive.wrapping_offset(1);
        }
        copy.handle = yaml_strdup(value.handle);
        copy.prefix = yaml_strdup(value.prefix);
        {
            if (*emitter).tag_directives.top == (*emitter).tag_directives.end {
                yaml_stack_extend(
                    &raw mut (*emitter).tag_directives.start as *mut *mut libc::c_void,
                    &raw mut (*emitter).tag_directives.top as *mut *mut libc::c_void,
                    &raw mut (*emitter).tag_directives.end as *mut *mut libc::c_void,
                );
            }
            ptr::write((*emitter).tag_directives.top, copy);
            (*emitter).tag_directives.top = (*emitter)
                .tag_directives
                .top
                .wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_emitter_increase_indent(
        emitter: *mut YamlEmitterT,
        flow: bool,
        indentless: bool,
    ) {
        {
            if (*emitter).indents.top == (*emitter).indents.end {
                yaml_stack_extend(
                    &raw mut (*emitter).indents.start as *mut *mut libc::c_void,
                    &raw mut (*emitter).indents.top as *mut *mut libc::c_void,
                    &raw mut (*emitter).indents.end as *mut *mut libc::c_void,
                );
            }
            ptr::write((*emitter).indents.top, (*emitter).indent);
            (*emitter).indents.top = (*emitter).indents.top.wrapping_offset(1);
        };
        if (*emitter).indent < 0 {
            (*emitter).indent = if flow { (*emitter).best_indent } else { 0 };
        } else if !indentless {
            (*emitter).indent += (*emitter).best_indent;
        }
    }
    unsafe fn yaml_emitter_state_machine(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
    ) -> Success {
        match (*emitter).state {
            YamlEmitStreamStartState => yaml_emitter_emit_stream_start(emitter, event),
            YamlEmitFirstDocumentStartState => {
                yaml_emitter_emit_document_start(emitter, event, true)
            }
            YamlEmitDocumentStartState => {
                yaml_emitter_emit_document_start(emitter, event, false)
            }
            YamlEmitDocumentContentState => {
                yaml_emitter_emit_document_content(emitter, event)
            }
            YamlEmitDocumentEndState => yaml_emitter_emit_document_end(emitter, event),
            YamlEmitFlowSequenceFirstItemState => {
                yaml_emitter_emit_flow_sequence_item(emitter, event, true)
            }
            YamlEmitFlowSequenceItemState => {
                yaml_emitter_emit_flow_sequence_item(emitter, event, false)
            }
            YamlEmitFlowMappingFirstKeyState => {
                yaml_emitter_emit_flow_mapping_key(emitter, event, true)
            }
            YamlEmitFlowMappingKeyState => {
                yaml_emitter_emit_flow_mapping_key(emitter, event, false)
            }
            YamlEmitFlowMappingSimpleValueState => {
                yaml_emitter_emit_flow_mapping_value(emitter, event, true)
            }
            YamlEmitFlowMappingValueState => {
                yaml_emitter_emit_flow_mapping_value(emitter, event, false)
            }
            YamlEmitBlockSequenceFirstItemState => {
                yaml_emitter_emit_block_sequence_item(emitter, event, true)
            }
            YamlEmitBlockSequenceItemState => {
                yaml_emitter_emit_block_sequence_item(emitter, event, false)
            }
            YamlEmitBlockMappingFirstKeyState => {
                yaml_emitter_emit_block_mapping_key(emitter, event, true)
            }
            YamlEmitBlockMappingKeyState => {
                yaml_emitter_emit_block_mapping_key(emitter, event, false)
            }
            YamlEmitBlockMappingSimpleValueState => {
                yaml_emitter_emit_block_mapping_value(emitter, event, true)
            }
            YamlEmitBlockMappingValueState => {
                yaml_emitter_emit_block_mapping_value(emitter, event, false)
            }
            YamlEmitEndState => {
                yaml_emitter_set_emitter_error(
                    emitter,
                    b"expected nothing after STREAM-END\0" as *const u8
                        as *const libc::c_char,
                )
            }
        }
    }
    unsafe fn yaml_emitter_emit_stream_start(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
    ) -> Success {
        (*emitter).open_ended = 0;
        if (*event).type_ == YamlStreamStartEvent {
            if (*emitter).encoding == YamlAnyEncoding {
                (*emitter).encoding = (*event).data.stream_start.encoding;
            }
            if (*emitter).encoding == YamlAnyEncoding {
                (*emitter).encoding = YamlUtf8Encoding;
            }
            if (*emitter).best_indent < 2 || (*emitter).best_indent > 9 {
                (*emitter).best_indent = 2;
            }
            if (*emitter).best_width >= 0
                && (*emitter).best_width <= (*emitter).best_indent.force_mul(2)
            {
                (*emitter).best_width = 80;
            }
            if (*emitter).best_width < 0 {
                (*emitter).best_width = libc::c_int::MAX;
            }
            if (*emitter).line_break == YamlAnyBreak {
                (*emitter).line_break = YamlLnBreak;
            }
            (*emitter).indent = -1;
            (*emitter).line = 0;
            (*emitter).column = 0;
            (*emitter).whitespace = true;
            (*emitter).indention = true;
            if (*emitter).encoding != YamlUtf8Encoding
                && yaml_emitter_write_bom(emitter).fail
            {
                return FAIL;
            }
            (*emitter).state = YamlEmitFirstDocumentStartState;
            return OK;
        }
        yaml_emitter_set_emitter_error(
            emitter,
            b"expected STREAM-START\0" as *const u8 as *const libc::c_char,
        )
    }
    unsafe fn yaml_emitter_emit_document_start(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
        first: bool,
    ) -> Success {
        if (*event).type_ == YamlDocumentStartEvent {
            let mut default_tag_directives: [YamlTagDirectiveT; 3] = [
                YamlTagDirectiveT {
                    handle: b"!\0" as *const u8 as *const libc::c_char
                        as *mut yaml_char_t,
                    prefix: b"!\0" as *const u8 as *const libc::c_char
                        as *mut yaml_char_t,
                },
                YamlTagDirectiveT {
                    handle: b"!!\0" as *const u8 as *const libc::c_char
                        as *mut yaml_char_t,
                    prefix: b"tag:yaml.org,2002:\0" as *const u8 as *const libc::c_char
                        as *mut yaml_char_t,
                },
                YamlTagDirectiveT {
                    handle: ptr::null_mut::<yaml_char_t>(),
                    prefix: ptr::null_mut::<yaml_char_t>(),
                },
            ];
            let mut tag_directive: *mut YamlTagDirectiveT;
            let mut implicit;
            if !(*event).data.document_start.version_directive.is_null()
                && yaml_emitter_analyze_version_directive(
                        emitter,
                        *(*event).data.document_start.version_directive,
                    )
                    .fail
            {
                return FAIL;
            }
            tag_directive = (*event).data.document_start.tag_directives.start;
            while tag_directive != (*event).data.document_start.tag_directives.end {
                if yaml_emitter_analyze_tag_directive(emitter, *tag_directive).fail {
                    return FAIL;
                }
                if yaml_emitter_append_tag_directive(emitter, *tag_directive, false).fail
                {
                    return FAIL;
                }
                tag_directive = tag_directive.wrapping_offset(1);
            }
            tag_directive = default_tag_directives.as_mut_ptr();
            while !(*tag_directive).handle.is_null() {
                if yaml_emitter_append_tag_directive(emitter, *tag_directive, true).fail
                {
                    return FAIL;
                }
                tag_directive = tag_directive.wrapping_offset(1);
            }
            implicit = (*event).data.document_start.implicit;
            if !first || (*emitter).canonical {
                implicit = false;
            }
            if (!(*event).data.document_start.version_directive.is_null()
                || (*event).data.document_start.tag_directives.start
                    != (*event).data.document_start.tag_directives.end)
                && (*emitter).open_ended != 0
            {
                if yaml_emitter_write_indicator(
                        emitter,
                        b"...\0" as *const u8 as *const libc::c_char,
                        true,
                        false,
                        false,
                    )
                    .fail
                {
                    return FAIL;
                }
                if yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
            }
            (*emitter).open_ended = 0;
            if !(*event).data.document_start.version_directive.is_null() {
                implicit = false;
                if yaml_emitter_write_indicator(
                        emitter,
                        b"%YAML\0" as *const u8 as *const libc::c_char,
                        true,
                        false,
                        false,
                    )
                    .fail
                {
                    return FAIL;
                }
                if (*(*event).data.document_start.version_directive).minor == 1 {
                    if yaml_emitter_write_indicator(
                            emitter,
                            b"1.1\0" as *const u8 as *const libc::c_char,
                            true,
                            false,
                            false,
                        )
                        .fail
                    {
                        return FAIL;
                    }
                } else if yaml_emitter_write_indicator(
                        emitter,
                        b"1.2\0" as *const u8 as *const libc::c_char,
                        true,
                        false,
                        false,
                    )
                    .fail
                {
                    return FAIL;
                }
                if yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
            }
            if (*event).data.document_start.tag_directives.start
                != (*event).data.document_start.tag_directives.end
            {
                implicit = false;
                tag_directive = (*event).data.document_start.tag_directives.start;
                while tag_directive != (*event).data.document_start.tag_directives.end {
                    if yaml_emitter_write_indicator(
                            emitter,
                            b"%TAG\0" as *const u8 as *const libc::c_char,
                            true,
                            false,
                            false,
                        )
                        .fail
                    {
                        return FAIL;
                    }
                    if yaml_emitter_write_tag_handle(
                            emitter,
                            (*tag_directive).handle,
                            strlen((*tag_directive).handle as *mut libc::c_char),
                        )
                        .fail
                    {
                        return FAIL;
                    }
                    if yaml_emitter_write_tag_content(
                            emitter,
                            (*tag_directive).prefix,
                            strlen((*tag_directive).prefix as *mut libc::c_char),
                            true,
                        )
                        .fail
                    {
                        return FAIL;
                    }
                    if yaml_emitter_write_indent(emitter).fail {
                        return FAIL;
                    }
                    tag_directive = tag_directive.wrapping_offset(1);
                }
            }
            if yaml_emitter_check_empty_document(emitter) {
                implicit = false;
            }
            if !implicit {
                if yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
                if yaml_emitter_write_indicator(
                        emitter,
                        b"---\0" as *const u8 as *const libc::c_char,
                        true,
                        false,
                        false,
                    )
                    .fail
                {
                    return FAIL;
                }
                if (*emitter).canonical && yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
            }
            (*emitter).state = YamlEmitDocumentContentState;
            (*emitter).open_ended = 0;
            return OK;
        } else if (*event).type_ == YamlStreamEndEvent {
            if (*emitter).open_ended == 2 {
                if yaml_emitter_write_indicator(
                        emitter,
                        b"...\0" as *const u8 as *const libc::c_char,
                        true,
                        false,
                        false,
                    )
                    .fail
                {
                    return FAIL;
                }
                (*emitter).open_ended = 0;
                if yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
            }
            if yaml_emitter_flush(emitter).fail {
                return FAIL;
            }
            (*emitter).state = YamlEmitEndState;
            return OK;
        }
        yaml_emitter_set_emitter_error(
            emitter,
            b"expected DOCUMENT-START or STREAM-END\0" as *const u8
                as *const libc::c_char,
        )
    }
    unsafe fn yaml_emitter_emit_document_content(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
    ) -> Success {
        {
            if (*emitter).states.top == (*emitter).states.end {
                yaml_stack_extend(
                    &raw mut (*emitter).states.start as *mut *mut libc::c_void,
                    &raw mut (*emitter).states.top as *mut *mut libc::c_void,
                    &raw mut (*emitter).states.end as *mut *mut libc::c_void,
                );
            }
            ptr::write((*emitter).states.top, YamlEmitDocumentEndState);
            (*emitter).states.top = (*emitter).states.top.wrapping_offset(1);
        };
        yaml_emitter_emit_node(emitter, event, true, false, false, false)
    }
    unsafe fn yaml_emitter_emit_document_end(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
    ) -> Success {
        if (*event).type_ == YamlDocumentEndEvent {
            if yaml_emitter_write_indent(emitter).fail {
                return FAIL;
            }
            if !(*event).data.document_end.implicit {
                if yaml_emitter_write_indicator(
                        emitter,
                        b"...\0" as *const u8 as *const libc::c_char,
                        true,
                        false,
                        false,
                    )
                    .fail
                {
                    return FAIL;
                }
                (*emitter).open_ended = 0;
                if yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
            } else if (*emitter).open_ended == 0 {
                (*emitter).open_ended = 1;
            }
            if yaml_emitter_flush(emitter).fail {
                return FAIL;
            }
            (*emitter).state = YamlEmitDocumentStartState;
            while !((*emitter).tag_directives.start == (*emitter).tag_directives.top) {
                let tag_directive = *{
                    (*emitter).tag_directives.top = (*emitter)
                        .tag_directives
                        .top
                        .offset(-1);
                    (*emitter).tag_directives.top
                };
                yaml_free(tag_directive.handle as *mut libc::c_void);
                yaml_free(tag_directive.prefix as *mut libc::c_void);
            }
            return OK;
        }
        yaml_emitter_set_emitter_error(
            emitter,
            b"expected DOCUMENT-END\0" as *const u8 as *const libc::c_char,
        )
    }
    unsafe fn yaml_emitter_emit_flow_sequence_item(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
        first: bool,
    ) -> Success {
        if first {
            if yaml_emitter_write_indicator(
                    emitter,
                    b"[\0" as *const u8 as *const libc::c_char,
                    true,
                    true,
                    false,
                )
                .fail
            {
                return FAIL;
            }
            yaml_emitter_increase_indent(emitter, true, false);
            let fresh12 = &raw mut (*emitter).flow_level;
            *fresh12 += 1;
        }
        if (*event).type_ == YamlSequenceEndEvent {
            let fresh13 = &raw mut (*emitter).flow_level;
            *fresh13 -= 1;
            (*emitter).indent = *{
                (*emitter).indents.top = (*emitter).indents.top.offset(-1);
                (*emitter).indents.top
            };
            if (*emitter).canonical && !first {
                if yaml_emitter_write_indicator(
                        emitter,
                        b",\0" as *const u8 as *const libc::c_char,
                        false,
                        false,
                        false,
                    )
                    .fail
                {
                    return FAIL;
                }
                if yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
            }
            if yaml_emitter_write_indicator(
                    emitter,
                    b"]\0" as *const u8 as *const libc::c_char,
                    false,
                    false,
                    false,
                )
                .fail
            {
                return FAIL;
            }
            (*emitter).state = *{
                (*emitter).states.top = (*emitter).states.top.offset(-1);
                (*emitter).states.top
            };
            return OK;
        }
        if !first
            && yaml_emitter_write_indicator(
                    emitter,
                    b",\0" as *const u8 as *const libc::c_char,
                    false,
                    false,
                    false,
                )
                .fail
        {
            return FAIL;
        }
        if ((*emitter).canonical || (*emitter).column > (*emitter).best_width)
            && yaml_emitter_write_indent(emitter).fail
        {
            return FAIL;
        }
        {
            if (*emitter).states.top == (*emitter).states.end {
                yaml_stack_extend(
                    &raw mut (*emitter).states.start as *mut *mut libc::c_void,
                    &raw mut (*emitter).states.top as *mut *mut libc::c_void,
                    &raw mut (*emitter).states.end as *mut *mut libc::c_void,
                );
            }
            ptr::write((*emitter).states.top, YamlEmitFlowSequenceItemState);
            (*emitter).states.top = (*emitter).states.top.wrapping_offset(1);
        };
        yaml_emitter_emit_node(emitter, event, false, true, false, false)
    }
    unsafe fn yaml_emitter_emit_flow_mapping_key(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
        first: bool,
    ) -> Success {
        if first {
            if yaml_emitter_write_indicator(
                    emitter,
                    b"{\0" as *const u8 as *const libc::c_char,
                    true,
                    true,
                    false,
                )
                .fail
            {
                return FAIL;
            }
            yaml_emitter_increase_indent(emitter, true, false);
            let fresh18 = &raw mut (*emitter).flow_level;
            *fresh18 += 1;
        }
        if (*event).type_ == YamlMappingEndEvent {
            if (*emitter).indents.start == (*emitter).indents.top {
                return FAIL;
            }
            let fresh19 = &raw mut (*emitter).flow_level;
            *fresh19 -= 1;
            (*emitter).indent = *{
                (*emitter).indents.top = (*emitter).indents.top.offset(-1);
                (*emitter).indents.top
            };
            if (*emitter).canonical && !first {
                if yaml_emitter_write_indicator(
                        emitter,
                        b",\0" as *const u8 as *const libc::c_char,
                        false,
                        false,
                        false,
                    )
                    .fail
                {
                    return FAIL;
                }
                if yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
            }
            if yaml_emitter_write_indicator(
                    emitter,
                    b"}\0" as *const u8 as *const libc::c_char,
                    false,
                    false,
                    false,
                )
                .fail
            {
                return FAIL;
            }
            (*emitter).state = *{
                (*emitter).states.top = (*emitter).states.top.offset(-1);
                (*emitter).states.top
            };
            return OK;
        }
        if !first
            && yaml_emitter_write_indicator(
                    emitter,
                    b",\0" as *const u8 as *const libc::c_char,
                    false,
                    false,
                    false,
                )
                .fail
        {
            return FAIL;
        }
        if ((*emitter).canonical || (*emitter).column > (*emitter).best_width)
            && yaml_emitter_write_indent(emitter).fail
        {
            return FAIL;
        }
        if !(*emitter).canonical && yaml_emitter_check_simple_key(emitter) {
            {
                if (*emitter).states.top == (*emitter).states.end {
                    yaml_stack_extend(
                        &raw mut (*emitter).states.start as *mut *mut libc::c_void,
                        &raw mut (*emitter).states.top as *mut *mut libc::c_void,
                        &raw mut (*emitter).states.end as *mut *mut libc::c_void,
                    );
                }
                ptr::write((*emitter).states.top, YamlEmitFlowMappingSimpleValueState);
                (*emitter).states.top = (*emitter).states.top.wrapping_offset(1);
            };
            yaml_emitter_emit_node(emitter, event, false, false, true, true)
        } else {
            if yaml_emitter_write_indicator(
                    emitter,
                    b"?\0" as *const u8 as *const libc::c_char,
                    true,
                    false,
                    false,
                )
                .fail
            {
                return FAIL;
            }
            {
                if (*emitter).states.top == (*emitter).states.end {
                    yaml_stack_extend(
                        &raw mut (*emitter).states.start as *mut *mut libc::c_void,
                        &raw mut (*emitter).states.top as *mut *mut libc::c_void,
                        &raw mut (*emitter).states.end as *mut *mut libc::c_void,
                    );
                }
                ptr::write((*emitter).states.top, YamlEmitFlowMappingValueState);
                (*emitter).states.top = (*emitter).states.top.wrapping_offset(1);
            };
            yaml_emitter_emit_node(emitter, event, false, false, true, false)
        }
    }
    unsafe fn yaml_emitter_emit_flow_mapping_value(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
        simple: bool,
    ) -> Success {
        if simple {
            if yaml_emitter_write_indicator(
                    emitter,
                    b":\0" as *const u8 as *const libc::c_char,
                    false,
                    false,
                    false,
                )
                .fail
            {
                return FAIL;
            }
        } else {
            if ((*emitter).canonical || (*emitter).column > (*emitter).best_width)
                && yaml_emitter_write_indent(emitter).fail
            {
                return FAIL;
            }
            if yaml_emitter_write_indicator(
                    emitter,
                    b":\0" as *const u8 as *const libc::c_char,
                    true,
                    false,
                    false,
                )
                .fail
            {
                return FAIL;
            }
        }
        {
            if (*emitter).states.top == (*emitter).states.end {
                yaml_stack_extend(
                    &raw mut (*emitter).states.start as *mut *mut libc::c_void,
                    &raw mut (*emitter).states.top as *mut *mut libc::c_void,
                    &raw mut (*emitter).states.end as *mut *mut libc::c_void,
                );
            }
            ptr::write((*emitter).states.top, YamlEmitFlowMappingKeyState);
            (*emitter).states.top = (*emitter).states.top.wrapping_offset(1);
        };
        yaml_emitter_emit_node(emitter, event, false, false, true, false)
    }
    unsafe fn yaml_emitter_emit_block_sequence_item(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
        first: bool,
    ) -> Success {
        if first {
            yaml_emitter_increase_indent(
                emitter,
                false,
                (*emitter).mapping_context && !(*emitter).indention,
            );
        }
        if (*event).type_ == YamlSequenceEndEvent {
            (*emitter).indent = *{
                (*emitter).indents.top = (*emitter).indents.top.offset(-1);
                (*emitter).indents.top
            };
            (*emitter).state = *{
                (*emitter).states.top = (*emitter).states.top.offset(-1);
                (*emitter).states.top
            };
            return OK;
        }
        if yaml_emitter_write_indent(emitter).fail {
            return FAIL;
        }
        if yaml_emitter_write_indicator(
                emitter,
                b"-\0" as *const u8 as *const libc::c_char,
                true,
                false,
                true,
            )
            .fail
        {
            return FAIL;
        }
        {
            if (*emitter).states.top == (*emitter).states.end {
                yaml_stack_extend(
                    &raw mut (*emitter).states.start as *mut *mut libc::c_void,
                    &raw mut (*emitter).states.top as *mut *mut libc::c_void,
                    &raw mut (*emitter).states.end as *mut *mut libc::c_void,
                );
            }
            ptr::write((*emitter).states.top, YamlEmitBlockSequenceItemState);
            (*emitter).states.top = (*emitter).states.top.wrapping_offset(1);
        };
        yaml_emitter_emit_node(emitter, event, false, true, false, false)
    }
    unsafe fn yaml_emitter_emit_block_mapping_key(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
        first: bool,
    ) -> Success {
        if first {
            yaml_emitter_increase_indent(emitter, false, false);
        }
        if (*event).type_ == YamlMappingEndEvent {
            (*emitter).indent = *{
                (*emitter).indents.top = (*emitter).indents.top.offset(-1);
                (*emitter).indents.top
            };
            (*emitter).state = *{
                (*emitter).states.top = (*emitter).states.top.offset(-1);
                (*emitter).states.top
            };
            return OK;
        }
        if yaml_emitter_write_indent(emitter).fail {
            return FAIL;
        }
        if yaml_emitter_check_simple_key(emitter) {
            {
                if (*emitter).states.top == (*emitter).states.end {
                    yaml_stack_extend(
                        &raw mut (*emitter).states.start as *mut *mut libc::c_void,
                        &raw mut (*emitter).states.top as *mut *mut libc::c_void,
                        &raw mut (*emitter).states.end as *mut *mut libc::c_void,
                    );
                }
                ptr::write((*emitter).states.top, YamlEmitBlockMappingSimpleValueState);
                (*emitter).states.top = (*emitter).states.top.wrapping_offset(1);
            };
            yaml_emitter_emit_node(emitter, event, false, false, true, true)
        } else {
            if yaml_emitter_write_indicator(
                    emitter,
                    b"?\0" as *const u8 as *const libc::c_char,
                    true,
                    false,
                    true,
                )
                .fail
            {
                return FAIL;
            }
            {
                if (*emitter).states.top == (*emitter).states.end {
                    yaml_stack_extend(
                        &raw mut (*emitter).states.start as *mut *mut libc::c_void,
                        &raw mut (*emitter).states.top as *mut *mut libc::c_void,
                        &raw mut (*emitter).states.end as *mut *mut libc::c_void,
                    );
                }
                ptr::write((*emitter).states.top, YamlEmitBlockMappingValueState);
                (*emitter).states.top = (*emitter).states.top.wrapping_offset(1);
            };
            yaml_emitter_emit_node(emitter, event, false, false, true, false)
        }
    }
    unsafe fn yaml_emitter_emit_block_mapping_value(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
        simple: bool,
    ) -> Success {
        if simple {
            if yaml_emitter_write_indicator(
                    emitter,
                    b":\0" as *const u8 as *const libc::c_char,
                    false,
                    false,
                    false,
                )
                .fail
            {
                return FAIL;
            }
        } else {
            if yaml_emitter_write_indent(emitter).fail {
                return FAIL;
            }
            if yaml_emitter_write_indicator(
                    emitter,
                    b":\0" as *const u8 as *const libc::c_char,
                    true,
                    false,
                    true,
                )
                .fail
            {
                return FAIL;
            }
        }
        {
            if (*emitter).states.top == (*emitter).states.end {
                yaml_stack_extend(
                    &raw mut (*emitter).states.start as *mut *mut libc::c_void,
                    &raw mut (*emitter).states.top as *mut *mut libc::c_void,
                    &raw mut (*emitter).states.end as *mut *mut libc::c_void,
                );
            }
            ptr::write((*emitter).states.top, YamlEmitBlockMappingKeyState);
            (*emitter).states.top = (*emitter).states.top.wrapping_offset(1);
        };
        yaml_emitter_emit_node(emitter, event, false, false, true, false)
    }
    unsafe fn yaml_emitter_emit_node(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
        root: bool,
        sequence: bool,
        mapping: bool,
        simple_key: bool,
    ) -> Success {
        (*emitter).root_context = root;
        (*emitter).sequence_context = sequence;
        (*emitter).mapping_context = mapping;
        (*emitter).simple_key_context = simple_key;
        match (*event).type_ {
            YamlAliasEvent => yaml_emitter_emit_alias(emitter, event),
            YamlScalarEvent => yaml_emitter_emit_scalar(emitter, event),
            YamlSequenceStartEvent => yaml_emitter_emit_sequence_start(emitter, event),
            YamlMappingStartEvent => yaml_emitter_emit_mapping_start(emitter, event),
            _ => {
                yaml_emitter_set_emitter_error(
                    emitter,
                    b"expected SCALAR, SEQUENCE-START, MAPPING-START, or ALIAS\0"
                        as *const u8 as *const libc::c_char,
                )
            }
        }
    }
    unsafe fn yaml_emitter_emit_alias(
        emitter: *mut YamlEmitterT,
        _event: *mut YamlEventT,
    ) -> Success {
        if yaml_emitter_process_anchor(emitter).fail {
            return FAIL;
        }
        if (*emitter).simple_key_context && put(emitter, b' ').fail {
            return FAIL;
        }
        (*emitter).state = *{
            (*emitter).states.top = (*emitter).states.top.offset(-1);
            (*emitter).states.top
        };
        OK
    }
    unsafe fn yaml_emitter_emit_scalar(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
    ) -> Success {
        if yaml_emitter_select_scalar_style(emitter, event).fail {
            return FAIL;
        }
        if yaml_emitter_process_anchor(emitter).fail {
            return FAIL;
        }
        if yaml_emitter_process_tag(emitter).fail {
            return FAIL;
        }
        yaml_emitter_increase_indent(emitter, true, false);
        if yaml_emitter_process_scalar(emitter).fail {
            return FAIL;
        }
        (*emitter).indent = *{
            (*emitter).indents.top = (*emitter).indents.top.offset(-1);
            (*emitter).indents.top
        };
        (*emitter).state = *{
            (*emitter).states.top = (*emitter).states.top.offset(-1);
            (*emitter).states.top
        };
        OK
    }
    unsafe fn yaml_emitter_emit_sequence_start(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
    ) -> Success {
        if yaml_emitter_process_anchor(emitter).fail {
            return FAIL;
        }
        if yaml_emitter_process_tag(emitter).fail {
            return FAIL;
        }
        if (*emitter).flow_level != 0 || (*emitter).canonical
            || (*event).data.sequence_start.style == YamlFlowSequenceStyle
            || yaml_emitter_check_empty_sequence(emitter)
        {
            (*emitter).state = YamlEmitFlowSequenceFirstItemState;
        } else {
            (*emitter).state = YamlEmitBlockSequenceFirstItemState;
        }
        OK
    }
    unsafe fn yaml_emitter_emit_mapping_start(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
    ) -> Success {
        if yaml_emitter_process_anchor(emitter).fail {
            return FAIL;
        }
        if yaml_emitter_process_tag(emitter).fail {
            return FAIL;
        }
        if (*emitter).flow_level != 0 || (*emitter).canonical
            || (*event).data.mapping_start.style == YamlFlowMappingStyle
            || yaml_emitter_check_empty_mapping(emitter)
        {
            (*emitter).state = YamlEmitFlowMappingFirstKeyState;
        } else {
            (*emitter).state = YamlEmitBlockMappingFirstKeyState;
        }
        OK
    }
    unsafe fn yaml_emitter_check_empty_document(_emitter: *mut YamlEmitterT) -> bool {
        false
    }
    unsafe fn yaml_emitter_check_empty_sequence(emitter: *mut YamlEmitterT) -> bool {
        if ((*emitter).events.tail.c_offset_from((*emitter).events.head) as libc::c_long)
            < 2_i64
        {
            return false;
        }
        (*(*emitter).events.head).type_ == YamlSequenceStartEvent
            && (*(*emitter).events.head.wrapping_offset(1_isize)).type_
                == YamlSequenceEndEvent
    }
    unsafe fn yaml_emitter_check_empty_mapping(emitter: *mut YamlEmitterT) -> bool {
        if ((*emitter).events.tail.c_offset_from((*emitter).events.head) as libc::c_long)
            < 2_i64
        {
            return false;
        }
        (*(*emitter).events.head).type_ == YamlMappingStartEvent
            && (*(*emitter).events.head.wrapping_offset(1_isize)).type_
                == YamlMappingEndEvent
    }
    unsafe fn yaml_emitter_check_simple_key(emitter: *mut YamlEmitterT) -> bool {
        let event: *mut YamlEventT = (*emitter).events.head;
        let mut length: size_t = 0_u64;
        match (*event).type_ {
            YamlAliasEvent => {
                length = length.force_add((*emitter).anchor_data.anchor_length);
            }
            YamlScalarEvent => {
                if (*emitter).scalar_data.multiline {
                    return false;
                }
                length = length
                    .force_add((*emitter).anchor_data.anchor_length)
                    .force_add((*emitter).tag_data.handle_length)
                    .force_add((*emitter).tag_data.suffix_length)
                    .force_add((*emitter).scalar_data.length);
            }
            YamlSequenceStartEvent => {
                if !yaml_emitter_check_empty_sequence(emitter) {
                    return false;
                }
                length = length
                    .force_add((*emitter).anchor_data.anchor_length)
                    .force_add((*emitter).tag_data.handle_length)
                    .force_add((*emitter).tag_data.suffix_length);
            }
            YamlMappingStartEvent => {
                if !yaml_emitter_check_empty_mapping(emitter) {
                    return false;
                }
                length = length
                    .force_add((*emitter).anchor_data.anchor_length)
                    .force_add((*emitter).tag_data.handle_length)
                    .force_add((*emitter).tag_data.suffix_length);
            }
            _ => return false,
        }
        if length > 128_u64 {
            return false;
        }
        true
    }
    unsafe fn yaml_emitter_select_scalar_style(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
    ) -> Success {
        let mut style: YamlScalarStyleT = (*event).data.scalar.style;
        let no_tag = (*emitter).tag_data.handle.is_null()
            && (*emitter).tag_data.suffix.is_null();
        if no_tag && !(*event).data.scalar.plain_implicit
            && !(*event).data.scalar.quoted_implicit
        {
            return yaml_emitter_set_emitter_error(
                emitter,
                b"neither tag nor implicit flags are specified\0" as *const u8
                    as *const libc::c_char,
            );
        }
        if style == YamlAnyScalarStyle {
            style = YamlPlainScalarStyle;
        }
        if (*emitter).canonical {
            style = YamlDoubleQuotedScalarStyle;
        }
        if (*emitter).simple_key_context && (*emitter).scalar_data.multiline {
            style = YamlDoubleQuotedScalarStyle;
        }
        if style == YamlPlainScalarStyle {
            if (*emitter).flow_level != 0 && !(*emitter).scalar_data.flow_plain_allowed
                || (*emitter).flow_level == 0
                    && !(*emitter).scalar_data.block_plain_allowed
            {
                style = YamlSingleQuotedScalarStyle;
            }
            if (*emitter).scalar_data.length == 0
                && ((*emitter).flow_level != 0 || (*emitter).simple_key_context)
            {
                style = YamlSingleQuotedScalarStyle;
            }
            if no_tag && !(*event).data.scalar.plain_implicit {
                style = YamlSingleQuotedScalarStyle;
            }
        }
        if style == YamlSingleQuotedScalarStyle
            && !(*emitter).scalar_data.single_quoted_allowed
        {
            style = YamlDoubleQuotedScalarStyle;
        }
        if (style == YamlLiteralScalarStyle || style == YamlFoldedScalarStyle)
            && (!(*emitter).scalar_data.block_allowed || (*emitter).flow_level != 0
                || (*emitter).simple_key_context)
        {
            style = YamlDoubleQuotedScalarStyle;
        }
        if no_tag && !(*event).data.scalar.quoted_implicit
            && style != YamlPlainScalarStyle
        {
            let fresh46 = &raw mut (*emitter).tag_data.handle;
            *fresh46 = b"!\0" as *const u8 as *const libc::c_char as *mut yaml_char_t;
            (*emitter).tag_data.handle_length = 1_u64;
        }
        (*emitter).scalar_data.style = style;
        OK
    }
    unsafe fn yaml_emitter_process_anchor(emitter: *mut YamlEmitterT) -> Success {
        if (*emitter).anchor_data.anchor.is_null() {
            return OK;
        }
        if yaml_emitter_write_indicator(
                emitter,
                if (*emitter).anchor_data.alias {
                    b"*\0" as *const u8 as *const libc::c_char
                } else {
                    b"&\0" as *const u8 as *const libc::c_char
                },
                true,
                false,
                false,
            )
            .fail
        {
            return FAIL;
        }
        yaml_emitter_write_anchor(
            emitter,
            (*emitter).anchor_data.anchor,
            (*emitter).anchor_data.anchor_length,
        )
    }
    unsafe fn yaml_emitter_process_tag(emitter: *mut YamlEmitterT) -> Success {
        if (*emitter).tag_data.handle.is_null() && (*emitter).tag_data.suffix.is_null() {
            return OK;
        }
        if !(*emitter).tag_data.handle.is_null() {
            if yaml_emitter_write_tag_handle(
                    emitter,
                    (*emitter).tag_data.handle,
                    (*emitter).tag_data.handle_length,
                )
                .fail
            {
                return FAIL;
            }
            if !(*emitter).tag_data.suffix.is_null()
                && yaml_emitter_write_tag_content(
                        emitter,
                        (*emitter).tag_data.suffix,
                        (*emitter).tag_data.suffix_length,
                        false,
                    )
                    .fail
            {
                return FAIL;
            }
        } else {
            if yaml_emitter_write_indicator(
                    emitter,
                    b"!<\0" as *const u8 as *const libc::c_char,
                    true,
                    false,
                    false,
                )
                .fail
            {
                return FAIL;
            }
            if yaml_emitter_write_tag_content(
                    emitter,
                    (*emitter).tag_data.suffix,
                    (*emitter).tag_data.suffix_length,
                    false,
                )
                .fail
            {
                return FAIL;
            }
            if yaml_emitter_write_indicator(
                    emitter,
                    b">\0" as *const u8 as *const libc::c_char,
                    false,
                    false,
                    false,
                )
                .fail
            {
                return FAIL;
            }
        }
        OK
    }
    unsafe fn yaml_emitter_process_scalar(emitter: *mut YamlEmitterT) -> Success {
        match (*emitter).scalar_data.style {
            YamlPlainScalarStyle => {
                return yaml_emitter_write_plain_scalar(
                    emitter,
                    (*emitter).scalar_data.value,
                    (*emitter).scalar_data.length,
                    !(*emitter).simple_key_context,
                );
            }
            YamlSingleQuotedScalarStyle => {
                return yaml_emitter_write_single_quoted_scalar(
                    emitter,
                    (*emitter).scalar_data.value,
                    (*emitter).scalar_data.length,
                    !(*emitter).simple_key_context,
                );
            }
            YamlDoubleQuotedScalarStyle => {
                return yaml_emitter_write_double_quoted_scalar(
                    emitter,
                    (*emitter).scalar_data.value,
                    (*emitter).scalar_data.length,
                    !(*emitter).simple_key_context,
                );
            }
            YamlLiteralScalarStyle => {
                return yaml_emitter_write_literal_scalar(
                    emitter,
                    (*emitter).scalar_data.value,
                    (*emitter).scalar_data.length,
                );
            }
            YamlFoldedScalarStyle => {
                return yaml_emitter_write_folded_scalar(
                    emitter,
                    (*emitter).scalar_data.value,
                    (*emitter).scalar_data.length,
                );
            }
            _ => {}
        }
        FAIL
    }
    unsafe fn yaml_emitter_analyze_version_directive(
        emitter: *mut YamlEmitterT,
        version_directive: YamlVersionDirectiveT,
    ) -> Success {
        if version_directive.major != 1
            || version_directive.minor != 1 && version_directive.minor != 2
        {
            return yaml_emitter_set_emitter_error(
                emitter,
                b"incompatible %YAML directive\0" as *const u8 as *const libc::c_char,
            );
        }
        OK
    }
    unsafe fn yaml_emitter_analyze_tag_directive(
        emitter: *mut YamlEmitterT,
        tag_directive: YamlTagDirectiveT,
    ) -> Success {
        let handle_length: size_t = strlen(tag_directive.handle as *mut libc::c_char);
        let prefix_length: size_t = strlen(tag_directive.prefix as *mut libc::c_char);
        let mut handle = YamlStringT {
            start: tag_directive.handle,
            end: tag_directive.handle.wrapping_offset(handle_length as isize),
            pointer: tag_directive.handle,
        };
        let prefix = YamlStringT {
            start: tag_directive.prefix,
            end: tag_directive.prefix.wrapping_offset(prefix_length as isize),
            pointer: tag_directive.prefix,
        };
        if handle.start == handle.end {
            return yaml_emitter_set_emitter_error(
                emitter,
                b"tag handle must not be empty\0" as *const u8 as *const libc::c_char,
            );
        }
        if *handle.start != b'!' {
            return yaml_emitter_set_emitter_error(
                emitter,
                b"tag handle must start with '!'\0" as *const u8 as *const libc::c_char,
            );
        }
        if *handle.end.wrapping_offset(-1_isize) != b'!' {
            return yaml_emitter_set_emitter_error(
                emitter,
                b"tag handle must end with '!'\0" as *const u8 as *const libc::c_char,
            );
        }
        handle.pointer = handle.pointer.wrapping_offset(1);
        while handle.pointer < handle.end.wrapping_offset(-1_isize) {
            if !(*handle.pointer >= b'0' && *handle.pointer <= b'9'
                || *handle.pointer >= b'A' && *handle.pointer <= b'Z'
                || *handle.pointer >= b'a' && *handle.pointer <= b'z'
                || *handle.pointer == b'_' || *handle.pointer == b'-')
            {
                return yaml_emitter_set_emitter_error(
                    emitter,
                    b"tag handle must contain alphanumerical characters only\0"
                        as *const u8 as *const libc::c_char,
                );
            }
            handle.pointer = handle
                .pointer
                .wrapping_offset(
                    if *handle.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                        1
                    } else if *handle.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                        2
                    } else if *handle.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                        3
                    } else if *handle.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                        4
                    } else {
                        0
                    },
                );
        }
        if prefix.start == prefix.end {
            return yaml_emitter_set_emitter_error(
                emitter,
                b"tag prefix must not be empty\0" as *const u8 as *const libc::c_char,
            );
        }
        OK
    }
    unsafe fn yaml_emitter_analyze_anchor(
        emitter: *mut YamlEmitterT,
        anchor: *mut yaml_char_t,
        alias: bool,
    ) -> Success {
        let anchor_length: size_t = strlen(anchor as *mut libc::c_char);
        let mut string = YamlStringT {
            start: anchor,
            end: anchor.wrapping_offset(anchor_length as isize),
            pointer: anchor,
        };
        if string.start == string.end {
            return yaml_emitter_set_emitter_error(
                emitter,
                if alias {
                    b"alias value must not be empty\0" as *const u8
                        as *const libc::c_char
                } else {
                    b"anchor value must not be empty\0" as *const u8
                        as *const libc::c_char
                },
            );
        }
        while string.pointer != string.end {
            if !(*string.pointer >= b'0' && *string.pointer <= b'9'
                || *string.pointer >= b'A' && *string.pointer <= b'Z'
                || *string.pointer >= b'a' && *string.pointer <= b'z'
                || *string.pointer == b'_' || *string.pointer == b'-')
            {
                return yaml_emitter_set_emitter_error(
                    emitter,
                    if alias {
                        b"alias value must contain alphanumerical characters only\0"
                            as *const u8 as *const libc::c_char
                    } else {
                        b"anchor value must contain alphanumerical characters only\0"
                            as *const u8 as *const libc::c_char
                    },
                );
            }
            string.pointer = string
                .pointer
                .wrapping_offset(
                    if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                        1
                    } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                        2
                    } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                        3
                    } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                        4
                    } else {
                        0
                    },
                );
        }
        let fresh47 = &raw mut (*emitter).anchor_data.anchor;
        *fresh47 = string.start;
        (*emitter).anchor_data.anchor_length = string.end.c_offset_from(string.start)
            as size_t;
        (*emitter).anchor_data.alias = alias;
        OK
    }
    unsafe fn yaml_emitter_analyze_tag(
        emitter: *mut YamlEmitterT,
        tag: *mut yaml_char_t,
    ) -> Success {
        let mut tag_directive: *mut YamlTagDirectiveT;
        let tag_length: size_t = strlen(tag as *mut libc::c_char);
        let string = YamlStringT {
            start: tag,
            end: tag.wrapping_offset(tag_length as isize),
            pointer: tag,
        };
        if string.start == string.end {
            return yaml_emitter_set_emitter_error(
                emitter,
                b"tag value must not be empty\0" as *const u8 as *const libc::c_char,
            );
        }
        tag_directive = (*emitter).tag_directives.start;
        while tag_directive != (*emitter).tag_directives.top {
            let prefix_length: size_t = strlen(
                (*tag_directive).prefix as *mut libc::c_char,
            );
            if prefix_length < string.end.c_offset_from(string.start) as size_t
                && strncmp(
                    (*tag_directive).prefix as *mut libc::c_char,
                    string.start as *mut libc::c_char,
                    prefix_length,
                ) == 0
            {
                let fresh48 = &raw mut (*emitter).tag_data.handle;
                *fresh48 = (*tag_directive).handle;
                (*emitter).tag_data.handle_length = strlen(
                    (*tag_directive).handle as *mut libc::c_char,
                );
                let fresh49 = &raw mut (*emitter).tag_data.suffix;
                *fresh49 = string.start.wrapping_offset(prefix_length as isize);
                (*emitter).tag_data.suffix_length = (string
                    .end
                    .c_offset_from(string.start) as libc::c_ulong)
                    .wrapping_sub(prefix_length);
                return OK;
            }
            tag_directive = tag_directive.wrapping_offset(1);
        }
        let fresh50 = &raw mut (*emitter).tag_data.suffix;
        *fresh50 = string.start;
        (*emitter).tag_data.suffix_length = string.end.c_offset_from(string.start)
            as size_t;
        OK
    }
    unsafe fn yaml_emitter_analyze_scalar(
        emitter: *mut YamlEmitterT,
        value: *mut yaml_char_t,
        length: size_t,
    ) -> Success {
        let mut block_indicators = false;
        let mut flow_indicators = false;
        let mut line_breaks = false;
        let mut special_characters = false;
        let mut leading_space = false;
        let mut leading_break = false;
        let mut trailing_space = false;
        let mut trailing_break = false;
        let mut break_space = false;
        let mut space_break = false;
        let mut preceded_by_whitespace;
        let mut followed_by_whitespace;
        let mut previous_space = false;
        let mut previous_break = false;
        let mut string = YamlStringT {
            start: value,
            end: value.wrapping_offset(length as isize),
            pointer: value,
        };
        let fresh51 = &raw mut (*emitter).scalar_data.value;
        *fresh51 = value;
        (*emitter).scalar_data.length = length;
        if string.start == string.end {
            (*emitter).scalar_data.multiline = false;
            (*emitter).scalar_data.flow_plain_allowed = false;
            (*emitter).scalar_data.block_plain_allowed = true;
            (*emitter).scalar_data.single_quoted_allowed = true;
            (*emitter).scalar_data.block_allowed = false;
            return OK;
        }
        if *string.pointer.offset(0) == b'-' && *string.pointer.offset(1) == b'-'
            && *string.pointer.offset(2) == b'-'
            || *string.pointer.offset(0) == b'.' && *string.pointer.offset(1) == b'.'
                && *string.pointer.offset(2) == b'.'
        {
            block_indicators = true;
            flow_indicators = true;
        }
        preceded_by_whitespace = true;
        followed_by_whitespace = *string
            .pointer
            .offset(
                if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                    1
                } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                    2
                } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                    3
                } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                    4
                } else {
                    0
                },
            ) == b' '
            || *string
                .pointer
                .offset(
                    if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                        1
                    } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                        2
                    } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                        3
                    } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                        4
                    } else {
                        0
                    },
                ) == b'\t'
            || (*string
                .pointer
                .offset(
                    if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                        1
                    } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                        2
                    } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                        3
                    } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                        4
                    } else {
                        0
                    },
                ) == b'\r'
                || *string
                    .pointer
                    .offset(
                        if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                            1
                        } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                            2
                        } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                            3
                        } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                            4
                        } else {
                            0
                        },
                    ) == b'\n'
                || *string
                    .pointer
                    .offset(
                        if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                            1
                        } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                            2
                        } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                            3
                        } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                            4
                        } else {
                            0
                        },
                    ) == b'\xC2'
                    && *string
                        .pointer
                        .offset(
                            (if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                1
                            } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                                2
                            } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                                3
                            } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                                4
                            } else {
                                0
                            } + 1)
                                .try_into()
                                .unwrap(),
                        ) == b'\x85'
                || *string
                    .pointer
                    .offset(
                        if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                            1
                        } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                            2
                        } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                            3
                        } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                            4
                        } else {
                            0
                        },
                    ) == b'\xE2'
                    && *string
                        .pointer
                        .offset(
                            (if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                1
                            } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                                2
                            } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                                3
                            } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                                4
                            } else {
                                0
                            } + 1)
                                .try_into()
                                .unwrap(),
                        ) == b'\x80'
                    && *string
                        .pointer
                        .offset(
                            (if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                1
                            } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                                2
                            } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                                3
                            } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                                4
                            } else {
                                0
                            } + 2)
                                .try_into()
                                .unwrap(),
                        ) == b'\xA8'
                || *string
                    .pointer
                    .offset(
                        if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                            1
                        } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                            2
                        } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                            3
                        } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                            4
                        } else {
                            0
                        },
                    ) == b'\xE2'
                    && *string
                        .pointer
                        .offset(
                            (if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                1
                            } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                                2
                            } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                                3
                            } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                                4
                            } else {
                                0
                            } + 1)
                                .try_into()
                                .unwrap(),
                        ) == b'\x80'
                    && *string
                        .pointer
                        .offset(
                            (if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                1
                            } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                                2
                            } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                                3
                            } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                                4
                            } else {
                                0
                            } + 2)
                                .try_into()
                                .unwrap(),
                        ) == b'\xA9'
                || *string
                    .pointer
                    .offset(
                        if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                            1
                        } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                            2
                        } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                            3
                        } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                            4
                        } else {
                            0
                        },
                    ) == b'\0');
        while string.pointer != string.end {
            if string.start == string.pointer {
                if *string.pointer == b'#' || *string.pointer == b','
                    || *string.pointer == b'[' || *string.pointer == b']'
                    || *string.pointer == b'{' || *string.pointer == b'}'
                    || *string.pointer == b'&' || *string.pointer == b'*'
                    || *string.pointer == b'!' || *string.pointer == b'|'
                    || *string.pointer == b'>' || *string.pointer == b'\''
                    || *string.pointer == b'"' || *string.pointer == b'%'
                    || *string.pointer == b'@' || *string.pointer == b'`'
                {
                    flow_indicators = true;
                    block_indicators = true;
                }
                if *string.pointer == b'?' || *string.pointer == b':' {
                    flow_indicators = true;
                    if followed_by_whitespace {
                        block_indicators = true;
                    }
                }
                if *string.pointer == b'-' && followed_by_whitespace {
                    flow_indicators = true;
                    block_indicators = true;
                }
            } else {
                if *string.pointer == b',' || *string.pointer == b'?'
                    || *string.pointer == b'[' || *string.pointer == b']'
                    || *string.pointer == b'{' || *string.pointer == b'}'
                {
                    flow_indicators = true;
                }
                if *string.pointer == b':' {
                    flow_indicators = true;
                    if followed_by_whitespace {
                        block_indicators = true;
                    }
                }
                if *string.pointer == b'#' && preceded_by_whitespace {
                    flow_indicators = true;
                    block_indicators = true;
                }
            }
            if !match *string.pointer {
                0x0A | 0x20..=0x7E => true,
                0xC2 => {
                    match *string.pointer.wrapping_offset(1) {
                        0xA0..=0xBF => true,
                        _ => false,
                    }
                }
                0xC3..=0xEC => true,
                0xED => {
                    match *string.pointer.wrapping_offset(1) {
                        0x00..=0x9F => true,
                        _ => false,
                    }
                }
                0xEE => true,
                0xEF => {
                    match *string.pointer.wrapping_offset(1) {
                        0xBB => {
                            match *string.pointer.wrapping_offset(2) {
                                0xBF => false,
                                _ => true,
                            }
                        }
                        0xBF => {
                            match *string.pointer.wrapping_offset(2) {
                                0xBE | 0xBF => false,
                                _ => true,
                            }
                        }
                        _ => true,
                    }
                }
                0xF0..=0xF4 => true,
                _ => false,
            } || !(*string.pointer <= b'\x7F') && !(*emitter).unicode
            {
                special_characters = true;
            }
            if *string.pointer.offset(0) == b'\r' || *string.pointer.offset(0) == b'\n'
                || *string.pointer.offset(0) == b'\xC2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x85'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA8'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA9'
            {
                line_breaks = true;
            }
            if *string.pointer.offset(0) == b' ' {
                if string.start == string.pointer {
                    leading_space = true;
                }
                if string
                    .pointer
                    .wrapping_offset(
                        if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                            1
                        } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                            2
                        } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                            3
                        } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                            4
                        } else {
                            0
                        } as isize,
                    ) == string.end
                {
                    trailing_space = true;
                }
                if previous_break {
                    break_space = true;
                }
                previous_space = true;
                previous_break = false;
            } else if *string.pointer.offset(0) == b'\r'
                || *string.pointer.offset(0) == b'\n'
                || *string.pointer.offset(0) == b'\xC2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x85'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA8'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA9'
            {
                if string.start == string.pointer {
                    leading_break = true;
                }
                if string
                    .pointer
                    .wrapping_offset(
                        if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                            1
                        } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                            2
                        } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                            3
                        } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                            4
                        } else {
                            0
                        } as isize,
                    ) == string.end
                {
                    trailing_break = true;
                }
                if previous_space {
                    space_break = true;
                }
                previous_space = false;
                previous_break = true;
            } else {
                previous_space = false;
                previous_break = false;
            }
            preceded_by_whitespace = *string.pointer.offset(0) == b' '
                || *string.pointer.offset(0) == b'\t'
                || (*string.pointer.offset(0) == b'\r'
                    || *string.pointer.offset(0) == b'\n'
                    || *string.pointer.offset(0) == b'\xC2'
                        && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x85'
                    || *string.pointer.offset(0) == b'\xE2'
                        && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                        && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA8'
                    || *string.pointer.offset(0) == b'\xE2'
                        && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                        && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA9'
                    || *string.pointer.offset(0) == b'\0');
            string.pointer = string
                .pointer
                .wrapping_offset(
                    if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                        1
                    } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                        2
                    } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                        3
                    } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                        4
                    } else {
                        0
                    },
                );
            if string.pointer != string.end {
                followed_by_whitespace = *string
                    .pointer
                    .offset(
                        if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                            1
                        } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                            2
                        } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                            3
                        } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                            4
                        } else {
                            0
                        },
                    ) == b' '
                    || *string
                        .pointer
                        .offset(
                            if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                1
                            } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                                2
                            } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                                3
                            } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                                4
                            } else {
                                0
                            },
                        ) == b'\t'
                    || (*string
                        .pointer
                        .offset(
                            if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                1
                            } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                                2
                            } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                                3
                            } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                                4
                            } else {
                                0
                            },
                        ) == b'\r'
                        || *string
                            .pointer
                            .offset(
                                if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                    1
                                } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0
                                {
                                    2
                                } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0
                                {
                                    3
                                } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0
                                {
                                    4
                                } else {
                                    0
                                },
                            ) == b'\n'
                        || *string
                            .pointer
                            .offset(
                                if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                    1
                                } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0
                                {
                                    2
                                } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0
                                {
                                    3
                                } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0
                                {
                                    4
                                } else {
                                    0
                                },
                            ) == b'\xC2'
                            && *string
                                .pointer
                                .offset(
                                    (if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                        1
                                    } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0
                                    {
                                        2
                                    } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0
                                    {
                                        3
                                    } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0
                                    {
                                        4
                                    } else {
                                        0
                                    } + 1)
                                        .try_into()
                                        .unwrap(),
                                ) == b'\x85'
                        || *string
                            .pointer
                            .offset(
                                if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                    1
                                } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0
                                {
                                    2
                                } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0
                                {
                                    3
                                } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0
                                {
                                    4
                                } else {
                                    0
                                },
                            ) == b'\xE2'
                            && *string
                                .pointer
                                .offset(
                                    (if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                        1
                                    } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0
                                    {
                                        2
                                    } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0
                                    {
                                        3
                                    } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0
                                    {
                                        4
                                    } else {
                                        0
                                    } + 1)
                                        .try_into()
                                        .unwrap(),
                                ) == b'\x80'
                            && *string
                                .pointer
                                .offset(
                                    (if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                        1
                                    } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0
                                    {
                                        2
                                    } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0
                                    {
                                        3
                                    } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0
                                    {
                                        4
                                    } else {
                                        0
                                    } + 2)
                                        .try_into()
                                        .unwrap(),
                                ) == b'\xA8'
                        || *string
                            .pointer
                            .offset(
                                if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                    1
                                } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0
                                {
                                    2
                                } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0
                                {
                                    3
                                } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0
                                {
                                    4
                                } else {
                                    0
                                },
                            ) == b'\xE2'
                            && *string
                                .pointer
                                .offset(
                                    (if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                        1
                                    } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0
                                    {
                                        2
                                    } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0
                                    {
                                        3
                                    } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0
                                    {
                                        4
                                    } else {
                                        0
                                    } + 1)
                                        .try_into()
                                        .unwrap(),
                                ) == b'\x80'
                            && *string
                                .pointer
                                .offset(
                                    (if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                        1
                                    } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0
                                    {
                                        2
                                    } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0
                                    {
                                        3
                                    } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0
                                    {
                                        4
                                    } else {
                                        0
                                    } + 2)
                                        .try_into()
                                        .unwrap(),
                                ) == b'\xA9'
                        || *string
                            .pointer
                            .offset(
                                if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                    1
                                } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0
                                {
                                    2
                                } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0
                                {
                                    3
                                } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0
                                {
                                    4
                                } else {
                                    0
                                },
                            ) == b'\0');
            }
        }
        (*emitter).scalar_data.multiline = line_breaks;
        (*emitter).scalar_data.flow_plain_allowed = true;
        (*emitter).scalar_data.block_plain_allowed = true;
        (*emitter).scalar_data.single_quoted_allowed = true;
        (*emitter).scalar_data.block_allowed = true;
        if leading_space || leading_break || trailing_space || trailing_break {
            (*emitter).scalar_data.flow_plain_allowed = false;
            (*emitter).scalar_data.block_plain_allowed = false;
        }
        if trailing_space {
            (*emitter).scalar_data.block_allowed = false;
        }
        if break_space {
            (*emitter).scalar_data.flow_plain_allowed = false;
            (*emitter).scalar_data.block_plain_allowed = false;
            (*emitter).scalar_data.single_quoted_allowed = false;
        }
        if space_break || special_characters {
            (*emitter).scalar_data.flow_plain_allowed = false;
            (*emitter).scalar_data.block_plain_allowed = false;
            (*emitter).scalar_data.single_quoted_allowed = false;
            (*emitter).scalar_data.block_allowed = false;
        }
        if line_breaks {
            (*emitter).scalar_data.flow_plain_allowed = false;
            (*emitter).scalar_data.block_plain_allowed = false;
        }
        if flow_indicators {
            (*emitter).scalar_data.flow_plain_allowed = false;
        }
        if block_indicators {
            (*emitter).scalar_data.block_plain_allowed = false;
        }
        OK
    }
    unsafe fn yaml_emitter_analyze_event(
        emitter: *mut YamlEmitterT,
        event: *mut YamlEventT,
    ) -> Success {
        let fresh52 = &raw mut (*emitter).anchor_data.anchor;
        *fresh52 = ptr::null_mut::<yaml_char_t>();
        (*emitter).anchor_data.anchor_length = 0_u64;
        let fresh53 = &raw mut (*emitter).tag_data.handle;
        *fresh53 = ptr::null_mut::<yaml_char_t>();
        (*emitter).tag_data.handle_length = 0_u64;
        let fresh54 = &raw mut (*emitter).tag_data.suffix;
        *fresh54 = ptr::null_mut::<yaml_char_t>();
        (*emitter).tag_data.suffix_length = 0_u64;
        let fresh55 = &raw mut (*emitter).scalar_data.value;
        *fresh55 = ptr::null_mut::<yaml_char_t>();
        (*emitter).scalar_data.length = 0_u64;
        match (*event).type_ {
            YamlAliasEvent => {
                yaml_emitter_analyze_anchor(emitter, (*event).data.alias.anchor, true)
            }
            YamlScalarEvent => {
                if !(*event).data.scalar.anchor.is_null()
                    && yaml_emitter_analyze_anchor(
                            emitter,
                            (*event).data.scalar.anchor,
                            false,
                        )
                        .fail
                {
                    return FAIL;
                }
                if !(*event).data.scalar.tag.is_null()
                    && ((*emitter).canonical
                        || !(*event).data.scalar.plain_implicit
                            && !(*event).data.scalar.quoted_implicit)
                    && yaml_emitter_analyze_tag(emitter, (*event).data.scalar.tag).fail
                {
                    return FAIL;
                }
                yaml_emitter_analyze_scalar(
                    emitter,
                    (*event).data.scalar.value,
                    (*event).data.scalar.length,
                )
            }
            YamlSequenceStartEvent => {
                if !(*event).data.sequence_start.anchor.is_null()
                    && yaml_emitter_analyze_anchor(
                            emitter,
                            (*event).data.sequence_start.anchor,
                            false,
                        )
                        .fail
                {
                    return FAIL;
                }
                if !(*event).data.sequence_start.tag.is_null()
                    && ((*emitter).canonical || !(*event).data.sequence_start.implicit)
                    && yaml_emitter_analyze_tag(
                            emitter,
                            (*event).data.sequence_start.tag,
                        )
                        .fail
                {
                    return FAIL;
                }
                OK
            }
            YamlMappingStartEvent => {
                if !(*event).data.mapping_start.anchor.is_null()
                    && yaml_emitter_analyze_anchor(
                            emitter,
                            (*event).data.mapping_start.anchor,
                            false,
                        )
                        .fail
                {
                    return FAIL;
                }
                if !(*event).data.mapping_start.tag.is_null()
                    && ((*emitter).canonical || !(*event).data.mapping_start.implicit)
                    && yaml_emitter_analyze_tag(emitter, (*event).data.mapping_start.tag)
                        .fail
                {
                    return FAIL;
                }
                OK
            }
            _ => OK,
        }
    }
    unsafe fn yaml_emitter_write_bom(emitter: *mut YamlEmitterT) -> Success {
        if flush(emitter).fail {
            return FAIL;
        }
        let fresh56 = &raw mut (*emitter).buffer.pointer;
        let fresh57 = *fresh56;
        *fresh56 = (*fresh56).wrapping_offset(1);
        *fresh57 = b'\xEF';
        let fresh58 = &raw mut (*emitter).buffer.pointer;
        let fresh59 = *fresh58;
        *fresh58 = (*fresh58).wrapping_offset(1);
        *fresh59 = b'\xBB';
        let fresh60 = &raw mut (*emitter).buffer.pointer;
        let fresh61 = *fresh60;
        *fresh60 = (*fresh60).wrapping_offset(1);
        *fresh61 = b'\xBF';
        OK
    }
    unsafe fn yaml_emitter_write_indent(emitter: *mut YamlEmitterT) -> Success {
        let indent: libc::c_int = if (*emitter).indent >= 0 {
            (*emitter).indent
        } else {
            0
        };
        if (!(*emitter).indention || (*emitter).column > indent
            || (*emitter).column == indent && !(*emitter).whitespace)
            && put_break(emitter).fail
        {
            return FAIL;
        }
        if (*emitter).column < indent {
            loop {
                if put(emitter, b' ').fail {
                    return FAIL;
                }
                if (*emitter).column >= indent {
                    break;
                }
            }
        }
        (*emitter).whitespace = true;
        (*emitter).indention = true;
        OK
    }
    unsafe fn yaml_emitter_write_indicator(
        emitter: *mut YamlEmitterT,
        indicator: *const libc::c_char,
        need_whitespace: bool,
        is_whitespace: bool,
        is_indention: bool,
    ) -> Success {
        let indicator_length: size_t = strlen(indicator);
        let mut string = YamlStringT {
            start: indicator as *mut yaml_char_t,
            end: (indicator as *mut yaml_char_t)
                .wrapping_offset(indicator_length as isize),
            pointer: indicator as *mut yaml_char_t,
        };
        if need_whitespace && !(*emitter).whitespace && put(emitter, b' ').fail {
            return FAIL;
        }
        while string.pointer != string.end {
            if write(emitter, &raw mut string).fail {
                return FAIL;
            }
        }
        (*emitter).whitespace = is_whitespace;
        (*emitter).indention = (*emitter).indention && is_indention;
        OK
    }
    unsafe fn yaml_emitter_write_anchor(
        emitter: *mut YamlEmitterT,
        value: *mut yaml_char_t,
        length: size_t,
    ) -> Success {
        let mut string = YamlStringT {
            start: value,
            end: value.wrapping_offset(length as isize),
            pointer: value,
        };
        while string.pointer != string.end {
            if write(emitter, &raw mut string).fail {
                return FAIL;
            }
        }
        (*emitter).whitespace = false;
        (*emitter).indention = false;
        OK
    }
    unsafe fn yaml_emitter_write_tag_handle(
        emitter: *mut YamlEmitterT,
        value: *mut yaml_char_t,
        length: size_t,
    ) -> Success {
        let mut string = YamlStringT {
            start: value,
            end: value.wrapping_offset(length as isize),
            pointer: value,
        };
        if !(*emitter).whitespace && put(emitter, b' ').fail {
            return FAIL;
        }
        while string.pointer != string.end {
            if write(emitter, &raw mut string).fail {
                return FAIL;
            }
        }
        (*emitter).whitespace = false;
        (*emitter).indention = false;
        OK
    }
    unsafe fn yaml_emitter_write_tag_content(
        emitter: *mut YamlEmitterT,
        value: *mut yaml_char_t,
        length: size_t,
        need_whitespace: bool,
    ) -> Success {
        let mut string = YamlStringT {
            start: value,
            end: value.wrapping_offset(length as isize),
            pointer: value,
        };
        if need_whitespace && !(*emitter).whitespace && put(emitter, b' ').fail {
            return FAIL;
        }
        while string.pointer != string.end {
            if *string.pointer >= b'0' && *string.pointer <= b'9'
                || *string.pointer >= b'A' && *string.pointer <= b'Z'
                || *string.pointer >= b'a' && *string.pointer <= b'z'
                || *string.pointer == b'_' || *string.pointer == b'-'
                || *string.pointer == b';' || *string.pointer == b'/'
                || *string.pointer == b'?' || *string.pointer == b':'
                || *string.pointer == b'@' || *string.pointer == b'&'
                || *string.pointer == b'=' || *string.pointer == b'+'
                || *string.pointer == b'$' || *string.pointer == b','
                || *string.pointer == b'_' || *string.pointer == b'.'
                || *string.pointer == b'~' || *string.pointer == b'*'
                || *string.pointer == b'\'' || *string.pointer == b'('
                || *string.pointer == b')' || *string.pointer == b'['
                || *string.pointer == b']'
            {
                if write(emitter, &raw mut string).fail {
                    return FAIL;
                }
            } else {
                let mut width = if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                    1
                } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                    2
                } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                    3
                } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                    4
                } else {
                    0
                };
                loop {
                    let fresh207 = width;
                    width -= 1;
                    if fresh207 == 0 {
                        break;
                    }
                    let fresh208 = string.pointer;
                    string.pointer = string.pointer.wrapping_offset(1);
                    let value = *fresh208;
                    if put(emitter, b'%').fail {
                        return FAIL;
                    }
                    if put(
                            emitter,
                            (value >> 4)
                                .force_add(if (value >> 4) < 10 { b'0' } else { b'A' - 10 }),
                        )
                        .fail
                    {
                        return FAIL;
                    }
                    if put(
                            emitter,
                            (value & 0x0F)
                                .force_add(
                                    if (value & 0x0F) < 10 { b'0' } else { b'A' - 10 },
                                ),
                        )
                        .fail
                    {
                        return FAIL;
                    }
                }
            }
        }
        (*emitter).whitespace = false;
        (*emitter).indention = false;
        OK
    }
    unsafe fn yaml_emitter_write_plain_scalar(
        emitter: *mut YamlEmitterT,
        value: *mut yaml_char_t,
        length: size_t,
        allow_breaks: bool,
    ) -> Success {
        let mut spaces = false;
        let mut breaks = false;
        let mut string = YamlStringT {
            start: value,
            end: value.wrapping_offset(length as isize),
            pointer: value,
        };
        if !(*emitter).whitespace && (length != 0 || (*emitter).flow_level != 0)
            && put(emitter, b' ').fail
        {
            return FAIL;
        }
        while string.pointer != string.end {
            if *string.pointer.offset(0) == b' ' {
                if allow_breaks && !spaces && (*emitter).column > (*emitter).best_width
                    && !(*string.pointer.offset(1) == b' ')
                {
                    if yaml_emitter_write_indent(emitter).fail {
                        return FAIL;
                    }
                    string.pointer = string
                        .pointer
                        .wrapping_offset(
                            if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                1
                            } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                                2
                            } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                                3
                            } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                                4
                            } else {
                                0
                            },
                        );
                } else if write(emitter, &raw mut string).fail {
                    return FAIL;
                }
                spaces = true;
            } else if *string.pointer.offset(0) == b'\r'
                || *string.pointer.offset(0) == b'\n'
                || *string.pointer.offset(0) == b'\xC2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x85'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA8'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA9'
            {
                if !breaks && *string.pointer == b'\n' && put_break(emitter).fail {
                    return FAIL;
                }
                if write_break(emitter, &raw mut string).fail {
                    return FAIL;
                }
                (*emitter).indention = true;
                breaks = true;
            } else {
                if breaks && yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
                if write(emitter, &raw mut string).fail {
                    return FAIL;
                }
                (*emitter).indention = false;
                spaces = false;
                breaks = false;
            }
        }
        (*emitter).whitespace = false;
        (*emitter).indention = false;
        OK
    }
    unsafe fn yaml_emitter_write_single_quoted_scalar(
        emitter: *mut YamlEmitterT,
        value: *mut yaml_char_t,
        length: size_t,
        allow_breaks: bool,
    ) -> Success {
        let mut spaces = false;
        let mut breaks = false;
        let mut string = YamlStringT {
            start: value,
            end: value.wrapping_offset(length as isize),
            pointer: value,
        };
        if yaml_emitter_write_indicator(
                emitter,
                b"'\0" as *const u8 as *const libc::c_char,
                true,
                false,
                false,
            )
            .fail
        {
            return FAIL;
        }
        while string.pointer != string.end {
            if *string.pointer.offset(0) == b' ' {
                if allow_breaks && !spaces && (*emitter).column > (*emitter).best_width
                    && string.pointer != string.start
                    && string.pointer != string.end.wrapping_offset(-1_isize)
                    && !(*string.pointer.offset(1) == b' ')
                {
                    if yaml_emitter_write_indent(emitter).fail {
                        return FAIL;
                    }
                    string.pointer = string
                        .pointer
                        .wrapping_offset(
                            if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                1
                            } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                                2
                            } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                                3
                            } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                                4
                            } else {
                                0
                            },
                        );
                } else if write(emitter, &raw mut string).fail {
                    return FAIL;
                }
                spaces = true;
            } else if *string.pointer.offset(0) == b'\r'
                || *string.pointer.offset(0) == b'\n'
                || *string.pointer.offset(0) == b'\xC2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x85'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA8'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA9'
            {
                if !breaks && *string.pointer == b'\n' && put_break(emitter).fail {
                    return FAIL;
                }
                if write_break(emitter, &raw mut string).fail {
                    return FAIL;
                }
                (*emitter).indention = true;
                breaks = true;
            } else {
                if breaks && yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
                if *string.pointer == b'\'' && put(emitter, b'\'').fail {
                    return FAIL;
                }
                if write(emitter, &raw mut string).fail {
                    return FAIL;
                }
                (*emitter).indention = false;
                spaces = false;
                breaks = false;
            }
        }
        if breaks && yaml_emitter_write_indent(emitter).fail {
            return FAIL;
        }
        if yaml_emitter_write_indicator(
                emitter,
                b"'\0" as *const u8 as *const libc::c_char,
                false,
                false,
                false,
            )
            .fail
        {
            return FAIL;
        }
        (*emitter).whitespace = false;
        (*emitter).indention = false;
        OK
    }
    unsafe fn yaml_emitter_write_double_quoted_scalar(
        emitter: *mut YamlEmitterT,
        value: *mut yaml_char_t,
        length: size_t,
        allow_breaks: bool,
    ) -> Success {
        let mut spaces = false;
        let mut string = YamlStringT {
            start: value,
            end: value.wrapping_offset(length as isize),
            pointer: value,
        };
        if yaml_emitter_write_indicator(
                emitter,
                b"\"\0" as *const u8 as *const libc::c_char,
                true,
                false,
                false,
            )
            .fail
        {
            return FAIL;
        }
        while string.pointer != string.end {
            if !match *string.pointer {
                0x0A | 0x20..=0x7E => true,
                0xC2 => {
                    match *string.pointer.wrapping_offset(1) {
                        0xA0..=0xBF => true,
                        _ => false,
                    }
                }
                0xC3..=0xEC => true,
                0xED => {
                    match *string.pointer.wrapping_offset(1) {
                        0x00..=0x9F => true,
                        _ => false,
                    }
                }
                0xEE => true,
                0xEF => {
                    match *string.pointer.wrapping_offset(1) {
                        0xBB => {
                            match *string.pointer.wrapping_offset(2) {
                                0xBF => false,
                                _ => true,
                            }
                        }
                        0xBF => {
                            match *string.pointer.wrapping_offset(2) {
                                0xBE | 0xBF => false,
                                _ => true,
                            }
                        }
                        _ => true,
                    }
                }
                0xF0..=0xF4 => true,
                _ => false,
            } || !(*emitter).unicode && !(*string.pointer <= b'\x7F')
                || *string.pointer.offset(0) == b'\xEF'
                    && *string.pointer.offset(1) == b'\xBB'
                    && *string.pointer.offset(2) == b'\xBF'
                || (*string.pointer.offset(0) == b'\r'
                    || *string.pointer.offset(0) == b'\n'
                    || *string.pointer.offset(0) == b'\xC2'
                        && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x85'
                    || *string.pointer.offset(0) == b'\xE2'
                        && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                        && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA8'
                    || *string.pointer.offset(0) == b'\xE2'
                        && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                        && *string.pointer.offset((0 + 2).try_into().unwrap())
                            == b'\xA9') || *string.pointer == b'"'
                || *string.pointer == b'\\'
            {
                let mut octet: libc::c_uchar;
                let mut width: libc::c_uint;
                let mut value_0: libc::c_uint;
                let mut k: libc::c_int;
                octet = *string.pointer;
                width = if octet & 0x80 == 0x00 {
                    1
                } else if octet & 0xE0 == 0xC0 {
                    2
                } else if octet & 0xF0 == 0xE0 {
                    3
                } else if octet & 0xF8 == 0xF0 {
                    4
                } else {
                    0
                };
                value_0 = if octet & 0x80 == 0 {
                    octet & 0x7F
                } else if octet & 0xE0 == 0xC0 {
                    octet & 0x1F
                } else if octet & 0xF0 == 0xE0 {
                    octet & 0x0F
                } else if octet & 0xF8 == 0xF0 {
                    octet & 0x07
                } else {
                    0
                } as libc::c_uint;
                k = 1;
                while k < width as libc::c_int {
                    octet = *string.pointer.wrapping_offset(k as isize);
                    value_0 = (value_0 << 6).force_add((octet & 0x3F) as libc::c_uint);
                    k += 1;
                }
                string.pointer = string.pointer.wrapping_offset(width as isize);
                if put(emitter, b'\\').fail {
                    return FAIL;
                }
                match value_0 {
                    0x00 => {
                        if put(emitter, b'0').fail {
                            return FAIL;
                        }
                    }
                    0x07 => {
                        if put(emitter, b'a').fail {
                            return FAIL;
                        }
                    }
                    0x08 => {
                        if put(emitter, b'b').fail {
                            return FAIL;
                        }
                    }
                    0x09 => {
                        if put(emitter, b't').fail {
                            return FAIL;
                        }
                    }
                    0x0A => {
                        if put(emitter, b'n').fail {
                            return FAIL;
                        }
                    }
                    0x0B => {
                        if put(emitter, b'v').fail {
                            return FAIL;
                        }
                    }
                    0x0C => {
                        if put(emitter, b'f').fail {
                            return FAIL;
                        }
                    }
                    0x0D => {
                        if put(emitter, b'r').fail {
                            return FAIL;
                        }
                    }
                    0x1B => {
                        if put(emitter, b'e').fail {
                            return FAIL;
                        }
                    }
                    0x22 => {
                        if put(emitter, b'"').fail {
                            return FAIL;
                        }
                    }
                    0x5C => {
                        if put(emitter, b'\\').fail {
                            return FAIL;
                        }
                    }
                    0x85 => {
                        if put(emitter, b'N').fail {
                            return FAIL;
                        }
                    }
                    0xA0 => {
                        if put(emitter, b'_').fail {
                            return FAIL;
                        }
                    }
                    0x2028 => {
                        if put(emitter, b'L').fail {
                            return FAIL;
                        }
                    }
                    0x2029 => {
                        if put(emitter, b'P').fail {
                            return FAIL;
                        }
                    }
                    _ => {
                        if value_0 <= 0xFF {
                            if put(emitter, b'x').fail {
                                return FAIL;
                            }
                            width = 2;
                        } else if value_0 <= 0xFFFF {
                            if put(emitter, b'u').fail {
                                return FAIL;
                            }
                            width = 4;
                        } else {
                            if put(emitter, b'U').fail {
                                return FAIL;
                            }
                            width = 8;
                        }
                        k = width.wrapping_sub(1).wrapping_mul(4) as libc::c_int;
                        while k >= 0 {
                            let digit: libc::c_int = (value_0 >> k & 0x0F)
                                as libc::c_int;
                            if put(
                                    emitter,
                                    (digit + if digit < 10 { b'0' } else { b'A' - 10 } as i32)
                                        as u8,
                                )
                                .fail
                            {
                                return FAIL;
                            }
                            k -= 4;
                        }
                    }
                }
                spaces = false;
            } else if *string.pointer.offset(0) == b' ' {
                if allow_breaks && !spaces && (*emitter).column > (*emitter).best_width
                    && string.pointer != string.start
                    && string.pointer != string.end.wrapping_offset(-1_isize)
                {
                    if yaml_emitter_write_indent(emitter).fail {
                        return FAIL;
                    }
                    if *string.pointer.offset(1) == b' ' && put(emitter, b'\\').fail {
                        return FAIL;
                    }
                    string.pointer = string
                        .pointer
                        .wrapping_offset(
                            if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                1
                            } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                                2
                            } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                                3
                            } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                                4
                            } else {
                                0
                            },
                        );
                } else if write(emitter, &raw mut string).fail {
                    return FAIL;
                }
                spaces = true;
            } else {
                if write(emitter, &raw mut string).fail {
                    return FAIL;
                }
                spaces = false;
            }
        }
        if yaml_emitter_write_indicator(
                emitter,
                b"\"\0" as *const u8 as *const libc::c_char,
                false,
                false,
                false,
            )
            .fail
        {
            return FAIL;
        }
        (*emitter).whitespace = false;
        (*emitter).indention = false;
        OK
    }
    unsafe fn yaml_emitter_write_block_scalar_hints(
        emitter: *mut YamlEmitterT,
        mut string: YamlStringT,
    ) -> Success {
        let mut indent_hint: [libc::c_char; 2] = [0; 2];
        let mut chomp_hint: *const libc::c_char = ptr::null::<libc::c_char>();
        if *string.pointer.offset(0) == b' '
            || (*string.pointer.offset(0) == b'\r' || *string.pointer.offset(0) == b'\n'
                || *string.pointer.offset(0) == b'\xC2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x85'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA8'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA9')
        {
            indent_hint[0] = (b'0' as libc::c_int + (*emitter).best_indent)
                as libc::c_char;
            indent_hint[1] = '\0' as libc::c_char;
            if yaml_emitter_write_indicator(
                    emitter,
                    indent_hint.as_mut_ptr(),
                    false,
                    false,
                    false,
                )
                .fail
            {
                return FAIL;
            }
        }
        (*emitter).open_ended = 0;
        string.pointer = string.end;
        if string.start == string.pointer {
            chomp_hint = b"-\0" as *const u8 as *const libc::c_char;
        } else {
            loop {
                string.pointer = string.pointer.wrapping_offset(-1);
                if *string.pointer & 0xC0 != 0x80 {
                    break;
                }
            }
            if !(*string.pointer.offset(0) == b'\r' || *string.pointer.offset(0) == b'\n'
                || *string.pointer.offset(0) == b'\xC2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x85'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA8'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA9')
            {
                chomp_hint = b"-\0" as *const u8 as *const libc::c_char;
            } else if string.start == string.pointer {
                chomp_hint = b"+\0" as *const u8 as *const libc::c_char;
                (*emitter).open_ended = 2;
            } else {
                loop {
                    string.pointer = string.pointer.wrapping_offset(-1);
                    if *string.pointer & 0xC0 != 0x80 {
                        break;
                    }
                }
                if *string.pointer.offset(0) == b'\r'
                    || *string.pointer.offset(0) == b'\n'
                    || *string.pointer.offset(0) == b'\xC2'
                        && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x85'
                    || *string.pointer.offset(0) == b'\xE2'
                        && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                        && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA8'
                    || *string.pointer.offset(0) == b'\xE2'
                        && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                        && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA9'
                {
                    chomp_hint = b"+\0" as *const u8 as *const libc::c_char;
                    (*emitter).open_ended = 2;
                }
            }
        }
        if !chomp_hint.is_null()
            && yaml_emitter_write_indicator(emitter, chomp_hint, false, false, false)
                .fail
        {
            return FAIL;
        }
        OK
    }
    unsafe fn yaml_emitter_write_literal_scalar(
        emitter: *mut YamlEmitterT,
        value: *mut yaml_char_t,
        length: size_t,
    ) -> Success {
        let mut breaks = true;
        let mut string = YamlStringT {
            start: value,
            end: value.wrapping_offset(length as isize),
            pointer: value,
        };
        if yaml_emitter_write_indicator(
                emitter,
                b"|\0" as *const u8 as *const libc::c_char,
                true,
                false,
                false,
            )
            .fail
        {
            return FAIL;
        }
        if yaml_emitter_write_block_scalar_hints(emitter, string).fail {
            return FAIL;
        }
        if put_break(emitter).fail {
            return FAIL;
        }
        (*emitter).indention = true;
        (*emitter).whitespace = true;
        while string.pointer != string.end {
            if *string.pointer.offset(0) == b'\r' || *string.pointer.offset(0) == b'\n'
                || *string.pointer.offset(0) == b'\xC2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x85'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA8'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA9'
            {
                if write_break(emitter, &raw mut string).fail {
                    return FAIL;
                }
                (*emitter).indention = true;
                breaks = true;
            } else {
                if breaks && yaml_emitter_write_indent(emitter).fail {
                    return FAIL;
                }
                if write(emitter, &raw mut string).fail {
                    return FAIL;
                }
                (*emitter).indention = false;
                breaks = false;
            }
        }
        OK
    }
    unsafe fn yaml_emitter_write_folded_scalar(
        emitter: *mut YamlEmitterT,
        value: *mut yaml_char_t,
        length: size_t,
    ) -> Success {
        let mut breaks = true;
        let mut leading_spaces = true;
        let mut string = YamlStringT {
            start: value,
            end: value.wrapping_offset(length as isize),
            pointer: value,
        };
        if yaml_emitter_write_indicator(
                emitter,
                b">\0" as *const u8 as *const libc::c_char,
                true,
                false,
                false,
            )
            .fail
        {
            return FAIL;
        }
        if yaml_emitter_write_block_scalar_hints(emitter, string).fail {
            return FAIL;
        }
        if put_break(emitter).fail {
            return FAIL;
        }
        (*emitter).indention = true;
        (*emitter).whitespace = true;
        while string.pointer != string.end {
            if *string.pointer.offset(0) == b'\r' || *string.pointer.offset(0) == b'\n'
                || *string.pointer.offset(0) == b'\xC2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x85'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA8'
                || *string.pointer.offset(0) == b'\xE2'
                    && *string.pointer.offset((0 + 1).try_into().unwrap()) == b'\x80'
                    && *string.pointer.offset((0 + 2).try_into().unwrap()) == b'\xA9'
            {
                if !breaks && !leading_spaces && *string.pointer == b'\n' {
                    let mut k: libc::c_int = 0;
                    while *string.pointer.offset(k as isize) == b'\r'
                        || *string.pointer.offset(k as isize) == b'\n'
                        || *string.pointer.offset(k as isize) == b'\xC2'
                            && *string
                                .pointer
                                .offset((k as isize + 1).try_into().unwrap()) == b'\x85'
                        || *string.pointer.offset(k as isize) == b'\xE2'
                            && *string
                                .pointer
                                .offset((k as isize + 1).try_into().unwrap()) == b'\x80'
                            && *string
                                .pointer
                                .offset((k as isize + 2).try_into().unwrap()) == b'\xA8'
                        || *string.pointer.offset(k as isize) == b'\xE2'
                            && *string
                                .pointer
                                .offset((k as isize + 1).try_into().unwrap()) == b'\x80'
                            && *string
                                .pointer
                                .offset((k as isize + 2).try_into().unwrap()) == b'\xA9'
                    {
                        k
                            += if *string.pointer.wrapping_offset(k as isize) & 0x80
                                == 0x00
                            {
                                1
                            } else if *string.pointer.wrapping_offset(k as isize) & 0xE0
                                == 0xC0
                            {
                                2
                            } else if *string.pointer.wrapping_offset(k as isize) & 0xF0
                                == 0xE0
                            {
                                3
                            } else if *string.pointer.wrapping_offset(k as isize) & 0xF8
                                == 0xF0
                            {
                                4
                            } else {
                                0
                            };
                    }
                    if !(*string.pointer.offset(k as isize) == b' '
                        || *string.pointer.offset(k as isize) == b'\t'
                        || (*string.pointer.offset(k as isize) == b'\r'
                            || *string.pointer.offset(k as isize) == b'\n'
                            || *string.pointer.offset(k as isize) == b'\xC2'
                                && *string
                                    .pointer
                                    .offset((k as isize + 1).try_into().unwrap()) == b'\x85'
                            || *string.pointer.offset(k as isize) == b'\xE2'
                                && *string
                                    .pointer
                                    .offset((k as isize + 1).try_into().unwrap()) == b'\x80'
                                && *string
                                    .pointer
                                    .offset((k as isize + 2).try_into().unwrap()) == b'\xA8'
                            || *string.pointer.offset(k as isize) == b'\xE2'
                                && *string
                                    .pointer
                                    .offset((k as isize + 1).try_into().unwrap()) == b'\x80'
                                && *string
                                    .pointer
                                    .offset((k as isize + 2).try_into().unwrap()) == b'\xA9'
                            || *string.pointer.offset(k as isize) == b'\0'))
                        && put_break(emitter).fail
                    {
                        return FAIL;
                    }
                }
                if write_break(emitter, &raw mut string).fail {
                    return FAIL;
                }
                (*emitter).indention = true;
                breaks = true;
            } else {
                if breaks {
                    if yaml_emitter_write_indent(emitter).fail {
                        return FAIL;
                    }
                    leading_spaces = *string.pointer.offset(0) == b' '
                        || *string.pointer.offset(0) == b'\t';
                }
                if !breaks && *string.pointer.offset(0) == b' '
                    && !(*string.pointer.offset(1) == b' ')
                    && (*emitter).column > (*emitter).best_width
                {
                    if yaml_emitter_write_indent(emitter).fail {
                        return FAIL;
                    }
                    string.pointer = string
                        .pointer
                        .wrapping_offset(
                            if *string.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                                1
                            } else if *string.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                                2
                            } else if *string.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                                3
                            } else if *string.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                                4
                            } else {
                                0
                            },
                        );
                } else if write(emitter, &raw mut string).fail {
                    return FAIL;
                }
                (*emitter).indention = false;
                breaks = false;
            }
        }
        OK
    }
}
/// Loader module for LibYML.
///
/// This module contains functions for loading YAML data.
pub mod loader {
    use crate::externs::{memset, strcmp};
    use crate::internal::yaml_stack_extend;
    use crate::memory::{yaml_free, yaml_malloc, yaml_strdup};
    use crate::success::{Success, FAIL, OK};
    use crate::yaml::yaml_char_t;
    use crate::{
        libc, yaml_document_delete, yaml_parser_parse, PointerExt, YamlAliasDataT,
        YamlAliasEvent, YamlComposerError, YamlDocumentEndEvent, YamlDocumentStartEvent,
        YamlDocumentT, YamlEventT, YamlMappingEndEvent, YamlMappingNode,
        YamlMappingStartEvent, YamlMarkT, YamlMemoryError, YamlNodeItemT, YamlNodePairT,
        YamlNodeT, YamlParserT, YamlScalarEvent, YamlScalarNode, YamlSequenceEndEvent,
        YamlSequenceNode, YamlSequenceStartEvent, YamlStreamEndEvent,
        YamlStreamStartEvent,
    };
    use core::mem::{size_of, MaybeUninit};
    use core::ptr::{self, addr_of_mut};
    #[repr(C)]
    struct LoaderCtx {
        start: *mut libc::c_int,
        end: *mut libc::c_int,
        top: *mut libc::c_int,
    }
    /// Parse the input stream and produce the next YAML document.
    ///
    /// Call this function subsequently to produce a sequence of documents
    /// constituting the input stream.
    ///
    /// If the produced document has no root node, it means that the document end
    /// has been reached.
    ///
    /// An application is responsible for freeing any data associated with the
    /// produced document object using the yaml_document_delete() function.
    ///
    /// An application must not alternate the calls of yaml_parser_load() with the
    /// calls of yaml_parser_scan() or yaml_parser_parse(). Doing this will break
    /// the parser.
    ///
    /// # Safety
    ///
    /// - `parser` must be a valid, non-null pointer to a properly initialized `YamlParserT` struct.
    /// - `document` must be a valid, non-null pointer to a `YamlDocumentT` struct that can be safely written to.
    /// - The `YamlParserT` and `YamlDocumentT` structs must be properly aligned and have the expected memory layout.
    /// - The caller must call `yaml_document_delete` to free any data associated with the produced document object.
    /// - The caller must not alternate calls to `yaml_parser_load` with calls to `yaml_parser_scan` or `yaml_parser_parse` on the same `YamlParserT` instance.
    pub unsafe fn yaml_parser_load(
        parser: *mut YamlParserT,
        document: *mut YamlDocumentT,
    ) -> Success {
        let current_block: u64;
        let mut event = MaybeUninit::<YamlEventT>::uninit();
        let event = event.as_mut_ptr();
        if !!parser.is_null() {
            crate::externs::__assert_fail("!parser.is_null()", "src/loader.rs", 55u32);
        }
        if !!document.is_null() {
            crate::externs::__assert_fail("!document.is_null()", "src/loader.rs", 56u32);
        }
        let _ = memset(
            document as *mut libc::c_void,
            0,
            size_of::<YamlDocumentT>() as libc::c_ulong,
        );
        {
            (*document).nodes.start = yaml_malloc(
                16 * size_of::<YamlNodeT>() as libc::c_ulong,
            ) as *mut YamlNodeT;
            (*document).nodes.top = (*document).nodes.start;
            (*document).nodes.end = (*document).nodes.start.offset(16_isize);
        };
        if !(*parser).stream_start_produced {
            if yaml_parser_parse(parser, event).fail {
                current_block = 6234624449317607669;
            } else {
                if !((*event).type_ == YamlStreamStartEvent) {
                    crate::externs::__assert_fail(
                        "(*event).type_ == YamlStreamStartEvent",
                        "src/loader.rs",
                        67u32,
                    );
                }
                current_block = 7815301370352969686;
            }
        } else {
            current_block = 7815301370352969686;
        }
        if current_block != 6234624449317607669 {
            if (*parser).stream_end_produced {
                return OK;
            }
            if yaml_parser_parse(parser, event).ok {
                if (*event).type_ == YamlStreamEndEvent {
                    return OK;
                }
                {
                    (*parser).aliases.start = yaml_malloc(
                        16 * size_of::<YamlAliasDataT>() as libc::c_ulong,
                    ) as *mut YamlAliasDataT;
                    (*parser).aliases.top = (*parser).aliases.start;
                    (*parser).aliases.end = (*parser).aliases.start.offset(16_isize);
                };
                let fresh6 = &raw mut (*parser).document;
                *fresh6 = document;
                if yaml_parser_load_document(parser, event).ok {
                    yaml_parser_delete_aliases(parser);
                    let fresh7 = &raw mut (*parser).document;
                    *fresh7 = ptr::null_mut::<YamlDocumentT>();
                    return OK;
                }
            }
        }
        yaml_parser_delete_aliases(parser);
        yaml_document_delete(document);
        let fresh8 = &raw mut (*parser).document;
        *fresh8 = ptr::null_mut::<YamlDocumentT>();
        FAIL
    }
    /// Sets the error type to `YamlComposerError` and stores the problem and its mark.
    ///
    /// # Parameters
    ///
    /// * `parser`: A mutable pointer to the `YamlParserT` struct.
    /// * `problem`: A pointer to a constant C string representing the problem.
    /// * `problem_mark`: A `YamlMarkT` struct representing the mark where the problem occurred.
    ///
    /// # Return
    ///
    /// Returns `FAIL` to indicate an error.
    ///
    /// # Safety
    ///
    /// - `parser` must be a valid, non-null pointer to a properly initialized `YamlParserT` struct.
    /// - `problem` must be a valid, non-null pointer to a constant C string.
    /// - `problem_mark` must be a valid `YamlMarkT` struct.
    /// - The `YamlParserT` struct must be properly aligned and have the expected memory layout.
    /// - The `problem` string must be null-terminated.
    /// - The `problem_mark` struct must be properly initialized.
    /// - The caller must handle the error and clean up any resources.
    ///
    pub unsafe fn yaml_parser_set_composer_error(
        parser: *mut YamlParserT,
        problem: *const libc::c_char,
        problem_mark: YamlMarkT,
    ) -> Success {
        (*parser).error = YamlComposerError;
        let fresh9 = &raw mut (*parser).problem;
        *fresh9 = problem;
        (*parser).problem_mark = problem_mark;
        FAIL
    }
    unsafe fn yaml_parser_set_composer_error_context(
        parser: *mut YamlParserT,
        context: *const libc::c_char,
        context_mark: YamlMarkT,
        problem: *const libc::c_char,
        problem_mark: YamlMarkT,
    ) -> Success {
        (*parser).error = YamlComposerError;
        let fresh10 = &raw mut (*parser).context;
        *fresh10 = context;
        (*parser).context_mark = context_mark;
        let fresh11 = &raw mut (*parser).problem;
        *fresh11 = problem;
        (*parser).problem_mark = problem_mark;
        FAIL
    }
    unsafe fn yaml_parser_delete_aliases(parser: *mut YamlParserT) {
        while !((*parser).aliases.start == (*parser).aliases.top) {
            yaml_free(
                (*{
                    (*parser).aliases.top = (*parser).aliases.top.offset(-1);
                    (*parser).aliases.top
                })
                    .anchor as *mut libc::c_void,
            );
        }
        yaml_free((*parser).aliases.start as *mut libc::c_void);
        (*parser).aliases.end = ptr::null_mut();
        (*parser).aliases.top = ptr::null_mut();
        (*parser).aliases.start = ptr::null_mut();
    }
    unsafe fn yaml_parser_load_document(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
    ) -> Success {
        let mut ctx = LoaderCtx {
            start: ptr::null_mut::<libc::c_int>(),
            end: ptr::null_mut::<libc::c_int>(),
            top: ptr::null_mut::<libc::c_int>(),
        };
        if !((*event).type_ == YamlDocumentStartEvent) {
            crate::externs::__assert_fail(
                "(*event).type_ == YamlDocumentStartEvent",
                "src/loader.rs",
                166u32,
            );
        }
        let fresh16 = &raw mut (*(*parser).document).version_directive;
        *fresh16 = (*event).data.document_start.version_directive;
        let fresh17 = &raw mut (*(*parser).document).tag_directives.start;
        *fresh17 = (*event).data.document_start.tag_directives.start;
        let fresh18 = &raw mut (*(*parser).document).tag_directives.end;
        *fresh18 = (*event).data.document_start.tag_directives.end;
        (*(*parser).document).start_implicit = (*event).data.document_start.implicit;
        (*(*parser).document).start_mark = (*event).start_mark;
        {
            ctx.start = yaml_malloc(16 * size_of::<libc::c_int>() as libc::c_ulong)
                as *mut libc::c_int;
            ctx.top = ctx.start;
            ctx.end = ctx.start.offset(16_isize);
        };
        if yaml_parser_load_nodes(parser, &raw mut ctx).fail {
            yaml_free(ctx.start as *mut libc::c_void);
            ctx.end = ptr::null_mut();
            ctx.top = ptr::null_mut();
            ctx.start = ptr::null_mut();
            return FAIL;
        }
        yaml_free(ctx.start as *mut libc::c_void);
        ctx.end = ptr::null_mut();
        ctx.top = ptr::null_mut();
        ctx.start = ptr::null_mut();
        OK
    }
    unsafe fn yaml_parser_load_nodes(
        parser: *mut YamlParserT,
        ctx: *mut LoaderCtx,
    ) -> Success {
        let mut event = MaybeUninit::<YamlEventT>::uninit();
        let event = event.as_mut_ptr();
        loop {
            if yaml_parser_parse(parser, event).fail {
                return FAIL;
            }
            match (*event).type_ {
                YamlAliasEvent => {
                    if yaml_parser_load_alias(parser, event, ctx).fail {
                        return FAIL;
                    }
                }
                YamlScalarEvent => {
                    if yaml_parser_load_scalar(parser, event, ctx).fail {
                        return FAIL;
                    }
                }
                YamlSequenceStartEvent => {
                    if yaml_parser_load_sequence(parser, event, ctx).fail {
                        return FAIL;
                    }
                }
                YamlSequenceEndEvent => {
                    if yaml_parser_load_sequence_end(parser, event, ctx).fail {
                        return FAIL;
                    }
                }
                YamlMappingStartEvent => {
                    if yaml_parser_load_mapping(parser, event, ctx).fail {
                        return FAIL;
                    }
                }
                YamlMappingEndEvent => {
                    if yaml_parser_load_mapping_end(parser, event, ctx).fail {
                        return FAIL;
                    }
                }
                YamlDocumentEndEvent => {}
                _ => {
                    crate::externs::__assert_fail("false", "src/loader.rs", 233u32);
                }
            }
            if (*event).type_ == YamlDocumentEndEvent {
                break;
            }
        }
        (*(*parser).document).end_implicit = (*event).data.document_end.implicit;
        (*(*parser).document).end_mark = (*event).end_mark;
        OK
    }
    unsafe fn yaml_parser_register_anchor(
        parser: *mut YamlParserT,
        index: libc::c_int,
        anchor: *mut yaml_char_t,
    ) -> Success {
        let mut data = MaybeUninit::<YamlAliasDataT>::uninit();
        let data = data.as_mut_ptr();
        let mut alias_data: *mut YamlAliasDataT;
        if anchor.is_null() {
            return OK;
        }
        (*data).anchor = anchor;
        (*data).index = index;
        (*data).mark = (*(*(*parser).document)
            .nodes
            .start
            .wrapping_offset((index - 1) as isize))
            .start_mark;
        alias_data = (*parser).aliases.start;
        while alias_data != (*parser).aliases.top {
            if strcmp(
                (*alias_data).anchor as *mut libc::c_char,
                anchor as *mut libc::c_char,
            ) == 0
            {
                yaml_free(anchor as *mut libc::c_void);
                return yaml_parser_set_composer_error_context(
                    parser,
                    b"found duplicate anchor; first occurrence\0" as *const u8
                        as *const libc::c_char,
                    (*alias_data).mark,
                    b"second occurrence\0" as *const u8 as *const libc::c_char,
                    (*data).mark,
                );
            }
            alias_data = alias_data.wrapping_offset(1);
        }
        {
            if (*parser).aliases.top == (*parser).aliases.end {
                yaml_stack_extend(
                    &raw mut (*parser).aliases.start as *mut *mut libc::c_void,
                    &raw mut (*parser).aliases.top as *mut *mut libc::c_void,
                    &raw mut (*parser).aliases.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(data, (*parser).aliases.top, 1);
            (*parser).aliases.top = (*parser).aliases.top.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_load_node_add(
        parser: *mut YamlParserT,
        ctx: *mut LoaderCtx,
        index: libc::c_int,
    ) -> Success {
        if (*ctx).start == (*ctx).top {
            return OK;
        }
        let parent_index: libc::c_int = *(*ctx).top.wrapping_offset(-1_isize);
        let parent: *mut YamlNodeT = &raw mut *((*(*parser).document).nodes.start)
            .wrapping_offset((parent_index - 1) as isize);
        let current_block_17: u64;
        match (*parent).type_ {
            YamlSequenceNode => {
                if if (*parent)
                    .data
                    .sequence
                    .items
                    .top
                    .c_offset_from((*parent).data.sequence.items.start)
                    < libc::c_int::MAX as isize - 1
                {
                    OK
                } else {
                    (*parser).error = YamlMemoryError;
                    FAIL
                }
                    .fail
                {
                    return FAIL;
                }
                {
                    if (*parent).data.sequence.items.top
                        == (*parent).data.sequence.items.end
                    {
                        yaml_stack_extend(
                            &raw mut (*parent).data.sequence.items.start
                                as *mut *mut libc::c_void,
                            &raw mut (*parent).data.sequence.items.top
                                as *mut *mut libc::c_void,
                            &raw mut (*parent).data.sequence.items.end
                                as *mut *mut libc::c_void,
                        );
                    }
                    ptr::write((*parent).data.sequence.items.top, index);
                    (*parent).data.sequence.items.top = (*parent)
                        .data
                        .sequence
                        .items
                        .top
                        .wrapping_offset(1);
                };
            }
            YamlMappingNode => {
                let mut pair = MaybeUninit::<YamlNodePairT>::uninit();
                let pair = pair.as_mut_ptr();
                if !((*parent).data.mapping.pairs.start
                    == (*parent).data.mapping.pairs.top)
                {
                    let p: *mut YamlNodePairT = (*parent)
                        .data
                        .mapping
                        .pairs
                        .top
                        .wrapping_offset(-1_isize);
                    if (*p).key != 0 && (*p).value == 0 {
                        (*p).value = index;
                        current_block_17 = 11307063007268554308;
                    } else {
                        current_block_17 = 17407779659766490442;
                    }
                } else {
                    current_block_17 = 17407779659766490442;
                }
                match current_block_17 {
                    11307063007268554308 => {}
                    _ => {
                        (*pair).key = index;
                        (*pair).value = 0;
                        if if (*parent)
                            .data
                            .mapping
                            .pairs
                            .top
                            .c_offset_from((*parent).data.mapping.pairs.start)
                            < libc::c_int::MAX as isize - 1
                        {
                            OK
                        } else {
                            (*parser).error = YamlMemoryError;
                            FAIL
                        }
                            .fail
                        {
                            return FAIL;
                        }
                        {
                            if (*parent).data.mapping.pairs.top
                                == (*parent).data.mapping.pairs.end
                            {
                                yaml_stack_extend(
                                    &raw mut (*parent).data.mapping.pairs.start
                                        as *mut *mut libc::c_void,
                                    &raw mut (*parent).data.mapping.pairs.top
                                        as *mut *mut libc::c_void,
                                    &raw mut (*parent).data.mapping.pairs.end
                                        as *mut *mut libc::c_void,
                                );
                            }
                            ptr::copy_nonoverlapping(
                                pair,
                                (*parent).data.mapping.pairs.top,
                                1,
                            );
                            (*parent).data.mapping.pairs.top = (*parent)
                                .data
                                .mapping
                                .pairs
                                .top
                                .wrapping_offset(1);
                        };
                    }
                }
            }
            _ => {
                crate::externs::__assert_fail("false", "src/loader.rs", 347u32);
            }
        }
        OK
    }
    unsafe fn yaml_parser_load_alias(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
        ctx: *mut LoaderCtx,
    ) -> Success {
        let anchor: *mut yaml_char_t = (*event).data.alias.anchor;
        let mut alias_data: *mut YamlAliasDataT;
        alias_data = (*parser).aliases.start;
        while alias_data != (*parser).aliases.top {
            if strcmp(
                (*alias_data).anchor as *mut libc::c_char,
                anchor as *mut libc::c_char,
            ) == 0
            {
                yaml_free(anchor as *mut libc::c_void);
                return yaml_parser_load_node_add(parser, ctx, (*alias_data).index);
            }
            alias_data = alias_data.wrapping_offset(1);
        }
        yaml_free(anchor as *mut libc::c_void);
        yaml_parser_set_composer_error(
            parser,
            b"found undefined alias\0" as *const u8 as *const libc::c_char,
            (*event).start_mark,
        )
    }
    unsafe fn yaml_parser_load_scalar(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
        ctx: *mut LoaderCtx,
    ) -> Success {
        let current_block: u64;
        let mut node = MaybeUninit::<YamlNodeT>::uninit();
        let node = node.as_mut_ptr();
        let index: libc::c_int;
        let mut tag: *mut yaml_char_t = (*event).data.scalar.tag;
        if if (*(*parser).document)
            .nodes
            .top
            .c_offset_from((*(*parser).document).nodes.start)
            < libc::c_int::MAX as isize - 1
        {
            OK
        } else {
            (*parser).error = YamlMemoryError;
            FAIL
        }
            .ok
        {
            if tag.is_null()
                || strcmp(
                    tag as *mut libc::c_char,
                    b"!\0" as *const u8 as *const libc::c_char,
                ) == 0
            {
                yaml_free(tag as *mut libc::c_void);
                tag = yaml_strdup(
                    b"tag:yaml.org,2002:str\0" as *const u8 as *const libc::c_char
                        as *mut yaml_char_t,
                );
                if tag.is_null() {
                    current_block = 10579931339944277179;
                } else {
                    current_block = 11006700562992250127;
                }
            } else {
                current_block = 11006700562992250127;
            }
            if current_block != 10579931339944277179 {
                let _ = memset(
                    node as *mut libc::c_void,
                    0,
                    size_of::<YamlNodeT>() as libc::c_ulong,
                );
                (*node).type_ = YamlScalarNode;
                (*node).tag = tag;
                (*node).start_mark = (*event).start_mark;
                (*node).end_mark = (*event).end_mark;
                (*node).data.scalar.value = (*event).data.scalar.value;
                (*node).data.scalar.length = (*event).data.scalar.length;
                (*node).data.scalar.style = (*event).data.scalar.style;
                {
                    if (*(*parser).document).nodes.top == (*(*parser).document).nodes.end
                    {
                        yaml_stack_extend(
                            &raw mut (*(*parser).document).nodes.start
                                as *mut *mut libc::c_void,
                            &raw mut (*(*parser).document).nodes.top
                                as *mut *mut libc::c_void,
                            &raw mut (*(*parser).document).nodes.end
                                as *mut *mut libc::c_void,
                        );
                    }
                    ptr::copy_nonoverlapping(node, (*(*parser).document).nodes.top, 1);
                    (*(*parser).document).nodes.top = (*(*parser).document)
                        .nodes
                        .top
                        .wrapping_offset(1);
                };
                index = (*(*parser).document)
                    .nodes
                    .top
                    .c_offset_from((*(*parser).document).nodes.start) as libc::c_int;
                if yaml_parser_register_anchor(
                        parser,
                        index,
                        (*event).data.scalar.anchor,
                    )
                    .fail
                {
                    return FAIL;
                }
                return yaml_parser_load_node_add(parser, ctx, index);
            }
        }
        yaml_free(tag as *mut libc::c_void);
        yaml_free((*event).data.scalar.anchor as *mut libc::c_void);
        yaml_free((*event).data.scalar.value as *mut libc::c_void);
        FAIL
    }
    unsafe fn yaml_parser_load_sequence(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
        ctx: *mut LoaderCtx,
    ) -> Success {
        let current_block: u64;
        let mut node = MaybeUninit::<YamlNodeT>::uninit();
        let node = node.as_mut_ptr();
        struct Items {
            start: *mut YamlNodeItemT,
            end: *mut YamlNodeItemT,
            top: *mut YamlNodeItemT,
        }
        let mut items = Items {
            start: ptr::null_mut::<YamlNodeItemT>(),
            end: ptr::null_mut::<YamlNodeItemT>(),
            top: ptr::null_mut::<YamlNodeItemT>(),
        };
        let index: libc::c_int;
        let mut tag: *mut yaml_char_t = (*event).data.sequence_start.tag;
        if if (*(*parser).document)
            .nodes
            .top
            .c_offset_from((*(*parser).document).nodes.start)
            < libc::c_int::MAX as isize - 1
        {
            OK
        } else {
            (*parser).error = YamlMemoryError;
            FAIL
        }
            .ok
        {
            if tag.is_null()
                || strcmp(
                    tag as *mut libc::c_char,
                    b"!\0" as *const u8 as *const libc::c_char,
                ) == 0
            {
                yaml_free(tag as *mut libc::c_void);
                tag = yaml_strdup(
                    b"tag:yaml.org,2002:seq\0" as *const u8 as *const libc::c_char
                        as *mut yaml_char_t,
                );
                if tag.is_null() {
                    current_block = 13474536459355229096;
                } else {
                    current_block = 6937071982253665452;
                }
            } else {
                current_block = 6937071982253665452;
            }
            if current_block != 13474536459355229096 {
                {
                    items.start = yaml_malloc(
                        16 * size_of::<YamlNodeItemT>() as libc::c_ulong,
                    ) as *mut YamlNodeItemT;
                    items.top = items.start;
                    items.end = items.start.offset(16_isize);
                };
                let _ = memset(
                    node as *mut libc::c_void,
                    0,
                    size_of::<YamlNodeT>() as libc::c_ulong,
                );
                (*node).type_ = YamlSequenceNode;
                (*node).tag = tag;
                (*node).start_mark = (*event).start_mark;
                (*node).end_mark = (*event).end_mark;
                (*node).data.sequence.items.start = items.start;
                (*node).data.sequence.items.end = items.end;
                (*node).data.sequence.items.top = items.start;
                (*node).data.sequence.style = (*event).data.sequence_start.style;
                {
                    if (*(*parser).document).nodes.top == (*(*parser).document).nodes.end
                    {
                        yaml_stack_extend(
                            &raw mut (*(*parser).document).nodes.start
                                as *mut *mut libc::c_void,
                            &raw mut (*(*parser).document).nodes.top
                                as *mut *mut libc::c_void,
                            &raw mut (*(*parser).document).nodes.end
                                as *mut *mut libc::c_void,
                        );
                    }
                    ptr::copy_nonoverlapping(node, (*(*parser).document).nodes.top, 1);
                    (*(*parser).document).nodes.top = (*(*parser).document)
                        .nodes
                        .top
                        .wrapping_offset(1);
                };
                index = (*(*parser).document)
                    .nodes
                    .top
                    .c_offset_from((*(*parser).document).nodes.start) as libc::c_int;
                if yaml_parser_register_anchor(
                        parser,
                        index,
                        (*event).data.sequence_start.anchor,
                    )
                    .fail
                {
                    return FAIL;
                }
                if yaml_parser_load_node_add(parser, ctx, index).fail {
                    return FAIL;
                }
                if if (*ctx).top.c_offset_from((*ctx).start)
                    < libc::c_int::MAX as isize - 1
                {
                    OK
                } else {
                    (*parser).error = YamlMemoryError;
                    FAIL
                }
                    .fail
                {
                    return FAIL;
                }
                {
                    if (*ctx).top == (*ctx).end {
                        yaml_stack_extend(
                            &raw mut (*ctx).start as *mut *mut libc::c_void,
                            &raw mut (*ctx).top as *mut *mut libc::c_void,
                            &raw mut (*ctx).end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::write((*ctx).top, index);
                    (*ctx).top = (*ctx).top.wrapping_offset(1);
                };
                return OK;
            }
        }
        yaml_free(tag as *mut libc::c_void);
        yaml_free((*event).data.sequence_start.anchor as *mut libc::c_void);
        FAIL
    }
    unsafe fn yaml_parser_load_sequence_end(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
        ctx: *mut LoaderCtx,
    ) -> Success {
        if !(((*ctx).top).c_offset_from((*ctx).start) as libc::c_long > 0_i64) {
            crate::externs::__assert_fail(
                "((*ctx).top).c_offset_from((*ctx).start) as libc::c_long > 0_i64",
                "src/loader.rs",
                544u32,
            );
        }
        let index: libc::c_int = *(*ctx).top.wrapping_offset(-1_isize);
        if !((*((*(*parser).document).nodes.start).wrapping_offset((index - 1) as isize))
            .type_ == YamlSequenceNode)
        {
            crate::externs::__assert_fail(
                "(*((*(*parser).document).nodes.start).wrapping_offset((index - 1) as\nisize)).type_ == YamlSequenceNode",
                "src/loader.rs",
                549u32,
            );
        }
        (*(*(*parser).document).nodes.start.wrapping_offset((index - 1) as isize))
            .end_mark = (*event).end_mark;
        let _ = *{
            (*ctx).top = (*ctx).top.offset(-1);
            (*ctx).top
        };
        OK
    }
    unsafe fn yaml_parser_load_mapping(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
        ctx: *mut LoaderCtx,
    ) -> Success {
        let current_block: u64;
        let mut node = MaybeUninit::<YamlNodeT>::uninit();
        let node = node.as_mut_ptr();
        struct Pairs {
            start: *mut YamlNodePairT,
            end: *mut YamlNodePairT,
            top: *mut YamlNodePairT,
        }
        let mut pairs = Pairs {
            start: ptr::null_mut::<YamlNodePairT>(),
            end: ptr::null_mut::<YamlNodePairT>(),
            top: ptr::null_mut::<YamlNodePairT>(),
        };
        let index: libc::c_int;
        let mut tag: *mut yaml_char_t = (*event).data.mapping_start.tag;
        if if (*(*parser).document)
            .nodes
            .top
            .c_offset_from((*(*parser).document).nodes.start)
            < libc::c_int::MAX as isize - 1
        {
            OK
        } else {
            (*parser).error = YamlMemoryError;
            FAIL
        }
            .ok
        {
            if tag.is_null()
                || strcmp(
                    tag as *mut libc::c_char,
                    b"!\0" as *const u8 as *const libc::c_char,
                ) == 0
            {
                yaml_free(tag as *mut libc::c_void);
                tag = yaml_strdup(
                    b"tag:yaml.org,2002:map\0" as *const u8 as *const libc::c_char
                        as *mut yaml_char_t,
                );
                if tag.is_null() {
                    current_block = 13635467803606088781;
                } else {
                    current_block = 6937071982253665452;
                }
            } else {
                current_block = 6937071982253665452;
            }
            if current_block != 13635467803606088781 {
                {
                    pairs.start = yaml_malloc(
                        16 * size_of::<YamlNodePairT>() as libc::c_ulong,
                    ) as *mut YamlNodePairT;
                    pairs.top = pairs.start;
                    pairs.end = pairs.start.offset(16_isize);
                };
                let _ = memset(
                    node as *mut libc::c_void,
                    0,
                    size_of::<YamlNodeT>() as libc::c_ulong,
                );
                (*node).type_ = YamlMappingNode;
                (*node).tag = tag;
                (*node).start_mark = (*event).start_mark;
                (*node).end_mark = (*event).end_mark;
                (*node).data.mapping.pairs.start = pairs.start;
                (*node).data.mapping.pairs.end = pairs.end;
                (*node).data.mapping.pairs.top = pairs.start;
                (*node).data.mapping.style = (*event).data.mapping_start.style;
                {
                    if (*(*parser).document).nodes.top == (*(*parser).document).nodes.end
                    {
                        yaml_stack_extend(
                            &raw mut (*(*parser).document).nodes.start
                                as *mut *mut libc::c_void,
                            &raw mut (*(*parser).document).nodes.top
                                as *mut *mut libc::c_void,
                            &raw mut (*(*parser).document).nodes.end
                                as *mut *mut libc::c_void,
                        );
                    }
                    ptr::copy_nonoverlapping(node, (*(*parser).document).nodes.top, 1);
                    (*(*parser).document).nodes.top = (*(*parser).document)
                        .nodes
                        .top
                        .wrapping_offset(1);
                };
                index = (*(*parser).document)
                    .nodes
                    .top
                    .c_offset_from((*(*parser).document).nodes.start) as libc::c_int;
                if yaml_parser_register_anchor(
                        parser,
                        index,
                        (*event).data.mapping_start.anchor,
                    )
                    .fail
                {
                    return FAIL;
                }
                if yaml_parser_load_node_add(parser, ctx, index).fail {
                    return FAIL;
                }
                if if (*ctx).top.c_offset_from((*ctx).start)
                    < libc::c_int::MAX as isize - 1
                {
                    OK
                } else {
                    (*parser).error = YamlMemoryError;
                    FAIL
                }
                    .fail
                {
                    return FAIL;
                }
                {
                    if (*ctx).top == (*ctx).end {
                        yaml_stack_extend(
                            &raw mut (*ctx).start as *mut *mut libc::c_void,
                            &raw mut (*ctx).top as *mut *mut libc::c_void,
                            &raw mut (*ctx).end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::write((*ctx).top, index);
                    (*ctx).top = (*ctx).top.wrapping_offset(1);
                };
                return OK;
            }
        }
        yaml_free(tag as *mut libc::c_void);
        yaml_free((*event).data.mapping_start.anchor as *mut libc::c_void);
        FAIL
    }
    unsafe fn yaml_parser_load_mapping_end(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
        ctx: *mut LoaderCtx,
    ) -> Success {
        if !(((*ctx).top).c_offset_from((*ctx).start) as libc::c_long > 0_i64) {
            crate::externs::__assert_fail(
                "((*ctx).top).c_offset_from((*ctx).start) as libc::c_long > 0_i64",
                "src/loader.rs",
                656u32,
            );
        }
        let index: libc::c_int = *(*ctx).top.wrapping_offset(-1_isize);
        if !((*((*(*parser).document).nodes.start).wrapping_offset((index - 1) as isize))
            .type_ == YamlMappingNode)
        {
            crate::externs::__assert_fail(
                "(*((*(*parser).document).nodes.start).wrapping_offset((index - 1) as\nisize)).type_ == YamlMappingNode",
                "src/loader.rs",
                661u32,
            );
        }
        (*(*(*parser).document).nodes.start.wrapping_offset((index - 1) as isize))
            .end_mark = (*event).end_mark;
        let _ = *{
            (*ctx).top = (*ctx).top.offset(-1);
            (*ctx).top
        };
        OK
    }
}
/// Decode module for LibYML.
///
/// This module contains functions for decoding YAML data.
pub mod decode {
    use crate::{
        libc, memory::{yaml_free, yaml_malloc},
        success::{Success, OK},
        yaml::{size_t, yaml_char_t},
        yaml_token_delete, YamlMarkT, YamlParserStateT, YamlParserT, YamlSimpleKeyT,
        YamlTagDirectiveT, YamlTokenT,
    };
    use crate::externs::memset;
    use core::{mem::size_of, ptr::{self, addr_of_mut}};
    const INPUT_RAW_BUFFER_SIZE: usize = 16384;
    const INPUT_BUFFER_SIZE: usize = INPUT_RAW_BUFFER_SIZE * 3;
    /// Initialize a parser.
    ///
    /// This function creates a new parser object. An application is responsible
    /// for destroying the object using the yaml_parser_delete() function.
    ///
    /// # Safety
    ///
    /// - `parser` must be a valid, non-null pointer to an uninitialized `YamlParserT` struct.
    /// - The `YamlParserT` struct must be properly aligned and have the expected memory layout.
    /// - The caller is responsible for properly destroying the parser object using `yaml_parser_delete`.
    ///
    pub unsafe fn yaml_parser_initialize(parser: *mut YamlParserT) -> Success {
        if !!parser.is_null() {
            crate::externs::__assert_fail("!parser.is_null()", "src/decode.rs", 38u32);
        }
        let _ = memset(
            parser as *mut libc::c_void,
            0,
            size_of::<YamlParserT>() as libc::c_ulong,
        );
        {
            let start = &raw mut (*parser).raw_buffer.start;
            *start = yaml_malloc(INPUT_RAW_BUFFER_SIZE as size_t) as *mut yaml_char_t;
            if !start.is_null() {
                let _ = memset(
                    *start as *mut libc::c_void,
                    0,
                    INPUT_RAW_BUFFER_SIZE as u64,
                );
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for buffer"),
                    );
                };
            }
            let pointer = &raw mut (*parser).raw_buffer.pointer;
            *pointer = *start;
            let last = &raw mut (*parser).raw_buffer.last;
            *last = *pointer;
            let end = &raw mut (*parser).raw_buffer.end;
            *end = (*start).wrapping_add(INPUT_RAW_BUFFER_SIZE);
        };
        {
            let start = &raw mut (*parser).buffer.start;
            *start = yaml_malloc(INPUT_BUFFER_SIZE as size_t) as *mut yaml_char_t;
            if !start.is_null() {
                let _ = memset(*start as *mut libc::c_void, 0, INPUT_BUFFER_SIZE as u64);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for buffer"),
                    );
                };
            }
            let pointer = &raw mut (*parser).buffer.pointer;
            *pointer = *start;
            let last = &raw mut (*parser).buffer.last;
            *last = *pointer;
            let end = &raw mut (*parser).buffer.end;
            *end = (*start).wrapping_add(INPUT_BUFFER_SIZE);
        };
        {
            (*parser).tokens.start = yaml_malloc(
                16 * size_of::<YamlTokenT>() as libc::c_ulong,
            ) as *mut YamlTokenT;
            (*parser).tokens.tail = (*parser).tokens.start;
            (*parser).tokens.head = (*parser).tokens.tail;
            (*parser).tokens.end = (*parser).tokens.start.offset(16_isize);
        };
        {
            (*parser).indents.start = yaml_malloc(
                16 * size_of::<libc::c_int>() as libc::c_ulong,
            ) as *mut libc::c_int;
            (*parser).indents.top = (*parser).indents.start;
            (*parser).indents.end = (*parser).indents.start.offset(16_isize);
        };
        {
            (*parser).simple_keys.start = yaml_malloc(
                16 * size_of::<YamlSimpleKeyT>() as libc::c_ulong,
            ) as *mut YamlSimpleKeyT;
            (*parser).simple_keys.top = (*parser).simple_keys.start;
            (*parser).simple_keys.end = (*parser).simple_keys.start.offset(16_isize);
        };
        {
            (*parser).states.start = yaml_malloc(
                16 * size_of::<YamlParserStateT>() as libc::c_ulong,
            ) as *mut YamlParserStateT;
            (*parser).states.top = (*parser).states.start;
            (*parser).states.end = (*parser).states.start.offset(16_isize);
        };
        {
            (*parser).marks.start = yaml_malloc(
                16 * size_of::<YamlMarkT>() as libc::c_ulong,
            ) as *mut YamlMarkT;
            (*parser).marks.top = (*parser).marks.start;
            (*parser).marks.end = (*parser).marks.start.offset(16_isize);
        };
        {
            (*parser).tag_directives.start = yaml_malloc(
                16 * size_of::<YamlTagDirectiveT>() as libc::c_ulong,
            ) as *mut YamlTagDirectiveT;
            (*parser).tag_directives.top = (*parser).tag_directives.start;
            (*parser).tag_directives.end = (*parser)
                .tag_directives
                .start
                .offset(16_isize);
        };
        OK
    }
    /// Destroy a parser.
    ///
    /// This function frees all memory associated with a parser object, including
    /// any dynamically allocated buffers, tokens, and other data structures.
    ///
    /// # Safety
    ///
    /// - `parser` must be a valid, non-null pointer to a properly initialized `YamlParserT` struct.
    /// - The `YamlParserT` struct and its associated data structures must have been properly initialized and their memory allocated correctly.
    /// - The `YamlParserT` struct and its associated data structures must be properly aligned and have the expected memory layout.
    /// - After calling this function, the `parser` pointer should be considered invalid and should not be used again.
    ///
    /// Destroy a parser.
    ///
    /// This function frees all memory associated with a parser object, including
    /// any dynamically allocated buffers, tokens, and other data structures.
    ///
    /// # Safety
    ///
    /// - `parser` must be a valid, non-null pointer to a properly initialized `YamlParserT` struct.
    /// - After calling this function, `parser` should be considered invalid and should not be used again.
    ///
    pub unsafe fn yaml_parser_delete(parser: *mut YamlParserT) {
        if !!parser.is_null() {
            crate::externs::__assert_fail("!parser.is_null()", "src/decode.rs", 78u32);
        }
        {
            yaml_free((*parser).raw_buffer.start as *mut libc::c_void);
            (*parser).raw_buffer.start = ptr::null_mut::<yaml_char_t>();
            (*parser).raw_buffer.pointer = ptr::null_mut::<yaml_char_t>();
            (*parser).raw_buffer.last = ptr::null_mut::<yaml_char_t>();
            (*parser).raw_buffer.end = ptr::null_mut::<yaml_char_t>();
        };
        {
            yaml_free((*parser).buffer.start as *mut libc::c_void);
            (*parser).buffer.start = ptr::null_mut::<yaml_char_t>();
            (*parser).buffer.pointer = ptr::null_mut::<yaml_char_t>();
            (*parser).buffer.last = ptr::null_mut::<yaml_char_t>();
            (*parser).buffer.end = ptr::null_mut::<yaml_char_t>();
        };
        while !((*parser).tokens.head == (*parser).tokens.tail) {
            yaml_token_delete(
                &raw mut *{
                    let head = (*parser).tokens.head;
                    (*parser).tokens.head = (*parser).tokens.head.wrapping_offset(1);
                    head
                },
            );
        }
        yaml_free((*parser).tokens.start as *mut libc::c_void);
        (*parser).tokens.end = ptr::null_mut();
        (*parser).tokens.tail = ptr::null_mut();
        (*parser).tokens.head = ptr::null_mut();
        (*parser).tokens.start = ptr::null_mut();
        yaml_free((*parser).indents.start as *mut libc::c_void);
        (*parser).indents.end = ptr::null_mut();
        (*parser).indents.top = ptr::null_mut();
        (*parser).indents.start = ptr::null_mut();
        yaml_free((*parser).simple_keys.start as *mut libc::c_void);
        (*parser).simple_keys.end = ptr::null_mut();
        (*parser).simple_keys.top = ptr::null_mut();
        (*parser).simple_keys.start = ptr::null_mut();
        yaml_free((*parser).states.start as *mut libc::c_void);
        (*parser).states.end = ptr::null_mut();
        (*parser).states.top = ptr::null_mut();
        (*parser).states.start = ptr::null_mut();
        yaml_free((*parser).marks.start as *mut libc::c_void);
        (*parser).marks.end = ptr::null_mut();
        (*parser).marks.top = ptr::null_mut();
        (*parser).marks.start = ptr::null_mut();
        while !((*parser).tag_directives.start == (*parser).tag_directives.top) {
            let tag_directive = *{
                (*parser).tag_directives.top = (*parser).tag_directives.top.offset(-1);
                (*parser).tag_directives.top
            };
            yaml_free(tag_directive.handle as *mut libc::c_void);
            yaml_free(tag_directive.prefix as *mut libc::c_void);
        }
        yaml_free((*parser).tag_directives.start as *mut libc::c_void);
        (*parser).tag_directives.end = ptr::null_mut();
        (*parser).tag_directives.top = ptr::null_mut();
        (*parser).tag_directives.start = ptr::null_mut();
        let _ = memset(
            parser as *mut libc::c_void,
            0,
            size_of::<YamlParserT>() as libc::c_ulong,
        );
    }
}
/// Document module for LibYML.
///
/// This module provides functions for working with YAML documents.
pub mod document {
    use crate::externs::{memcpy, memset, strlen};
    use crate::internal::yaml_check_utf8;
    use crate::internal::yaml_stack_extend;
    use crate::memory::{yaml_free, yaml_malloc, yaml_strdup};
    use crate::ops::ForceAdd;
    use crate::success::{Success, FAIL, OK};
    use crate::yaml::{size_t, yaml_char_t};
    use crate::YamlEventT;
    use crate::YamlEventTypeT::YamlDocumentEndEvent;
    use crate::YamlEventTypeT::YamlDocumentStartEvent;
    use crate::{
        libc, PointerExt, YamlDocumentT, YamlMappingNode, YamlMappingStyleT, YamlMarkT,
        YamlNodeItemT, YamlNodePairT, YamlNodeT, YamlScalarNode, YamlScalarStyleT,
        YamlSequenceNode, YamlSequenceStyleT, YamlTagDirectiveT, YamlVersionDirectiveT,
    };
    use core::mem::{size_of, MaybeUninit};
    use core::ptr::{self, addr_of_mut};
    /// Create a YAML document.
    ///
    /// This function initializes a `YamlDocumentT` struct with the provided version directive,
    /// tag directives, and implicit flags. It allocates memory for the document data and
    /// copies the provided directives.
    ///
    /// # Safety
    ///
    /// - `document` must be a valid, non-null pointer to a `YamlDocumentT` struct that can be safely written to.
    /// - `version_directive`, if not null, must point to a valid `YamlVersionDirectiveT` struct.
    /// - `tag_directives_start` and `tag_directives_end` must be valid pointers to `YamlTagDirectiveT` structs, or both must be null.
    /// - If `tag_directives_start` and `tag_directives_end` are not null, the range they define must contain valid `YamlTagDirectiveT` structs with non-null `handle` and `prefix` members, and the `handle` and `prefix` strings must be valid UTF-8.
    /// - The `YamlDocumentT`, `YamlVersionDirectiveT`, and `YamlTagDirectiveT` structs must be properly aligned and have the expected memory layout.
    /// - The caller is responsible for freeing the memory allocated for the document using `yaml_document_delete`.
    ///
    pub unsafe fn yaml_document_initialize(
        document: *mut YamlDocumentT,
        version_directive: *mut YamlVersionDirectiveT,
        tag_directives_start: *mut YamlTagDirectiveT,
        tag_directives_end: *mut YamlTagDirectiveT,
        start_implicit: bool,
        end_implicit: bool,
    ) -> Success {
        let current_block: u64;
        struct Nodes {
            start: *mut YamlNodeT,
            end: *mut YamlNodeT,
            top: *mut YamlNodeT,
        }
        let mut nodes = Nodes {
            start: ptr::null_mut::<YamlNodeT>(),
            end: ptr::null_mut::<YamlNodeT>(),
            top: ptr::null_mut::<YamlNodeT>(),
        };
        let mut version_directive_copy: *mut YamlVersionDirectiveT = ptr::null_mut::<
            YamlVersionDirectiveT,
        >();
        struct TagDirectivesCopy {
            start: *mut YamlTagDirectiveT,
            end: *mut YamlTagDirectiveT,
            top: *mut YamlTagDirectiveT,
        }
        let mut tag_directives_copy = TagDirectivesCopy {
            start: ptr::null_mut::<YamlTagDirectiveT>(),
            end: ptr::null_mut::<YamlTagDirectiveT>(),
            top: ptr::null_mut::<YamlTagDirectiveT>(),
        };
        let mut value = YamlTagDirectiveT {
            handle: ptr::null_mut::<yaml_char_t>(),
            prefix: ptr::null_mut::<yaml_char_t>(),
        };
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        if !!document.is_null() {
            crate::externs::__assert_fail(
                "!document.is_null()",
                "src/document.rs",
                74u32,
            );
        }
        if !(!tag_directives_start.is_null() && !tag_directives_end.is_null()
            || tag_directives_start == tag_directives_end)
        {
            crate::externs::__assert_fail(
                "!tag_directives_start.is_null() && !tag_directives_end.is_null() ||\ntag_directives_start == tag_directives_end",
                "src/document.rs",
                75u32,
            );
        }
        {
            nodes.start = yaml_malloc(16 * size_of::<YamlNodeT>() as libc::c_ulong)
                as *mut YamlNodeT;
            nodes.top = nodes.start;
            nodes.end = nodes.start.offset(16_isize);
        };
        if !version_directive.is_null() {
            version_directive_copy = yaml_malloc(
                size_of::<YamlVersionDirectiveT>() as libc::c_ulong,
            ) as *mut YamlVersionDirectiveT;
            (*version_directive_copy).major = (*version_directive).major;
            (*version_directive_copy).minor = (*version_directive).minor;
        }
        if tag_directives_start != tag_directives_end {
            let mut tag_directive: *mut YamlTagDirectiveT;
            {
                tag_directives_copy.start = yaml_malloc(
                    16 * size_of::<YamlTagDirectiveT>() as libc::c_ulong,
                ) as *mut YamlTagDirectiveT;
                tag_directives_copy.top = tag_directives_copy.start;
                tag_directives_copy.end = tag_directives_copy.start.offset(16_isize);
            };
            tag_directive = tag_directives_start;
            loop {
                if tag_directive == tag_directives_end {
                    current_block = 14818589718467733107;
                    break;
                }
                if !!((*tag_directive).handle).is_null() {
                    crate::externs::__assert_fail(
                        "!((*tag_directive).handle).is_null()",
                        "src/document.rs",
                        98u32,
                    );
                }
                if !!((*tag_directive).prefix).is_null() {
                    crate::externs::__assert_fail(
                        "!((*tag_directive).prefix).is_null()",
                        "src/document.rs",
                        99u32,
                    );
                }
                if yaml_check_utf8(
                        (*tag_directive).handle,
                        strlen((*tag_directive).handle as *mut libc::c_char),
                    )
                    .fail
                {
                    current_block = 8142820162064489797;
                    break;
                }
                if yaml_check_utf8(
                        (*tag_directive).prefix,
                        strlen((*tag_directive).prefix as *mut libc::c_char),
                    )
                    .fail
                {
                    current_block = 8142820162064489797;
                    break;
                }
                value.handle = yaml_strdup((*tag_directive).handle);
                value.prefix = yaml_strdup((*tag_directive).prefix);
                if value.handle.is_null() || value.prefix.is_null() {
                    current_block = 8142820162064489797;
                    break;
                }
                {
                    if tag_directives_copy.top == tag_directives_copy.end {
                        yaml_stack_extend(
                            &raw mut tag_directives_copy.start as *mut *mut libc::c_void,
                            &raw mut tag_directives_copy.top as *mut *mut libc::c_void,
                            &raw mut tag_directives_copy.end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::write(tag_directives_copy.top, value);
                    tag_directives_copy.top = tag_directives_copy.top.wrapping_offset(1);
                };
                value.handle = ptr::null_mut::<yaml_char_t>();
                value.prefix = ptr::null_mut::<yaml_char_t>();
                tag_directive = tag_directive.wrapping_offset(1);
            }
        } else {
            current_block = 14818589718467733107;
        }
        if current_block != 8142820162064489797 {
            let _ = memset(
                document as *mut libc::c_void,
                0,
                size_of::<YamlDocumentT>() as libc::c_ulong,
            );
            let fresh176 = &raw mut (*document).nodes.start;
            *fresh176 = nodes.start;
            let fresh177 = &raw mut (*document).nodes.end;
            *fresh177 = nodes.end;
            let fresh178 = &raw mut (*document).nodes.top;
            *fresh178 = nodes.start;
            let fresh179 = &raw mut (*document).version_directive;
            *fresh179 = version_directive_copy;
            let fresh180 = &raw mut (*document).tag_directives.start;
            *fresh180 = tag_directives_copy.start;
            let fresh181 = &raw mut (*document).tag_directives.end;
            *fresh181 = tag_directives_copy.top;
            (*document).start_implicit = start_implicit;
            (*document).end_implicit = end_implicit;
            (*document).start_mark = mark;
            (*document).end_mark = mark;
            return OK;
        }
        yaml_free(nodes.start as *mut libc::c_void);
        nodes.end = ptr::null_mut();
        nodes.top = ptr::null_mut();
        nodes.start = ptr::null_mut();
        yaml_free(version_directive_copy as *mut libc::c_void);
        while !(tag_directives_copy.start == tag_directives_copy.top) {
            let value = *{
                tag_directives_copy.top = tag_directives_copy.top.offset(-1);
                tag_directives_copy.top
            };
            yaml_free(value.handle as *mut libc::c_void);
            yaml_free(value.prefix as *mut libc::c_void);
        }
        yaml_free(tag_directives_copy.start as *mut libc::c_void);
        tag_directives_copy.end = ptr::null_mut();
        tag_directives_copy.top = ptr::null_mut();
        tag_directives_copy.start = ptr::null_mut();
        yaml_free(value.handle as *mut libc::c_void);
        yaml_free(value.prefix as *mut libc::c_void);
        FAIL
    }
    /// Delete a YAML document and all its nodes.
    ///
    /// This function frees the memory allocated for a `YamlDocumentT` struct and all its associated
    /// nodes, including scalar values, sequences, and mappings.
    ///
    /// # Safety
    ///
    /// - `document` must be a valid, non-null pointer to a `YamlDocumentT` struct.
    /// - The `YamlDocumentT` struct and its associated nodes must have been properly initialized and their memory allocated correctly.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_document_delete(document: *mut YamlDocumentT) {
        if document.is_null() {
            return;
        }
        let mut tag_directive: *mut YamlTagDirectiveT;
        while !((*document).nodes.start == (*document).nodes.top) {
            let mut node = *{
                (*document).nodes.top = (*document).nodes.top.offset(-1);
                (*document).nodes.top
            };
            yaml_free(node.tag as *mut libc::c_void);
            match node.type_ {
                YamlScalarNode => {
                    yaml_free(node.data.scalar.value as *mut libc::c_void);
                }
                YamlSequenceNode => {
                    yaml_free(node.data.sequence.items.start as *mut libc::c_void);
                    node.data.sequence.items.end = ptr::null_mut();
                    node.data.sequence.items.top = ptr::null_mut();
                    node.data.sequence.items.start = ptr::null_mut();
                }
                YamlMappingNode => {
                    yaml_free(node.data.mapping.pairs.start as *mut libc::c_void);
                    node.data.mapping.pairs.end = ptr::null_mut();
                    node.data.mapping.pairs.top = ptr::null_mut();
                    node.data.mapping.pairs.start = ptr::null_mut();
                }
                _ => {
                    crate::externs::__assert_fail("false", "src/document.rs", 203u32);
                }
            }
        }
        yaml_free((*document).nodes.start as *mut libc::c_void);
        (*document).nodes.end = ptr::null_mut();
        (*document).nodes.top = ptr::null_mut();
        (*document).nodes.start = ptr::null_mut();
        yaml_free((*document).version_directive as *mut libc::c_void);
        tag_directive = (*document).tag_directives.start;
        while tag_directive != (*document).tag_directives.end {
            yaml_free((*tag_directive).handle as *mut libc::c_void);
            yaml_free((*tag_directive).prefix as *mut libc::c_void);
            tag_directive = tag_directive.wrapping_offset(1);
        }
        yaml_free((*document).tag_directives.start as *mut libc::c_void);
        let _ = memset(
            document as *mut libc::c_void,
            0,
            size_of::<YamlDocumentT>() as libc::c_ulong,
        );
    }
    /// Get a node of a YAML document.
    ///
    /// This function returns a pointer to the node at the specified `index` in the document's node
    /// stack. The pointer returned by this function is valid until any of the functions modifying the
    /// document are called.
    ///
    /// Returns the node object or NULL if `index` is out of range.
    ///
    /// # Safety
    ///
    /// - `document` must be a valid, non-null pointer to a `YamlDocumentT` struct.
    /// - `index` must be a valid index within the range of nodes in the `YamlDocumentT` struct.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly initialized and their memory allocated correctly.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly aligned and have the expected memory layout.
    /// - The caller must not modify or free the returned pointer, as it is owned by the `YamlDocumentT` struct.
    ///
    pub unsafe fn yaml_document_get_node(
        document: *mut YamlDocumentT,
        index: libc::c_int,
    ) -> *mut YamlNodeT {
        if !!document.is_null() {
            crate::externs::__assert_fail(
                "!document.is_null()",
                "src/document.rs",
                247u32,
            );
        }
        if index > 0
            && (*document).nodes.start.wrapping_offset(index as isize)
                <= (*document).nodes.top
        {
            return (*document)
                .nodes
                .start
                .wrapping_offset(index as isize)
                .wrapping_offset(-1_isize);
        }
        ptr::null_mut::<YamlNodeT>()
    }
    /// Get the root of a YAML document node.
    ///
    /// This function returns a pointer to the root node of the YAML document. The root object is the
    /// first object added to the document.
    ///
    /// The pointer returned by this function is valid until any of the functions modifying the
    /// document are called.
    ///
    /// An empty document produced by the parser signifies the end of a YAML stream.
    ///
    /// Returns the node object or NULL if the document is empty.
    ///
    /// # Safety
    ///
    /// - `document` must be a valid, non-null pointer to a `YamlDocumentT` struct.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly initialized and their memory allocated correctly.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly aligned and have the expected memory layout.
    /// - The caller must not modify or free the returned pointer, as it is owned by the `YamlDocumentT` struct.
    ///
    pub unsafe fn yaml_document_get_root_node(
        document: *mut YamlDocumentT,
    ) -> *mut YamlNodeT {
        if !!document.is_null() {
            crate::externs::__assert_fail(
                "!document.is_null()",
                "src/document.rs",
                283u32,
            );
        }
        if (*document).nodes.top != (*document).nodes.start {
            return (*document).nodes.start;
        }
        ptr::null_mut::<YamlNodeT>()
    }
    /// Create a SCALAR node and attach it to the document.
    ///
    /// This function creates a new SCALAR node with the provided `tag`, `value`, and `style`, and
    /// adds it to the document's node stack.
    ///
    /// The `style` argument may be ignored by the emitter.
    ///
    /// Returns the node id or 0 on error.
    ///
    /// # Safety
    ///
    /// - `document` must be a valid, non-null pointer to a `YamlDocumentT` struct.
    /// - `value` must be a valid, non-null pointer to a null-terminated UTF-8 string.
    /// - `tag`, if not null, must be a valid pointer to a null-terminated UTF-8 string.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly initialized and their memory allocated correctly.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly aligned and have the expected memory layout.
    /// - The caller is responsible for freeing the memory allocated for the document using `yaml_document_delete`.
    ///
    #[must_use]
    pub unsafe fn yaml_document_add_scalar(
        document: *mut YamlDocumentT,
        mut tag: *const yaml_char_t,
        value: *const yaml_char_t,
        mut length: libc::c_int,
        style: YamlScalarStyleT,
    ) -> libc::c_int {
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        let mut tag_copy: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let mut value_copy: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let mut node = MaybeUninit::<YamlNodeT>::uninit();
        let node = node.as_mut_ptr();
        if !!document.is_null() {
            crate::externs::__assert_fail(
                "!document.is_null()",
                "src/document.rs",
                326u32,
            );
        }
        if !!value.is_null() {
            crate::externs::__assert_fail("!value.is_null()", "src/document.rs", 327u32);
        }
        if tag.is_null() {
            tag = b"tag:yaml.org,2002:str\0" as *const u8 as *const libc::c_char
                as *mut yaml_char_t;
        }
        if yaml_check_utf8(tag, strlen(tag as *mut libc::c_char)).ok {
            tag_copy = yaml_strdup(tag);
            if !tag_copy.is_null() {
                if length < 0 {
                    length = strlen(value as *mut libc::c_char) as libc::c_int;
                }
                if yaml_check_utf8(value, length as size_t).ok {
                    value_copy = yaml_malloc(length.force_add(1) as size_t)
                        as *mut yaml_char_t;
                    let _ = memcpy(
                        value_copy as *mut libc::c_void,
                        value as *const libc::c_void,
                        length as libc::c_ulong,
                    );
                    *value_copy.wrapping_offset(length as isize) = b'\0';
                    let _ = memset(
                        node as *mut libc::c_void,
                        0,
                        size_of::<YamlNodeT>() as libc::c_ulong,
                    );
                    (*node).type_ = YamlScalarNode;
                    (*node).tag = tag_copy;
                    (*node).start_mark = mark;
                    (*node).end_mark = mark;
                    (*node).data.scalar.value = value_copy;
                    (*node).data.scalar.length = length as size_t;
                    (*node).data.scalar.style = style;
                    {
                        if (*document).nodes.top == (*document).nodes.end {
                            yaml_stack_extend(
                                &raw mut (*document).nodes.start as *mut *mut libc::c_void,
                                &raw mut (*document).nodes.top as *mut *mut libc::c_void,
                                &raw mut (*document).nodes.end as *mut *mut libc::c_void,
                            );
                        }
                        ptr::copy_nonoverlapping(node, (*document).nodes.top, 1);
                        (*document).nodes.top = (*document).nodes.top.wrapping_offset(1);
                    };
                    return (*document).nodes.top.c_offset_from((*document).nodes.start)
                        as libc::c_int;
                }
            }
        }
        yaml_free(tag_copy as *mut libc::c_void);
        yaml_free(value_copy as *mut libc::c_void);
        0
    }
    /// Create a SEQUENCE node and attach it to the document.
    ///
    /// This function creates a new SEQUENCE node with the provided `tag` and `style`, and adds it to
    /// the document's node stack.
    ///
    /// The `style` argument may be ignored by the emitter.
    ///
    /// Returns the node id or 0 on error.
    ///
    /// # Safety
    ///
    /// - `document` must be a valid, non-null pointer to a `YamlDocumentT` struct.
    /// - `tag`, if not null, must be a valid pointer to a null-terminated UTF-8 string.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly initialized and their memory allocated correctly.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly aligned and have the expected memory layout.
    /// - The caller is responsible for freeing the memory allocated for the document using `yaml_document_delete`.
    ///
    #[must_use]
    pub unsafe fn yaml_document_add_sequence(
        document: *mut YamlDocumentT,
        mut tag: *const yaml_char_t,
        style: YamlSequenceStyleT,
    ) -> libc::c_int {
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        let mut tag_copy: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        struct Items {
            start: *mut YamlNodeItemT,
            end: *mut YamlNodeItemT,
            top: *mut YamlNodeItemT,
        }
        let mut items = Items {
            start: ptr::null_mut::<YamlNodeItemT>(),
            end: ptr::null_mut::<YamlNodeItemT>(),
            top: ptr::null_mut::<YamlNodeItemT>(),
        };
        let mut node = MaybeUninit::<YamlNodeT>::uninit();
        let node = node.as_mut_ptr();
        if !!document.is_null() {
            crate::externs::__assert_fail(
                "!document.is_null()",
                "src/document.rs",
                415u32,
            );
        }
        if tag.is_null() {
            tag = b"tag:yaml.org,2002:seq\0" as *const u8 as *const libc::c_char
                as *mut yaml_char_t;
        }
        if yaml_check_utf8(tag, strlen(tag as *mut libc::c_char)).ok {
            tag_copy = yaml_strdup(tag);
            if !tag_copy.is_null() {
                {
                    items.start = yaml_malloc(
                        16 * size_of::<YamlNodeItemT>() as libc::c_ulong,
                    ) as *mut YamlNodeItemT;
                    items.top = items.start;
                    items.end = items.start.offset(16_isize);
                };
                let _ = memset(
                    node as *mut libc::c_void,
                    0,
                    size_of::<YamlNodeT>() as libc::c_ulong,
                );
                (*node).type_ = YamlSequenceNode;
                (*node).tag = tag_copy;
                (*node).start_mark = mark;
                (*node).end_mark = mark;
                (*node).data.sequence.items.start = items.start;
                (*node).data.sequence.items.end = items.end;
                (*node).data.sequence.items.top = items.start;
                (*node).data.sequence.style = style;
                {
                    if (*document).nodes.top == (*document).nodes.end {
                        yaml_stack_extend(
                            &raw mut (*document).nodes.start as *mut *mut libc::c_void,
                            &raw mut (*document).nodes.top as *mut *mut libc::c_void,
                            &raw mut (*document).nodes.end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::copy_nonoverlapping(node, (*document).nodes.top, 1);
                    (*document).nodes.top = (*document).nodes.top.wrapping_offset(1);
                };
                return (*document).nodes.top.c_offset_from((*document).nodes.start)
                    as libc::c_int;
            }
        }
        yaml_free(items.start as *mut libc::c_void);
        items.end = ptr::null_mut();
        items.top = ptr::null_mut();
        items.start = ptr::null_mut();
        yaml_free(tag_copy as *mut libc::c_void);
        0
    }
    /// Create a MAPPING node and attach it to the document.
    ///
    /// This function creates a new MAPPING node with the provided `tag` and `style`, and adds it to
    /// the document's node stack.
    ///
    /// The `style` argument may be ignored by the emitter.
    ///
    /// Returns the node id or 0 on error.
    ///
    /// # Safety
    ///
    /// - `document` must be a valid, non-null pointer to a `YamlDocumentT` struct.
    /// - `tag`, if not null, must be a valid pointer to a null-terminated UTF-8 string.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly initialized and their memory allocated correctly.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly aligned and have the expected memory layout.
    /// - The caller is responsible for freeing the memory allocated for the document using `yaml_document_delete`.
    ///
    #[must_use]
    pub unsafe fn yaml_document_add_mapping(
        document: *mut YamlDocumentT,
        mut tag: *const yaml_char_t,
        style: YamlMappingStyleT,
    ) -> libc::c_int {
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        let mut tag_copy: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        struct Pairs {
            start: *mut YamlNodePairT,
            end: *mut YamlNodePairT,
            top: *mut YamlNodePairT,
        }
        let mut pairs = Pairs {
            start: ptr::null_mut::<YamlNodePairT>(),
            end: ptr::null_mut::<YamlNodePairT>(),
            top: ptr::null_mut::<YamlNodePairT>(),
        };
        let mut node = MaybeUninit::<YamlNodeT>::uninit();
        let node = node.as_mut_ptr();
        if !!document.is_null() {
            crate::externs::__assert_fail(
                "!document.is_null()",
                "src/document.rs",
                491u32,
            );
        }
        if tag.is_null() {
            tag = b"tag:yaml.org,2002:map\0" as *const u8 as *const libc::c_char
                as *mut yaml_char_t;
        }
        if yaml_check_utf8(tag, strlen(tag as *mut libc::c_char)).ok {
            tag_copy = yaml_strdup(tag);
            if !tag_copy.is_null() {
                {
                    pairs.start = yaml_malloc(
                        16 * size_of::<YamlNodePairT>() as libc::c_ulong,
                    ) as *mut YamlNodePairT;
                    pairs.top = pairs.start;
                    pairs.end = pairs.start.offset(16_isize);
                };
                let _ = memset(
                    node as *mut libc::c_void,
                    0,
                    size_of::<YamlNodeT>() as libc::c_ulong,
                );
                (*node).type_ = YamlMappingNode;
                (*node).tag = tag_copy;
                (*node).start_mark = mark;
                (*node).end_mark = mark;
                (*node).data.mapping.pairs.start = pairs.start;
                (*node).data.mapping.pairs.end = pairs.end;
                (*node).data.mapping.pairs.top = pairs.start;
                (*node).data.mapping.style = style;
                {
                    if (*document).nodes.top == (*document).nodes.end {
                        yaml_stack_extend(
                            &raw mut (*document).nodes.start as *mut *mut libc::c_void,
                            &raw mut (*document).nodes.top as *mut *mut libc::c_void,
                            &raw mut (*document).nodes.end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::copy_nonoverlapping(node, (*document).nodes.top, 1);
                    (*document).nodes.top = (*document).nodes.top.wrapping_offset(1);
                };
                return (*document).nodes.top.c_offset_from((*document).nodes.start)
                    as libc::c_int;
            }
        }
        yaml_free(pairs.start as *mut libc::c_void);
        pairs.end = ptr::null_mut();
        pairs.top = ptr::null_mut();
        pairs.start = ptr::null_mut();
        yaml_free(tag_copy as *mut libc::c_void);
        0
    }
    /// Add an item to a SEQUENCE node.
    ///
    /// This function adds a node with the given `item` id to the sequence node with the given
    /// `sequence` id in the document.
    ///
    /// # Safety
    ///
    /// - `document` must be a valid, non-null pointer to a `YamlDocumentT` struct.
    /// - `sequence` must be a valid index within the range of nodes in the `YamlDocumentT` struct, and the node at that index must be a `YamlSequenceNode`.
    /// - `item` must be a valid index within the range of nodes in the `YamlDocumentT` struct.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly initialized and their memory allocated correctly.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_document_append_sequence_item(
        document: *mut YamlDocumentT,
        sequence: libc::c_int,
        item: libc::c_int,
    ) -> Success {
        if !!document.is_null() {
            crate::externs::__assert_fail(
                "!document.is_null()",
                "src/document.rs",
                544u32,
            );
        }
        if !(sequence > 0
            && ((*document).nodes.start).wrapping_offset(sequence as isize)
                <= (*document).nodes.top)
        {
            crate::externs::__assert_fail(
                "sequence > 0 && ((*document).nodes.start).wrapping_offset(sequence as isize)\n<= (*document).nodes.top",
                "src/document.rs",
                545u32,
            );
        }
        if !((*((*document).nodes.start).wrapping_offset((sequence - 1) as isize)).type_
            == YamlSequenceNode)
        {
            crate::externs::__assert_fail(
                "(*((*document).nodes.start).wrapping_offset((sequence - 1) as isize)).type_ ==\nYamlSequenceNode",
                "src/document.rs",
                551u32,
            );
        }
        if !(item > 0
            && ((*document).nodes.start).wrapping_offset(item as isize)
                <= (*document).nodes.top)
        {
            crate::externs::__assert_fail(
                "item > 0 && ((*document).nodes.start).wrapping_offset(item as isize) <=\n(*document).nodes.top",
                "src/document.rs",
                557u32,
            );
        }
        {
            if (*((*document).nodes.start).wrapping_offset((sequence - 1) as isize))
                .data
                .sequence
                .items
                .top
                == (*((*document).nodes.start).wrapping_offset((sequence - 1) as isize))
                    .data
                    .sequence
                    .items
                    .end
            {
                yaml_stack_extend(
                    &raw mut (*((*document).nodes.start)
                        .wrapping_offset((sequence - 1) as isize))
                        .data
                        .sequence
                        .items
                        .start as *mut *mut libc::c_void,
                    &raw mut (*((*document).nodes.start)
                        .wrapping_offset((sequence - 1) as isize))
                        .data
                        .sequence
                        .items
                        .top as *mut *mut libc::c_void,
                    &raw mut (*((*document).nodes.start)
                        .wrapping_offset((sequence - 1) as isize))
                        .data
                        .sequence
                        .items
                        .end as *mut *mut libc::c_void,
                );
            }
            ptr::write(
                (*((*document).nodes.start).wrapping_offset((sequence - 1) as isize))
                    .data
                    .sequence
                    .items
                    .top,
                item,
            );
            (*((*document).nodes.start).wrapping_offset((sequence - 1) as isize))
                .data
                .sequence
                .items
                .top = (*((*document).nodes.start)
                .wrapping_offset((sequence - 1) as isize))
                .data
                .sequence
                .items
                .top
                .wrapping_offset(1);
        };
        OK
    }
    /// Add a pair of a key and a value to a MAPPING node.
    ///
    /// This function adds a key-value pair to the mapping node with the given `mapping` id in the
    /// document. The `key` and `value` arguments are the ids of the nodes to be used as the key and
    /// value, respectively.
    ///
    /// # Safety
    ///
    /// - `document` must be a valid, non-null pointer to a `YamlDocumentT` struct.
    /// - `mapping` must be a valid index within the range of nodes in the `YamlDocumentT` struct, and the node at that index must be a `YamlMappingNode`.
    /// - `key` and `value` must be valid indices within the range of nodes in the `YamlDocumentT` struct.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly initialized and their memory allocated correctly.
    /// - The `YamlDocumentT` struct and its associated nodes must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_document_append_mapping_pair(
        document: *mut YamlDocumentT,
        mapping: libc::c_int,
        key: libc::c_int,
        value: libc::c_int,
    ) -> Success {
        if !!document.is_null() {
            crate::externs::__assert_fail(
                "!document.is_null()",
                "src/document.rs",
                593u32,
            );
        }
        if !(mapping > 0
            && ((*document).nodes.start).wrapping_offset(mapping as isize)
                <= (*document).nodes.top)
        {
            crate::externs::__assert_fail(
                "mapping > 0 && ((*document).nodes.start).wrapping_offset(mapping as isize) <=\n(*document).nodes.top",
                "src/document.rs",
                594u32,
            );
        }
        if !((*((*document).nodes.start).wrapping_offset((mapping - 1) as isize)).type_
            == YamlMappingNode)
        {
            crate::externs::__assert_fail(
                "(*((*document).nodes.start).wrapping_offset((mapping - 1) as isize)).type_ ==\nYamlMappingNode",
                "src/document.rs",
                600u32,
            );
        }
        if !(key > 0
            && ((*document).nodes.start).wrapping_offset(key as isize)
                <= (*document).nodes.top)
        {
            crate::externs::__assert_fail(
                "key > 0 && ((*document).nodes.start).wrapping_offset(key as isize) <=\n(*document).nodes.top",
                "src/document.rs",
                606u32,
            );
        }
        if !(value > 0
            && ((*document).nodes.start).wrapping_offset(value as isize)
                <= (*document).nodes.top)
        {
            crate::externs::__assert_fail(
                "value > 0 && ((*document).nodes.start).wrapping_offset(value as isize) <=\n(*document).nodes.top",
                "src/document.rs",
                611u32,
            );
        }
        let pair = YamlNodePairT { key, value };
        {
            if (*((*document).nodes.start).wrapping_offset((mapping - 1) as isize))
                .data
                .mapping
                .pairs
                .top
                == (*((*document).nodes.start).wrapping_offset((mapping - 1) as isize))
                    .data
                    .mapping
                    .pairs
                    .end
            {
                yaml_stack_extend(
                    &raw mut (*((*document).nodes.start)
                        .wrapping_offset((mapping - 1) as isize))
                        .data
                        .mapping
                        .pairs
                        .start as *mut *mut libc::c_void,
                    &raw mut (*((*document).nodes.start)
                        .wrapping_offset((mapping - 1) as isize))
                        .data
                        .mapping
                        .pairs
                        .top as *mut *mut libc::c_void,
                    &raw mut (*((*document).nodes.start)
                        .wrapping_offset((mapping - 1) as isize))
                        .data
                        .mapping
                        .pairs
                        .end as *mut *mut libc::c_void,
                );
            }
            ptr::write(
                (*((*document).nodes.start).wrapping_offset((mapping - 1) as isize))
                    .data
                    .mapping
                    .pairs
                    .top,
                pair,
            );
            (*((*document).nodes.start).wrapping_offset((mapping - 1) as isize))
                .data
                .mapping
                .pairs
                .top = (*((*document).nodes.start)
                .wrapping_offset((mapping - 1) as isize))
                .data
                .mapping
                .pairs
                .top
                .wrapping_offset(1);
        };
        OK
    }
    /// Create the DOCUMENT-END event.
    ///
    /// The `implicit` argument is considered as a stylistic parameter and may be
    /// ignored by the emitter.
    ///
    /// # Safety
    ///
    /// - `event` must be a valid, non-null pointer to a `YamlEventT` struct that can be safely written to.
    /// - The `YamlEventT` struct must be properly aligned and have the expected memory layout.
    ///
    pub unsafe fn yaml_document_end_event_initialize(
        event: *mut YamlEventT,
        implicit: bool,
    ) -> Success {
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        if !!event.is_null() {
            crate::externs::__assert_fail("!event.is_null()", "src/document.rs", 648u32);
        }
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlDocumentEndEvent;
        (*event).start_mark = mark;
        (*event).end_mark = mark;
        (*event).data.document_end.implicit = implicit;
        OK
    }
    /// Create the DOCUMENT-START event.
    ///
    /// The `implicit` argument is considered as a stylistic parameter and may be
    /// ignored by the emitter.
    ///
    /// # Safety
    ///
    /// - `event` must be a valid, non-null pointer to a `YamlEventT` struct that can be safely written to.
    /// - `version_directive`, if not null, must point to a valid `YamlVersionDirectiveT` struct.
    /// - `tag_directives_start` and `tag_directives_end` must be valid pointers to `YamlTagDirectiveT` structs, or both must be null.
    /// - If `tag_directives_start` and `tag_directives_end` are not null, the range they define must contain valid `YamlTagDirectiveT` structs with non-null `handle` and `prefix` members, and the `handle` and `prefix` strings must be valid UTF-8.
    /// - The `YamlEventT`, `YamlVersionDirectiveT`, and `YamlTagDirectiveT` structs must be properly aligned and have the expected memory layout.
    /// - The caller is responsible for freeing any dynamically allocated memory associated with the event using `yaml_event_delete`.
    ///
    pub unsafe fn yaml_document_start_event_initialize(
        event: *mut YamlEventT,
        version_directive: *mut YamlVersionDirectiveT,
        tag_directives_start: *mut YamlTagDirectiveT,
        tag_directives_end: *mut YamlTagDirectiveT,
        implicit: bool,
    ) -> Success {
        let current_block: u64;
        let mark = YamlMarkT {
            index: 0_u64,
            line: 0_u64,
            column: 0_u64,
        };
        let mut version_directive_copy: *mut YamlVersionDirectiveT = ptr::null_mut::<
            YamlVersionDirectiveT,
        >();
        struct TagDirectivesCopy {
            start: *mut YamlTagDirectiveT,
            end: *mut YamlTagDirectiveT,
            top: *mut YamlTagDirectiveT,
        }
        let mut tag_directives_copy = TagDirectivesCopy {
            start: ptr::null_mut::<YamlTagDirectiveT>(),
            end: ptr::null_mut::<YamlTagDirectiveT>(),
            top: ptr::null_mut::<YamlTagDirectiveT>(),
        };
        let mut value = YamlTagDirectiveT {
            handle: ptr::null_mut::<yaml_char_t>(),
            prefix: ptr::null_mut::<yaml_char_t>(),
        };
        if !!event.is_null() {
            crate::externs::__assert_fail("!event.is_null()", "src/document.rs", 704u32);
        }
        if !(!tag_directives_start.is_null() && !tag_directives_end.is_null()
            || tag_directives_start == tag_directives_end)
        {
            crate::externs::__assert_fail(
                "!tag_directives_start.is_null() && !tag_directives_end.is_null() ||\ntag_directives_start == tag_directives_end",
                "src/document.rs",
                705u32,
            );
        }
        if !version_directive.is_null() {
            version_directive_copy = yaml_malloc(
                size_of::<YamlVersionDirectiveT>() as libc::c_ulong,
            ) as *mut YamlVersionDirectiveT;
            (*version_directive_copy).major = (*version_directive).major;
            (*version_directive_copy).minor = (*version_directive).minor;
        }
        if tag_directives_start != tag_directives_end {
            let mut tag_directive: *mut YamlTagDirectiveT;
            {
                tag_directives_copy.start = yaml_malloc(
                    16 * size_of::<YamlTagDirectiveT>() as libc::c_ulong,
                ) as *mut YamlTagDirectiveT;
                tag_directives_copy.top = tag_directives_copy.start;
                tag_directives_copy.end = tag_directives_copy.start.offset(16_isize);
            };
            tag_directive = tag_directives_start;
            loop {
                if tag_directive == tag_directives_end {
                    current_block = 16203760046146113240;
                    break;
                }
                if !!((*tag_directive).handle).is_null() {
                    crate::externs::__assert_fail(
                        "!((*tag_directive).handle).is_null()",
                        "src/document.rs",
                        727u32,
                    );
                }
                if !!((*tag_directive).prefix).is_null() {
                    crate::externs::__assert_fail(
                        "!((*tag_directive).prefix).is_null()",
                        "src/document.rs",
                        728u32,
                    );
                }
                if yaml_check_utf8(
                        (*tag_directive).handle,
                        strlen((*tag_directive).handle as *mut libc::c_char),
                    )
                    .fail
                {
                    current_block = 14964981520188694172;
                    break;
                }
                if yaml_check_utf8(
                        (*tag_directive).prefix,
                        strlen((*tag_directive).prefix as *mut libc::c_char),
                    )
                    .fail
                {
                    current_block = 14964981520188694172;
                    break;
                }
                value.handle = yaml_strdup((*tag_directive).handle);
                value.prefix = yaml_strdup((*tag_directive).prefix);
                if value.handle.is_null() || value.prefix.is_null() {
                    current_block = 14964981520188694172;
                    break;
                }
                {
                    if tag_directives_copy.top == tag_directives_copy.end {
                        yaml_stack_extend(
                            &raw mut tag_directives_copy.start as *mut *mut libc::c_void,
                            &raw mut tag_directives_copy.top as *mut *mut libc::c_void,
                            &raw mut tag_directives_copy.end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::write(tag_directives_copy.top, value);
                    tag_directives_copy.top = tag_directives_copy.top.wrapping_offset(1);
                };
                value.handle = ptr::null_mut::<yaml_char_t>();
                value.prefix = ptr::null_mut::<yaml_char_t>();
                tag_directive = tag_directive.wrapping_offset(1);
            }
        } else {
            current_block = 16203760046146113240;
        }
        if current_block != 14964981520188694172 {
            let _ = memset(
                event as *mut libc::c_void,
                0,
                size_of::<YamlEventT>() as libc::c_ulong,
            );
            (*event).type_ = YamlDocumentStartEvent;
            (*event).start_mark = mark;
            (*event).end_mark = mark;
            let fresh164 = &raw mut (*event).data.document_start.version_directive;
            *fresh164 = version_directive_copy;
            let fresh165 = &raw mut (*event).data.document_start.tag_directives.start;
            *fresh165 = tag_directives_copy.start;
            let fresh166 = &raw mut (*event).data.document_start.tag_directives.end;
            *fresh166 = tag_directives_copy.top;
            (*event).data.document_start.implicit = implicit;
            return OK;
        }
        yaml_free(version_directive_copy as *mut libc::c_void);
        while !(tag_directives_copy.start == tag_directives_copy.top) {
            let value = *{
                tag_directives_copy.top = tag_directives_copy.top.offset(-1);
                tag_directives_copy.top
            };
            yaml_free(value.handle as *mut libc::c_void);
            yaml_free(value.prefix as *mut libc::c_void);
        }
        yaml_free(tag_directives_copy.start as *mut libc::c_void);
        tag_directives_copy.end = ptr::null_mut();
        tag_directives_copy.top = ptr::null_mut();
        tag_directives_copy.start = ptr::null_mut();
        yaml_free(value.handle as *mut libc::c_void);
        yaml_free(value.prefix as *mut libc::c_void);
        FAIL
    }
    /// Deletes a YAML document start event.
    ///
    /// This function frees any dynamically allocated memory associated with the given YAML document start event.
    ///
    /// # Parameters
    ///
    /// * `event`: A mutable pointer to a `YamlEventT` struct representing the YAML document start event to be deleted.
    ///
    /// # Safety
    ///
    /// This function must be called with a valid, non-null pointer to a `YamlEventT` struct that represents a YAML document start event.
    ///
    /// # Panics
    ///
    /// This function does not panic.
    pub unsafe fn yaml_document_start_event_delete(event: *mut YamlEventT) {
        if event.is_null() {
            return;
        }
        let version_ptr = (*event).data.document_start.version_directive;
        if !version_ptr.is_null() {
            yaml_free(version_ptr as *mut libc::c_void);
            (*event).data.document_start.version_directive = ptr::null_mut();
        }
        let mut directives_start = (*event).data.document_start.tag_directives.start;
        let directives_end = (*event).data.document_start.tag_directives.end;
        while directives_start < directives_end {
            yaml_free((*directives_start).handle as *mut libc::c_void);
            yaml_free((*directives_start).prefix as *mut libc::c_void);
            directives_start = directives_start.add(1);
        }
        if !(*event).data.document_start.tag_directives.start.is_null() {
            yaml_free(
                (*event).data.document_start.tag_directives.start as *mut libc::c_void,
            );
        }
        (*event).data.document_start.tag_directives.start = ptr::null_mut();
        (*event).data.document_start.tag_directives.end = ptr::null_mut();
    }
}
/// Internal utilities for LibYML.
///
/// This module contains internal utility functions and structures for the library.
pub mod internal {
    use crate::{
        externs::memmove, libc, memory::yaml_realloc,
        ops::{ForceAdd as _, ForceMul as _},
        success::{Success, FAIL, OK},
        yaml::{size_t, yaml_char_t},
        PointerExt,
    };
    /// Extend a stack by reallocating and copying the existing data.
    ///
    /// This function is used to grow a stack when more space is needed.
    ///
    /// # Safety
    ///
    /// - This function is unsafe because it directly calls the system's `realloc` function,
    ///   which can lead to undefined behaviour if misused.
    /// - The caller must ensure that `start`, `top`, and `end` are valid pointers into the
    ///   same allocated memory block.
    /// - The caller must ensure that the memory block being extended is large enough to
    ///   accommodate the new size.
    /// - The caller is responsible for properly freeing the extended memory block using
    ///   the corresponding `yaml_free` function when it is no longer needed.
    ///
    pub unsafe fn yaml_stack_extend(
        start: *mut *mut libc::c_void,
        top: *mut *mut libc::c_void,
        end: *mut *mut libc::c_void,
    ) {
        let new_start: *mut libc::c_void = yaml_realloc(
            *start,
            (((*end as *mut libc::c_char).c_offset_from(*start as *mut libc::c_char)
                as libc::c_long)
                .force_mul(2_i64)) as size_t,
        );
        *top = (new_start as *mut libc::c_char)
            .wrapping_offset(
                (*top as *mut libc::c_char).c_offset_from(*start as *mut libc::c_char)
                    as libc::c_long as isize,
            ) as *mut libc::c_void;
        *end = (new_start as *mut libc::c_char)
            .wrapping_offset(
                (((*end as *mut libc::c_char).c_offset_from(*start as *mut libc::c_char)
                    as libc::c_long)
                    .force_mul(2_i64)) as isize,
            ) as *mut libc::c_void;
        *start = new_start;
    }
    /// Extend a queue by reallocating (doubling) if necessary or moving existing data.
    ///
    /// # Safety
    ///
    /// - The caller must ensure `start`, `head`, `tail`, and `end` all point into
    ///   the same allocated memory block.
    /// - If `tail == end`, this function will attempt to move data (if there is front space),
    ///   or reallocate with double the current capacity.
    /// - The caller is responsible for eventually freeing the block with `yaml_free`.
    ///
    pub unsafe fn yaml_queue_extend(
        start: *mut *mut libc::c_void,
        head: *mut *mut libc::c_void,
        tail: *mut *mut libc::c_void,
        end: *mut *mut libc::c_void,
    ) {
        use crate::memory::yaml_realloc;
        use crate::libc;
        if *tail == *end {
            let used_bytes = (*tail as *mut libc::c_char)
                .c_offset_from(*head as *mut libc::c_char);
            let capacity = (*end as *mut libc::c_char)
                .c_offset_from(*start as *mut libc::c_char);
            if *head != *start {
                let _ = memmove(*start, *head, used_bytes as libc::c_ulong);
                *tail = (*start as *mut libc::c_char).add(used_bytes as usize)
                    as *mut libc::c_void;
                *head = *start;
            } else {
                let new_capacity = capacity * 2;
                let new_start = yaml_realloc(
                    *start,
                    (new_capacity as usize).try_into().unwrap(),
                );
                if new_start.is_null() {
                    {
                        ::core::panicking::panic_fmt(
                            format_args!("yaml_queue_extend: reallocation failed"),
                        );
                    };
                }
                let old_start_char = *start as *mut libc::c_char;
                let head_offset = (*head as *mut libc::c_char)
                    .c_offset_from(old_start_char);
                let tail_offset = (*tail as *mut libc::c_char)
                    .c_offset_from(old_start_char);
                *start = new_start;
                let new_start_char = new_start as *mut libc::c_char;
                *head = new_start_char.add(head_offset as usize) as *mut libc::c_void;
                *tail = new_start_char.add(tail_offset as usize) as *mut libc::c_void;
                *end = new_start_char.add(new_capacity as usize) as *mut libc::c_void;
            }
        }
    }
    /// Checks if the provided UTF-8 encoded string is valid according to the UTF-8 specification.
    ///
    /// # Parameters
    ///
    /// * `start`: A pointer to the start of the UTF-8 encoded string.
    /// * `length`: The length of the UTF-8 encoded string in bytes.
    ///
    /// # Return
    ///
    /// Returns `Success::OK` if the string is valid UTF-8, otherwise returns `Success::FAIL`.
    ///
    /// # Safety
    ///
    /// - `start` must be a valid, non-null pointer to a null-terminated UTF-8 string.
    /// - The UTF-8 encoded string must be properly formatted and not contain any invalid characters.
    /// - The string must be properly null-terminated.
    /// - The string must not contain any invalid characters or sequences.
    ///
    pub unsafe fn yaml_check_utf8(start: *const yaml_char_t, length: size_t) -> Success {
        let end: *const yaml_char_t = start.wrapping_offset(length as isize);
        let mut pointer: *const yaml_char_t = start;
        while pointer < end {
            let mut octet: libc::c_uchar;
            let mut value: libc::c_uint;
            let mut k: size_t;
            octet = *pointer;
            let width: libc::c_uint = if octet & 0x80 == 0 {
                1
            } else if octet & 0xE0 == 0xC0 {
                2
            } else if octet & 0xF0 == 0xE0 {
                3
            } else if octet & 0xF8 == 0xF0 {
                4
            } else {
                0
            } as libc::c_uint;
            value = if octet & 0x80 == 0 {
                octet & 0x7F
            } else if octet & 0xE0 == 0xC0 {
                octet & 0x1F
            } else if octet & 0xF0 == 0xE0 {
                octet & 0xF
            } else if octet & 0xF8 == 0xF0 {
                octet & 0x7
            } else {
                0
            } as libc::c_uint;
            if width == 0 {
                return FAIL;
            }
            if pointer.wrapping_offset(width as isize) > end {
                return FAIL;
            }
            k = 1_u64;
            while k < width as libc::c_ulong {
                octet = *pointer.wrapping_offset(k as isize);
                if octet & 0xC0 != 0x80 {
                    return FAIL;
                }
                value = (value << 6).force_add((octet & 0x3F) as libc::c_uint);
                k = k.force_add(1);
            }
            if !(width == 1 || width == 2 && value >= 0x80
                || width == 3 && value >= 0x800 || width == 4 && value >= 0x10000)
            {
                return FAIL;
            }
            pointer = pointer.wrapping_offset(width as isize);
        }
        OK
    }
}
/// Memory management for LibYML.
///
/// This module provides functions for managing memory within the library.
pub mod memory {
    use crate::{
        externs::{free, malloc, realloc, strlen},
        libc, yaml::{size_t, yaml_char_t},
    };
    use core::{mem::size_of, ptr};
    use libc::c_void;
    /// Allocate memory using the system's `malloc` function.
    ///
    /// This function allocates `size` bytes of uninitialized memory and returns a pointer to it.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of bytes to allocate.
    ///
    /// # Returns
    ///
    /// Returns a pointer to the allocated memory, or a null pointer if the allocation failed.
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - It directly calls the system's `malloc` function, which is not memory-safe.
    /// - The caller must ensure that the allocated memory is properly freed using `yaml_free`.
    /// - The caller is responsible for initializing the allocated memory before use.
    ///
    /// # Examples
    ///
    /// ```
    /// use libyml::memory::yaml_malloc;
    /// use libyml::yaml::size_t;
    /// use libyml::memory::yaml_free;
    ///
    /// unsafe {
    ///     let size: size_t = 1024;
    ///     let ptr = yaml_malloc(size);
    ///     if !ptr.is_null() {
    ///         // Use the allocated memory
    ///         // ...
    ///         yaml_free(ptr);
    ///     }
    /// }
    /// ```
    pub unsafe fn yaml_malloc(size: size_t) -> *mut c_void {
        malloc(size)
    }
    /// Reallocate memory using the system's `realloc` function.
    ///
    /// This function changes the size of the memory block pointed to by `ptr` to `size` bytes.
    ///
    /// # Arguments
    ///
    /// * `ptr` - A pointer to the memory block to reallocate. If null, this function behaves like `yaml_malloc`.
    /// * `size` - The new size of the memory block in bytes.
    ///
    /// # Returns
    ///
    /// Returns a pointer to the reallocated memory, which may be different from `ptr`, or a null pointer if the reallocation failed.
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - It directly calls the system's `realloc` function, which is not memory-safe.
    /// - The caller must ensure that `ptr` is either null or was previously allocated by `yaml_malloc` or `yaml_realloc`.
    /// - The caller must ensure that the reallocated memory is properly freed using `yaml_free`.
    /// - The contents of the reallocated memory beyond the original size are undefined.
    ///
    /// # Note
    ///
    /// If the reallocation fails, the original memory block is left untouched and a null pointer is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use libyml::memory::{yaml_malloc, yaml_realloc, yaml_free};
    /// use libyml::yaml::size_t;
    ///
    /// unsafe {
    ///     let mut size: size_t = 1024;
    ///     let mut ptr = yaml_malloc(size);
    ///     if !ptr.is_null() {
    ///         // Use the allocated memory
    ///         // ...
    ///         size = 2048;
    ///         ptr = yaml_realloc(ptr, size);
    ///         if !ptr.is_null() {
    ///             // Use the reallocated memory
    ///             // ...
    ///             yaml_free(ptr);
    ///         }
    ///     }
    /// }
    /// ```
    pub unsafe fn yaml_realloc(ptr: *mut c_void, size: size_t) -> *mut c_void {
        if !ptr.is_null() { realloc(ptr, size) } else { malloc(size) }
    }
    /// Free memory allocated by `yaml_malloc` or `yaml_realloc`.
    ///
    /// This function deallocates the memory previously allocated by `yaml_malloc` or `yaml_realloc`.
    ///
    /// # Arguments
    ///
    /// * `ptr` - A pointer to the memory block to free. If null, no operation is performed.
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - It directly calls the system's `free` function, which is not memory-safe.
    /// - The caller must ensure that `ptr` was allocated by `yaml_malloc` or `yaml_realloc`.
    /// - After calling this function, `ptr` becomes invalid and must not be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use libyml::memory::{yaml_malloc, yaml_free};
    /// use libyml::yaml::size_t;
    ///
    /// unsafe {
    ///     let size: size_t = 1024;
    ///     let ptr = yaml_malloc(size);
    ///     if !ptr.is_null() {
    ///         // Use the allocated memory
    ///         // ...
    ///         yaml_free(ptr);
    ///         // ptr is now invalid and must not be used
    ///     }
    /// }
    /// ```
    pub unsafe fn yaml_free(ptr: *mut c_void) {
        if !ptr.is_null() {
            free(ptr);
        }
    }
    /// Duplicate a string using the system's `malloc` function and manual copy due to type mismatch.
    ///
    /// This function creates a new copy of the input string, allocating new memory for it.
    ///
    /// # Arguments
    ///
    /// * `str` - A pointer to the null-terminated string to duplicate.
    ///
    /// # Returns
    ///
    /// Returns a pointer to the newly allocated string, or a null pointer if the allocation failed or the input was null.
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - It involves memory allocation and raw pointer manipulation.
    /// - The caller must ensure that `str` is a valid, null-terminated string.
    /// - The caller is responsible for freeing the returned pointer using `yaml_free`.
    ///
    /// # Examples
    ///
    /// ```
    /// use libyml::memory::{yaml_strdup, yaml_free};
    /// use libyml::yaml::yaml_char_t;
    /// use core::ffi::c_void;
    ///
    /// unsafe {
    ///     // Note: The cast to *const yaml_char_t is necessary because yaml_char_t
    ///     // might not be the same as u8 on all systems.
    ///     let original: *const yaml_char_t = b"Hello, world!\0".as_ptr() as *const yaml_char_t;
    ///     let copy = yaml_strdup(original);
    ///     if !copy.is_null() {
    ///         // Use the duplicated string
    ///         // ...
    ///         yaml_free(copy as *mut c_void);
    ///     }
    /// }
    /// ```
    pub unsafe fn yaml_strdup(str: *const yaml_char_t) -> *mut yaml_char_t {
        if str.is_null() {
            return ptr::null_mut();
        }
        let len = strlen(str as *const libc::c_char) as usize;
        let new_size = (len + 1) * size_of::<yaml_char_t>();
        let new_str = malloc(new_size.try_into().unwrap()) as *mut yaml_char_t;
        if new_str.is_null() {
            return ptr::null_mut();
        }
        ptr::copy_nonoverlapping(str, new_str, len + 1);
        new_str
    }
}
mod ops {
    pub(crate) trait ForceAdd: Sized {
        fn force_add(self, rhs: Self) -> Self;
    }
    impl ForceAdd for u8 {
        fn force_add(self, rhs: Self) -> Self {
            self.checked_add(rhs).unwrap_or_else(die)
        }
    }
    impl ForceAdd for i32 {
        fn force_add(self, rhs: Self) -> Self {
            self.checked_add(rhs).unwrap_or_else(die)
        }
    }
    impl ForceAdd for u32 {
        fn force_add(self, rhs: Self) -> Self {
            self.checked_add(rhs).unwrap_or_else(die)
        }
    }
    impl ForceAdd for u64 {
        fn force_add(self, rhs: Self) -> Self {
            self.checked_add(rhs).unwrap_or_else(die)
        }
    }
    impl ForceAdd for usize {
        fn force_add(self, rhs: Self) -> Self {
            self.checked_add(rhs).unwrap_or_else(die)
        }
    }
    pub(crate) trait ForceMul: Sized {
        fn force_mul(self, rhs: Self) -> Self;
    }
    impl ForceMul for i32 {
        fn force_mul(self, rhs: Self) -> Self {
            self.checked_mul(rhs).unwrap_or_else(die)
        }
    }
    impl ForceMul for i64 {
        fn force_mul(self, rhs: Self) -> Self {
            self.checked_mul(rhs).unwrap_or_else(die)
        }
    }
    impl ForceMul for u64 {
        fn force_mul(self, rhs: Self) -> Self {
            self.checked_mul(rhs).unwrap_or_else(die)
        }
    }
    pub(crate) trait ForceInto {
        fn force_into<U>(self) -> U
        where
            Self: TryInto<U>;
    }
    impl<T> ForceInto for T {
        fn force_into<U>(self) -> U
        where
            Self: TryInto<U>,
        {
            <Self as TryInto<U>>::try_into(self).ok().unwrap_or_else(die)
        }
    }
    #[cold]
    pub(crate) fn die<T>() -> T {
        struct PanicAgain;
        impl Drop for PanicAgain {
            fn drop(&mut self) {
                {
                    ::core::panicking::panic_fmt(format_args!("arithmetic overflow"));
                };
            }
        }
        fn do_die() -> ! {
            let _panic_again = PanicAgain;
            {
                ::core::panicking::panic_fmt(format_args!("arithmetic overflow"));
            };
        }
        do_die();
    }
}
mod parser {
    use crate::externs::{memcpy, memset, strcmp, strlen};
    use crate::internal::yaml_stack_extend;
    use crate::memory::{yaml_free, yaml_malloc, yaml_strdup};
    use crate::ops::ForceAdd as _;
    use crate::scanner::yaml_parser_fetch_more_tokens;
    use crate::success::{Success, FAIL, OK};
    use crate::yaml::{size_t, yaml_char_t};
    use crate::{
        libc, YamlAliasEvent, YamlAliasToken, YamlAnchorToken, YamlBlockEndToken,
        YamlBlockEntryToken, YamlBlockMappingStartToken, YamlBlockMappingStyle,
        YamlBlockSequenceStartToken, YamlBlockSequenceStyle, YamlDocumentEndEvent,
        YamlDocumentEndToken, YamlDocumentStartEvent, YamlDocumentStartToken, YamlEventT,
        YamlFlowEntryToken, YamlFlowMappingEndToken, YamlFlowMappingStartToken,
        YamlFlowMappingStyle, YamlFlowSequenceEndToken, YamlFlowSequenceStartToken,
        YamlFlowSequenceStyle, YamlKeyToken, YamlMappingEndEvent, YamlMappingStartEvent,
        YamlMarkT, YamlNoError, YamlParseBlockMappingFirstKeyState,
        YamlParseBlockMappingKeyState, YamlParseBlockMappingValueState,
        YamlParseBlockNodeOrIndentlessSequenceState, YamlParseBlockNodeState,
        YamlParseBlockSequenceEntryState, YamlParseBlockSequenceFirstEntryState,
        YamlParseDocumentContentState, YamlParseDocumentEndState,
        YamlParseDocumentStartState, YamlParseEndState,
        YamlParseFlowMappingEmptyValueState, YamlParseFlowMappingFirstKeyState,
        YamlParseFlowMappingKeyState, YamlParseFlowMappingValueState,
        YamlParseFlowNodeState, YamlParseFlowSequenceEntryMappingEndState,
        YamlParseFlowSequenceEntryMappingKeyState,
        YamlParseFlowSequenceEntryMappingValueState, YamlParseFlowSequenceEntryState,
        YamlParseFlowSequenceFirstEntryState, YamlParseImplicitDocumentStartState,
        YamlParseIndentlessSequenceEntryState, YamlParseStreamStartState,
        YamlParserError, YamlParserT, YamlPlainScalarStyle, YamlScalarEvent,
        YamlScalarToken, YamlSequenceEndEvent, YamlSequenceStartEvent,
        YamlStreamEndEvent, YamlStreamEndToken, YamlStreamStartEvent,
        YamlStreamStartToken, YamlTagDirectiveT, YamlTagDirectiveToken, YamlTagToken,
        YamlTokenT, YamlValueToken, YamlVersionDirectiveT, YamlVersionDirectiveToken,
    };
    use core::mem::size_of;
    use core::ptr::{self, addr_of_mut};
    unsafe fn peek_token(parser: *mut YamlParserT) -> *mut YamlTokenT {
        if (*parser).token_available || yaml_parser_fetch_more_tokens(parser).ok {
            (*parser).tokens.head
        } else {
            ptr::null_mut::<YamlTokenT>()
        }
    }
    unsafe fn skip_token(parser: *mut YamlParserT) {
        (*parser).token_available = false;
        let fresh3 = &raw mut (*parser).tokens_parsed;
        *fresh3 = (*fresh3).wrapping_add(1);
        (*parser).stream_end_produced = (*(*parser).tokens.head).type_
            == YamlStreamEndToken;
        let fresh4 = &raw mut (*parser).tokens.head;
        *fresh4 = (*fresh4).wrapping_offset(1);
    }
    /// Parse the input stream and produce the next parsing event.
    ///
    /// This function should be called repeatedly to produce a sequence of events
    /// corresponding to the input stream. The initial event will be of type
    /// `YamlStreamStartEvent`, and the final event will be of type `YamlStreamEndEvent`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because:
    /// - It operates on raw pointers.
    /// - It assumes certain memory layouts and alignments.
    /// - It may cause undefined behavior if the input pointers are invalid or if the
    ///   function is misused.
    ///
    /// # Arguments
    ///
    /// * `parser` - A pointer to a properly initialized `YamlParserT` struct.
    /// * `event` - A pointer to a `YamlEventT` struct that will be filled with the next event.
    ///
    /// # Returns
    ///
    /// Returns `OK` if an event was successfully parsed, or `FAIL` if:
    /// - The stream has ended (stream_end_produced is true)
    /// - There's an existing error in the parser
    /// - The parser is in the end state
    ///
    /// # Errors
    ///
    /// This function will return `FAIL` if any of the above error conditions are met.
    /// The caller should check the parser's error state for more details on the failure.
    ///
    /// # Notes
    ///
    /// - The caller is responsible for freeing any buffers associated with the produced
    ///   event using the `yaml_event_delete()` function.
    /// - Do not alternate calls to `yaml_parser_parse()` with calls to `yaml_parser_scan()`
    ///   or `yaml_parser_load()`. Doing so will break the parser.
    ///
    pub unsafe fn yaml_parser_parse(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
    ) -> Success {
        if !!parser.is_null() {
            crate::externs::__assert_fail("!parser.is_null()", "src/parser.rs", 108u32);
        }
        if !!event.is_null() {
            crate::externs::__assert_fail("!event.is_null()", "src/parser.rs", 109u32);
        }
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        if (*parser).stream_end_produced {
            return FAIL;
        }
        if (*parser).error != YamlNoError {
            return FAIL;
        }
        if (*parser).state == YamlParseEndState {
            return FAIL;
        }
        yaml_parser_state_machine(parser, event)
    }
    unsafe fn yaml_parser_set_parser_error(
        parser: *mut YamlParserT,
        problem: *const libc::c_char,
        problem_mark: YamlMarkT,
    ) {
        (*parser).error = YamlParserError;
        let fresh0 = &raw mut (*parser).problem;
        *fresh0 = problem;
        (*parser).problem_mark = problem_mark;
    }
    unsafe fn yaml_parser_set_parser_error_context(
        parser: *mut YamlParserT,
        context: *const libc::c_char,
        context_mark: YamlMarkT,
        problem: *const libc::c_char,
        problem_mark: YamlMarkT,
    ) {
        (*parser).error = YamlParserError;
        let fresh1 = &raw mut (*parser).context;
        *fresh1 = context;
        (*parser).context_mark = context_mark;
        let fresh2 = &raw mut (*parser).problem;
        *fresh2 = problem;
        (*parser).problem_mark = problem_mark;
    }
    unsafe fn yaml_parser_state_machine(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
    ) -> Success {
        match (*parser).state {
            YamlParseStreamStartState => yaml_parser_parse_stream_start(parser, event),
            YamlParseImplicitDocumentStartState => {
                yaml_parser_parse_document_start(parser, event, true)
            }
            YamlParseDocumentStartState => {
                yaml_parser_parse_document_start(parser, event, false)
            }
            YamlParseDocumentContentState => {
                yaml_parser_parse_document_content(parser, event)
            }
            YamlParseDocumentEndState => yaml_parser_parse_document_end(parser, event),
            YamlParseBlockNodeState => yaml_parser_parse_node(parser, event, true, false),
            YamlParseBlockNodeOrIndentlessSequenceState => {
                yaml_parser_parse_node(parser, event, true, true)
            }
            YamlParseFlowNodeState => yaml_parser_parse_node(parser, event, false, false),
            YamlParseBlockSequenceFirstEntryState => {
                yaml_parser_parse_block_sequence_entry(parser, event, true)
            }
            YamlParseBlockSequenceEntryState => {
                yaml_parser_parse_block_sequence_entry(parser, event, false)
            }
            YamlParseIndentlessSequenceEntryState => {
                yaml_parser_parse_indentless_sequence_entry(parser, event)
            }
            YamlParseBlockMappingFirstKeyState => {
                yaml_parser_parse_block_mapping_key(parser, event, true)
            }
            YamlParseBlockMappingKeyState => {
                yaml_parser_parse_block_mapping_key(parser, event, false)
            }
            YamlParseBlockMappingValueState => {
                yaml_parser_parse_block_mapping_value(parser, event)
            }
            YamlParseFlowSequenceFirstEntryState => {
                yaml_parser_parse_flow_sequence_entry(parser, event, true)
            }
            YamlParseFlowSequenceEntryState => {
                yaml_parser_parse_flow_sequence_entry(parser, event, false)
            }
            YamlParseFlowSequenceEntryMappingKeyState => {
                yaml_parser_parse_flow_sequence_entry_mapping_key(parser, event)
            }
            YamlParseFlowSequenceEntryMappingValueState => {
                yaml_parser_parse_flow_sequence_entry_mapping_value(parser, event)
            }
            YamlParseFlowSequenceEntryMappingEndState => {
                yaml_parser_parse_flow_sequence_entry_mapping_end(parser, event)
            }
            YamlParseFlowMappingFirstKeyState => {
                yaml_parser_parse_flow_mapping_key(parser, event, true)
            }
            YamlParseFlowMappingKeyState => {
                yaml_parser_parse_flow_mapping_key(parser, event, false)
            }
            YamlParseFlowMappingValueState => {
                yaml_parser_parse_flow_mapping_value(parser, event, false)
            }
            YamlParseFlowMappingEmptyValueState => {
                yaml_parser_parse_flow_mapping_value(parser, event, true)
            }
            _ => FAIL,
        }
    }
    unsafe fn yaml_parser_parse_stream_start(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
    ) -> Success {
        let token: *mut YamlTokenT = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ != YamlStreamStartToken {
            yaml_parser_set_parser_error(
                parser,
                b"did not find expected <stream-start>\0" as *const u8
                    as *const libc::c_char,
                (*token).start_mark,
            );
            return FAIL;
        }
        (*parser).state = YamlParseImplicitDocumentStartState;
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlStreamStartEvent;
        (*event).start_mark = (*token).start_mark;
        (*event).end_mark = (*token).start_mark;
        (*event).data.stream_start.encoding = (*token).data.stream_start.encoding;
        skip_token(parser);
        OK
    }
    unsafe fn yaml_parser_parse_document_start(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
        implicit: bool,
    ) -> Success {
        let mut token: *mut YamlTokenT;
        let mut version_directive: *mut YamlVersionDirectiveT = ptr::null_mut::<
            YamlVersionDirectiveT,
        >();
        struct TagDirectives {
            start: *mut YamlTagDirectiveT,
            end: *mut YamlTagDirectiveT,
        }
        let mut tag_directives = TagDirectives {
            start: ptr::null_mut::<YamlTagDirectiveT>(),
            end: ptr::null_mut::<YamlTagDirectiveT>(),
        };
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if !implicit {
            while (*token).type_ == YamlDocumentEndToken {
                skip_token(parser);
                token = peek_token(parser);
                if token.is_null() {
                    return FAIL;
                }
            }
        }
        if implicit && (*token).type_ != YamlVersionDirectiveToken
            && (*token).type_ != YamlTagDirectiveToken
            && (*token).type_ != YamlDocumentStartToken
            && (*token).type_ != YamlStreamEndToken
        {
            if yaml_parser_process_directives(
                    parser,
                    ptr::null_mut::<*mut YamlVersionDirectiveT>(),
                    ptr::null_mut::<*mut YamlTagDirectiveT>(),
                    ptr::null_mut::<*mut YamlTagDirectiveT>(),
                )
                .fail
            {
                return FAIL;
            }
            {
                if (*parser).states.top == (*parser).states.end {
                    yaml_stack_extend(
                        &raw mut (*parser).states.start as *mut *mut libc::c_void,
                        &raw mut (*parser).states.top as *mut *mut libc::c_void,
                        &raw mut (*parser).states.end as *mut *mut libc::c_void,
                    );
                }
                ptr::write((*parser).states.top, YamlParseDocumentEndState);
                (*parser).states.top = (*parser).states.top.wrapping_offset(1);
            };
            (*parser).state = YamlParseBlockNodeState;
            let _ = memset(
                event as *mut libc::c_void,
                0,
                size_of::<YamlEventT>() as libc::c_ulong,
            );
            (*event).type_ = YamlDocumentStartEvent;
            (*event).start_mark = (*token).start_mark;
            (*event).end_mark = (*token).start_mark;
            let fresh9 = &raw mut (*event).data.document_start.version_directive;
            *fresh9 = ptr::null_mut::<YamlVersionDirectiveT>();
            let fresh10 = &raw mut (*event).data.document_start.tag_directives.start;
            *fresh10 = ptr::null_mut::<YamlTagDirectiveT>();
            let fresh11 = &raw mut (*event).data.document_start.tag_directives.end;
            *fresh11 = ptr::null_mut::<YamlTagDirectiveT>();
            (*event).data.document_start.implicit = true;
            OK
        } else if (*token).type_ != YamlStreamEndToken {
            let end_mark: YamlMarkT;
            let start_mark: YamlMarkT = (*token).start_mark;
            if yaml_parser_process_directives(
                    parser,
                    &raw mut version_directive,
                    &raw mut tag_directives.start,
                    &raw mut tag_directives.end,
                )
                .fail
            {
                return FAIL;
            }
            token = peek_token(parser);
            if !token.is_null() {
                if (*token).type_ != YamlDocumentStartToken {
                    yaml_parser_set_parser_error(
                        parser,
                        b"did not find expected <document start>\0" as *const u8
                            as *const libc::c_char,
                        (*token).start_mark,
                    );
                } else {
                    {
                        if (*parser).states.top == (*parser).states.end {
                            yaml_stack_extend(
                                &raw mut (*parser).states.start as *mut *mut libc::c_void,
                                &raw mut (*parser).states.top as *mut *mut libc::c_void,
                                &raw mut (*parser).states.end as *mut *mut libc::c_void,
                            );
                        }
                        ptr::write((*parser).states.top, YamlParseDocumentEndState);
                        (*parser).states.top = (*parser).states.top.wrapping_offset(1);
                    };
                    (*parser).state = YamlParseDocumentContentState;
                    end_mark = (*token).end_mark;
                    let _ = memset(
                        event as *mut libc::c_void,
                        0,
                        size_of::<YamlEventT>() as libc::c_ulong,
                    );
                    (*event).type_ = YamlDocumentStartEvent;
                    (*event).start_mark = start_mark;
                    (*event).end_mark = end_mark;
                    let fresh14 = &raw mut (*event)
                        .data
                        .document_start
                        .version_directive;
                    *fresh14 = version_directive;
                    let fresh15 = &raw mut (*event)
                        .data
                        .document_start
                        .tag_directives
                        .start;
                    *fresh15 = tag_directives.start;
                    let fresh16 = &raw mut (*event)
                        .data
                        .document_start
                        .tag_directives
                        .end;
                    *fresh16 = tag_directives.end;
                    (*event).data.document_start.implicit = false;
                    skip_token(parser);
                    tag_directives.end = ptr::null_mut::<YamlTagDirectiveT>();
                    tag_directives.start = tag_directives.end;
                    return OK;
                }
            }
            yaml_free(version_directive as *mut libc::c_void);
            while tag_directives.start != tag_directives.end {
                yaml_free(
                    (*tag_directives.end.wrapping_offset(-1_isize)).handle
                        as *mut libc::c_void,
                );
                yaml_free(
                    (*tag_directives.end.wrapping_offset(-1_isize)).prefix
                        as *mut libc::c_void,
                );
                tag_directives.end = tag_directives.end.wrapping_offset(-1);
            }
            yaml_free(tag_directives.start as *mut libc::c_void);
            FAIL
        } else {
            (*parser).state = YamlParseEndState;
            let _ = memset(
                event as *mut libc::c_void,
                0,
                size_of::<YamlEventT>() as libc::c_ulong,
            );
            (*event).type_ = YamlStreamEndEvent;
            (*event).start_mark = (*token).start_mark;
            (*event).end_mark = (*token).end_mark;
            skip_token(parser);
            OK
        }
    }
    unsafe fn yaml_parser_parse_document_content(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
    ) -> Success {
        let token: *mut YamlTokenT = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ == YamlVersionDirectiveToken
            || (*token).type_ == YamlTagDirectiveToken
            || (*token).type_ == YamlDocumentStartToken
            || (*token).type_ == YamlDocumentEndToken
            || (*token).type_ == YamlStreamEndToken
        {
            (*parser).state = *{
                (*parser).states.top = (*parser).states.top.offset(-1);
                (*parser).states.top
            };
            yaml_parser_process_empty_scalar(event, (*token).start_mark)
        } else {
            yaml_parser_parse_node(parser, event, true, false)
        }
    }
    unsafe fn yaml_parser_parse_document_end(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
    ) -> Success {
        let mut end_mark: YamlMarkT;
        let mut implicit = true;
        let token: *mut YamlTokenT = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        end_mark = (*token).start_mark;
        let start_mark: YamlMarkT = end_mark;
        if (*token).type_ == YamlDocumentEndToken {
            end_mark = (*token).end_mark;
            skip_token(parser);
            implicit = false;
        }
        while !((*parser).tag_directives.start == (*parser).tag_directives.top) {
            let tag_directive = *{
                (*parser).tag_directives.top = (*parser).tag_directives.top.offset(-1);
                (*parser).tag_directives.top
            };
            yaml_free(tag_directive.handle as *mut libc::c_void);
            yaml_free(tag_directive.prefix as *mut libc::c_void);
        }
        (*parser).state = YamlParseDocumentStartState;
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlDocumentEndEvent;
        (*event).start_mark = start_mark;
        (*event).end_mark = end_mark;
        (*event).data.document_end.implicit = implicit;
        OK
    }
    unsafe fn yaml_parser_parse_node(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
        block: bool,
        indentless_sequence: bool,
    ) -> Success {
        let mut current_block: u64;
        let mut token: *mut YamlTokenT;
        let mut anchor: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let mut tag_handle: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let mut tag_suffix: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let mut tag: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let mut start_mark: YamlMarkT;
        let mut end_mark: YamlMarkT;
        let mut tag_mark = YamlMarkT {
            index: 0,
            line: 0,
            column: 0,
        };
        let implicit;
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ == YamlAliasToken {
            (*parser).state = *{
                (*parser).states.top = (*parser).states.top.offset(-1);
                (*parser).states.top
            };
            let _ = memset(
                event as *mut libc::c_void,
                0,
                size_of::<YamlEventT>() as libc::c_ulong,
            );
            (*event).type_ = YamlAliasEvent;
            (*event).start_mark = (*token).start_mark;
            (*event).end_mark = (*token).end_mark;
            let fresh26 = &raw mut (*event).data.alias.anchor;
            *fresh26 = (*token).data.alias.value;
            skip_token(parser);
            OK
        } else {
            end_mark = (*token).start_mark;
            start_mark = end_mark;
            if (*token).type_ == YamlAnchorToken {
                anchor = (*token).data.anchor.value;
                start_mark = (*token).start_mark;
                end_mark = (*token).end_mark;
                skip_token(parser);
                token = peek_token(parser);
                if token.is_null() {
                    current_block = 17786380918591080555;
                } else if (*token).type_ == YamlTagToken {
                    tag_handle = (*token).data.tag.handle;
                    tag_suffix = (*token).data.tag.suffix;
                    tag_mark = (*token).start_mark;
                    end_mark = (*token).end_mark;
                    skip_token(parser);
                    token = peek_token(parser);
                    if token.is_null() {
                        current_block = 17786380918591080555;
                    } else {
                        current_block = 11743904203796629665;
                    }
                } else {
                    current_block = 11743904203796629665;
                }
            } else if (*token).type_ == YamlTagToken {
                tag_handle = (*token).data.tag.handle;
                tag_suffix = (*token).data.tag.suffix;
                tag_mark = (*token).start_mark;
                start_mark = tag_mark;
                end_mark = (*token).end_mark;
                skip_token(parser);
                token = peek_token(parser);
                if token.is_null() {
                    current_block = 17786380918591080555;
                } else if (*token).type_ == YamlAnchorToken {
                    anchor = (*token).data.anchor.value;
                    end_mark = (*token).end_mark;
                    skip_token(parser);
                    token = peek_token(parser);
                    if token.is_null() {
                        current_block = 17786380918591080555;
                    } else {
                        current_block = 11743904203796629665;
                    }
                } else {
                    current_block = 11743904203796629665;
                }
            } else {
                current_block = 11743904203796629665;
            }
            if current_block == 11743904203796629665 {
                if !tag_handle.is_null() {
                    if *tag_handle == 0 {
                        tag = tag_suffix;
                        yaml_free(tag_handle as *mut libc::c_void);
                        tag_suffix = ptr::null_mut::<yaml_char_t>();
                        tag_handle = tag_suffix;
                        current_block = 9437013279121998969;
                    } else {
                        let mut tag_directive: *mut YamlTagDirectiveT;
                        tag_directive = (*parser).tag_directives.start;
                        loop {
                            if tag_directive == (*parser).tag_directives.top {
                                current_block = 17728966195399430138;
                                break;
                            }
                            if strcmp(
                                (*tag_directive).handle as *mut libc::c_char,
                                tag_handle as *mut libc::c_char,
                            ) == 0
                            {
                                let prefix_len: size_t = strlen(
                                    (*tag_directive).prefix as *mut libc::c_char,
                                );
                                let suffix_len: size_t = strlen(
                                    tag_suffix as *mut libc::c_char,
                                );
                                tag = yaml_malloc(
                                    prefix_len.force_add(suffix_len).force_add(1_u64),
                                ) as *mut yaml_char_t;
                                let _ = memcpy(
                                    tag as *mut libc::c_void,
                                    (*tag_directive).prefix as *const libc::c_void,
                                    prefix_len,
                                );
                                let _ = memcpy(
                                    tag.wrapping_offset(prefix_len as isize)
                                        as *mut libc::c_void,
                                    tag_suffix as *const libc::c_void,
                                    suffix_len,
                                );
                                *tag
                                    .wrapping_offset(
                                        prefix_len.force_add(suffix_len) as isize,
                                    ) = b'\0';
                                yaml_free(tag_handle as *mut libc::c_void);
                                yaml_free(tag_suffix as *mut libc::c_void);
                                tag_suffix = ptr::null_mut::<yaml_char_t>();
                                tag_handle = tag_suffix;
                                current_block = 17728966195399430138;
                                break;
                            } else {
                                tag_directive = tag_directive.wrapping_offset(1);
                            }
                        }
                        if current_block != 17786380918591080555 {
                            if tag.is_null() {
                                yaml_parser_set_parser_error_context(
                                    parser,
                                    b"while parsing a node\0" as *const u8
                                        as *const libc::c_char,
                                    start_mark,
                                    b"found undefined tag handle\0" as *const u8
                                        as *const libc::c_char,
                                    tag_mark,
                                );
                                current_block = 17786380918591080555;
                            } else {
                                current_block = 9437013279121998969;
                            }
                        }
                    }
                } else {
                    current_block = 9437013279121998969;
                }
                if current_block != 17786380918591080555 {
                    implicit = tag.is_null() || *tag == 0;
                    if indentless_sequence && (*token).type_ == YamlBlockEntryToken {
                        end_mark = (*token).end_mark;
                        (*parser).state = YamlParseIndentlessSequenceEntryState;
                        let _ = memset(
                            event as *mut libc::c_void,
                            0,
                            size_of::<YamlEventT>() as libc::c_ulong,
                        );
                        (*event).type_ = YamlSequenceStartEvent;
                        (*event).start_mark = start_mark;
                        (*event).end_mark = end_mark;
                        let fresh37 = &raw mut (*event).data.sequence_start.anchor;
                        *fresh37 = anchor;
                        let fresh38 = &raw mut (*event).data.sequence_start.tag;
                        *fresh38 = tag;
                        (*event).data.sequence_start.implicit = implicit;
                        (*event).data.sequence_start.style = YamlBlockSequenceStyle;
                        return OK;
                    } else if (*token).type_ == YamlScalarToken {
                        let mut plain_implicit = false;
                        let mut quoted_implicit = false;
                        end_mark = (*token).end_mark;
                        if (*token).data.scalar.style == YamlPlainScalarStyle
                            && tag.is_null()
                            || !tag.is_null()
                                && strcmp(
                                    tag as *mut libc::c_char,
                                    b"!\0" as *const u8 as *const libc::c_char,
                                ) == 0
                        {
                            plain_implicit = true;
                        } else if tag.is_null() {
                            quoted_implicit = true;
                        }
                        (*parser).state = *{
                            (*parser).states.top = (*parser).states.top.offset(-1);
                            (*parser).states.top
                        };
                        let _ = memset(
                            event as *mut libc::c_void,
                            0,
                            size_of::<YamlEventT>() as libc::c_ulong,
                        );
                        (*event).type_ = YamlScalarEvent;
                        (*event).start_mark = start_mark;
                        (*event).end_mark = end_mark;
                        let fresh40 = &raw mut (*event).data.scalar.anchor;
                        *fresh40 = anchor;
                        let fresh41 = &raw mut (*event).data.scalar.tag;
                        *fresh41 = tag;
                        let fresh42 = &raw mut (*event).data.scalar.value;
                        *fresh42 = (*token).data.scalar.value;
                        (*event).data.scalar.length = (*token).data.scalar.length;
                        (*event).data.scalar.plain_implicit = plain_implicit;
                        (*event).data.scalar.quoted_implicit = quoted_implicit;
                        (*event).data.scalar.style = (*token).data.scalar.style;
                        skip_token(parser);
                        return OK;
                    } else if (*token).type_ == YamlFlowSequenceStartToken {
                        end_mark = (*token).end_mark;
                        (*parser).state = YamlParseFlowSequenceFirstEntryState;
                        let _ = memset(
                            event as *mut libc::c_void,
                            0,
                            size_of::<YamlEventT>() as libc::c_ulong,
                        );
                        (*event).type_ = YamlSequenceStartEvent;
                        (*event).start_mark = start_mark;
                        (*event).end_mark = end_mark;
                        let fresh45 = &raw mut (*event).data.sequence_start.anchor;
                        *fresh45 = anchor;
                        let fresh46 = &raw mut (*event).data.sequence_start.tag;
                        *fresh46 = tag;
                        (*event).data.sequence_start.implicit = implicit;
                        (*event).data.sequence_start.style = YamlFlowSequenceStyle;
                        return OK;
                    } else if (*token).type_ == YamlFlowMappingStartToken {
                        end_mark = (*token).end_mark;
                        (*parser).state = YamlParseFlowMappingFirstKeyState;
                        let _ = memset(
                            event as *mut libc::c_void,
                            0,
                            size_of::<YamlEventT>() as libc::c_ulong,
                        );
                        (*event).type_ = YamlMappingStartEvent;
                        (*event).start_mark = start_mark;
                        (*event).end_mark = end_mark;
                        let fresh47 = &raw mut (*event).data.mapping_start.anchor;
                        *fresh47 = anchor;
                        let fresh48 = &raw mut (*event).data.mapping_start.tag;
                        *fresh48 = tag;
                        (*event).data.mapping_start.implicit = implicit;
                        (*event).data.mapping_start.style = YamlFlowMappingStyle;
                        return OK;
                    } else if block && (*token).type_ == YamlBlockSequenceStartToken {
                        end_mark = (*token).end_mark;
                        (*parser).state = YamlParseBlockSequenceFirstEntryState;
                        let _ = memset(
                            event as *mut libc::c_void,
                            0,
                            size_of::<YamlEventT>() as libc::c_ulong,
                        );
                        (*event).type_ = YamlSequenceStartEvent;
                        (*event).start_mark = start_mark;
                        (*event).end_mark = end_mark;
                        let fresh49 = &raw mut (*event).data.sequence_start.anchor;
                        *fresh49 = anchor;
                        let fresh50 = &raw mut (*event).data.sequence_start.tag;
                        *fresh50 = tag;
                        (*event).data.sequence_start.implicit = implicit;
                        (*event).data.sequence_start.style = YamlBlockSequenceStyle;
                        return OK;
                    } else if block && (*token).type_ == YamlBlockMappingStartToken {
                        end_mark = (*token).end_mark;
                        (*parser).state = YamlParseBlockMappingFirstKeyState;
                        let _ = memset(
                            event as *mut libc::c_void,
                            0,
                            size_of::<YamlEventT>() as libc::c_ulong,
                        );
                        (*event).type_ = YamlMappingStartEvent;
                        (*event).start_mark = start_mark;
                        (*event).end_mark = end_mark;
                        let fresh51 = &raw mut (*event).data.mapping_start.anchor;
                        *fresh51 = anchor;
                        let fresh52 = &raw mut (*event).data.mapping_start.tag;
                        *fresh52 = tag;
                        (*event).data.mapping_start.implicit = implicit;
                        (*event).data.mapping_start.style = YamlBlockMappingStyle;
                        return OK;
                    } else if !anchor.is_null() || !tag.is_null() {
                        let value: *mut yaml_char_t = yaml_malloc(1_u64)
                            as *mut yaml_char_t;
                        *value = b'\0';
                        (*parser).state = *{
                            (*parser).states.top = (*parser).states.top.offset(-1);
                            (*parser).states.top
                        };
                        let _ = memset(
                            event as *mut libc::c_void,
                            0,
                            size_of::<YamlEventT>() as libc::c_ulong,
                        );
                        (*event).type_ = YamlScalarEvent;
                        (*event).start_mark = start_mark;
                        (*event).end_mark = end_mark;
                        let fresh54 = &raw mut (*event).data.scalar.anchor;
                        *fresh54 = anchor;
                        let fresh55 = &raw mut (*event).data.scalar.tag;
                        *fresh55 = tag;
                        let fresh56 = &raw mut (*event).data.scalar.value;
                        *fresh56 = value;
                        (*event).data.scalar.length = 0_u64;
                        (*event).data.scalar.plain_implicit = implicit;
                        (*event).data.scalar.quoted_implicit = false;
                        (*event).data.scalar.style = YamlPlainScalarStyle;
                        return OK;
                    } else {
                        yaml_parser_set_parser_error_context(
                            parser,
                            if block {
                                b"while parsing a block node\0" as *const u8
                                    as *const libc::c_char
                            } else {
                                b"while parsing a flow node\0" as *const u8
                                    as *const libc::c_char
                            },
                            start_mark,
                            b"did not find expected node content\0" as *const u8
                                as *const libc::c_char,
                            (*token).start_mark,
                        );
                    }
                }
            }
            yaml_free(anchor as *mut libc::c_void);
            yaml_free(tag_handle as *mut libc::c_void);
            yaml_free(tag_suffix as *mut libc::c_void);
            yaml_free(tag as *mut libc::c_void);
            FAIL
        }
    }
    unsafe fn yaml_parser_parse_block_sequence_entry(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
        first: bool,
    ) -> Success {
        let mut token: *mut YamlTokenT;
        if first {
            token = peek_token(parser);
            {
                if (*parser).marks.top == (*parser).marks.end {
                    yaml_stack_extend(
                        &raw mut (*parser).marks.start as *mut *mut libc::c_void,
                        &raw mut (*parser).marks.top as *mut *mut libc::c_void,
                        &raw mut (*parser).marks.end as *mut *mut libc::c_void,
                    );
                }
                ptr::write((*parser).marks.top, (*token).start_mark);
                (*parser).marks.top = (*parser).marks.top.wrapping_offset(1);
            };
            skip_token(parser);
        }
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ == YamlBlockEntryToken {
            let mark: YamlMarkT = (*token).end_mark;
            skip_token(parser);
            token = peek_token(parser);
            if token.is_null() {
                return FAIL;
            }
            if (*token).type_ != YamlBlockEntryToken
                && (*token).type_ != YamlBlockEndToken
            {
                {
                    if (*parser).states.top == (*parser).states.end {
                        yaml_stack_extend(
                            &raw mut (*parser).states.start as *mut *mut libc::c_void,
                            &raw mut (*parser).states.top as *mut *mut libc::c_void,
                            &raw mut (*parser).states.end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::write((*parser).states.top, YamlParseBlockSequenceEntryState);
                    (*parser).states.top = (*parser).states.top.wrapping_offset(1);
                };
                yaml_parser_parse_node(parser, event, true, false)
            } else {
                (*parser).state = YamlParseBlockSequenceEntryState;
                yaml_parser_process_empty_scalar(event, mark)
            }
        } else if (*token).type_ == YamlBlockEndToken {
            (*parser).state = *{
                (*parser).states.top = (*parser).states.top.offset(-1);
                (*parser).states.top
            };
            let _ = *{
                (*parser).marks.top = (*parser).marks.top.offset(-1);
                (*parser).marks.top
            };
            let _ = memset(
                event as *mut libc::c_void,
                0,
                size_of::<YamlEventT>() as libc::c_ulong,
            );
            (*event).type_ = YamlSequenceEndEvent;
            (*event).start_mark = (*token).start_mark;
            (*event).end_mark = (*token).end_mark;
            skip_token(parser);
            OK
        } else {
            yaml_parser_set_parser_error_context(
                parser,
                b"while parsing a block collection\0" as *const u8
                    as *const libc::c_char,
                *{
                    (*parser).marks.top = (*parser).marks.top.offset(-1);
                    (*parser).marks.top
                },
                b"did not find expected '-' indicator\0" as *const u8
                    as *const libc::c_char,
                (*token).start_mark,
            );
            FAIL
        }
    }
    unsafe fn yaml_parser_parse_indentless_sequence_entry(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
    ) -> Success {
        let mut token: *mut YamlTokenT;
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ == YamlBlockEntryToken {
            let mark: YamlMarkT = (*token).end_mark;
            skip_token(parser);
            token = peek_token(parser);
            if token.is_null() {
                return FAIL;
            }
            if (*token).type_ != YamlBlockEntryToken && (*token).type_ != YamlKeyToken
                && (*token).type_ != YamlValueToken
                && (*token).type_ != YamlBlockEndToken
            {
                {
                    if (*parser).states.top == (*parser).states.end {
                        yaml_stack_extend(
                            &raw mut (*parser).states.start as *mut *mut libc::c_void,
                            &raw mut (*parser).states.top as *mut *mut libc::c_void,
                            &raw mut (*parser).states.end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::write(
                        (*parser).states.top,
                        YamlParseIndentlessSequenceEntryState,
                    );
                    (*parser).states.top = (*parser).states.top.wrapping_offset(1);
                };
                yaml_parser_parse_node(parser, event, true, false)
            } else {
                (*parser).state = YamlParseIndentlessSequenceEntryState;
                yaml_parser_process_empty_scalar(event, mark)
            }
        } else {
            (*parser).state = *{
                (*parser).states.top = (*parser).states.top.offset(-1);
                (*parser).states.top
            };
            let _ = memset(
                event as *mut libc::c_void,
                0,
                size_of::<YamlEventT>() as libc::c_ulong,
            );
            (*event).type_ = YamlSequenceEndEvent;
            (*event).start_mark = (*token).start_mark;
            (*event).end_mark = (*token).start_mark;
            OK
        }
    }
    unsafe fn yaml_parser_parse_block_mapping_key(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
        first: bool,
    ) -> Success {
        let mut token: *mut YamlTokenT;
        if first {
            token = peek_token(parser);
            {
                if (*parser).marks.top == (*parser).marks.end {
                    yaml_stack_extend(
                        &raw mut (*parser).marks.start as *mut *mut libc::c_void,
                        &raw mut (*parser).marks.top as *mut *mut libc::c_void,
                        &raw mut (*parser).marks.end as *mut *mut libc::c_void,
                    );
                }
                ptr::write((*parser).marks.top, (*token).start_mark);
                (*parser).marks.top = (*parser).marks.top.wrapping_offset(1);
            };
            skip_token(parser);
        }
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ == YamlKeyToken {
            let mark: YamlMarkT = (*token).end_mark;
            skip_token(parser);
            token = peek_token(parser);
            if token.is_null() {
                return FAIL;
            }
            if (*token).type_ != YamlKeyToken && (*token).type_ != YamlValueToken
                && (*token).type_ != YamlBlockEndToken
            {
                {
                    if (*parser).states.top == (*parser).states.end {
                        yaml_stack_extend(
                            &raw mut (*parser).states.start as *mut *mut libc::c_void,
                            &raw mut (*parser).states.top as *mut *mut libc::c_void,
                            &raw mut (*parser).states.end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::write((*parser).states.top, YamlParseBlockMappingValueState);
                    (*parser).states.top = (*parser).states.top.wrapping_offset(1);
                };
                yaml_parser_parse_node(parser, event, true, true)
            } else {
                (*parser).state = YamlParseBlockMappingValueState;
                yaml_parser_process_empty_scalar(event, mark)
            }
        } else if (*token).type_ == YamlBlockEndToken {
            (*parser).state = *{
                (*parser).states.top = (*parser).states.top.offset(-1);
                (*parser).states.top
            };
            let _ = *{
                (*parser).marks.top = (*parser).marks.top.offset(-1);
                (*parser).marks.top
            };
            let _ = memset(
                event as *mut libc::c_void,
                0,
                size_of::<YamlEventT>() as libc::c_ulong,
            );
            (*event).type_ = YamlMappingEndEvent;
            (*event).start_mark = (*token).start_mark;
            (*event).end_mark = (*token).end_mark;
            skip_token(parser);
            OK
        } else {
            yaml_parser_set_parser_error_context(
                parser,
                b"while parsing a block mapping\0" as *const u8 as *const libc::c_char,
                *{
                    (*parser).marks.top = (*parser).marks.top.offset(-1);
                    (*parser).marks.top
                },
                b"did not find expected key\0" as *const u8 as *const libc::c_char,
                (*token).start_mark,
            );
            FAIL
        }
    }
    unsafe fn yaml_parser_parse_block_mapping_value(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
    ) -> Success {
        let mut token: *mut YamlTokenT;
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ == YamlValueToken {
            let mark: YamlMarkT = (*token).end_mark;
            skip_token(parser);
            token = peek_token(parser);
            if token.is_null() {
                return FAIL;
            }
            if (*token).type_ != YamlKeyToken && (*token).type_ != YamlValueToken
                && (*token).type_ != YamlBlockEndToken
            {
                {
                    if (*parser).states.top == (*parser).states.end {
                        yaml_stack_extend(
                            &raw mut (*parser).states.start as *mut *mut libc::c_void,
                            &raw mut (*parser).states.top as *mut *mut libc::c_void,
                            &raw mut (*parser).states.end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::write((*parser).states.top, YamlParseBlockMappingKeyState);
                    (*parser).states.top = (*parser).states.top.wrapping_offset(1);
                };
                yaml_parser_parse_node(parser, event, true, true)
            } else {
                (*parser).state = YamlParseBlockMappingKeyState;
                yaml_parser_process_empty_scalar(event, mark)
            }
        } else {
            (*parser).state = YamlParseBlockMappingKeyState;
            yaml_parser_process_empty_scalar(event, (*token).start_mark)
        }
    }
    unsafe fn yaml_parser_parse_flow_sequence_entry(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
        first: bool,
    ) -> Success {
        let mut token: *mut YamlTokenT;
        if first {
            token = peek_token(parser);
            {
                if (*parser).marks.top == (*parser).marks.end {
                    yaml_stack_extend(
                        &raw mut (*parser).marks.start as *mut *mut libc::c_void,
                        &raw mut (*parser).marks.top as *mut *mut libc::c_void,
                        &raw mut (*parser).marks.end as *mut *mut libc::c_void,
                    );
                }
                ptr::write((*parser).marks.top, (*token).start_mark);
                (*parser).marks.top = (*parser).marks.top.wrapping_offset(1);
            };
            skip_token(parser);
        }
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ != YamlFlowSequenceEndToken {
            if !first {
                if (*token).type_ == YamlFlowEntryToken {
                    skip_token(parser);
                    token = peek_token(parser);
                    if token.is_null() {
                        return FAIL;
                    }
                } else {
                    yaml_parser_set_parser_error_context(
                        parser,
                        b"while parsing a flow sequence\0" as *const u8
                            as *const libc::c_char,
                        *{
                            (*parser).marks.top = (*parser).marks.top.offset(-1);
                            (*parser).marks.top
                        },
                        b"did not find expected ',' or ']'\0" as *const u8
                            as *const libc::c_char,
                        (*token).start_mark,
                    );
                    return FAIL;
                }
            }
            if (*token).type_ == YamlKeyToken {
                (*parser).state = YamlParseFlowSequenceEntryMappingKeyState;
                let _ = memset(
                    event as *mut libc::c_void,
                    0,
                    size_of::<YamlEventT>() as libc::c_ulong,
                );
                (*event).type_ = YamlMappingStartEvent;
                (*event).start_mark = (*token).start_mark;
                (*event).end_mark = (*token).end_mark;
                let fresh99 = &raw mut (*event).data.mapping_start.anchor;
                *fresh99 = ptr::null_mut::<yaml_char_t>();
                let fresh100 = &raw mut (*event).data.mapping_start.tag;
                *fresh100 = ptr::null_mut::<yaml_char_t>();
                (*event).data.mapping_start.implicit = true;
                (*event).data.mapping_start.style = YamlFlowMappingStyle;
                skip_token(parser);
                return OK;
            } else if (*token).type_ != YamlFlowSequenceEndToken {
                {
                    if (*parser).states.top == (*parser).states.end {
                        yaml_stack_extend(
                            &raw mut (*parser).states.start as *mut *mut libc::c_void,
                            &raw mut (*parser).states.top as *mut *mut libc::c_void,
                            &raw mut (*parser).states.end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::write((*parser).states.top, YamlParseFlowSequenceEntryState);
                    (*parser).states.top = (*parser).states.top.wrapping_offset(1);
                };
                return yaml_parser_parse_node(parser, event, false, false);
            }
        }
        (*parser).state = *{
            (*parser).states.top = (*parser).states.top.offset(-1);
            (*parser).states.top
        };
        let _ = *{
            (*parser).marks.top = (*parser).marks.top.offset(-1);
            (*parser).marks.top
        };
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlSequenceEndEvent;
        (*event).start_mark = (*token).start_mark;
        (*event).end_mark = (*token).end_mark;
        skip_token(parser);
        OK
    }
    unsafe fn yaml_parser_parse_flow_sequence_entry_mapping_key(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
    ) -> Success {
        let token: *mut YamlTokenT = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ != YamlValueToken && (*token).type_ != YamlFlowEntryToken
            && (*token).type_ != YamlFlowSequenceEndToken
        {
            {
                if (*parser).states.top == (*parser).states.end {
                    yaml_stack_extend(
                        &raw mut (*parser).states.start as *mut *mut libc::c_void,
                        &raw mut (*parser).states.top as *mut *mut libc::c_void,
                        &raw mut (*parser).states.end as *mut *mut libc::c_void,
                    );
                }
                ptr::write(
                    (*parser).states.top,
                    YamlParseFlowSequenceEntryMappingValueState,
                );
                (*parser).states.top = (*parser).states.top.wrapping_offset(1);
            };
            yaml_parser_parse_node(parser, event, false, false)
        } else {
            let mark: YamlMarkT = (*token).end_mark;
            skip_token(parser);
            (*parser).state = YamlParseFlowSequenceEntryMappingValueState;
            yaml_parser_process_empty_scalar(event, mark)
        }
    }
    unsafe fn yaml_parser_parse_flow_sequence_entry_mapping_value(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
    ) -> Success {
        let mut token: *mut YamlTokenT;
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ == YamlValueToken {
            skip_token(parser);
            token = peek_token(parser);
            if token.is_null() {
                return FAIL;
            }
            if (*token).type_ != YamlFlowEntryToken
                && (*token).type_ != YamlFlowSequenceEndToken
            {
                {
                    if (*parser).states.top == (*parser).states.end {
                        yaml_stack_extend(
                            &raw mut (*parser).states.start as *mut *mut libc::c_void,
                            &raw mut (*parser).states.top as *mut *mut libc::c_void,
                            &raw mut (*parser).states.end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::write(
                        (*parser).states.top,
                        YamlParseFlowSequenceEntryMappingEndState,
                    );
                    (*parser).states.top = (*parser).states.top.wrapping_offset(1);
                };
                return yaml_parser_parse_node(parser, event, false, false);
            }
        }
        (*parser).state = YamlParseFlowSequenceEntryMappingEndState;
        yaml_parser_process_empty_scalar(event, (*token).start_mark)
    }
    unsafe fn yaml_parser_parse_flow_sequence_entry_mapping_end(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
    ) -> Success {
        let token: *mut YamlTokenT = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        (*parser).state = YamlParseFlowSequenceEntryState;
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlMappingEndEvent;
        (*event).start_mark = (*token).start_mark;
        (*event).end_mark = (*token).start_mark;
        OK
    }
    unsafe fn yaml_parser_parse_flow_mapping_key(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
        first: bool,
    ) -> Success {
        let mut token: *mut YamlTokenT;
        if first {
            token = peek_token(parser);
            {
                if (*parser).marks.top == (*parser).marks.end {
                    yaml_stack_extend(
                        &raw mut (*parser).marks.start as *mut *mut libc::c_void,
                        &raw mut (*parser).marks.top as *mut *mut libc::c_void,
                        &raw mut (*parser).marks.end as *mut *mut libc::c_void,
                    );
                }
                ptr::write((*parser).marks.top, (*token).start_mark);
                (*parser).marks.top = (*parser).marks.top.wrapping_offset(1);
            };
            skip_token(parser);
        }
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if (*token).type_ != YamlFlowMappingEndToken {
            if !first {
                if (*token).type_ == YamlFlowEntryToken {
                    skip_token(parser);
                    token = peek_token(parser);
                    if token.is_null() {
                        return FAIL;
                    }
                } else {
                    yaml_parser_set_parser_error_context(
                        parser,
                        b"while parsing a flow mapping\0" as *const u8
                            as *const libc::c_char,
                        *{
                            (*parser).marks.top = (*parser).marks.top.offset(-1);
                            (*parser).marks.top
                        },
                        b"did not find expected ',' or '}'\0" as *const u8
                            as *const libc::c_char,
                        (*token).start_mark,
                    );
                    return FAIL;
                }
            }
            if (*token).type_ == YamlKeyToken {
                skip_token(parser);
                token = peek_token(parser);
                if token.is_null() {
                    return FAIL;
                }
                if (*token).type_ != YamlValueToken
                    && (*token).type_ != YamlFlowEntryToken
                    && (*token).type_ != YamlFlowMappingEndToken
                {
                    {
                        if (*parser).states.top == (*parser).states.end {
                            yaml_stack_extend(
                                &raw mut (*parser).states.start as *mut *mut libc::c_void,
                                &raw mut (*parser).states.top as *mut *mut libc::c_void,
                                &raw mut (*parser).states.end as *mut *mut libc::c_void,
                            );
                        }
                        ptr::write((*parser).states.top, YamlParseFlowMappingValueState);
                        (*parser).states.top = (*parser).states.top.wrapping_offset(1);
                    };
                    return yaml_parser_parse_node(parser, event, false, false);
                } else {
                    (*parser).state = YamlParseFlowMappingValueState;
                    return yaml_parser_process_empty_scalar(event, (*token).start_mark);
                }
            } else if (*token).type_ != YamlFlowMappingEndToken {
                {
                    if (*parser).states.top == (*parser).states.end {
                        yaml_stack_extend(
                            &raw mut (*parser).states.start as *mut *mut libc::c_void,
                            &raw mut (*parser).states.top as *mut *mut libc::c_void,
                            &raw mut (*parser).states.end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::write(
                        (*parser).states.top,
                        YamlParseFlowMappingEmptyValueState,
                    );
                    (*parser).states.top = (*parser).states.top.wrapping_offset(1);
                };
                return yaml_parser_parse_node(parser, event, false, false);
            }
        }
        (*parser).state = *{
            (*parser).states.top = (*parser).states.top.offset(-1);
            (*parser).states.top
        };
        let _ = *{
            (*parser).marks.top = (*parser).marks.top.offset(-1);
            (*parser).marks.top
        };
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlMappingEndEvent;
        (*event).start_mark = (*token).start_mark;
        (*event).end_mark = (*token).end_mark;
        skip_token(parser);
        OK
    }
    unsafe fn yaml_parser_parse_flow_mapping_value(
        parser: *mut YamlParserT,
        event: *mut YamlEventT,
        empty: bool,
    ) -> Success {
        let mut token: *mut YamlTokenT;
        token = peek_token(parser);
        if token.is_null() {
            return FAIL;
        }
        if empty {
            (*parser).state = YamlParseFlowMappingKeyState;
            return yaml_parser_process_empty_scalar(event, (*token).start_mark);
        }
        if (*token).type_ == YamlValueToken {
            skip_token(parser);
            token = peek_token(parser);
            if token.is_null() {
                return FAIL;
            }
            if (*token).type_ != YamlFlowEntryToken
                && (*token).type_ != YamlFlowMappingEndToken
            {
                {
                    if (*parser).states.top == (*parser).states.end {
                        yaml_stack_extend(
                            &raw mut (*parser).states.start as *mut *mut libc::c_void,
                            &raw mut (*parser).states.top as *mut *mut libc::c_void,
                            &raw mut (*parser).states.end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::write((*parser).states.top, YamlParseFlowMappingKeyState);
                    (*parser).states.top = (*parser).states.top.wrapping_offset(1);
                };
                return yaml_parser_parse_node(parser, event, false, false);
            }
        }
        (*parser).state = YamlParseFlowMappingKeyState;
        yaml_parser_process_empty_scalar(event, (*token).start_mark)
    }
    unsafe fn yaml_parser_process_empty_scalar(
        event: *mut YamlEventT,
        mark: YamlMarkT,
    ) -> Success {
        let value: *mut yaml_char_t = yaml_malloc(1_u64) as *mut yaml_char_t;
        *value = b'\0';
        let _ = memset(
            event as *mut libc::c_void,
            0,
            size_of::<YamlEventT>() as libc::c_ulong,
        );
        (*event).type_ = YamlScalarEvent;
        (*event).start_mark = mark;
        (*event).end_mark = mark;
        let fresh138 = &raw mut (*event).data.scalar.anchor;
        *fresh138 = ptr::null_mut::<yaml_char_t>();
        let fresh139 = &raw mut (*event).data.scalar.tag;
        *fresh139 = ptr::null_mut::<yaml_char_t>();
        let fresh140 = &raw mut (*event).data.scalar.value;
        *fresh140 = value;
        (*event).data.scalar.length = 0_u64;
        (*event).data.scalar.plain_implicit = true;
        (*event).data.scalar.quoted_implicit = false;
        (*event).data.scalar.style = YamlPlainScalarStyle;
        OK
    }
    unsafe fn yaml_parser_process_directives(
        parser: *mut YamlParserT,
        version_directive_ref: *mut *mut YamlVersionDirectiveT,
        tag_directives_start_ref: *mut *mut YamlTagDirectiveT,
        tag_directives_end_ref: *mut *mut YamlTagDirectiveT,
    ) -> Success {
        let mut current_block: u64;
        let mut default_tag_directives: [YamlTagDirectiveT; 3] = [
            YamlTagDirectiveT {
                handle: b"!\0" as *const u8 as *const libc::c_char as *mut yaml_char_t,
                prefix: b"!\0" as *const u8 as *const libc::c_char as *mut yaml_char_t,
            },
            YamlTagDirectiveT {
                handle: b"!!\0" as *const u8 as *const libc::c_char as *mut yaml_char_t,
                prefix: b"tag:yaml.org,2002:\0" as *const u8 as *const libc::c_char
                    as *mut yaml_char_t,
            },
            YamlTagDirectiveT {
                handle: ptr::null_mut::<yaml_char_t>(),
                prefix: ptr::null_mut::<yaml_char_t>(),
            },
        ];
        let mut default_tag_directive: *mut YamlTagDirectiveT;
        let mut version_directive: *mut YamlVersionDirectiveT = ptr::null_mut::<
            YamlVersionDirectiveT,
        >();
        struct TagDirectives {
            start: *mut YamlTagDirectiveT,
            end: *mut YamlTagDirectiveT,
            top: *mut YamlTagDirectiveT,
        }
        let mut tag_directives = TagDirectives {
            start: ptr::null_mut::<YamlTagDirectiveT>(),
            end: ptr::null_mut::<YamlTagDirectiveT>(),
            top: ptr::null_mut::<YamlTagDirectiveT>(),
        };
        let mut token: *mut YamlTokenT;
        {
            tag_directives.start = yaml_malloc(
                16 * size_of::<YamlTagDirectiveT>() as libc::c_ulong,
            ) as *mut YamlTagDirectiveT;
            tag_directives.top = tag_directives.start;
            tag_directives.end = tag_directives.start.offset(16_isize);
        };
        token = peek_token(parser);
        if !token.is_null() {
            loop {
                if !((*token).type_ == YamlVersionDirectiveToken
                    || (*token).type_ == YamlTagDirectiveToken)
                {
                    current_block = 16924917904204750491;
                    break;
                }
                if (*token).type_ == YamlVersionDirectiveToken {
                    if !version_directive.is_null() {
                        yaml_parser_set_parser_error(
                            parser,
                            b"found duplicate %YAML directive\0" as *const u8
                                as *const libc::c_char,
                            (*token).start_mark,
                        );
                        current_block = 17143798186130252483;
                        break;
                    } else if (*token).data.version_directive.major != 1
                        || (*token).data.version_directive.minor != 1
                            && (*token).data.version_directive.minor != 2
                    {
                        yaml_parser_set_parser_error(
                            parser,
                            b"found incompatible YAML document\0" as *const u8
                                as *const libc::c_char,
                            (*token).start_mark,
                        );
                        current_block = 17143798186130252483;
                        break;
                    } else {
                        version_directive = yaml_malloc(
                            size_of::<YamlVersionDirectiveT>() as libc::c_ulong,
                        ) as *mut YamlVersionDirectiveT;
                        (*version_directive).major = (*token)
                            .data
                            .version_directive
                            .major;
                        (*version_directive).minor = (*token)
                            .data
                            .version_directive
                            .minor;
                    }
                } else if (*token).type_ == YamlTagDirectiveToken {
                    let value = YamlTagDirectiveT {
                        handle: (*token).data.tag_directive.handle,
                        prefix: (*token).data.tag_directive.prefix,
                    };
                    if yaml_parser_append_tag_directive(
                            parser,
                            value,
                            false,
                            (*token).start_mark,
                        )
                        .fail
                    {
                        current_block = 17143798186130252483;
                        break;
                    }
                    {
                        if tag_directives.top == tag_directives.end {
                            yaml_stack_extend(
                                &raw mut tag_directives.start as *mut *mut libc::c_void,
                                &raw mut tag_directives.top as *mut *mut libc::c_void,
                                &raw mut tag_directives.end as *mut *mut libc::c_void,
                            );
                        }
                        ptr::write(tag_directives.top, value);
                        tag_directives.top = tag_directives.top.wrapping_offset(1);
                    };
                }
                skip_token(parser);
                token = peek_token(parser);
                if token.is_null() {
                    current_block = 17143798186130252483;
                    break;
                }
            }
            if current_block != 17143798186130252483 {
                default_tag_directive = default_tag_directives.as_mut_ptr();
                loop {
                    if (*default_tag_directive).handle.is_null() {
                        current_block = 18377268871191777778;
                        break;
                    }
                    if yaml_parser_append_tag_directive(
                            parser,
                            *default_tag_directive,
                            true,
                            (*token).start_mark,
                        )
                        .fail
                    {
                        current_block = 17143798186130252483;
                        break;
                    }
                    default_tag_directive = default_tag_directive.wrapping_offset(1);
                }
                if current_block != 17143798186130252483 {
                    if !version_directive_ref.is_null() {
                        *version_directive_ref = version_directive;
                    }
                    if !tag_directives_start_ref.is_null() {
                        if tag_directives.start == tag_directives.top {
                            *tag_directives_end_ref = ptr::null_mut::<
                                YamlTagDirectiveT,
                            >();
                            *tag_directives_start_ref = *tag_directives_end_ref;
                            yaml_free(tag_directives.start as *mut libc::c_void);
                            tag_directives.end = ptr::null_mut();
                            tag_directives.top = ptr::null_mut();
                            tag_directives.start = ptr::null_mut();
                        } else {
                            *tag_directives_start_ref = tag_directives.start;
                            *tag_directives_end_ref = tag_directives.top;
                        }
                    } else {
                        yaml_free(tag_directives.start as *mut libc::c_void);
                        tag_directives.end = ptr::null_mut();
                        tag_directives.top = ptr::null_mut();
                        tag_directives.start = ptr::null_mut();
                    }
                    if version_directive_ref.is_null() {
                        yaml_free(version_directive as *mut libc::c_void);
                    }
                    return OK;
                }
            }
        }
        yaml_free(version_directive as *mut libc::c_void);
        while !(tag_directives.start == tag_directives.top) {
            let tag_directive = *{
                tag_directives.top = tag_directives.top.offset(-1);
                tag_directives.top
            };
            yaml_free(tag_directive.handle as *mut libc::c_void);
            yaml_free(tag_directive.prefix as *mut libc::c_void);
        }
        yaml_free(tag_directives.start as *mut libc::c_void);
        tag_directives.end = ptr::null_mut();
        tag_directives.top = ptr::null_mut();
        tag_directives.start = ptr::null_mut();
        FAIL
    }
    unsafe fn yaml_parser_append_tag_directive(
        parser: *mut YamlParserT,
        value: YamlTagDirectiveT,
        allow_duplicates: bool,
        mark: YamlMarkT,
    ) -> Success {
        let mut tag_directive: *mut YamlTagDirectiveT;
        let mut copy = YamlTagDirectiveT {
            handle: ptr::null_mut::<yaml_char_t>(),
            prefix: ptr::null_mut::<yaml_char_t>(),
        };
        tag_directive = (*parser).tag_directives.start;
        while tag_directive != (*parser).tag_directives.top {
            if strcmp(
                value.handle as *mut libc::c_char,
                (*tag_directive).handle as *mut libc::c_char,
            ) == 0
            {
                if allow_duplicates {
                    return OK;
                }
                yaml_parser_set_parser_error(
                    parser,
                    b"found duplicate %TAG directive\0" as *const u8
                        as *const libc::c_char,
                    mark,
                );
                return FAIL;
            }
            tag_directive = tag_directive.wrapping_offset(1);
        }
        copy.handle = yaml_strdup(value.handle);
        copy.prefix = yaml_strdup(value.prefix);
        {
            if (*parser).tag_directives.top == (*parser).tag_directives.end {
                yaml_stack_extend(
                    &raw mut (*parser).tag_directives.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tag_directives.top as *mut *mut libc::c_void,
                    &raw mut (*parser).tag_directives.end as *mut *mut libc::c_void,
                );
            }
            ptr::write((*parser).tag_directives.top, copy);
            (*parser).tag_directives.top = (*parser)
                .tag_directives
                .top
                .wrapping_offset(1);
        };
        OK
    }
}
mod reader {
    use crate::externs::{memcmp, memmove};
    use crate::ops::ForceAdd as _;
    use crate::success::{Success, FAIL, OK};
    use crate::yaml::{size_t, yaml_char_t};
    use crate::{
        libc, PointerExt, YamlAnyEncoding, YamlParserT, YamlReaderError,
        YamlUtf16beEncoding, YamlUtf16leEncoding, YamlUtf8Encoding,
    };
    use core::ptr::addr_of_mut;
    unsafe fn yaml_parser_set_reader_error(
        parser: *mut YamlParserT,
        problem: *const libc::c_char,
        offset: size_t,
        value: libc::c_int,
    ) -> Success {
        (*parser).error = YamlReaderError;
        let fresh0 = &raw mut (*parser).problem;
        *fresh0 = problem;
        (*parser).problem_offset = offset;
        (*parser).problem_value = value;
        FAIL
    }
    const BOM_UTF8: *const libc::c_char = b"\xEF\xBB\xBF\0" as *const u8
        as *const libc::c_char;
    const BOM_UTF16LE: *const libc::c_char = b"\xFF\xFE\0" as *const u8
        as *const libc::c_char;
    const BOM_UTF16BE: *const libc::c_char = b"\xFE\xFF\0" as *const u8
        as *const libc::c_char;
    unsafe fn yaml_parser_determine_encoding(parser: *mut YamlParserT) -> Success {
        while !(*parser).eof
            && ((*parser).raw_buffer.last.c_offset_from((*parser).raw_buffer.pointer)
                as libc::c_long) < 3_i64
        {
            if yaml_parser_update_raw_buffer(parser).fail {
                return FAIL;
            }
        }
        if (*parser).raw_buffer.last.c_offset_from((*parser).raw_buffer.pointer)
            as libc::c_long >= 2_i64
            && memcmp(
                (*parser).raw_buffer.pointer as *const libc::c_void,
                BOM_UTF16LE as *const libc::c_void,
                2_u64,
            ) == 0
        {
            (*parser).encoding = YamlUtf16leEncoding;
            let fresh1 = &raw mut (*parser).raw_buffer.pointer;
            *fresh1 = (*fresh1).wrapping_offset(2_isize);
            let fresh2 = &raw mut (*parser).offset;
            *fresh2 = (*fresh2).force_add(3_u64);
        } else if (*parser).raw_buffer.last.c_offset_from((*parser).raw_buffer.pointer)
            as libc::c_long >= 2_i64
            && memcmp(
                (*parser).raw_buffer.pointer as *const libc::c_void,
                BOM_UTF16BE as *const libc::c_void,
                2_u64,
            ) == 0
        {
            (*parser).encoding = YamlUtf16beEncoding;
            let fresh3 = &raw mut (*parser).raw_buffer.pointer;
            *fresh3 = (*fresh3).wrapping_offset(2_isize);
            let fresh4 = &raw mut (*parser).offset;
            *fresh4 = (*fresh4).force_add(3_u64);
        } else if (*parser).raw_buffer.last.c_offset_from((*parser).raw_buffer.pointer)
            as libc::c_long >= 3_i64
            && memcmp(
                (*parser).raw_buffer.pointer as *const libc::c_void,
                BOM_UTF8 as *const libc::c_void,
                3_u64,
            ) == 0
        {
            (*parser).encoding = YamlUtf8Encoding;
            let fresh5 = &raw mut (*parser).raw_buffer.pointer;
            *fresh5 = (*fresh5).wrapping_offset(3_isize);
            let fresh6 = &raw mut (*parser).offset;
            *fresh6 = (*fresh6).force_add(3_u64);
        } else {
            (*parser).encoding = YamlUtf8Encoding;
        }
        OK
    }
    unsafe fn yaml_parser_update_raw_buffer(parser: *mut YamlParserT) -> Success {
        let mut size_read: size_t = 0_u64;
        if (*parser).raw_buffer.start == (*parser).raw_buffer.pointer
            && (*parser).raw_buffer.last == (*parser).raw_buffer.end
        {
            return OK;
        }
        if (*parser).eof {
            return OK;
        }
        if (*parser).raw_buffer.start < (*parser).raw_buffer.pointer
            && (*parser).raw_buffer.pointer < (*parser).raw_buffer.last
        {
            let _ = memmove(
                (*parser).raw_buffer.start as *mut libc::c_void,
                (*parser).raw_buffer.pointer as *const libc::c_void,
                (*parser).raw_buffer.last.c_offset_from((*parser).raw_buffer.pointer)
                    as libc::c_long as libc::c_ulong,
            );
        }
        let fresh7 = &raw mut (*parser).raw_buffer.last;
        *fresh7 = (*fresh7)
            .wrapping_offset(
                -((*parser).raw_buffer.pointer.c_offset_from((*parser).raw_buffer.start)
                    as libc::c_long as isize),
            );
        let fresh8 = &raw mut (*parser).raw_buffer.pointer;
        *fresh8 = (*parser).raw_buffer.start;
        if (*parser)
            .read_handler
            .expect(
                "non-null function pointer",
            )(
            (*parser).read_handler_data,
            (*parser).raw_buffer.last,
            (*parser).raw_buffer.end.c_offset_from((*parser).raw_buffer.last) as size_t,
            &raw mut size_read,
        ) == 0
        {
            return yaml_parser_set_reader_error(
                parser,
                b"input error\0" as *const u8 as *const libc::c_char,
                (*parser).offset,
                -1,
            );
        }
        let fresh9 = &raw mut (*parser).raw_buffer.last;
        *fresh9 = (*fresh9).wrapping_offset(size_read as isize);
        if size_read == 0 {
            (*parser).eof = true;
        }
        OK
    }
    pub(crate) unsafe fn yaml_parser_update_buffer(
        parser: *mut YamlParserT,
        length: size_t,
    ) -> Success {
        let mut first = true;
        if !((*parser).read_handler).is_some() {
            crate::externs::__assert_fail(
                "((*parser).read_handler).is_some()",
                "src/reader.rs",
                169u32,
            );
        }
        if (*parser).eof && (*parser).raw_buffer.pointer == (*parser).raw_buffer.last {
            return OK;
        }
        if (*parser).unread >= length {
            return OK;
        }
        if (*parser).encoding == YamlAnyEncoding
            && yaml_parser_determine_encoding(parser).fail
        {
            return FAIL;
        }
        if (*parser).buffer.start < (*parser).buffer.pointer
            && (*parser).buffer.pointer < (*parser).buffer.last
        {
            let size: size_t = (*parser)
                .buffer
                .last
                .c_offset_from((*parser).buffer.pointer) as size_t;
            let _ = memmove(
                (*parser).buffer.start as *mut libc::c_void,
                (*parser).buffer.pointer as *const libc::c_void,
                size,
            );
            let fresh10 = &raw mut (*parser).buffer.pointer;
            *fresh10 = (*parser).buffer.start;
            let fresh11 = &raw mut (*parser).buffer.last;
            *fresh11 = (*parser).buffer.start.wrapping_offset(size as isize);
        } else if (*parser).buffer.pointer == (*parser).buffer.last {
            let fresh12 = &raw mut (*parser).buffer.pointer;
            *fresh12 = (*parser).buffer.start;
            let fresh13 = &raw mut (*parser).buffer.last;
            *fresh13 = (*parser).buffer.start;
        }
        while (*parser).unread < length {
            if (!first || (*parser).raw_buffer.pointer == (*parser).raw_buffer.last)
                && yaml_parser_update_raw_buffer(parser).fail
            {
                return FAIL;
            }
            first = false;
            while (*parser).raw_buffer.pointer != (*parser).raw_buffer.last {
                let mut value: libc::c_uint = 0;
                let value2: libc::c_uint;
                let mut incomplete = false;
                let mut octet: libc::c_uchar;
                let mut width: libc::c_uint = 0;
                let low: libc::c_int;
                let high: libc::c_int;
                let mut k: size_t;
                let raw_unread: size_t = (*parser)
                    .raw_buffer
                    .last
                    .c_offset_from((*parser).raw_buffer.pointer) as size_t;
                match (*parser).encoding {
                    YamlUtf8Encoding => {
                        octet = *(*parser).raw_buffer.pointer;
                        width = if octet & 0x80 == 0 {
                            1
                        } else if octet & 0xE0 == 0xC0 {
                            2
                        } else if octet & 0xF0 == 0xE0 {
                            3
                        } else if octet & 0xF8 == 0xF0 {
                            4
                        } else {
                            0
                        } as libc::c_uint;
                        if width == 0 {
                            return yaml_parser_set_reader_error(
                                parser,
                                b"invalid leading UTF-8 octet\0" as *const u8
                                    as *const libc::c_char,
                                (*parser).offset,
                                octet as libc::c_int,
                            );
                        }
                        if width as libc::c_ulong > raw_unread {
                            if (*parser).eof {
                                return yaml_parser_set_reader_error(
                                    parser,
                                    b"incomplete UTF-8 octet sequence\0" as *const u8
                                        as *const libc::c_char,
                                    (*parser).offset,
                                    -1,
                                );
                            }
                            incomplete = true;
                        } else {
                            value = if octet & 0x80 == 0 {
                                octet & 0x7F
                            } else if octet & 0xE0 == 0xC0 {
                                octet & 0x1F
                            } else if octet & 0xF0 == 0xE0 {
                                octet & 0xF
                            } else if octet & 0xF8 == 0xF0 {
                                octet & 0x7
                            } else {
                                0
                            } as libc::c_uint;
                            k = 1_u64;
                            while k < width as libc::c_ulong {
                                octet = *(*parser)
                                    .raw_buffer
                                    .pointer
                                    .wrapping_offset(k as isize);
                                if octet & 0xC0 != 0x80 {
                                    return yaml_parser_set_reader_error(
                                        parser,
                                        b"invalid trailing UTF-8 octet\0" as *const u8
                                            as *const libc::c_char,
                                        (*parser).offset.force_add(k),
                                        octet as libc::c_int,
                                    );
                                }
                                value = (value << 6)
                                    .force_add((octet & 0x3F) as libc::c_uint);
                                k = k.force_add(1);
                            }
                            if !(width == 1 || width == 2 && value >= 0x80
                                || width == 3 && value >= 0x800
                                || width == 4 && value >= 0x10000)
                            {
                                return yaml_parser_set_reader_error(
                                    parser,
                                    b"invalid length of a UTF-8 sequence\0" as *const u8
                                        as *const libc::c_char,
                                    (*parser).offset,
                                    -1,
                                );
                            }
                            if (0xD800..=0xDFFF).contains(&value) || value > 0x10FFFF {
                                return yaml_parser_set_reader_error(
                                    parser,
                                    b"invalid Unicode character\0" as *const u8
                                        as *const libc::c_char,
                                    (*parser).offset,
                                    value as libc::c_int,
                                );
                            }
                        }
                    }
                    YamlUtf16leEncoding | YamlUtf16beEncoding => {
                        low = if (*parser).encoding == YamlUtf16leEncoding {
                            0
                        } else {
                            1
                        };
                        high = if (*parser).encoding == YamlUtf16leEncoding {
                            1
                        } else {
                            0
                        };
                        if raw_unread < 2_u64 {
                            if (*parser).eof {
                                return yaml_parser_set_reader_error(
                                    parser,
                                    b"incomplete UTF-16 character\0" as *const u8
                                        as *const libc::c_char,
                                    (*parser).offset,
                                    -1,
                                );
                            }
                            incomplete = true;
                        } else {
                            value = (*(*parser)
                                .raw_buffer
                                .pointer
                                .wrapping_offset(low as isize) as libc::c_int
                                + ((*(*parser)
                                    .raw_buffer
                                    .pointer
                                    .wrapping_offset(high as isize) as libc::c_int) << 8))
                                as libc::c_uint;
                            if value & 0xFC00 == 0xDC00 {
                                return yaml_parser_set_reader_error(
                                    parser,
                                    b"unexpected low surrogate area\0" as *const u8
                                        as *const libc::c_char,
                                    (*parser).offset,
                                    value as libc::c_int,
                                );
                            }
                            if value & 0xFC00 == 0xD800 {
                                width = 4;
                                if raw_unread < 4_u64 {
                                    if (*parser).eof {
                                        return yaml_parser_set_reader_error(
                                            parser,
                                            b"incomplete UTF-16 surrogate pair\0" as *const u8
                                                as *const libc::c_char,
                                            (*parser).offset,
                                            -1,
                                        );
                                    }
                                    incomplete = true;
                                } else {
                                    value2 = (*(*parser)
                                        .raw_buffer
                                        .pointer
                                        .wrapping_offset((low + 2) as isize) as libc::c_int
                                        + ((*(*parser)
                                            .raw_buffer
                                            .pointer
                                            .wrapping_offset((high + 2) as isize) as libc::c_int) << 8))
                                        as libc::c_uint;
                                    if value2 & 0xFC00 != 0xDC00 {
                                        return yaml_parser_set_reader_error(
                                            parser,
                                            b"expected low surrogate area\0" as *const u8
                                                as *const libc::c_char,
                                            (*parser).offset.force_add(2_u64),
                                            value2 as libc::c_int,
                                        );
                                    }
                                    value = 0x10000_u32
                                        .force_add((value & 0x3FF) << 10)
                                        .force_add(value2 & 0x3FF);
                                }
                            } else {
                                width = 2;
                            }
                        }
                    }
                    _ => {}
                }
                if incomplete {
                    break;
                }
                if !(value == 0x9 || value == 0xA || value == 0xD
                    || (0x20..=0x7E).contains(&value) || value == 0x85
                    || (0xA0..=0xD7FF).contains(&value)
                    || (0xE000..=0xFFFD).contains(&value)
                    || (0x10000..=0x10FFFF).contains(&value))
                {
                    return yaml_parser_set_reader_error(
                        parser,
                        b"control characters are not allowed\0" as *const u8
                            as *const libc::c_char,
                        (*parser).offset,
                        value as libc::c_int,
                    );
                }
                let fresh14 = &raw mut (*parser).raw_buffer.pointer;
                *fresh14 = (*fresh14).wrapping_offset(width as isize);
                let fresh15 = &raw mut (*parser).offset;
                *fresh15 = (*fresh15).force_add(width as size_t);
                if value <= 0x7F {
                    let fresh16 = &raw mut (*parser).buffer.last;
                    let fresh17 = *fresh16;
                    *fresh16 = (*fresh16).wrapping_offset(1);
                    *fresh17 = value as yaml_char_t;
                } else if value <= 0x7FF {
                    let fresh18 = &raw mut (*parser).buffer.last;
                    let fresh19 = *fresh18;
                    *fresh18 = (*fresh18).wrapping_offset(1);
                    *fresh19 = 0xC0_u32.force_add(value >> 6) as yaml_char_t;
                    let fresh20 = &raw mut (*parser).buffer.last;
                    let fresh21 = *fresh20;
                    *fresh20 = (*fresh20).wrapping_offset(1);
                    *fresh21 = 0x80_u32.force_add(value & 0x3F) as yaml_char_t;
                } else if value <= 0xFFFF {
                    let fresh22 = &raw mut (*parser).buffer.last;
                    let fresh23 = *fresh22;
                    *fresh22 = (*fresh22).wrapping_offset(1);
                    *fresh23 = 0xE0_u32.force_add(value >> 12) as yaml_char_t;
                    let fresh24 = &raw mut (*parser).buffer.last;
                    let fresh25 = *fresh24;
                    *fresh24 = (*fresh24).wrapping_offset(1);
                    *fresh25 = 0x80_u32.force_add(value >> 6 & 0x3F) as yaml_char_t;
                    let fresh26 = &raw mut (*parser).buffer.last;
                    let fresh27 = *fresh26;
                    *fresh26 = (*fresh26).wrapping_offset(1);
                    *fresh27 = 0x80_u32.force_add(value & 0x3F) as yaml_char_t;
                } else {
                    let fresh28 = &raw mut (*parser).buffer.last;
                    let fresh29 = *fresh28;
                    *fresh28 = (*fresh28).wrapping_offset(1);
                    *fresh29 = 0xF0_u32.force_add(value >> 18) as yaml_char_t;
                    let fresh30 = &raw mut (*parser).buffer.last;
                    let fresh31 = *fresh30;
                    *fresh30 = (*fresh30).wrapping_offset(1);
                    *fresh31 = 0x80_u32.force_add(value >> 12 & 0x3F) as yaml_char_t;
                    let fresh32 = &raw mut (*parser).buffer.last;
                    let fresh33 = *fresh32;
                    *fresh32 = (*fresh32).wrapping_offset(1);
                    *fresh33 = 0x80_u32.force_add(value >> 6 & 0x3F) as yaml_char_t;
                    let fresh34 = &raw mut (*parser).buffer.last;
                    let fresh35 = *fresh34;
                    *fresh34 = (*fresh34).wrapping_offset(1);
                    *fresh35 = 0x80_u32.force_add(value & 0x3F) as yaml_char_t;
                }
                let fresh36 = &raw mut (*parser).unread;
                *fresh36 = (*fresh36).force_add(1);
            }
            if (*parser).eof {
                let fresh37 = &raw mut (*parser).buffer.last;
                let fresh38 = *fresh37;
                *fresh37 = (*fresh37).wrapping_offset(1);
                *fresh38 = b'\0';
                let fresh39 = &raw mut (*parser).unread;
                *fresh39 = (*fresh39).force_add(1);
                return OK;
            }
        }
        if (*parser).offset >= (!0_u64).wrapping_div(2_u64) {
            return yaml_parser_set_reader_error(
                parser,
                b"input is too long\0" as *const u8 as *const libc::c_char,
                (*parser).offset,
                -1,
            );
        }
        OK
    }
}
mod scanner {
    use crate::externs::{memcpy, memmove, memset, strcmp, strlen};
    use crate::internal::{yaml_queue_extend, yaml_stack_extend};
    use crate::memory::yaml_free;
    use crate::memory::yaml_malloc;
    use crate::ops::{ForceAdd as _, ForceMul as _};
    use crate::reader::yaml_parser_update_buffer;
    use crate::string::{yaml_string_extend, yaml_string_join};
    use crate::success::{Success, FAIL, OK};
    use crate::yaml::{ptrdiff_t, size_t, yaml_char_t, YamlStringT, NULL_STRING};
    use crate::{
        libc, PointerExt, YamlAliasToken, YamlAnchorToken, YamlBlockEndToken,
        YamlBlockEntryToken, YamlBlockMappingStartToken, YamlBlockSequenceStartToken,
        YamlDocumentEndToken, YamlDocumentStartToken, YamlDoubleQuotedScalarStyle,
        YamlFlowEntryToken, YamlFlowMappingEndToken, YamlFlowMappingStartToken,
        YamlFlowSequenceEndToken, YamlFlowSequenceStartToken, YamlFoldedScalarStyle,
        YamlKeyToken, YamlLiteralScalarStyle, YamlMarkT, YamlMemoryError, YamlNoError,
        YamlParserT, YamlPlainScalarStyle, YamlScalarToken, YamlScannerError,
        YamlSimpleKeyT, YamlSingleQuotedScalarStyle, YamlStreamEndToken,
        YamlStreamStartToken, YamlTagDirectiveToken, YamlTagToken, YamlTokenT,
        YamlTokenTypeT, YamlValueToken, YamlVersionDirectiveToken,
    };
    use core::mem::{size_of, MaybeUninit};
    use core::ptr::{self, addr_of_mut};
    unsafe fn cache(parser: *mut YamlParserT, length: size_t) -> Success {
        if (*parser).unread >= length {
            OK
        } else {
            yaml_parser_update_buffer(parser, length)
        }
    }
    unsafe fn skip(parser: *mut YamlParserT) {
        let width = if *(*parser).buffer.pointer.wrapping_offset(0) & 0x80 == 0x00 {
            1
        } else if *(*parser).buffer.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
            2
        } else if *(*parser).buffer.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
            3
        } else if *(*parser).buffer.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
            4
        } else {
            0
        };
        (*parser).mark.index = (*parser).mark.index.force_add(width as u64);
        (*parser).mark.column = (*parser).mark.column.force_add(1);
        (*parser).unread = (*parser).unread.wrapping_sub(1);
        (*parser).buffer.pointer = (*parser)
            .buffer
            .pointer
            .wrapping_offset(width as isize);
    }
    unsafe fn skip_line(parser: *mut YamlParserT) {
        if *(*parser).buffer.pointer.offset(0) == b'\r'
            && *(*parser).buffer.pointer.offset(1) == b'\n'
        {
            (*parser).mark.index = (*parser).mark.index.force_add(2);
            (*parser).mark.column = 0;
            (*parser).mark.line = (*parser).mark.line.force_add(1);
            (*parser).unread = (*parser).unread.wrapping_sub(2);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(2);
        } else if *(*parser).buffer.pointer.offset(0) == b'\r'
            || *(*parser).buffer.pointer.offset(0) == b'\n'
            || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                    == b'\x85'
            || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                    == b'\x80'
                && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                    == b'\xA8'
            || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                    == b'\x80'
                && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                    == b'\xA9'
        {
            let width = if *(*parser).buffer.pointer.wrapping_offset(0) & 0x80 == 0x00 {
                1
            } else if *(*parser).buffer.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
                2
            } else if *(*parser).buffer.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
                3
            } else if *(*parser).buffer.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
                4
            } else {
                0
            };
            (*parser).mark.index = (*parser).mark.index.force_add(width as u64);
            (*parser).mark.column = 0;
            (*parser).mark.line = (*parser).mark.line.force_add(1);
            (*parser).unread = (*parser).unread.wrapping_sub(1);
            (*parser).buffer.pointer = (*parser)
                .buffer
                .pointer
                .wrapping_offset(width as isize);
        }
    }
    unsafe fn read(parser: *mut YamlParserT, string: *mut YamlStringT) {
        let new_end = (*string).pointer.wrapping_add(5);
        if new_end >= (*string).end {
            yaml_string_extend(
                &raw mut (*string).start,
                &raw mut (*string).pointer,
                &raw mut (*string).end,
            );
        }
        let width = if *(*parser).buffer.pointer.wrapping_offset(0) & 0x80 == 0x00 {
            1
        } else if *(*parser).buffer.pointer.wrapping_offset(0) & 0xE0 == 0xC0 {
            2
        } else if *(*parser).buffer.pointer.wrapping_offset(0) & 0xF0 == 0xE0 {
            3
        } else if *(*parser).buffer.pointer.wrapping_offset(0) & 0xF8 == 0xF0 {
            4
        } else {
            0
        };
        if *(*parser).buffer.pointer & 0x80 == 0x00 {
            *(*string).pointer = *(*parser).buffer.pointer;
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(1);
        } else if *(*parser).buffer.pointer & 0xE0 == 0xC0 {
            *(*string).pointer = *(*parser).buffer.pointer;
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(1);
            *(*string).pointer = *(*parser).buffer.pointer;
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(1);
        } else if *(*parser).buffer.pointer & 0xF0 == 0xE0 {
            *(*string).pointer = *(*parser).buffer.pointer;
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(1);
            *(*string).pointer = *(*parser).buffer.pointer;
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(1);
            *(*string).pointer = *(*parser).buffer.pointer;
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(1);
        } else if *(*parser).buffer.pointer & 0xF8 == 0xF0 {
            *(*string).pointer = *(*parser).buffer.pointer;
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(1);
            *(*string).pointer = *(*parser).buffer.pointer;
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(1);
            *(*string).pointer = *(*parser).buffer.pointer;
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(1);
            *(*string).pointer = *(*parser).buffer.pointer;
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(1);
        }
        (*parser).mark.index = (*parser).mark.index.force_add(width as u64);
        (*parser).mark.column = (*parser).mark.column.force_add(1);
        (*parser).unread = (*parser).unread.wrapping_sub(1);
    }
    unsafe fn read_line(parser: *mut YamlParserT, string: *mut YamlStringT) {
        let new_end = (*string).pointer.wrapping_add(5);
        if new_end >= (*string).end {
            yaml_string_extend(
                &raw mut (*string).start,
                &raw mut (*string).pointer,
                &raw mut (*string).end,
            );
        }
        if *(*parser).buffer.pointer.offset(0) == b'\r'
            && *(*parser).buffer.pointer.offset(1) == b'\n'
        {
            *(*string).pointer = b'\n';
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(2);
            (*parser).mark.index = (*parser).mark.index.force_add(2);
            (*parser).mark.column = 0;
            (*parser).mark.line = (*parser).mark.line.force_add(1);
            (*parser).unread = (*parser).unread.wrapping_sub(2);
        } else if *(*parser).buffer.pointer.offset(0) == b'\r'
            || *(*parser).buffer.pointer.offset(0) == b'\n'
        {
            *(*string).pointer = b'\n';
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(1);
            (*parser).mark.index = (*parser).mark.index.force_add(1);
            (*parser).mark.column = 0;
            (*parser).mark.line = (*parser).mark.line.force_add(1);
            (*parser).unread = (*parser).unread.wrapping_sub(1);
        } else if *(*parser).buffer.pointer.offset(0) == b'\xC2'
            && *(*parser).buffer.pointer.offset(1) == b'\x85'
        {
            *(*string).pointer = b'\n';
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(2);
            (*parser).mark.index = (*parser).mark.index.force_add(2);
            (*parser).mark.column = 0;
            (*parser).mark.line = (*parser).mark.line.force_add(1);
            (*parser).unread = (*parser).unread.wrapping_sub(1);
        } else if *(*parser).buffer.pointer.offset(0) == b'\xE2'
            && *(*parser).buffer.pointer.offset(1) == b'\x80'
            && (*(*parser).buffer.pointer.offset(2) == b'\xA8'
                || *(*parser).buffer.pointer.offset(2) == b'\xA9')
        {
            *(*string).pointer = *(*parser).buffer.pointer;
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(1);
            *(*string).pointer = *(*parser).buffer.pointer;
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(1);
            *(*string).pointer = *(*parser).buffer.pointer;
            (*string).pointer = (*string).pointer.wrapping_offset(1);
            (*parser).buffer.pointer = (*parser).buffer.pointer.wrapping_offset(1);
            (*parser).mark.index = (*parser).mark.index.force_add(3);
            (*parser).mark.column = 0;
            (*parser).mark.line = (*parser).mark.line.force_add(1);
            (*parser).unread = (*parser).unread.wrapping_sub(1);
        }
    }
    /// Scan the input stream and produce the next token.
    ///
    /// Call the function subsequently to produce a sequence of tokens corresponding
    /// to the input stream. The initial token has the type YamlStreamStartToken
    /// while the ending token has the type YamlStreamEndToken.
    ///
    /// An application is responsible for freeing any buffers associated with the
    /// produced token object using the yaml_token_delete function.
    ///
    /// An application must not alternate the calls of yaml_parser_scan() with the
    /// calls of yaml_parser_parse() or yaml_parser_load(). Doing this will break
    /// the parser.
    ///
    /// # Safety
    ///
    /// - The `parser` and `token` pointers must be valid and non-null.
    /// - The `parser` must be properly initialized and not already in an error state.
    /// - The `token` must be properly allocated and have enough capacity to store the
    ///   produced token.
    /// - The function should not be called alternately with `yaml_parser_parse()` or
    ///   `yaml_parser_load()`, as it may break the parser state.
    /// - The caller is responsible for freeing any buffers associated with the produced
    ///   token using the `yaml_token_delete` function.
    ///
    pub unsafe fn yaml_parser_scan(
        parser: *mut YamlParserT,
        token: *mut YamlTokenT,
    ) -> Success {
        if !!parser.is_null() {
            crate::externs::__assert_fail("!parser.is_null()", "src/scanner.rs", 178u32);
        }
        if !!token.is_null() {
            crate::externs::__assert_fail("!token.is_null()", "src/scanner.rs", 179u32);
        }
        let _ = memset(
            token as *mut libc::c_void,
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
        if (*parser).stream_end_produced || (*parser).error != YamlNoError {
            return OK;
        }
        if !(*parser).token_available && yaml_parser_fetch_more_tokens(parser).fail {
            return FAIL;
        }
        *token = *{
            let head = (*parser).tokens.head;
            (*parser).tokens.head = (*parser).tokens.head.wrapping_offset(1);
            head
        };
        (*parser).token_available = false;
        let fresh2 = &raw mut (*parser).tokens_parsed;
        *fresh2 = (*fresh2).force_add(1);
        if (*token).type_ == YamlStreamEndToken {
            (*parser).stream_end_produced = true;
        }
        OK
    }
    unsafe fn yaml_parser_set_scanner_error(
        parser: *mut YamlParserT,
        context: *const libc::c_char,
        context_mark: YamlMarkT,
        problem: *const libc::c_char,
    ) {
        (*parser).error = YamlScannerError;
        let fresh3 = &raw mut (*parser).context;
        *fresh3 = context;
        (*parser).context_mark = context_mark;
        let fresh4 = &raw mut (*parser).problem;
        *fresh4 = problem;
        (*parser).problem_mark = (*parser).mark;
    }
    pub(crate) unsafe fn yaml_parser_fetch_more_tokens(
        parser: *mut YamlParserT,
    ) -> Success {
        let mut need_more_tokens;
        loop {
            need_more_tokens = false;
            if (*parser).tokens.head == (*parser).tokens.tail {
                need_more_tokens = true;
            } else {
                let mut simple_key: *mut YamlSimpleKeyT;
                if yaml_parser_stale_simple_keys(parser).fail {
                    return FAIL;
                }
                simple_key = (*parser)
                    .simple_keys
                    .start
                    .add((*parser).not_simple_keys as usize);
                while simple_key != (*parser).simple_keys.top {
                    if (*simple_key).possible
                        && (*simple_key).token_number == (*parser).tokens_parsed
                    {
                        need_more_tokens = true;
                        break;
                    } else {
                        simple_key = simple_key.wrapping_offset(1);
                    }
                }
            }
            if !need_more_tokens {
                break;
            }
            if yaml_parser_fetch_next_token(parser).fail {
                return FAIL;
            }
        }
        (*parser).token_available = true;
        OK
    }
    unsafe fn yaml_parser_fetch_next_token(parser: *mut YamlParserT) -> Success {
        if cache(parser, 1_u64).fail {
            return FAIL;
        }
        if !(*parser).stream_start_produced {
            yaml_parser_fetch_stream_start(parser);
            return OK;
        }
        if yaml_parser_scan_to_next_token(parser).fail {
            return FAIL;
        }
        if yaml_parser_stale_simple_keys(parser).fail {
            return FAIL;
        }
        yaml_parser_unroll_indent(parser, (*parser).mark.column as ptrdiff_t);
        if cache(parser, 4_u64).fail {
            return FAIL;
        }
        if *(*parser).buffer.pointer.offset(0) == b'\0' {
            return yaml_parser_fetch_stream_end(parser);
        }
        if (*parser).mark.column == 0_u64 && *(*parser).buffer.pointer == b'%' {
            return yaml_parser_fetch_directive(parser);
        }
        if (*parser).mark.column == 0_u64 && *(*parser).buffer.pointer.offset(0) == b'-'
            && *(*parser).buffer.pointer.offset(1) == b'-'
            && *(*parser).buffer.pointer.offset(2) == b'-'
            && (*(*parser).buffer.pointer.offset(3) == b' '
                || *(*parser).buffer.pointer.offset(3) == b'\t'
                || (*(*parser).buffer.pointer.offset(3) == b'\r'
                    || *(*parser).buffer.pointer.offset(3) == b'\n'
                    || *(*parser).buffer.pointer.offset(3) == b'\xC2'
                        && *(*parser).buffer.pointer.offset((3 + 1).try_into().unwrap())
                            == b'\x85'
                    || *(*parser).buffer.pointer.offset(3) == b'\xE2'
                        && *(*parser).buffer.pointer.offset((3 + 1).try_into().unwrap())
                            == b'\x80'
                        && *(*parser).buffer.pointer.offset((3 + 2).try_into().unwrap())
                            == b'\xA8'
                    || *(*parser).buffer.pointer.offset(3) == b'\xE2'
                        && *(*parser).buffer.pointer.offset((3 + 1).try_into().unwrap())
                            == b'\x80'
                        && *(*parser).buffer.pointer.offset((3 + 2).try_into().unwrap())
                            == b'\xA9' || *(*parser).buffer.pointer.offset(3) == b'\0'))
        {
            return yaml_parser_fetch_document_indicator(parser, YamlDocumentStartToken);
        }
        if (*parser).mark.column == 0_u64 && *(*parser).buffer.pointer.offset(0) == b'.'
            && *(*parser).buffer.pointer.offset(1) == b'.'
            && *(*parser).buffer.pointer.offset(2) == b'.'
            && (*(*parser).buffer.pointer.offset(3) == b' '
                || *(*parser).buffer.pointer.offset(3) == b'\t'
                || (*(*parser).buffer.pointer.offset(3) == b'\r'
                    || *(*parser).buffer.pointer.offset(3) == b'\n'
                    || *(*parser).buffer.pointer.offset(3) == b'\xC2'
                        && *(*parser).buffer.pointer.offset((3 + 1).try_into().unwrap())
                            == b'\x85'
                    || *(*parser).buffer.pointer.offset(3) == b'\xE2'
                        && *(*parser).buffer.pointer.offset((3 + 1).try_into().unwrap())
                            == b'\x80'
                        && *(*parser).buffer.pointer.offset((3 + 2).try_into().unwrap())
                            == b'\xA8'
                    || *(*parser).buffer.pointer.offset(3) == b'\xE2'
                        && *(*parser).buffer.pointer.offset((3 + 1).try_into().unwrap())
                            == b'\x80'
                        && *(*parser).buffer.pointer.offset((3 + 2).try_into().unwrap())
                            == b'\xA9' || *(*parser).buffer.pointer.offset(3) == b'\0'))
        {
            return yaml_parser_fetch_document_indicator(parser, YamlDocumentEndToken);
        }
        if *(*parser).buffer.pointer == b'[' {
            return yaml_parser_fetch_flow_collection_start(
                parser,
                YamlFlowSequenceStartToken,
            );
        }
        if *(*parser).buffer.pointer == b'{' {
            return yaml_parser_fetch_flow_collection_start(
                parser,
                YamlFlowMappingStartToken,
            );
        }
        if *(*parser).buffer.pointer == b']' {
            return yaml_parser_fetch_flow_collection_end(
                parser,
                YamlFlowSequenceEndToken,
            );
        }
        if *(*parser).buffer.pointer == b'}' {
            return yaml_parser_fetch_flow_collection_end(
                parser,
                YamlFlowMappingEndToken,
            );
        }
        if *(*parser).buffer.pointer == b',' {
            return yaml_parser_fetch_flow_entry(parser);
        }
        if *(*parser).buffer.pointer == b'-'
            && (*(*parser).buffer.pointer.offset(1) == b' '
                || *(*parser).buffer.pointer.offset(1) == b'\t'
                || (*(*parser).buffer.pointer.offset(1) == b'\r'
                    || *(*parser).buffer.pointer.offset(1) == b'\n'
                    || *(*parser).buffer.pointer.offset(1) == b'\xC2'
                        && *(*parser).buffer.pointer.offset((1 + 1).try_into().unwrap())
                            == b'\x85'
                    || *(*parser).buffer.pointer.offset(1) == b'\xE2'
                        && *(*parser).buffer.pointer.offset((1 + 1).try_into().unwrap())
                            == b'\x80'
                        && *(*parser).buffer.pointer.offset((1 + 2).try_into().unwrap())
                            == b'\xA8'
                    || *(*parser).buffer.pointer.offset(1) == b'\xE2'
                        && *(*parser).buffer.pointer.offset((1 + 1).try_into().unwrap())
                            == b'\x80'
                        && *(*parser).buffer.pointer.offset((1 + 2).try_into().unwrap())
                            == b'\xA9' || *(*parser).buffer.pointer.offset(1) == b'\0'))
        {
            return yaml_parser_fetch_block_entry(parser);
        }
        if *(*parser).buffer.pointer == b'?'
            && ((*parser).flow_level != 0
                || (*(*parser).buffer.pointer.offset(1) == b' '
                    || *(*parser).buffer.pointer.offset(1) == b'\t'
                    || (*(*parser).buffer.pointer.offset(1) == b'\r'
                        || *(*parser).buffer.pointer.offset(1) == b'\n'
                        || *(*parser).buffer.pointer.offset(1) == b'\xC2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 1).try_into().unwrap()) == b'\x85'
                        || *(*parser).buffer.pointer.offset(1) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 2).try_into().unwrap()) == b'\xA8'
                        || *(*parser).buffer.pointer.offset(1) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 2).try_into().unwrap()) == b'\xA9'
                        || *(*parser).buffer.pointer.offset(1) == b'\0')))
        {
            return yaml_parser_fetch_key(parser);
        }
        if *(*parser).buffer.pointer == b':'
            && ((*parser).flow_level != 0
                || (*(*parser).buffer.pointer.offset(1) == b' '
                    || *(*parser).buffer.pointer.offset(1) == b'\t'
                    || (*(*parser).buffer.pointer.offset(1) == b'\r'
                        || *(*parser).buffer.pointer.offset(1) == b'\n'
                        || *(*parser).buffer.pointer.offset(1) == b'\xC2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 1).try_into().unwrap()) == b'\x85'
                        || *(*parser).buffer.pointer.offset(1) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 2).try_into().unwrap()) == b'\xA8'
                        || *(*parser).buffer.pointer.offset(1) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 2).try_into().unwrap()) == b'\xA9'
                        || *(*parser).buffer.pointer.offset(1) == b'\0')))
        {
            return yaml_parser_fetch_value(parser);
        }
        if *(*parser).buffer.pointer == b'*' {
            return yaml_parser_fetch_anchor(parser, YamlAliasToken);
        }
        if *(*parser).buffer.pointer == b'&' {
            return yaml_parser_fetch_anchor(parser, YamlAnchorToken);
        }
        if *(*parser).buffer.pointer == b'!' {
            return yaml_parser_fetch_tag(parser);
        }
        if *(*parser).buffer.pointer == b'|' && (*parser).flow_level == 0 {
            return yaml_parser_fetch_block_scalar(parser, true);
        }
        if *(*parser).buffer.pointer == b'>' && (*parser).flow_level == 0 {
            return yaml_parser_fetch_block_scalar(parser, false);
        }
        if *(*parser).buffer.pointer == b'\'' {
            return yaml_parser_fetch_flow_scalar(parser, true);
        }
        if *(*parser).buffer.pointer == b'"' {
            return yaml_parser_fetch_flow_scalar(parser, false);
        }
        if !(*(*parser).buffer.pointer.offset(0) == b' '
            || *(*parser).buffer.pointer.offset(0) == b'\t'
            || (*(*parser).buffer.pointer.offset(0) == b'\r'
                || *(*parser).buffer.pointer.offset(0) == b'\n'
                || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                    && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                        == b'\x85'
                || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                    && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                        == b'\x80'
                    && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                        == b'\xA8'
                || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                    && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                        == b'\x80'
                    && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                        == b'\xA9' || *(*parser).buffer.pointer.offset(0) == b'\0')
            || *(*parser).buffer.pointer == b'-' || *(*parser).buffer.pointer == b'?'
            || *(*parser).buffer.pointer == b':' || *(*parser).buffer.pointer == b','
            || *(*parser).buffer.pointer == b'[' || *(*parser).buffer.pointer == b']'
            || *(*parser).buffer.pointer == b'{' || *(*parser).buffer.pointer == b'}'
            || *(*parser).buffer.pointer == b'#' || *(*parser).buffer.pointer == b'&'
            || *(*parser).buffer.pointer == b'*' || *(*parser).buffer.pointer == b'!'
            || *(*parser).buffer.pointer == b'|' || *(*parser).buffer.pointer == b'>'
            || *(*parser).buffer.pointer == b'\'' || *(*parser).buffer.pointer == b'"'
            || *(*parser).buffer.pointer == b'%' || *(*parser).buffer.pointer == b'@'
            || *(*parser).buffer.pointer == b'`')
            || *(*parser).buffer.pointer == b'-'
                && !(*(*parser).buffer.pointer.offset(1) == b' '
                    || *(*parser).buffer.pointer.offset(1) == b'\t')
            || (*parser).flow_level == 0
                && (*(*parser).buffer.pointer == b'?'
                    || *(*parser).buffer.pointer == b':')
                && !(*(*parser).buffer.pointer.offset(1) == b' '
                    || *(*parser).buffer.pointer.offset(1) == b'\t'
                    || (*(*parser).buffer.pointer.offset(1) == b'\r'
                        || *(*parser).buffer.pointer.offset(1) == b'\n'
                        || *(*parser).buffer.pointer.offset(1) == b'\xC2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 1).try_into().unwrap()) == b'\x85'
                        || *(*parser).buffer.pointer.offset(1) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 2).try_into().unwrap()) == b'\xA8'
                        || *(*parser).buffer.pointer.offset(1) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((1 + 2).try_into().unwrap()) == b'\xA9'
                        || *(*parser).buffer.pointer.offset(1) == b'\0'))
        {
            return yaml_parser_fetch_plain_scalar(parser);
        }
        yaml_parser_set_scanner_error(
            parser,
            b"while scanning for the next token\0" as *const u8 as *const libc::c_char,
            (*parser).mark,
            b"found character that cannot start any token\0" as *const u8
                as *const libc::c_char,
        );
        FAIL
    }
    unsafe fn yaml_parser_stale_simple_keys(parser: *mut YamlParserT) -> Success {
        let mut simple_key: *mut YamlSimpleKeyT;
        simple_key = (*parser).simple_keys.start.add((*parser).not_simple_keys as usize);
        while simple_key != (*parser).simple_keys.top {
            if (*simple_key).possible
                && ((*simple_key).mark.line < (*parser).mark.line
                    || (*simple_key).mark.index.force_add(1024_u64)
                        < (*parser).mark.index)
            {
                if (*simple_key).required {
                    yaml_parser_set_scanner_error(
                        parser,
                        b"while scanning a simple key\0" as *const u8
                            as *const libc::c_char,
                        (*simple_key).mark,
                        b"could not find expected ':'\0" as *const u8
                            as *const libc::c_char,
                    );
                    return FAIL;
                }
                (*simple_key).possible = false;
                if (*parser).simple_keys.start.add((*parser).not_simple_keys as usize)
                    == simple_key
                {
                    (*parser).not_simple_keys += 1;
                }
            }
            simple_key = simple_key.wrapping_offset(1);
        }
        OK
    }
    unsafe fn yaml_parser_save_simple_key(parser: *mut YamlParserT) -> Success {
        let required = (*parser).flow_level == 0
            && (*parser).indent as libc::c_long == (*parser).mark.column as ptrdiff_t;
        if (*parser).simple_key_allowed {
            let simple_key = YamlSimpleKeyT {
                possible: true,
                required,
                token_number: (*parser)
                    .tokens_parsed
                    .force_add(
                        (*parser).tokens.tail.c_offset_from((*parser).tokens.head)
                            as libc::c_ulong,
                    ),
                mark: (*parser).mark,
            };
            if yaml_parser_remove_simple_key(parser).fail {
                return FAIL;
            }
            *(*parser).simple_keys.top.wrapping_offset(-1_isize) = simple_key;
            if (*parser).simple_keys.start.add((*parser).not_simple_keys as usize)
                == (*parser).simple_keys.top
            {
                (*parser).not_simple_keys -= 1;
            }
        }
        OK
    }
    unsafe fn yaml_parser_remove_simple_key(parser: *mut YamlParserT) -> Success {
        let simple_key: *mut YamlSimpleKeyT = (*parser)
            .simple_keys
            .top
            .wrapping_offset(-1_isize);
        if (*simple_key).possible && (*simple_key).required {
            yaml_parser_set_scanner_error(
                parser,
                b"while scanning a simple key\0" as *const u8 as *const libc::c_char,
                (*simple_key).mark,
                b"could not find expected ':'\0" as *const u8 as *const libc::c_char,
            );
            return FAIL;
        }
        (*simple_key).possible = false;
        OK
    }
    unsafe fn yaml_parser_increase_flow_level(parser: *mut YamlParserT) -> Success {
        let empty_simple_key = YamlSimpleKeyT {
            possible: false,
            required: false,
            token_number: 0_u64,
            mark: YamlMarkT {
                index: 0_u64,
                line: 0_u64,
                column: 0_u64,
            },
        };
        {
            if (*parser).simple_keys.top == (*parser).simple_keys.end {
                yaml_stack_extend(
                    &raw mut (*parser).simple_keys.start as *mut *mut libc::c_void,
                    &raw mut (*parser).simple_keys.top as *mut *mut libc::c_void,
                    &raw mut (*parser).simple_keys.end as *mut *mut libc::c_void,
                );
            }
            ptr::write((*parser).simple_keys.top, empty_simple_key);
            (*parser).simple_keys.top = (*parser).simple_keys.top.wrapping_offset(1);
        };
        if (*parser).flow_level == libc::c_int::MAX {
            (*parser).error = YamlMemoryError;
            return FAIL;
        }
        let fresh7 = &raw mut (*parser).flow_level;
        *fresh7 += 1;
        OK
    }
    unsafe fn yaml_parser_decrease_flow_level(parser: *mut YamlParserT) {
        if (*parser).flow_level != 0 {
            let fresh8 = &raw mut (*parser).flow_level;
            *fresh8 -= 1;
            if (*parser).simple_keys.start.add((*parser).not_simple_keys as usize)
                == (*parser).simple_keys.top
            {
                (*parser).not_simple_keys -= 1;
            }
            let _ = *{
                (*parser).simple_keys.top = (*parser).simple_keys.top.offset(-1);
                (*parser).simple_keys.top
            };
        }
    }
    unsafe fn yaml_parser_roll_indent(
        parser: *mut YamlParserT,
        column: ptrdiff_t,
        number: ptrdiff_t,
        type_: YamlTokenTypeT,
        mark: YamlMarkT,
    ) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        if (*parser).flow_level != 0 {
            return OK;
        }
        if ((*parser).indent as libc::c_long) < column {
            {
                if (*parser).indents.top == (*parser).indents.end {
                    yaml_stack_extend(
                        &raw mut (*parser).indents.start as *mut *mut libc::c_void,
                        &raw mut (*parser).indents.top as *mut *mut libc::c_void,
                        &raw mut (*parser).indents.end as *mut *mut libc::c_void,
                    );
                }
                ptr::write((*parser).indents.top, (*parser).indent);
                (*parser).indents.top = (*parser).indents.top.wrapping_offset(1);
            };
            if column > ptrdiff_t::from(libc::c_int::MAX) {
                (*parser).error = YamlMemoryError;
                return FAIL;
            }
            (*parser).indent = column as libc::c_int;
            let _ = memset(
                token as *mut libc::c_void,
                0,
                size_of::<YamlTokenT>() as libc::c_ulong,
            );
            (*token).type_ = type_;
            (*token).start_mark = mark;
            (*token).end_mark = mark;
            if number == -1_i64 {
                {
                    if (*parser).tokens.tail == (*parser).tokens.end {
                        yaml_queue_extend(
                            &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                            &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                            &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                            &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                        );
                    }
                    ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
                    (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
                };
            } else {
                {
                    if (*parser).tokens.tail == (*parser).tokens.end {
                        yaml_queue_extend(
                            &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                            &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                            &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                            &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                        );
                    }
                    let _ = memmove(
                        (*parser)
                            .tokens
                            .head
                            .wrapping_offset(
                                (number as libc::c_ulong)
                                    .wrapping_sub((*parser).tokens_parsed) as isize,
                            )
                            .wrapping_offset(1_isize) as *mut libc::c_void,
                        (*parser)
                            .tokens
                            .head
                            .wrapping_offset(
                                (number as libc::c_ulong)
                                    .wrapping_sub((*parser).tokens_parsed) as isize,
                            ) as *const libc::c_void,
                        ((*parser).tokens.tail.c_offset_from((*parser).tokens.head)
                            as libc::c_ulong)
                            .wrapping_sub(
                                (number as libc::c_ulong)
                                    .wrapping_sub((*parser).tokens_parsed),
                            )
                            .wrapping_mul(size_of::<YamlTokenT>() as libc::c_ulong),
                    );
                    *(*parser)
                        .tokens
                        .head
                        .wrapping_offset(
                            (number as libc::c_ulong)
                                .wrapping_sub((*parser).tokens_parsed) as isize,
                        ) = *token;
                    let fresh14 = &raw mut (*parser).tokens.tail;
                    *fresh14 = (*fresh14).wrapping_offset(1);
                };
            }
        }
        OK
    }
    unsafe fn yaml_parser_unroll_indent(parser: *mut YamlParserT, column: ptrdiff_t) {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        if (*parser).flow_level != 0 {
            return;
        }
        while (*parser).indent as libc::c_long > column {
            let _ = memset(
                token as *mut libc::c_void,
                0,
                size_of::<YamlTokenT>() as libc::c_ulong,
            );
            (*token).type_ = YamlBlockEndToken;
            (*token).start_mark = (*parser).mark;
            (*token).end_mark = (*parser).mark;
            {
                if (*parser).tokens.tail == (*parser).tokens.end {
                    yaml_queue_extend(
                        &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                        &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                        &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                        &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                    );
                }
                ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
                (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
            };
            (*parser).indent = *{
                (*parser).indents.top = (*parser).indents.top.offset(-1);
                (*parser).indents.top
            };
        }
    }
    unsafe fn yaml_parser_fetch_stream_start(parser: *mut YamlParserT) {
        let simple_key = YamlSimpleKeyT {
            possible: false,
            required: false,
            token_number: 0_u64,
            mark: YamlMarkT {
                index: 0_u64,
                line: 0_u64,
                column: 0_u64,
            },
        };
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        (*parser).indent = -1;
        {
            if (*parser).simple_keys.top == (*parser).simple_keys.end {
                yaml_stack_extend(
                    &raw mut (*parser).simple_keys.start as *mut *mut libc::c_void,
                    &raw mut (*parser).simple_keys.top as *mut *mut libc::c_void,
                    &raw mut (*parser).simple_keys.end as *mut *mut libc::c_void,
                );
            }
            ptr::write((*parser).simple_keys.top, simple_key);
            (*parser).simple_keys.top = (*parser).simple_keys.top.wrapping_offset(1);
        };
        (*parser).not_simple_keys = 1;
        (*parser).simple_key_allowed = true;
        (*parser).stream_start_produced = true;
        let _ = memset(
            token as *mut libc::c_void,
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
        (*token).type_ = YamlStreamStartToken;
        (*token).start_mark = (*parser).mark;
        (*token).end_mark = (*parser).mark;
        (*token).data.stream_start.encoding = (*parser).encoding;
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
    }
    unsafe fn yaml_parser_fetch_stream_end(parser: *mut YamlParserT) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        if (*parser).mark.column != 0_u64 {
            (*parser).mark.column = 0_u64;
            let fresh22 = &raw mut (*parser).mark.line;
            *fresh22 = (*fresh22).force_add(1);
        }
        yaml_parser_unroll_indent(parser, -1_i64);
        if yaml_parser_remove_simple_key(parser).fail {
            return FAIL;
        }
        (*parser).simple_key_allowed = false;
        let _ = memset(
            token as *mut libc::c_void,
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
        (*token).type_ = YamlStreamEndToken;
        (*token).start_mark = (*parser).mark;
        (*token).end_mark = (*parser).mark;
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_fetch_directive(parser: *mut YamlParserT) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        yaml_parser_unroll_indent(parser, -1_i64);
        if yaml_parser_remove_simple_key(parser).fail {
            return FAIL;
        }
        (*parser).simple_key_allowed = false;
        if yaml_parser_scan_directive(parser, token).fail {
            return FAIL;
        }
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_fetch_document_indicator(
        parser: *mut YamlParserT,
        type_: YamlTokenTypeT,
    ) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        yaml_parser_unroll_indent(parser, -1_i64);
        if yaml_parser_remove_simple_key(parser).fail {
            return FAIL;
        }
        (*parser).simple_key_allowed = false;
        let start_mark: YamlMarkT = (*parser).mark;
        skip(parser);
        skip(parser);
        skip(parser);
        let end_mark: YamlMarkT = (*parser).mark;
        let _ = memset(
            token as *mut libc::c_void,
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
        (*token).type_ = type_;
        (*token).start_mark = start_mark;
        (*token).end_mark = end_mark;
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_fetch_flow_collection_start(
        parser: *mut YamlParserT,
        type_: YamlTokenTypeT,
    ) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        if yaml_parser_save_simple_key(parser).fail {
            return FAIL;
        }
        if yaml_parser_increase_flow_level(parser).fail {
            return FAIL;
        }
        (*parser).simple_key_allowed = true;
        let start_mark: YamlMarkT = (*parser).mark;
        skip(parser);
        let end_mark: YamlMarkT = (*parser).mark;
        let _ = memset(
            token as *mut libc::c_void,
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
        (*token).type_ = type_;
        (*token).start_mark = start_mark;
        (*token).end_mark = end_mark;
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_fetch_flow_collection_end(
        parser: *mut YamlParserT,
        type_: YamlTokenTypeT,
    ) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        if yaml_parser_remove_simple_key(parser).fail {
            return FAIL;
        }
        yaml_parser_decrease_flow_level(parser);
        (*parser).simple_key_allowed = false;
        let start_mark: YamlMarkT = (*parser).mark;
        skip(parser);
        let end_mark: YamlMarkT = (*parser).mark;
        let _ = memset(
            token as *mut libc::c_void,
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
        (*token).type_ = type_;
        (*token).start_mark = start_mark;
        (*token).end_mark = end_mark;
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_fetch_flow_entry(parser: *mut YamlParserT) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        if yaml_parser_remove_simple_key(parser).fail {
            return FAIL;
        }
        (*parser).simple_key_allowed = true;
        let start_mark: YamlMarkT = (*parser).mark;
        skip(parser);
        let end_mark: YamlMarkT = (*parser).mark;
        let _ = memset(
            token as *mut libc::c_void,
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
        (*token).type_ = YamlFlowEntryToken;
        (*token).start_mark = start_mark;
        (*token).end_mark = end_mark;
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_fetch_block_entry(parser: *mut YamlParserT) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        if (*parser).flow_level == 0 {
            if !(*parser).simple_key_allowed {
                yaml_parser_set_scanner_error(
                    parser,
                    ptr::null::<libc::c_char>(),
                    (*parser).mark,
                    b"block sequence entries are not allowed in this context\0"
                        as *const u8 as *const libc::c_char,
                );
                return FAIL;
            }
            if yaml_parser_roll_indent(
                    parser,
                    (*parser).mark.column as ptrdiff_t,
                    -1_i64,
                    YamlBlockSequenceStartToken,
                    (*parser).mark,
                )
                .fail
            {
                return FAIL;
            }
        }
        if yaml_parser_remove_simple_key(parser).fail {
            return FAIL;
        }
        (*parser).simple_key_allowed = true;
        let start_mark: YamlMarkT = (*parser).mark;
        skip(parser);
        let end_mark: YamlMarkT = (*parser).mark;
        let _ = memset(
            token as *mut libc::c_void,
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
        (*token).type_ = YamlBlockEntryToken;
        (*token).start_mark = start_mark;
        (*token).end_mark = end_mark;
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_fetch_key(parser: *mut YamlParserT) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        if (*parser).flow_level == 0 {
            if !(*parser).simple_key_allowed {
                yaml_parser_set_scanner_error(
                    parser,
                    ptr::null::<libc::c_char>(),
                    (*parser).mark,
                    b"mapping keys are not allowed in this context\0" as *const u8
                        as *const libc::c_char,
                );
                return FAIL;
            }
            if yaml_parser_roll_indent(
                    parser,
                    (*parser).mark.column as ptrdiff_t,
                    -1_i64,
                    YamlBlockMappingStartToken,
                    (*parser).mark,
                )
                .fail
            {
                return FAIL;
            }
        }
        if yaml_parser_remove_simple_key(parser).fail {
            return FAIL;
        }
        (*parser).simple_key_allowed = (*parser).flow_level == 0;
        let start_mark: YamlMarkT = (*parser).mark;
        skip(parser);
        let end_mark: YamlMarkT = (*parser).mark;
        let _ = memset(
            token as *mut libc::c_void,
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
        (*token).type_ = YamlKeyToken;
        (*token).start_mark = start_mark;
        (*token).end_mark = end_mark;
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_fetch_value(parser: *mut YamlParserT) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        let simple_key: *mut YamlSimpleKeyT = (*parser)
            .simple_keys
            .top
            .wrapping_offset(-1_isize);
        if (*simple_key).possible {
            let _ = memset(
                token as *mut libc::c_void,
                0,
                size_of::<YamlTokenT>() as libc::c_ulong,
            );
            (*token).type_ = YamlKeyToken;
            (*token).start_mark = (*simple_key).mark;
            (*token).end_mark = (*simple_key).mark;
            {
                if (*parser).tokens.tail == (*parser).tokens.end {
                    yaml_queue_extend(
                        &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                        &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                        &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                        &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                    );
                }
                let _ = memmove(
                    (*parser)
                        .tokens
                        .head
                        .wrapping_offset(
                            ((*simple_key).token_number)
                                .wrapping_sub((*parser).tokens_parsed) as isize,
                        )
                        .wrapping_offset(1_isize) as *mut libc::c_void,
                    (*parser)
                        .tokens
                        .head
                        .wrapping_offset(
                            ((*simple_key).token_number)
                                .wrapping_sub((*parser).tokens_parsed) as isize,
                        ) as *const libc::c_void,
                    ((*parser).tokens.tail.c_offset_from((*parser).tokens.head)
                        as libc::c_ulong)
                        .wrapping_sub(
                            ((*simple_key).token_number)
                                .wrapping_sub((*parser).tokens_parsed),
                        )
                        .wrapping_mul(size_of::<YamlTokenT>() as libc::c_ulong),
                );
                *(*parser)
                    .tokens
                    .head
                    .wrapping_offset(
                        ((*simple_key).token_number)
                            .wrapping_sub((*parser).tokens_parsed) as isize,
                    ) = *token;
                let fresh14 = &raw mut (*parser).tokens.tail;
                *fresh14 = (*fresh14).wrapping_offset(1);
            };
            if yaml_parser_roll_indent(
                    parser,
                    (*simple_key).mark.column as ptrdiff_t,
                    (*simple_key).token_number as ptrdiff_t,
                    YamlBlockMappingStartToken,
                    (*simple_key).mark,
                )
                .fail
            {
                return FAIL;
            }
            (*simple_key).possible = false;
            (*parser).simple_key_allowed = false;
        } else {
            if (*parser).flow_level == 0 {
                if !(*parser).simple_key_allowed {
                    yaml_parser_set_scanner_error(
                        parser,
                        ptr::null::<libc::c_char>(),
                        (*parser).mark,
                        b"mapping values are not allowed in this context\0" as *const u8
                            as *const libc::c_char,
                    );
                    return FAIL;
                }
                if yaml_parser_roll_indent(
                        parser,
                        (*parser).mark.column as ptrdiff_t,
                        -1_i64,
                        YamlBlockMappingStartToken,
                        (*parser).mark,
                    )
                    .fail
                {
                    return FAIL;
                }
            }
            (*parser).simple_key_allowed = (*parser).flow_level == 0;
        }
        let start_mark: YamlMarkT = (*parser).mark;
        skip(parser);
        let end_mark: YamlMarkT = (*parser).mark;
        let _ = memset(
            token as *mut libc::c_void,
            0,
            size_of::<YamlTokenT>() as libc::c_ulong,
        );
        (*token).type_ = YamlValueToken;
        (*token).start_mark = start_mark;
        (*token).end_mark = end_mark;
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_fetch_anchor(
        parser: *mut YamlParserT,
        type_: YamlTokenTypeT,
    ) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        if yaml_parser_save_simple_key(parser).fail {
            return FAIL;
        }
        (*parser).simple_key_allowed = false;
        if yaml_parser_scan_anchor(parser, token, type_).fail {
            return FAIL;
        }
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_fetch_tag(parser: *mut YamlParserT) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        if yaml_parser_save_simple_key(parser).fail {
            return FAIL;
        }
        (*parser).simple_key_allowed = false;
        if yaml_parser_scan_tag(parser, token).fail {
            return FAIL;
        }
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_fetch_block_scalar(
        parser: *mut YamlParserT,
        literal: bool,
    ) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        if yaml_parser_remove_simple_key(parser).fail {
            return FAIL;
        }
        (*parser).simple_key_allowed = true;
        if yaml_parser_scan_block_scalar(parser, token, literal).fail {
            return FAIL;
        }
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_fetch_flow_scalar(
        parser: *mut YamlParserT,
        single: bool,
    ) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        if yaml_parser_save_simple_key(parser).fail {
            return FAIL;
        }
        (*parser).simple_key_allowed = false;
        if yaml_parser_scan_flow_scalar(parser, token, single).fail {
            return FAIL;
        }
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_fetch_plain_scalar(parser: *mut YamlParserT) -> Success {
        let mut token = MaybeUninit::<YamlTokenT>::uninit();
        let token = token.as_mut_ptr();
        if yaml_parser_save_simple_key(parser).fail {
            return FAIL;
        }
        (*parser).simple_key_allowed = false;
        if yaml_parser_scan_plain_scalar(parser, token).fail {
            return FAIL;
        }
        {
            if (*parser).tokens.tail == (*parser).tokens.end {
                yaml_queue_extend(
                    &raw mut (*parser).tokens.start as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.head as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.tail as *mut *mut libc::c_void,
                    &raw mut (*parser).tokens.end as *mut *mut libc::c_void,
                );
            }
            ptr::copy_nonoverlapping(token, (*parser).tokens.tail, 1);
            (*parser).tokens.tail = (*parser).tokens.tail.wrapping_offset(1);
        };
        OK
    }
    unsafe fn yaml_parser_scan_to_next_token(parser: *mut YamlParserT) -> Success {
        loop {
            if cache(parser, 1_u64).fail {
                return FAIL;
            }
            if (*parser).mark.column == 0_u64
                && (*(*parser).buffer.pointer.offset(0) == b'\xEF'
                    && *(*parser).buffer.pointer.offset(1) == b'\xBB'
                    && *(*parser).buffer.pointer.offset(2) == b'\xBF')
            {
                skip(parser);
            }
            if cache(parser, 1_u64).fail {
                return FAIL;
            }
            let mut should_continue = true;
            while should_continue {
                if *(*parser).buffer.pointer == b' '
                    || ((*parser).flow_level != 0 || !(*parser).simple_key_allowed)
                        && *(*parser).buffer.pointer == b'\t'
                {
                    skip(parser);
                    if cache(parser, 1_u64).fail {
                        return FAIL;
                    }
                } else {
                    should_continue = false;
                }
            }
            if *(*parser).buffer.pointer == b'#' {
                while !(*(*parser).buffer.pointer.offset(0) == b'\r'
                    || *(*parser).buffer.pointer.offset(0) == b'\n'
                    || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                        && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                            == b'\x85'
                    || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                        && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                            == b'\x80'
                        && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                            == b'\xA8'
                    || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                        && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                            == b'\x80'
                        && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                            == b'\xA9' || *(*parser).buffer.pointer.offset(0) == b'\0')
                {
                    skip(parser);
                    if cache(parser, 1_u64).fail {
                        return FAIL;
                    }
                }
            }
            if !(*(*parser).buffer.pointer.offset(0) == b'\r'
                || *(*parser).buffer.pointer.offset(0) == b'\n'
                || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                    && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                        == b'\x85'
                || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                    && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                        == b'\x80'
                    && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                        == b'\xA8'
                || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                    && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                        == b'\x80'
                    && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                        == b'\xA9')
            {
                break;
            }
            if cache(parser, 2_u64).fail {
                return FAIL;
            }
            skip_line(parser);
            if (*parser).flow_level == 0 {
                (*parser).simple_key_allowed = true;
            }
        }
        OK
    }
    unsafe fn yaml_parser_scan_directive(
        parser: *mut YamlParserT,
        token: *mut YamlTokenT,
    ) -> Success {
        let mut current_block: u64;
        let end_mark: YamlMarkT;
        let mut name: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let mut major: libc::c_int = 0;
        let mut minor: libc::c_int = 0;
        let mut handle: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let mut prefix: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let start_mark: YamlMarkT = (*parser).mark;
        skip(parser);
        if yaml_parser_scan_directive_name(parser, start_mark, &raw mut name).ok {
            if strcmp(
                name as *mut libc::c_char,
                b"YAML\0" as *const u8 as *const libc::c_char,
            ) == 0
            {
                if yaml_parser_scan_version_directive_value(
                        parser,
                        start_mark,
                        &raw mut major,
                        &raw mut minor,
                    )
                    .fail
                {
                    current_block = 11397968426844348457;
                } else {
                    end_mark = (*parser).mark;
                    let _ = memset(
                        token as *mut libc::c_void,
                        0,
                        size_of::<YamlTokenT>() as libc::c_ulong,
                    );
                    (*token).type_ = YamlVersionDirectiveToken;
                    (*token).start_mark = start_mark;
                    (*token).end_mark = end_mark;
                    (*token).data.version_directive.major = major;
                    (*token).data.version_directive.minor = minor;
                    current_block = 17407779659766490442;
                }
            } else if strcmp(
                name as *mut libc::c_char,
                b"TAG\0" as *const u8 as *const libc::c_char,
            ) == 0
            {
                if yaml_parser_scan_tag_directive_value(
                        parser,
                        start_mark,
                        &raw mut handle,
                        &raw mut prefix,
                    )
                    .fail
                {
                    current_block = 11397968426844348457;
                } else {
                    end_mark = (*parser).mark;
                    let _ = memset(
                        token as *mut libc::c_void,
                        0,
                        size_of::<YamlTokenT>() as libc::c_ulong,
                    );
                    (*token).type_ = YamlTagDirectiveToken;
                    (*token).start_mark = start_mark;
                    (*token).end_mark = end_mark;
                    let fresh112 = &raw mut (*token).data.tag_directive.handle;
                    *fresh112 = handle;
                    let fresh113 = &raw mut (*token).data.tag_directive.prefix;
                    *fresh113 = prefix;
                    current_block = 17407779659766490442;
                }
            } else {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a directive\0" as *const u8 as *const libc::c_char,
                    start_mark,
                    b"found unknown directive name\0" as *const u8 as *const libc::c_char,
                );
                current_block = 11397968426844348457;
            }
            if current_block != 11397968426844348457 && cache(parser, 1_u64).ok {
                loop {
                    if !(*(*parser).buffer.pointer.offset(0) == b' '
                        || *(*parser).buffer.pointer.offset(0) == b'\t')
                    {
                        current_block = 11584701595673473500;
                        break;
                    }
                    skip(parser);
                    if cache(parser, 1_u64).fail {
                        current_block = 11397968426844348457;
                        break;
                    }
                }
                if current_block != 11397968426844348457 {
                    if *(*parser).buffer.pointer == b'#' {
                        loop {
                            if *(*parser).buffer.pointer.offset(0) == b'\r'
                                || *(*parser).buffer.pointer.offset(0) == b'\n'
                                || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 1).try_into().unwrap()) == b'\x85'
                                || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 2).try_into().unwrap()) == b'\xA8'
                                || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 2).try_into().unwrap()) == b'\xA9'
                                || *(*parser).buffer.pointer.offset(0) == b'\0'
                            {
                                current_block = 6669252993407410313;
                                break;
                            }
                            skip(parser);
                            if cache(parser, 1_u64).fail {
                                current_block = 11397968426844348457;
                                break;
                            }
                        }
                    } else {
                        current_block = 6669252993407410313;
                    }
                    if current_block != 11397968426844348457 {
                        if !(*(*parser).buffer.pointer.offset(0) == b'\r'
                            || *(*parser).buffer.pointer.offset(0) == b'\n'
                            || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 1).try_into().unwrap()) == b'\x85'
                            || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 2).try_into().unwrap()) == b'\xA8'
                            || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 2).try_into().unwrap()) == b'\xA9'
                            || *(*parser).buffer.pointer.offset(0) == b'\0')
                        {
                            yaml_parser_set_scanner_error(
                                parser,
                                b"while scanning a directive\0" as *const u8
                                    as *const libc::c_char,
                                start_mark,
                                b"did not find expected comment or line break\0"
                                    as *const u8 as *const libc::c_char,
                            );
                        } else {
                            if *(*parser).buffer.pointer.offset(0) == b'\r'
                                || *(*parser).buffer.pointer.offset(0) == b'\n'
                                || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 1).try_into().unwrap()) == b'\x85'
                                || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 2).try_into().unwrap()) == b'\xA8'
                                || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 2).try_into().unwrap()) == b'\xA9'
                            {
                                if cache(parser, 2_u64).fail {
                                    current_block = 11397968426844348457;
                                } else {
                                    skip_line(parser);
                                    current_block = 652864300344834934;
                                }
                            } else {
                                current_block = 652864300344834934;
                            }
                            if current_block != 11397968426844348457 {
                                yaml_free(name as *mut libc::c_void);
                                return OK;
                            }
                        }
                    }
                }
            }
        }
        yaml_free(prefix as *mut libc::c_void);
        yaml_free(handle as *mut libc::c_void);
        yaml_free(name as *mut libc::c_void);
        FAIL
    }
    unsafe fn yaml_parser_scan_directive_name(
        parser: *mut YamlParserT,
        start_mark: YamlMarkT,
        name: *mut *mut yaml_char_t,
    ) -> Success {
        let current_block: u64;
        let mut string = NULL_STRING;
        {
            string.start = yaml_malloc(16) as *mut yaml_char_t;
            if !string.start.is_null() {
                let _ = memset(string.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            string.pointer = string.start;
            string.end = string.start.wrapping_add(16);
            let _ = memset(string.start as *mut libc::c_void, 0, 16);
        };
        if cache(parser, 1_u64).ok {
            loop {
                if !(*(*parser).buffer.pointer >= b'0'
                    && *(*parser).buffer.pointer <= b'9'
                    || *(*parser).buffer.pointer >= b'A'
                        && *(*parser).buffer.pointer <= b'Z'
                    || *(*parser).buffer.pointer >= b'a'
                        && *(*parser).buffer.pointer <= b'z'
                    || *(*parser).buffer.pointer == b'_'
                    || *(*parser).buffer.pointer == b'-')
                {
                    current_block = 10879442775620481940;
                    break;
                }
                read(parser, &raw mut string);
                if cache(parser, 1_u64).fail {
                    current_block = 8318012024179131575;
                    break;
                }
            }
            if current_block != 8318012024179131575 {
                if string.start == string.pointer {
                    yaml_parser_set_scanner_error(
                        parser,
                        b"while scanning a directive\0" as *const u8
                            as *const libc::c_char,
                        start_mark,
                        b"could not find expected directive name\0" as *const u8
                            as *const libc::c_char,
                    );
                } else if !(*(*parser).buffer.pointer.offset(0) == b' '
                    || *(*parser).buffer.pointer.offset(0) == b'\t'
                    || (*(*parser).buffer.pointer.offset(0) == b'\r'
                        || *(*parser).buffer.pointer.offset(0) == b'\n'
                        || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 1).try_into().unwrap()) == b'\x85'
                        || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 2).try_into().unwrap()) == b'\xA8'
                        || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 2).try_into().unwrap()) == b'\xA9'
                        || *(*parser).buffer.pointer.offset(0) == b'\0'))
                {
                    yaml_parser_set_scanner_error(
                        parser,
                        b"while scanning a directive\0" as *const u8
                            as *const libc::c_char,
                        start_mark,
                        b"found unexpected non-alphabetical character\0" as *const u8
                            as *const libc::c_char,
                    );
                } else {
                    *name = string.start;
                    return OK;
                }
            }
        }
        {
            yaml_free(string.start as *mut libc::c_void);
            string.end = ptr::null_mut::<yaml_char_t>();
            string.pointer = string.end;
            string.start = string.pointer;
        };
        FAIL
    }
    unsafe fn yaml_parser_scan_version_directive_value(
        parser: *mut YamlParserT,
        start_mark: YamlMarkT,
        major: *mut libc::c_int,
        minor: *mut libc::c_int,
    ) -> Success {
        if cache(parser, 1_u64).fail {
            return FAIL;
        }
        while *(*parser).buffer.pointer.offset(0) == b' '
            || *(*parser).buffer.pointer.offset(0) == b'\t'
        {
            skip(parser);
            if cache(parser, 1_u64).fail {
                return FAIL;
            }
        }
        if yaml_parser_scan_version_directive_number(parser, start_mark, major).fail {
            return FAIL;
        }
        if !(*(*parser).buffer.pointer == b'.') {
            yaml_parser_set_scanner_error(
                parser,
                b"while scanning a %YAML directive\0" as *const u8
                    as *const libc::c_char,
                start_mark,
                b"did not find expected digit or '.' character\0" as *const u8
                    as *const libc::c_char,
            );
            return FAIL;
        }
        skip(parser);
        yaml_parser_scan_version_directive_number(parser, start_mark, minor)
    }
    const MAX_NUMBER_LENGTH: u64 = 9_u64;
    unsafe fn yaml_parser_scan_version_directive_number(
        parser: *mut YamlParserT,
        start_mark: YamlMarkT,
        number: *mut libc::c_int,
    ) -> Success {
        let mut value: libc::c_int = 0;
        let mut length: size_t = 0_u64;
        if cache(parser, 1_u64).fail {
            return FAIL;
        }
        while !(*parser).buffer.is_empty()
            && (*(*parser).buffer.pointer >= b'0' && *(*parser).buffer.pointer <= b'9')
        {
            length = length.force_add(1);
            if length > MAX_NUMBER_LENGTH {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a %YAML directive\0" as *const u8
                        as *const libc::c_char,
                    start_mark,
                    b"found extremely long version number\0" as *const u8
                        as *const libc::c_char,
                );
                return FAIL;
            }
            value = value
                .force_mul(10)
                .force_add((*(*parser).buffer.pointer - b'0') as libc::c_int);
            (*parser).buffer.next();
            if cache(parser, 1_u64).fail {
                return FAIL;
            }
        }
        if length == 0 {
            yaml_parser_set_scanner_error(
                parser,
                b"while scanning a %YAML directive\0" as *const u8
                    as *const libc::c_char,
                start_mark,
                b"did not find expected version number\0" as *const u8
                    as *const libc::c_char,
            );
            return FAIL;
        }
        *number = value;
        OK
    }
    unsafe fn yaml_parser_scan_tag_directive_value(
        parser: *mut YamlParserT,
        start_mark: YamlMarkT,
        handle: *mut *mut yaml_char_t,
        prefix: *mut *mut yaml_char_t,
    ) -> Success {
        let mut current_block: u64;
        let mut handle_value: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let mut prefix_value: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        if cache(parser, 1_u64).fail {
            current_block = 5231181710497607163;
        } else {
            current_block = 14916268686031723178;
        }
        'c_34337: loop {
            match current_block {
                5231181710497607163 => {
                    yaml_free(handle_value as *mut libc::c_void);
                    yaml_free(prefix_value as *mut libc::c_void);
                    return FAIL;
                }
                _ => {
                    if *(*parser).buffer.pointer.offset(0) == b' '
                        || *(*parser).buffer.pointer.offset(0) == b'\t'
                    {
                        skip(parser);
                        if cache(parser, 1_u64).fail {
                            current_block = 5231181710497607163;
                        } else {
                            current_block = 14916268686031723178;
                        }
                    } else {
                        if yaml_parser_scan_tag_handle(
                                parser,
                                true,
                                start_mark,
                                &raw mut handle_value,
                            )
                            .fail
                        {
                            current_block = 5231181710497607163;
                            continue;
                        }
                        if cache(parser, 1_u64).fail {
                            current_block = 5231181710497607163;
                            continue;
                        }
                        if !(*(*parser).buffer.pointer.offset(0) == b' '
                            || *(*parser).buffer.pointer.offset(0) == b'\t')
                        {
                            yaml_parser_set_scanner_error(
                                parser,
                                b"while scanning a %TAG directive\0" as *const u8
                                    as *const libc::c_char,
                                start_mark,
                                b"did not find expected whitespace\0" as *const u8
                                    as *const libc::c_char,
                            );
                            current_block = 5231181710497607163;
                        } else {
                            while *(*parser).buffer.pointer.offset(0) == b' '
                                || *(*parser).buffer.pointer.offset(0) == b'\t'
                            {
                                skip(parser);
                                if cache(parser, 1_u64).fail {
                                    current_block = 5231181710497607163;
                                    continue 'c_34337;
                                }
                            }
                            if yaml_parser_scan_tag_uri(
                                    parser,
                                    true,
                                    true,
                                    ptr::null_mut::<yaml_char_t>(),
                                    start_mark,
                                    &raw mut prefix_value,
                                )
                                .fail
                            {
                                current_block = 5231181710497607163;
                                continue;
                            }
                            if cache(parser, 1_u64).fail {
                                current_block = 5231181710497607163;
                                continue;
                            }
                            if !(*(*parser).buffer.pointer.offset(0) == b' '
                                || *(*parser).buffer.pointer.offset(0) == b'\t'
                                || (*(*parser).buffer.pointer.offset(0) == b'\r'
                                    || *(*parser).buffer.pointer.offset(0) == b'\n'
                                    || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                                        && *(*parser)
                                            .buffer
                                            .pointer
                                            .offset((0 + 1).try_into().unwrap()) == b'\x85'
                                    || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                        && *(*parser)
                                            .buffer
                                            .pointer
                                            .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                        && *(*parser)
                                            .buffer
                                            .pointer
                                            .offset((0 + 2).try_into().unwrap()) == b'\xA8'
                                    || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                        && *(*parser)
                                            .buffer
                                            .pointer
                                            .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                        && *(*parser)
                                            .buffer
                                            .pointer
                                            .offset((0 + 2).try_into().unwrap()) == b'\xA9'
                                    || *(*parser).buffer.pointer.offset(0) == b'\0'))
                            {
                                yaml_parser_set_scanner_error(
                                    parser,
                                    b"while scanning a %TAG directive\0" as *const u8
                                        as *const libc::c_char,
                                    start_mark,
                                    b"did not find expected whitespace or line break\0"
                                        as *const u8 as *const libc::c_char,
                                );
                                current_block = 5231181710497607163;
                            } else {
                                *handle = handle_value;
                                *prefix = prefix_value;
                                return OK;
                            }
                        }
                    }
                }
            }
        }
    }
    unsafe fn yaml_parser_scan_anchor(
        parser: *mut YamlParserT,
        token: *mut YamlTokenT,
        type_: YamlTokenTypeT,
    ) -> Success {
        let current_block: u64;
        let mut length: libc::c_int = 0;
        let end_mark: YamlMarkT;
        let mut string = NULL_STRING;
        {
            string.start = yaml_malloc(16) as *mut yaml_char_t;
            if !string.start.is_null() {
                let _ = memset(string.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            string.pointer = string.start;
            string.end = string.start.wrapping_add(16);
            let _ = memset(string.start as *mut libc::c_void, 0, 16);
        };
        let start_mark: YamlMarkT = (*parser).mark;
        skip(parser);
        if cache(parser, 1_u64).ok {
            loop {
                if !(*(*parser).buffer.pointer >= b'0'
                    && *(*parser).buffer.pointer <= b'9'
                    || *(*parser).buffer.pointer >= b'A'
                        && *(*parser).buffer.pointer <= b'Z'
                    || *(*parser).buffer.pointer >= b'a'
                        && *(*parser).buffer.pointer <= b'z'
                    || *(*parser).buffer.pointer == b'_'
                    || *(*parser).buffer.pointer == b'-')
                {
                    current_block = 2868539653012386629;
                    break;
                }
                read(parser, &raw mut string);
                if cache(parser, 1_u64).fail {
                    current_block = 5883759901342942623;
                    break;
                }
                length += 1;
            }
            if current_block != 5883759901342942623 {
                end_mark = (*parser).mark;
                if length == 0
                    || !(*(*parser).buffer.pointer.offset(0) == b' '
                        || *(*parser).buffer.pointer.offset(0) == b'\t'
                        || (*(*parser).buffer.pointer.offset(0) == b'\r'
                            || *(*parser).buffer.pointer.offset(0) == b'\n'
                            || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 1).try_into().unwrap()) == b'\x85'
                            || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 2).try_into().unwrap()) == b'\xA8'
                            || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 2).try_into().unwrap()) == b'\xA9'
                            || *(*parser).buffer.pointer.offset(0) == b'\0')
                        || *(*parser).buffer.pointer == b'?'
                        || *(*parser).buffer.pointer == b':'
                        || *(*parser).buffer.pointer == b','
                        || *(*parser).buffer.pointer == b']'
                        || *(*parser).buffer.pointer == b'}'
                        || *(*parser).buffer.pointer == b'%'
                        || *(*parser).buffer.pointer == b'@'
                        || *(*parser).buffer.pointer == b'`')
                {
                    yaml_parser_set_scanner_error(
                        parser,
                        if type_ == YamlAnchorToken {
                            b"while scanning an anchor\0" as *const u8
                                as *const libc::c_char
                        } else {
                            b"while scanning an alias\0" as *const u8
                                as *const libc::c_char
                        },
                        start_mark,
                        b"did not find expected alphabetic or numeric character\0"
                            as *const u8 as *const libc::c_char,
                    );
                } else {
                    if type_ == YamlAnchorToken {
                        let _ = memset(
                            token as *mut libc::c_void,
                            0,
                            size_of::<YamlTokenT>() as libc::c_ulong,
                        );
                        (*token).type_ = YamlAnchorToken;
                        (*token).start_mark = start_mark;
                        (*token).end_mark = end_mark;
                        let fresh220 = &raw mut (*token).data.anchor.value;
                        *fresh220 = string.start;
                    } else {
                        let _ = memset(
                            token as *mut libc::c_void,
                            0,
                            size_of::<YamlTokenT>() as libc::c_ulong,
                        );
                        (*token).type_ = YamlAliasToken;
                        (*token).start_mark = start_mark;
                        (*token).end_mark = end_mark;
                        let fresh221 = &raw mut (*token).data.alias.value;
                        *fresh221 = string.start;
                    }
                    return OK;
                }
            }
        }
        {
            yaml_free(string.start as *mut libc::c_void);
            string.end = ptr::null_mut::<yaml_char_t>();
            string.pointer = string.end;
            string.start = string.pointer;
        };
        FAIL
    }
    unsafe fn yaml_parser_scan_tag(
        parser: *mut YamlParserT,
        token: *mut YamlTokenT,
    ) -> Success {
        let mut current_block: u64;
        let mut handle: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let mut suffix: *mut yaml_char_t = ptr::null_mut::<yaml_char_t>();
        let end_mark: YamlMarkT;
        let start_mark: YamlMarkT = (*parser).mark;
        if cache(parser, 2_u64).ok {
            if *(*parser).buffer.pointer.offset(1) == b'<' {
                handle = yaml_malloc(1_u64) as *mut yaml_char_t;
                *handle = b'\0';
                skip(parser);
                skip(parser);
                if yaml_parser_scan_tag_uri(
                        parser,
                        true,
                        false,
                        ptr::null_mut::<yaml_char_t>(),
                        start_mark,
                        &raw mut suffix,
                    )
                    .fail
                {
                    current_block = 17708497480799081542;
                } else if !(*(*parser).buffer.pointer == b'>') {
                    yaml_parser_set_scanner_error(
                        parser,
                        b"while scanning a tag\0" as *const u8 as *const libc::c_char,
                        start_mark,
                        b"did not find the expected '>'\0" as *const u8
                            as *const libc::c_char,
                    );
                    current_block = 17708497480799081542;
                } else {
                    skip(parser);
                    current_block = 4488286894823169796;
                }
            } else if yaml_parser_scan_tag_handle(
                    parser,
                    false,
                    start_mark,
                    &raw mut handle,
                )
                .fail
            {
                current_block = 17708497480799081542;
            } else if *handle == b'!' && *handle.wrapping_offset(1_isize) != b'\0'
                && *handle
                    .wrapping_offset(
                        strlen(handle as *mut libc::c_char).wrapping_sub(1_u64) as isize,
                    ) == b'!'
            {
                if yaml_parser_scan_tag_uri(
                        parser,
                        false,
                        false,
                        ptr::null_mut::<yaml_char_t>(),
                        start_mark,
                        &raw mut suffix,
                    )
                    .fail
                {
                    current_block = 17708497480799081542;
                } else {
                    current_block = 4488286894823169796;
                }
            } else if yaml_parser_scan_tag_uri(
                    parser,
                    false,
                    false,
                    handle,
                    start_mark,
                    &raw mut suffix,
                )
                .fail
            {
                current_block = 17708497480799081542;
            } else {
                yaml_free(handle as *mut libc::c_void);
                handle = yaml_malloc(2_u64) as *mut yaml_char_t;
                *handle = b'!';
                *handle.wrapping_offset(1_isize) = b'\0';
                if *suffix == b'\0' {
                    core::mem::swap(&mut handle, &mut suffix);
                }
                current_block = 4488286894823169796;
            }
            if current_block != 17708497480799081542 && cache(parser, 1_u64).ok {
                if !(*(*parser).buffer.pointer.offset(0) == b' '
                    || *(*parser).buffer.pointer.offset(0) == b'\t'
                    || (*(*parser).buffer.pointer.offset(0) == b'\r'
                        || *(*parser).buffer.pointer.offset(0) == b'\n'
                        || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 1).try_into().unwrap()) == b'\x85'
                        || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 2).try_into().unwrap()) == b'\xA8'
                        || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 2).try_into().unwrap()) == b'\xA9'
                        || *(*parser).buffer.pointer.offset(0) == b'\0'))
                {
                    if (*parser).flow_level == 0 || !(*(*parser).buffer.pointer == b',')
                    {
                        yaml_parser_set_scanner_error(
                            parser,
                            b"while scanning a tag\0" as *const u8
                                as *const libc::c_char,
                            start_mark,
                            b"did not find expected whitespace or line break\0"
                                as *const u8 as *const libc::c_char,
                        );
                        current_block = 17708497480799081542;
                    } else {
                        current_block = 7333393191927787629;
                    }
                } else {
                    current_block = 7333393191927787629;
                }
                if current_block != 17708497480799081542 {
                    end_mark = (*parser).mark;
                    let _ = memset(
                        token as *mut libc::c_void,
                        0,
                        size_of::<YamlTokenT>() as libc::c_ulong,
                    );
                    (*token).type_ = YamlTagToken;
                    (*token).start_mark = start_mark;
                    (*token).end_mark = end_mark;
                    let fresh234 = &raw mut (*token).data.tag.handle;
                    *fresh234 = handle;
                    let fresh235 = &raw mut (*token).data.tag.suffix;
                    *fresh235 = suffix;
                    return OK;
                }
            }
        }
        yaml_free(handle as *mut libc::c_void);
        yaml_free(suffix as *mut libc::c_void);
        FAIL
    }
    unsafe fn yaml_parser_scan_tag_handle(
        parser: *mut YamlParserT,
        directive: bool,
        start_mark: YamlMarkT,
        handle: *mut *mut yaml_char_t,
    ) -> Success {
        let mut current_block: u64;
        let mut string = NULL_STRING;
        {
            string.start = yaml_malloc(16) as *mut yaml_char_t;
            if !string.start.is_null() {
                let _ = memset(string.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            string.pointer = string.start;
            string.end = string.start.wrapping_add(16);
            let _ = memset(string.start as *mut libc::c_void, 0, 16);
        };
        if cache(parser, 1_u64).ok {
            if !(*(*parser).buffer.pointer == b'!') {
                yaml_parser_set_scanner_error(
                    parser,
                    if directive {
                        b"while scanning a tag directive\0" as *const u8
                            as *const libc::c_char
                    } else {
                        b"while scanning a tag\0" as *const u8 as *const libc::c_char
                    },
                    start_mark,
                    b"did not find expected '!'\0" as *const u8 as *const libc::c_char,
                );
            } else {
                read(parser, &raw mut string);
                if cache(parser, 1_u64).ok {
                    loop {
                        if !(*(*parser).buffer.pointer >= b'0'
                            && *(*parser).buffer.pointer <= b'9'
                            || *(*parser).buffer.pointer >= b'A'
                                && *(*parser).buffer.pointer <= b'Z'
                            || *(*parser).buffer.pointer >= b'a'
                                && *(*parser).buffer.pointer <= b'z'
                            || *(*parser).buffer.pointer == b'_'
                            || *(*parser).buffer.pointer == b'-')
                        {
                            current_block = 7651349459974463963;
                            break;
                        }
                        read(parser, &raw mut string);
                        if cache(parser, 1_u64).fail {
                            current_block = 1771849829115608806;
                            break;
                        }
                    }
                    if current_block != 1771849829115608806 {
                        if *(*parser).buffer.pointer == b'!' {
                            read(parser, &raw mut string);
                            current_block = 5689001924483802034;
                        } else if directive
                            && !(*string.start == b'!'
                                && *string.start.wrapping_offset(1_isize) == b'\0')
                        {
                            yaml_parser_set_scanner_error(
                                parser,
                                b"while parsing a tag directive\0" as *const u8
                                    as *const libc::c_char,
                                start_mark,
                                b"did not find expected '!'\0" as *const u8
                                    as *const libc::c_char,
                            );
                            current_block = 1771849829115608806;
                        } else {
                            current_block = 5689001924483802034;
                        }
                        if current_block != 1771849829115608806 {
                            *handle = string.start;
                            return OK;
                        }
                    }
                }
            }
        }
        {
            yaml_free(string.start as *mut libc::c_void);
            string.end = ptr::null_mut::<yaml_char_t>();
            string.pointer = string.end;
            string.start = string.pointer;
        };
        FAIL
    }
    unsafe fn yaml_parser_scan_tag_uri(
        parser: *mut YamlParserT,
        uri_char: bool,
        directive: bool,
        head: *mut yaml_char_t,
        start_mark: YamlMarkT,
        uri: *mut *mut yaml_char_t,
    ) -> Success {
        let mut current_block: u64;
        let mut length: size_t = if !head.is_null() {
            strlen(head as *mut libc::c_char)
        } else {
            0_u64
        };
        let mut string = NULL_STRING;
        {
            string.start = yaml_malloc(16) as *mut yaml_char_t;
            if !string.start.is_null() {
                let _ = memset(string.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            string.pointer = string.start;
            string.end = string.start.wrapping_add(16);
            let _ = memset(string.start as *mut libc::c_void, 0, 16);
        };
        current_block = 14916268686031723178;
        'c_21953: loop {
            match current_block {
                15265153392498847348 => {
                    {
                        yaml_free(string.start as *mut libc::c_void);
                        string.end = ptr::null_mut::<yaml_char_t>();
                        string.pointer = string.end;
                        string.start = string.pointer;
                    };
                    return FAIL;
                }
                _ => {
                    if string.end.c_offset_from(string.start) as size_t <= length {
                        yaml_string_extend(
                            &raw mut string.start,
                            &raw mut string.pointer,
                            &raw mut string.end,
                        );
                        current_block = 14916268686031723178;
                        continue;
                    } else {
                        if length > 1_u64 {
                            let _ = memcpy(
                                string.start as *mut libc::c_void,
                                head.wrapping_offset(1_isize) as *const libc::c_void,
                                length.wrapping_sub(1_u64),
                            );
                            string.pointer = string
                                .pointer
                                .wrapping_offset(length.wrapping_sub(1_u64) as isize);
                        }
                        if cache(parser, 1_u64).fail {
                            current_block = 15265153392498847348;
                            continue;
                        }
                        while !(*parser).buffer.is_empty()
                            && (*(*parser).buffer.pointer >= b'0'
                                && *(*parser).buffer.pointer <= b'9'
                                || *(*parser).buffer.pointer >= b'A'
                                    && *(*parser).buffer.pointer <= b'Z'
                                || *(*parser).buffer.pointer >= b'a'
                                    && *(*parser).buffer.pointer <= b'z'
                                || *(*parser).buffer.pointer == b'_'
                                || *(*parser).buffer.pointer == b'-'
                                || *(*parser).buffer.pointer == b';'
                                || *(*parser).buffer.pointer == b'/'
                                || *(*parser).buffer.pointer == b'?'
                                || *(*parser).buffer.pointer == b':'
                                || *(*parser).buffer.pointer == b'@'
                                || *(*parser).buffer.pointer == b'&'
                                || *(*parser).buffer.pointer == b'='
                                || *(*parser).buffer.pointer == b'+'
                                || *(*parser).buffer.pointer == b'$'
                                || *(*parser).buffer.pointer == b'.'
                                || *(*parser).buffer.pointer == b'%'
                                || *(*parser).buffer.pointer == b'!'
                                || *(*parser).buffer.pointer == b'~'
                                || *(*parser).buffer.pointer == b'*'
                                || *(*parser).buffer.pointer == b'\''
                                || *(*parser).buffer.pointer == b'('
                                || *(*parser).buffer.pointer == b')'
                                || uri_char
                                    && (*(*parser).buffer.pointer == b','
                                        || *(*parser).buffer.pointer == b'['
                                        || *(*parser).buffer.pointer == b']'))
                        {
                            if *(*parser).buffer.pointer == b'%' {
                                let new_end = string.pointer.wrapping_add(5);
                                if new_end >= string.end {
                                    yaml_string_extend(
                                        &raw mut string.start,
                                        &raw mut string.pointer,
                                        &raw mut string.end,
                                    );
                                }
                                if yaml_parser_scan_uri_escapes(
                                        parser,
                                        directive,
                                        start_mark,
                                        &raw mut string,
                                    )
                                    .fail
                                {
                                    current_block = 15265153392498847348;
                                    continue 'c_21953;
                                }
                            } else {
                                read(parser, &raw mut string);
                            }
                            length = length.force_add(1);
                            if cache(parser, 1_u64).fail {
                                current_block = 15265153392498847348;
                                continue 'c_21953;
                            }
                        }
                        if length == 0 {
                            let new_end = string.pointer.wrapping_add(5);
                            if new_end >= string.end {
                                yaml_string_extend(
                                    &raw mut string.start,
                                    &raw mut string.pointer,
                                    &raw mut string.end,
                                );
                            }
                            yaml_parser_set_scanner_error(
                                parser,
                                if directive {
                                    b"while parsing a %TAG directive\0" as *const u8
                                        as *const libc::c_char
                                } else {
                                    b"while parsing a tag\0" as *const u8 as *const libc::c_char
                                },
                                start_mark,
                                b"did not find expected tag URI\0" as *const u8
                                    as *const libc::c_char,
                            );
                            current_block = 15265153392498847348;
                        } else {
                            *uri = string.start;
                            return OK;
                        }
                    }
                }
            }
        }
    }
    unsafe fn yaml_parser_scan_uri_escapes(
        parser: *mut YamlParserT,
        directive: bool,
        start_mark: YamlMarkT,
        string: *mut YamlStringT,
    ) -> Success {
        let mut width: libc::c_int = 0;
        loop {
            if cache(parser, 3_u64).fail {
                return FAIL;
            }
            if !(*(*parser).buffer.pointer == b'%'
                && (*(*parser).buffer.pointer.wrapping_offset(1) >= b'0'
                    && *(*parser).buffer.pointer.wrapping_offset(1) <= b'9'
                    || *(*parser).buffer.pointer.wrapping_offset(1) >= b'A'
                        && *(*parser).buffer.pointer.wrapping_offset(1) <= b'F'
                    || *(*parser).buffer.pointer.wrapping_offset(1) >= b'a'
                        && *(*parser).buffer.pointer.wrapping_offset(1) <= b'f')
                && (*(*parser).buffer.pointer.wrapping_offset(2) >= b'0'
                    && *(*parser).buffer.pointer.wrapping_offset(2) <= b'9'
                    || *(*parser).buffer.pointer.wrapping_offset(2) >= b'A'
                        && *(*parser).buffer.pointer.wrapping_offset(2) <= b'F'
                    || *(*parser).buffer.pointer.wrapping_offset(2) >= b'a'
                        && *(*parser).buffer.pointer.wrapping_offset(2) <= b'f'))
            {
                yaml_parser_set_scanner_error(
                    parser,
                    if directive {
                        b"while parsing a %TAG directive\0" as *const u8
                            as *const libc::c_char
                    } else {
                        b"while parsing a tag\0" as *const u8 as *const libc::c_char
                    },
                    start_mark,
                    b"did not find URI escaped octet\0" as *const u8
                        as *const libc::c_char,
                );
                return FAIL;
            }
            let octet: libc::c_uchar = (((if *(*parser).buffer.pointer.wrapping_offset(1)
                >= b'A' && *(*parser).buffer.pointer.wrapping_offset(1) <= b'F'
            {
                *(*parser).buffer.pointer.wrapping_offset(1) - b'A' + 10
            } else if *(*parser).buffer.pointer.wrapping_offset(1) >= b'a'
                && *(*parser).buffer.pointer.wrapping_offset(1) <= b'f'
            {
                *(*parser).buffer.pointer.wrapping_offset(1) - b'a' + 10
            } else {
                *(*parser).buffer.pointer.wrapping_offset(1) - b'0'
            } as libc::c_int) << 4)
                + if *(*parser).buffer.pointer.wrapping_offset(2) >= b'A'
                    && *(*parser).buffer.pointer.wrapping_offset(2) <= b'F'
                {
                    *(*parser).buffer.pointer.wrapping_offset(2) - b'A' + 10
                } else if *(*parser).buffer.pointer.wrapping_offset(2) >= b'a'
                    && *(*parser).buffer.pointer.wrapping_offset(2) <= b'f'
                {
                    *(*parser).buffer.pointer.wrapping_offset(2) - b'a' + 10
                } else {
                    *(*parser).buffer.pointer.wrapping_offset(2) - b'0'
                } as libc::c_int) as libc::c_uchar;
            if width == 0 {
                width = if octet & 0x80 == 0 {
                    1
                } else if octet & 0xE0 == 0xC0 {
                    2
                } else if octet & 0xF0 == 0xE0 {
                    3
                } else if octet & 0xF8 == 0xF0 {
                    4
                } else {
                    0
                };
                if width == 0 {
                    yaml_parser_set_scanner_error(
                        parser,
                        if directive {
                            b"while parsing a %TAG directive\0" as *const u8
                                as *const libc::c_char
                        } else {
                            b"while parsing a tag\0" as *const u8 as *const libc::c_char
                        },
                        start_mark,
                        b"found an incorrect leading UTF-8 octet\0" as *const u8
                            as *const libc::c_char,
                    );
                    return FAIL;
                }
            } else if octet & 0xC0 != 0x80 {
                yaml_parser_set_scanner_error(
                    parser,
                    if directive {
                        b"while parsing a %TAG directive\0" as *const u8
                            as *const libc::c_char
                    } else {
                        b"while parsing a tag\0" as *const u8 as *const libc::c_char
                    },
                    start_mark,
                    b"found an incorrect trailing UTF-8 octet\0" as *const u8
                        as *const libc::c_char,
                );
                return FAIL;
            }
            let fresh368 = &raw mut (*string).pointer;
            let fresh369 = *fresh368;
            *fresh368 = (*fresh368).wrapping_offset(1);
            *fresh369 = octet;
            skip(parser);
            skip(parser);
            skip(parser);
            width -= 1;
            if width == 0 {
                break;
            }
        }
        OK
    }
    unsafe fn yaml_parser_scan_block_scalar(
        parser: *mut YamlParserT,
        token: *mut YamlTokenT,
        literal: bool,
    ) -> Success {
        let mut current_block: u64;
        let mut end_mark: YamlMarkT;
        let mut string = NULL_STRING;
        let mut leading_break = NULL_STRING;
        let mut trailing_breaks = NULL_STRING;
        let mut chomping: libc::c_int = 0;
        let mut increment: libc::c_int = 0;
        let mut indent: libc::c_int = 0;
        let mut leading_blank: libc::c_int = 0;
        let mut trailing_blank: libc::c_int;
        {
            string.start = yaml_malloc(16) as *mut yaml_char_t;
            if !string.start.is_null() {
                let _ = memset(string.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            string.pointer = string.start;
            string.end = string.start.wrapping_add(16);
            let _ = memset(string.start as *mut libc::c_void, 0, 16);
        };
        {
            leading_break.start = yaml_malloc(16) as *mut yaml_char_t;
            if !leading_break.start.is_null() {
                let _ = memset(leading_break.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            leading_break.pointer = leading_break.start;
            leading_break.end = leading_break.start.wrapping_add(16);
            let _ = memset(leading_break.start as *mut libc::c_void, 0, 16);
        };
        {
            trailing_breaks.start = yaml_malloc(16) as *mut yaml_char_t;
            if !trailing_breaks.start.is_null() {
                let _ = memset(trailing_breaks.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            trailing_breaks.pointer = trailing_breaks.start;
            trailing_breaks.end = trailing_breaks.start.wrapping_add(16);
            let _ = memset(trailing_breaks.start as *mut libc::c_void, 0, 16);
        };
        let start_mark: YamlMarkT = (*parser).mark;
        skip(parser);
        if cache(parser, 1_u64).ok {
            if *(*parser).buffer.pointer == b'+' || *(*parser).buffer.pointer == b'-' {
                chomping = if *(*parser).buffer.pointer == b'+' { 1 } else { -1 };
                skip(parser);
                if cache(parser, 1_u64).fail {
                    current_block = 14984465786483313892;
                } else if *(*parser).buffer.pointer >= b'0'
                    && *(*parser).buffer.pointer <= b'9'
                {
                    if *(*parser).buffer.pointer == b'0' {
                        yaml_parser_set_scanner_error(
                            parser,
                            b"while scanning a block scalar\0" as *const u8
                                as *const libc::c_char,
                            start_mark,
                            b"found an indentation indicator equal to 0\0" as *const u8
                                as *const libc::c_char,
                        );
                        current_block = 14984465786483313892;
                    } else {
                        increment = (*(*parser).buffer.pointer - b'0') as libc::c_int;
                        skip(parser);
                        current_block = 11913429853522160501;
                    }
                } else {
                    current_block = 11913429853522160501;
                }
            } else if *(*parser).buffer.pointer >= b'0'
                && *(*parser).buffer.pointer <= b'9'
            {
                if *(*parser).buffer.pointer == b'0' {
                    yaml_parser_set_scanner_error(
                        parser,
                        b"while scanning a block scalar\0" as *const u8
                            as *const libc::c_char,
                        start_mark,
                        b"found an indentation indicator equal to 0\0" as *const u8
                            as *const libc::c_char,
                    );
                    current_block = 14984465786483313892;
                } else {
                    increment = (*(*parser).buffer.pointer - b'0') as libc::c_int;
                    skip(parser);
                    if cache(parser, 1_u64).fail {
                        current_block = 14984465786483313892;
                    } else {
                        if *(*parser).buffer.pointer == b'+'
                            || *(*parser).buffer.pointer == b'-'
                        {
                            chomping = if *(*parser).buffer.pointer == b'+' {
                                1
                            } else {
                                -1
                            };
                            skip(parser);
                        }
                        current_block = 11913429853522160501;
                    }
                }
            } else {
                current_block = 11913429853522160501;
            }
            if current_block != 14984465786483313892 && cache(parser, 1_u64).ok {
                loop {
                    if !(*(*parser).buffer.pointer.offset(0) == b' '
                        || *(*parser).buffer.pointer.offset(0) == b'\t')
                    {
                        current_block = 4090602189656566074;
                        break;
                    }
                    skip(parser);
                    if cache(parser, 1_u64).fail {
                        current_block = 14984465786483313892;
                        break;
                    }
                }
                if current_block != 14984465786483313892 {
                    if *(*parser).buffer.pointer == b'#' {
                        loop {
                            if *(*parser).buffer.pointer.offset(0) == b'\r'
                                || *(*parser).buffer.pointer.offset(0) == b'\n'
                                || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 1).try_into().unwrap()) == b'\x85'
                                || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 2).try_into().unwrap()) == b'\xA8'
                                || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 2).try_into().unwrap()) == b'\xA9'
                                || *(*parser).buffer.pointer.offset(0) == b'\0'
                            {
                                current_block = 12997042908615822766;
                                break;
                            }
                            skip(parser);
                            if cache(parser, 1_u64).fail {
                                current_block = 14984465786483313892;
                                break;
                            }
                        }
                    } else {
                        current_block = 12997042908615822766;
                    }
                    if current_block != 14984465786483313892 {
                        if !(*(*parser).buffer.pointer.offset(0) == b'\r'
                            || *(*parser).buffer.pointer.offset(0) == b'\n'
                            || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 1).try_into().unwrap()) == b'\x85'
                            || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 2).try_into().unwrap()) == b'\xA8'
                            || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                && *(*parser)
                                    .buffer
                                    .pointer
                                    .offset((0 + 2).try_into().unwrap()) == b'\xA9'
                            || *(*parser).buffer.pointer.offset(0) == b'\0')
                        {
                            yaml_parser_set_scanner_error(
                                parser,
                                b"while scanning a block scalar\0" as *const u8
                                    as *const libc::c_char,
                                start_mark,
                                b"did not find expected comment or line break\0"
                                    as *const u8 as *const libc::c_char,
                            );
                        } else {
                            if *(*parser).buffer.pointer.offset(0) == b'\r'
                                || *(*parser).buffer.pointer.offset(0) == b'\n'
                                || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 1).try_into().unwrap()) == b'\x85'
                                || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 2).try_into().unwrap()) == b'\xA8'
                                || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((0 + 2).try_into().unwrap()) == b'\xA9'
                            {
                                if cache(parser, 2_u64).fail {
                                    current_block = 14984465786483313892;
                                } else {
                                    skip_line(parser);
                                    current_block = 13619784596304402172;
                                }
                            } else {
                                current_block = 13619784596304402172;
                            }
                            if current_block != 14984465786483313892 {
                                end_mark = (*parser).mark;
                                if increment != 0 {
                                    indent = if (*parser).indent >= 0 {
                                        (*parser).indent + increment
                                    } else {
                                        increment
                                    };
                                }
                                if yaml_parser_scan_block_scalar_breaks(
                                        parser,
                                        &raw mut indent,
                                        &raw mut trailing_breaks,
                                        start_mark,
                                        &raw mut end_mark,
                                    )
                                    .ok && cache(parser, 1_u64).ok
                                {
                                    's_281: loop {
                                        if (*parser).mark.column as libc::c_int != indent
                                            || *(*parser).buffer.pointer.offset(0) == b'\0'
                                        {
                                            current_block = 5793491756164225964;
                                            break;
                                        }
                                        trailing_blank = (*(*parser).buffer.pointer.offset(0)
                                            == b' ' || *(*parser).buffer.pointer.offset(0) == b'\t')
                                            as libc::c_int;
                                        if !literal && *leading_break.start == b'\n'
                                            && leading_blank == 0 && trailing_blank == 0
                                        {
                                            if *trailing_breaks.start == b'\0' {
                                                let new_end = string.pointer.wrapping_add(5);
                                                if new_end >= string.end {
                                                    yaml_string_extend(
                                                        &raw mut string.start,
                                                        &raw mut string.pointer,
                                                        &raw mut string.end,
                                                    );
                                                }
                                                let fresh418 = string.pointer;
                                                string.pointer = string.pointer.wrapping_offset(1);
                                                *fresh418 = b' ';
                                            }
                                            {
                                                leading_break.pointer = leading_break.start;
                                                let _ = memset(
                                                    leading_break.start as *mut libc::c_void,
                                                    0,
                                                    leading_break.end.offset_from(leading_break.start)
                                                        as libc::c_ulong,
                                                );
                                            };
                                        } else {
                                            {
                                                let a_len = string.pointer.offset_from(string.start)
                                                    as usize;
                                                let b_len = leading_break
                                                    .pointer
                                                    .offset_from(leading_break.start) as usize;
                                                if a_len.checked_add(b_len).is_some()
                                                    && string.pointer.add(b_len) <= string.end
                                                {
                                                    yaml_string_join(
                                                        &raw mut string.start,
                                                        &raw mut string.pointer,
                                                        &raw mut string.end,
                                                        &raw mut leading_break.start,
                                                        &raw mut leading_break.pointer,
                                                        &raw mut leading_break.end,
                                                    );
                                                    leading_break.pointer = leading_break.start;
                                                } else {
                                                    {
                                                        ::core::panicking::panic_fmt(
                                                            format_args!("String join would overflow memory bounds"),
                                                        );
                                                    };
                                                }
                                            };
                                            {
                                                leading_break.pointer = leading_break.start;
                                                let _ = memset(
                                                    leading_break.start as *mut libc::c_void,
                                                    0,
                                                    leading_break.end.offset_from(leading_break.start)
                                                        as libc::c_ulong,
                                                );
                                            };
                                        }
                                        {
                                            let a_len = string.pointer.offset_from(string.start)
                                                as usize;
                                            let b_len = trailing_breaks
                                                .pointer
                                                .offset_from(trailing_breaks.start) as usize;
                                            if a_len.checked_add(b_len).is_some()
                                                && string.pointer.add(b_len) <= string.end
                                            {
                                                yaml_string_join(
                                                    &raw mut string.start,
                                                    &raw mut string.pointer,
                                                    &raw mut string.end,
                                                    &raw mut trailing_breaks.start,
                                                    &raw mut trailing_breaks.pointer,
                                                    &raw mut trailing_breaks.end,
                                                );
                                                trailing_breaks.pointer = trailing_breaks.start;
                                            } else {
                                                {
                                                    ::core::panicking::panic_fmt(
                                                        format_args!("String join would overflow memory bounds"),
                                                    );
                                                };
                                            }
                                        };
                                        {
                                            trailing_breaks.pointer = trailing_breaks.start;
                                            let _ = memset(
                                                trailing_breaks.start as *mut libc::c_void,
                                                0,
                                                trailing_breaks.end.offset_from(trailing_breaks.start)
                                                    as libc::c_ulong,
                                            );
                                        };
                                        leading_blank = (*(*parser).buffer.pointer.offset(0) == b' '
                                            || *(*parser).buffer.pointer.offset(0) == b'\t')
                                            as libc::c_int;
                                        while !(*(*parser).buffer.pointer.offset(0) == b'\r'
                                            || *(*parser).buffer.pointer.offset(0) == b'\n'
                                            || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                                                && *(*parser)
                                                    .buffer
                                                    .pointer
                                                    .offset((0 + 1).try_into().unwrap()) == b'\x85'
                                            || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                                && *(*parser)
                                                    .buffer
                                                    .pointer
                                                    .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                                && *(*parser)
                                                    .buffer
                                                    .pointer
                                                    .offset((0 + 2).try_into().unwrap()) == b'\xA8'
                                            || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                                                && *(*parser)
                                                    .buffer
                                                    .pointer
                                                    .offset((0 + 1).try_into().unwrap()) == b'\x80'
                                                && *(*parser)
                                                    .buffer
                                                    .pointer
                                                    .offset((0 + 2).try_into().unwrap()) == b'\xA9'
                                            || *(*parser).buffer.pointer.offset(0) == b'\0')
                                        {
                                            read(parser, &raw mut string);
                                            if cache(parser, 1_u64).fail {
                                                current_block = 14984465786483313892;
                                                break 's_281;
                                            }
                                        }
                                        if cache(parser, 2_u64).fail {
                                            current_block = 14984465786483313892;
                                            break;
                                        }
                                        read_line(parser, &raw mut leading_break);
                                        if yaml_parser_scan_block_scalar_breaks(
                                                parser,
                                                &raw mut indent,
                                                &raw mut trailing_breaks,
                                                start_mark,
                                                &raw mut end_mark,
                                            )
                                            .fail
                                        {
                                            current_block = 14984465786483313892;
                                            break;
                                        }
                                    }
                                    if current_block != 14984465786483313892 {
                                        if chomping != -1 {
                                            {
                                                let a_len = string.pointer.offset_from(string.start)
                                                    as usize;
                                                let b_len = leading_break
                                                    .pointer
                                                    .offset_from(leading_break.start) as usize;
                                                if a_len.checked_add(b_len).is_some()
                                                    && string.pointer.add(b_len) <= string.end
                                                {
                                                    yaml_string_join(
                                                        &raw mut string.start,
                                                        &raw mut string.pointer,
                                                        &raw mut string.end,
                                                        &raw mut leading_break.start,
                                                        &raw mut leading_break.pointer,
                                                        &raw mut leading_break.end,
                                                    );
                                                    leading_break.pointer = leading_break.start;
                                                } else {
                                                    {
                                                        ::core::panicking::panic_fmt(
                                                            format_args!("String join would overflow memory bounds"),
                                                        );
                                                    };
                                                }
                                            };
                                            current_block = 17787701279558130514;
                                        } else {
                                            current_block = 17787701279558130514;
                                        }
                                        if current_block != 14984465786483313892 {
                                            if chomping == 1 {
                                                {
                                                    let a_len = string.pointer.offset_from(string.start)
                                                        as usize;
                                                    let b_len = trailing_breaks
                                                        .pointer
                                                        .offset_from(trailing_breaks.start) as usize;
                                                    if a_len.checked_add(b_len).is_some()
                                                        && string.pointer.add(b_len) <= string.end
                                                    {
                                                        yaml_string_join(
                                                            &raw mut string.start,
                                                            &raw mut string.pointer,
                                                            &raw mut string.end,
                                                            &raw mut trailing_breaks.start,
                                                            &raw mut trailing_breaks.pointer,
                                                            &raw mut trailing_breaks.end,
                                                        );
                                                        trailing_breaks.pointer = trailing_breaks.start;
                                                    } else {
                                                        {
                                                            ::core::panicking::panic_fmt(
                                                                format_args!("String join would overflow memory bounds"),
                                                            );
                                                        };
                                                    }
                                                };
                                            }
                                            let _ = memset(
                                                token as *mut libc::c_void,
                                                0,
                                                size_of::<YamlTokenT>() as libc::c_ulong,
                                            );
                                            (*token).type_ = YamlScalarToken;
                                            (*token).start_mark = start_mark;
                                            (*token).end_mark = end_mark;
                                            let fresh479 = &raw mut (*token).data.scalar.value;
                                            *fresh479 = string.start;
                                            (*token).data.scalar.length = string
                                                .pointer
                                                .c_offset_from(string.start) as size_t;
                                            (*token).data.scalar.style = if literal {
                                                YamlLiteralScalarStyle
                                            } else {
                                                YamlFoldedScalarStyle
                                            };
                                            {
                                                yaml_free(leading_break.start as *mut libc::c_void);
                                                leading_break.end = ptr::null_mut::<yaml_char_t>();
                                                leading_break.pointer = leading_break.end;
                                                leading_break.start = leading_break.pointer;
                                            };
                                            {
                                                yaml_free(trailing_breaks.start as *mut libc::c_void);
                                                trailing_breaks.end = ptr::null_mut::<yaml_char_t>();
                                                trailing_breaks.pointer = trailing_breaks.end;
                                                trailing_breaks.start = trailing_breaks.pointer;
                                            };
                                            return OK;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        {
            yaml_free(string.start as *mut libc::c_void);
            string.end = ptr::null_mut::<yaml_char_t>();
            string.pointer = string.end;
            string.start = string.pointer;
        };
        {
            yaml_free(leading_break.start as *mut libc::c_void);
            leading_break.end = ptr::null_mut::<yaml_char_t>();
            leading_break.pointer = leading_break.end;
            leading_break.start = leading_break.pointer;
        };
        {
            yaml_free(trailing_breaks.start as *mut libc::c_void);
            trailing_breaks.end = ptr::null_mut::<yaml_char_t>();
            trailing_breaks.pointer = trailing_breaks.end;
            trailing_breaks.start = trailing_breaks.pointer;
        };
        FAIL
    }
    unsafe fn yaml_parser_scan_block_scalar_breaks(
        parser: *mut YamlParserT,
        indent: *mut libc::c_int,
        breaks: *mut YamlStringT,
        start_mark: YamlMarkT,
        end_mark: *mut YamlMarkT,
    ) -> Success {
        let mut max_indent: libc::c_int = 0;
        *end_mark = (*parser).mark;
        loop {
            if cache(parser, 1_u64).fail {
                return FAIL;
            }
            while (*indent == 0 || ((*parser).mark.column as libc::c_int) < *indent)
                && *(*parser).buffer.pointer.offset(0) == b' '
            {
                skip(parser);
                if cache(parser, 1_u64).fail {
                    return FAIL;
                }
            }
            if (*parser).mark.column as libc::c_int > max_indent {
                max_indent = (*parser).mark.column as libc::c_int;
            }
            if (*indent == 0 || ((*parser).mark.column as libc::c_int) < *indent)
                && *(*parser).buffer.pointer.offset(0) == b'\t'
            {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a block scalar\0" as *const u8
                        as *const libc::c_char,
                    start_mark,
                    b"found a tab character where an indentation space is expected\0"
                        as *const u8 as *const libc::c_char,
                );
                return FAIL;
            }
            if !(*(*parser).buffer.pointer.offset(0) == b'\r'
                || *(*parser).buffer.pointer.offset(0) == b'\n'
                || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                    && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                        == b'\x85'
                || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                    && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                        == b'\x80'
                    && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                        == b'\xA8'
                || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                    && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                        == b'\x80'
                    && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                        == b'\xA9')
            {
                break;
            }
            if cache(parser, 2_u64).fail {
                return FAIL;
            }
            read_line(parser, &raw mut *breaks);
            *end_mark = (*parser).mark;
        }
        if *indent == 0 {
            *indent = max_indent;
            if *indent < (*parser).indent + 1 {
                *indent = (*parser).indent + 1;
            }
            if *indent < 1 {
                *indent = 1;
            }
        }
        OK
    }
    unsafe fn yaml_parser_scan_flow_scalar(
        parser: *mut YamlParserT,
        token: *mut YamlTokenT,
        single: bool,
    ) -> Success {
        let current_block: u64;
        let end_mark: YamlMarkT;
        let mut string = NULL_STRING;
        let mut leading_break = NULL_STRING;
        let mut trailing_breaks = NULL_STRING;
        let mut whitespaces = NULL_STRING;
        let mut leading_blanks;
        {
            string.start = yaml_malloc(16) as *mut yaml_char_t;
            if !string.start.is_null() {
                let _ = memset(string.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            string.pointer = string.start;
            string.end = string.start.wrapping_add(16);
            let _ = memset(string.start as *mut libc::c_void, 0, 16);
        };
        {
            leading_break.start = yaml_malloc(16) as *mut yaml_char_t;
            if !leading_break.start.is_null() {
                let _ = memset(leading_break.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            leading_break.pointer = leading_break.start;
            leading_break.end = leading_break.start.wrapping_add(16);
            let _ = memset(leading_break.start as *mut libc::c_void, 0, 16);
        };
        {
            trailing_breaks.start = yaml_malloc(16) as *mut yaml_char_t;
            if !trailing_breaks.start.is_null() {
                let _ = memset(trailing_breaks.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            trailing_breaks.pointer = trailing_breaks.start;
            trailing_breaks.end = trailing_breaks.start.wrapping_add(16);
            let _ = memset(trailing_breaks.start as *mut libc::c_void, 0, 16);
        };
        {
            whitespaces.start = yaml_malloc(16) as *mut yaml_char_t;
            if !whitespaces.start.is_null() {
                let _ = memset(whitespaces.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            whitespaces.pointer = whitespaces.start;
            whitespaces.end = whitespaces.start.wrapping_add(16);
            let _ = memset(whitespaces.start as *mut libc::c_void, 0, 16);
        };
        let start_mark: YamlMarkT = (*parser).mark;
        skip(parser);
        's_58: loop {
            if cache(parser, 4_u64).fail {
                current_block = 8114179180390253173;
                break;
            }
            if (*parser).mark.column == 0_u64
                && (*(*parser).buffer.pointer.offset(0) == b'-'
                    && *(*parser).buffer.pointer.offset(1) == b'-'
                    && *(*parser).buffer.pointer.offset(2) == b'-'
                    || *(*parser).buffer.pointer.offset(0) == b'.'
                        && *(*parser).buffer.pointer.offset(1) == b'.'
                        && *(*parser).buffer.pointer.offset(2) == b'.')
                && (*(*parser).buffer.pointer.offset(3) == b' '
                    || *(*parser).buffer.pointer.offset(3) == b'\t'
                    || (*(*parser).buffer.pointer.offset(3) == b'\r'
                        || *(*parser).buffer.pointer.offset(3) == b'\n'
                        || *(*parser).buffer.pointer.offset(3) == b'\xC2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((3 + 1).try_into().unwrap()) == b'\x85'
                        || *(*parser).buffer.pointer.offset(3) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((3 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((3 + 2).try_into().unwrap()) == b'\xA8'
                        || *(*parser).buffer.pointer.offset(3) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((3 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((3 + 2).try_into().unwrap()) == b'\xA9'
                        || *(*parser).buffer.pointer.offset(3) == b'\0'))
            {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a quoted scalar\0" as *const u8
                        as *const libc::c_char,
                    start_mark,
                    b"found unexpected document indicator\0" as *const u8
                        as *const libc::c_char,
                );
                current_block = 8114179180390253173;
                break;
            } else if *(*parser).buffer.pointer.offset(0) == b'\0' {
                yaml_parser_set_scanner_error(
                    parser,
                    b"while scanning a quoted scalar\0" as *const u8
                        as *const libc::c_char,
                    start_mark,
                    b"found unexpected end of stream\0" as *const u8
                        as *const libc::c_char,
                );
                current_block = 8114179180390253173;
                break;
            } else {
                if cache(parser, 2_u64).fail {
                    current_block = 8114179180390253173;
                    break;
                }
                leading_blanks = false;
                while !(*(*parser).buffer.pointer.offset(0) == b' '
                    || *(*parser).buffer.pointer.offset(0) == b'\t'
                    || (*(*parser).buffer.pointer.offset(0) == b'\r'
                        || *(*parser).buffer.pointer.offset(0) == b'\n'
                        || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 1).try_into().unwrap()) == b'\x85'
                        || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 2).try_into().unwrap()) == b'\xA8'
                        || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 2).try_into().unwrap()) == b'\xA9'
                        || *(*parser).buffer.pointer.offset(0) == b'\0'))
                {
                    if single && *(*parser).buffer.pointer.offset(0) == b'\''
                        && *(*parser).buffer.pointer.offset(1) == b'\''
                    {
                        let new_end = string.pointer.wrapping_add(5);
                        if new_end >= string.end {
                            yaml_string_extend(
                                &raw mut string.start,
                                &raw mut string.pointer,
                                &raw mut string.end,
                            );
                        }
                        let fresh521 = string.pointer;
                        string.pointer = string.pointer.wrapping_offset(1);
                        *fresh521 = b'\'';
                        skip(parser);
                        skip(parser);
                    } else {
                        if *(*parser).buffer.pointer == if single { b'\'' } else { b'"' }
                        {
                            break;
                        }
                        if !single && *(*parser).buffer.pointer == b'\\'
                            && (*(*parser).buffer.pointer.offset(1) == b'\r'
                                || *(*parser).buffer.pointer.offset(1) == b'\n'
                                || *(*parser).buffer.pointer.offset(1) == b'\xC2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((1 + 1).try_into().unwrap()) == b'\x85'
                                || *(*parser).buffer.pointer.offset(1) == b'\xE2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((1 + 1).try_into().unwrap()) == b'\x80'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((1 + 2).try_into().unwrap()) == b'\xA8'
                                || *(*parser).buffer.pointer.offset(1) == b'\xE2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((1 + 1).try_into().unwrap()) == b'\x80'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((1 + 2).try_into().unwrap()) == b'\xA9')
                        {
                            if cache(parser, 3_u64).fail {
                                current_block = 8114179180390253173;
                                break 's_58;
                            }
                            skip(parser);
                            skip_line(parser);
                            leading_blanks = true;
                            break;
                        } else if !single && *(*parser).buffer.pointer == b'\\' {
                            let mut code_length: size_t = 0_u64;
                            let new_end = string.pointer.wrapping_add(5);
                            if new_end >= string.end {
                                yaml_string_extend(
                                    &raw mut string.start,
                                    &raw mut string.pointer,
                                    &raw mut string.end,
                                );
                            }
                            match *(*parser).buffer.pointer.wrapping_offset(1_isize) {
                                b'0' => {
                                    let fresh542 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh542 = b'\0';
                                }
                                b'a' => {
                                    let fresh543 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh543 = b'\x07';
                                }
                                b'b' => {
                                    let fresh544 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh544 = b'\x08';
                                }
                                b't' | b'\t' => {
                                    let fresh545 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh545 = b'\t';
                                }
                                b'n' => {
                                    let fresh546 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh546 = b'\n';
                                }
                                b'v' => {
                                    let fresh547 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh547 = b'\x0B';
                                }
                                b'f' => {
                                    let fresh548 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh548 = b'\x0C';
                                }
                                b'r' => {
                                    let fresh549 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh549 = b'\r';
                                }
                                b'e' => {
                                    let fresh550 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh550 = b'\x1B';
                                }
                                b' ' => {
                                    let fresh551 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh551 = b' ';
                                }
                                b'"' => {
                                    let fresh552 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh552 = b'"';
                                }
                                b'/' => {
                                    let fresh553 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh553 = b'/';
                                }
                                b'\\' => {
                                    let fresh554 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh554 = b'\\';
                                }
                                b'N' => {
                                    let fresh555 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh555 = b'\xC2';
                                    let fresh556 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh556 = b'\x85';
                                }
                                b'_' => {
                                    let fresh557 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh557 = b'\xC2';
                                    let fresh558 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh558 = b'\xA0';
                                }
                                b'L' => {
                                    let fresh559 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh559 = b'\xE2';
                                    let fresh560 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh560 = b'\x80';
                                    let fresh561 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh561 = b'\xA8';
                                }
                                b'P' => {
                                    let fresh562 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh562 = b'\xE2';
                                    let fresh563 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh563 = b'\x80';
                                    let fresh564 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh564 = b'\xA9';
                                }
                                b'x' => {
                                    code_length = 2_u64;
                                }
                                b'u' => {
                                    code_length = 4_u64;
                                }
                                b'U' => {
                                    code_length = 8_u64;
                                }
                                _ => {
                                    yaml_parser_set_scanner_error(
                                        parser,
                                        b"while parsing a quoted scalar\0" as *const u8
                                            as *const libc::c_char,
                                        start_mark,
                                        b"found unknown escape character\0" as *const u8
                                            as *const libc::c_char,
                                    );
                                    current_block = 8114179180390253173;
                                    break 's_58;
                                }
                            }
                            skip(parser);
                            skip(parser);
                            if code_length != 0 {
                                let mut value: libc::c_uint = 0;
                                let mut k: size_t;
                                if cache(parser, code_length).fail {
                                    current_block = 8114179180390253173;
                                    break 's_58;
                                }
                                k = 0_u64;
                                while k < code_length {
                                    if !(*(*parser).buffer.pointer.wrapping_offset(k as isize)
                                        >= b'0'
                                        && *(*parser).buffer.pointer.wrapping_offset(k as isize)
                                            <= b'9'
                                        || *(*parser).buffer.pointer.wrapping_offset(k as isize)
                                            >= b'A'
                                            && *(*parser).buffer.pointer.wrapping_offset(k as isize)
                                                <= b'F'
                                        || *(*parser).buffer.pointer.wrapping_offset(k as isize)
                                            >= b'a'
                                            && *(*parser).buffer.pointer.wrapping_offset(k as isize)
                                                <= b'f')
                                    {
                                        yaml_parser_set_scanner_error(
                                            parser,
                                            b"while parsing a quoted scalar\0" as *const u8
                                                as *const libc::c_char,
                                            start_mark,
                                            b"did not find expected hexadecimal number\0" as *const u8
                                                as *const libc::c_char,
                                        );
                                        current_block = 8114179180390253173;
                                        break 's_58;
                                    } else {
                                        value = (value << 4)
                                            .force_add(
                                                if *(*parser).buffer.pointer.wrapping_offset(k as isize)
                                                    >= b'A'
                                                    && *(*parser).buffer.pointer.wrapping_offset(k as isize)
                                                        <= b'F'
                                                {
                                                    *(*parser).buffer.pointer.wrapping_offset(k as isize) - b'A'
                                                        + 10
                                                } else if *(*parser)
                                                    .buffer
                                                    .pointer
                                                    .wrapping_offset(k as isize) >= b'a'
                                                    && *(*parser).buffer.pointer.wrapping_offset(k as isize)
                                                        <= b'f'
                                                {
                                                    *(*parser).buffer.pointer.wrapping_offset(k as isize) - b'a'
                                                        + 10
                                                } else {
                                                    *(*parser).buffer.pointer.wrapping_offset(k as isize) - b'0'
                                                } as libc::c_int as libc::c_uint,
                                            );
                                        k = k.force_add(1);
                                    }
                                }
                                if (0xD800..=0xDFFF).contains(&value) || value > 0x10FFFF {
                                    yaml_parser_set_scanner_error(
                                        parser,
                                        b"while parsing a quoted scalar\0" as *const u8
                                            as *const libc::c_char,
                                        start_mark,
                                        b"found invalid Unicode character escape code\0"
                                            as *const u8 as *const libc::c_char,
                                    );
                                    current_block = 8114179180390253173;
                                    break 's_58;
                                } else {
                                    if value <= 0x7F {
                                        let fresh573 = string.pointer;
                                        string.pointer = string.pointer.wrapping_offset(1);
                                        *fresh573 = value as yaml_char_t;
                                    } else if value <= 0x7FF {
                                        let fresh574 = string.pointer;
                                        string.pointer = string.pointer.wrapping_offset(1);
                                        *fresh574 = 0xC0_u32.force_add(value >> 6) as yaml_char_t;
                                        let fresh575 = string.pointer;
                                        string.pointer = string.pointer.wrapping_offset(1);
                                        *fresh575 = 0x80_u32.force_add(value & 0x3F) as yaml_char_t;
                                    } else if value <= 0xFFFF {
                                        let fresh576 = string.pointer;
                                        string.pointer = string.pointer.wrapping_offset(1);
                                        *fresh576 = 0xE0_u32.force_add(value >> 12) as yaml_char_t;
                                        let fresh577 = string.pointer;
                                        string.pointer = string.pointer.wrapping_offset(1);
                                        *fresh577 = 0x80_u32.force_add(value >> 6 & 0x3F)
                                            as yaml_char_t;
                                        let fresh578 = string.pointer;
                                        string.pointer = string.pointer.wrapping_offset(1);
                                        *fresh578 = 0x80_u32.force_add(value & 0x3F) as yaml_char_t;
                                    } else {
                                        let fresh579 = string.pointer;
                                        string.pointer = string.pointer.wrapping_offset(1);
                                        *fresh579 = 0xF0_u32.force_add(value >> 18) as yaml_char_t;
                                        let fresh580 = string.pointer;
                                        string.pointer = string.pointer.wrapping_offset(1);
                                        *fresh580 = 0x80_u32.force_add(value >> 12 & 0x3F)
                                            as yaml_char_t;
                                        let fresh581 = string.pointer;
                                        string.pointer = string.pointer.wrapping_offset(1);
                                        *fresh581 = 0x80_u32.force_add(value >> 6 & 0x3F)
                                            as yaml_char_t;
                                        let fresh582 = string.pointer;
                                        string.pointer = string.pointer.wrapping_offset(1);
                                        *fresh582 = 0x80_u32.force_add(value & 0x3F) as yaml_char_t;
                                    }
                                    k = 0_u64;
                                    while k < code_length {
                                        skip(parser);
                                        k = k.force_add(1);
                                    }
                                }
                            }
                        } else {
                            read(parser, &raw mut string);
                        }
                    }
                    if cache(parser, 2_u64).fail {
                        current_block = 8114179180390253173;
                        break 's_58;
                    }
                }
                if cache(parser, 1_u64).fail {
                    current_block = 8114179180390253173;
                    break;
                }
                if *(*parser).buffer.pointer == if single { b'\'' } else { b'"' } {
                    current_block = 7468767852762055642;
                    break;
                }
                if cache(parser, 1_u64).fail {
                    current_block = 8114179180390253173;
                    break;
                }
                while *(*parser).buffer.pointer.offset(0) == b' '
                    || *(*parser).buffer.pointer.offset(0) == b'\t'
                    || (*(*parser).buffer.pointer.offset(0) == b'\r'
                        || *(*parser).buffer.pointer.offset(0) == b'\n'
                        || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 1).try_into().unwrap()) == b'\x85'
                        || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 2).try_into().unwrap()) == b'\xA8'
                        || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((0 + 2).try_into().unwrap()) == b'\xA9')
                {
                    if *(*parser).buffer.pointer.offset(0) == b' '
                        || *(*parser).buffer.pointer.offset(0) == b'\t'
                    {
                        if !leading_blanks {
                            read(parser, &raw mut whitespaces);
                        } else {
                            skip(parser);
                        }
                    } else {
                        if cache(parser, 2_u64).fail {
                            current_block = 8114179180390253173;
                            break 's_58;
                        }
                        if !leading_blanks {
                            {
                                whitespaces.pointer = whitespaces.start;
                                let _ = memset(
                                    whitespaces.start as *mut libc::c_void,
                                    0,
                                    whitespaces.end.offset_from(whitespaces.start)
                                        as libc::c_ulong,
                                );
                            };
                            read_line(parser, &raw mut leading_break);
                            leading_blanks = true;
                        } else {
                            read_line(parser, &raw mut trailing_breaks);
                        }
                    }
                    if cache(parser, 1_u64).fail {
                        current_block = 8114179180390253173;
                        break 's_58;
                    }
                }
                if leading_blanks {
                    if *leading_break.start == b'\n' {
                        if *trailing_breaks.start == b'\0' {
                            let new_end = string.pointer.wrapping_add(5);
                            if new_end >= string.end {
                                yaml_string_extend(
                                    &raw mut string.start,
                                    &raw mut string.pointer,
                                    &raw mut string.end,
                                );
                            }
                            let fresh711 = string.pointer;
                            string.pointer = string.pointer.wrapping_offset(1);
                            *fresh711 = b' ';
                        } else {
                            {
                                let a_len = string.pointer.offset_from(string.start)
                                    as usize;
                                let b_len = trailing_breaks
                                    .pointer
                                    .offset_from(trailing_breaks.start) as usize;
                                if a_len.checked_add(b_len).is_some()
                                    && string.pointer.add(b_len) <= string.end
                                {
                                    yaml_string_join(
                                        &raw mut string.start,
                                        &raw mut string.pointer,
                                        &raw mut string.end,
                                        &raw mut trailing_breaks.start,
                                        &raw mut trailing_breaks.pointer,
                                        &raw mut trailing_breaks.end,
                                    );
                                    trailing_breaks.pointer = trailing_breaks.start;
                                } else {
                                    {
                                        ::core::panicking::panic_fmt(
                                            format_args!("String join would overflow memory bounds"),
                                        );
                                    };
                                }
                            };
                            {
                                trailing_breaks.pointer = trailing_breaks.start;
                                let _ = memset(
                                    trailing_breaks.start as *mut libc::c_void,
                                    0,
                                    trailing_breaks.end.offset_from(trailing_breaks.start)
                                        as libc::c_ulong,
                                );
                            };
                        }
                        {
                            leading_break.pointer = leading_break.start;
                            let _ = memset(
                                leading_break.start as *mut libc::c_void,
                                0,
                                leading_break.end.offset_from(leading_break.start)
                                    as libc::c_ulong,
                            );
                        };
                    } else {
                        {
                            let a_len = string.pointer.offset_from(string.start)
                                as usize;
                            let b_len = leading_break
                                .pointer
                                .offset_from(leading_break.start) as usize;
                            if a_len.checked_add(b_len).is_some()
                                && string.pointer.add(b_len) <= string.end
                            {
                                yaml_string_join(
                                    &raw mut string.start,
                                    &raw mut string.pointer,
                                    &raw mut string.end,
                                    &raw mut leading_break.start,
                                    &raw mut leading_break.pointer,
                                    &raw mut leading_break.end,
                                );
                                leading_break.pointer = leading_break.start;
                            } else {
                                {
                                    ::core::panicking::panic_fmt(
                                        format_args!("String join would overflow memory bounds"),
                                    );
                                };
                            }
                        };
                        {
                            let a_len = string.pointer.offset_from(string.start)
                                as usize;
                            let b_len = trailing_breaks
                                .pointer
                                .offset_from(trailing_breaks.start) as usize;
                            if a_len.checked_add(b_len).is_some()
                                && string.pointer.add(b_len) <= string.end
                            {
                                yaml_string_join(
                                    &raw mut string.start,
                                    &raw mut string.pointer,
                                    &raw mut string.end,
                                    &raw mut trailing_breaks.start,
                                    &raw mut trailing_breaks.pointer,
                                    &raw mut trailing_breaks.end,
                                );
                                trailing_breaks.pointer = trailing_breaks.start;
                            } else {
                                {
                                    ::core::panicking::panic_fmt(
                                        format_args!("String join would overflow memory bounds"),
                                    );
                                };
                            }
                        };
                        {
                            leading_break.pointer = leading_break.start;
                            let _ = memset(
                                leading_break.start as *mut libc::c_void,
                                0,
                                leading_break.end.offset_from(leading_break.start)
                                    as libc::c_ulong,
                            );
                        };
                        {
                            trailing_breaks.pointer = trailing_breaks.start;
                            let _ = memset(
                                trailing_breaks.start as *mut libc::c_void,
                                0,
                                trailing_breaks.end.offset_from(trailing_breaks.start)
                                    as libc::c_ulong,
                            );
                        };
                    }
                } else {
                    {
                        let a_len = string.pointer.offset_from(string.start) as usize;
                        let b_len = whitespaces.pointer.offset_from(whitespaces.start)
                            as usize;
                        if a_len.checked_add(b_len).is_some()
                            && string.pointer.add(b_len) <= string.end
                        {
                            yaml_string_join(
                                &raw mut string.start,
                                &raw mut string.pointer,
                                &raw mut string.end,
                                &raw mut whitespaces.start,
                                &raw mut whitespaces.pointer,
                                &raw mut whitespaces.end,
                            );
                            whitespaces.pointer = whitespaces.start;
                        } else {
                            {
                                ::core::panicking::panic_fmt(
                                    format_args!("String join would overflow memory bounds"),
                                );
                            };
                        }
                    };
                    {
                        whitespaces.pointer = whitespaces.start;
                        let _ = memset(
                            whitespaces.start as *mut libc::c_void,
                            0,
                            whitespaces.end.offset_from(whitespaces.start)
                                as libc::c_ulong,
                        );
                    };
                }
            }
        }
        if current_block != 8114179180390253173 {
            skip(parser);
            end_mark = (*parser).mark;
            let _ = memset(
                token as *mut libc::c_void,
                0,
                size_of::<YamlTokenT>() as libc::c_ulong,
            );
            (*token).type_ = YamlScalarToken;
            (*token).start_mark = start_mark;
            (*token).end_mark = end_mark;
            let fresh716 = &raw mut (*token).data.scalar.value;
            *fresh716 = string.start;
            (*token).data.scalar.length = string.pointer.c_offset_from(string.start)
                as size_t;
            (*token).data.scalar.style = if single {
                YamlSingleQuotedScalarStyle
            } else {
                YamlDoubleQuotedScalarStyle
            };
            {
                yaml_free(leading_break.start as *mut libc::c_void);
                leading_break.end = ptr::null_mut::<yaml_char_t>();
                leading_break.pointer = leading_break.end;
                leading_break.start = leading_break.pointer;
            };
            {
                yaml_free(trailing_breaks.start as *mut libc::c_void);
                trailing_breaks.end = ptr::null_mut::<yaml_char_t>();
                trailing_breaks.pointer = trailing_breaks.end;
                trailing_breaks.start = trailing_breaks.pointer;
            };
            {
                yaml_free(whitespaces.start as *mut libc::c_void);
                whitespaces.end = ptr::null_mut::<yaml_char_t>();
                whitespaces.pointer = whitespaces.end;
                whitespaces.start = whitespaces.pointer;
            };
            return OK;
        }
        {
            yaml_free(string.start as *mut libc::c_void);
            string.end = ptr::null_mut::<yaml_char_t>();
            string.pointer = string.end;
            string.start = string.pointer;
        };
        {
            yaml_free(leading_break.start as *mut libc::c_void);
            leading_break.end = ptr::null_mut::<yaml_char_t>();
            leading_break.pointer = leading_break.end;
            leading_break.start = leading_break.pointer;
        };
        {
            yaml_free(trailing_breaks.start as *mut libc::c_void);
            trailing_breaks.end = ptr::null_mut::<yaml_char_t>();
            trailing_breaks.pointer = trailing_breaks.end;
            trailing_breaks.start = trailing_breaks.pointer;
        };
        {
            yaml_free(whitespaces.start as *mut libc::c_void);
            whitespaces.end = ptr::null_mut::<yaml_char_t>();
            whitespaces.pointer = whitespaces.end;
            whitespaces.start = whitespaces.pointer;
        };
        FAIL
    }
    unsafe fn yaml_parser_scan_plain_scalar(
        parser: *mut YamlParserT,
        token: *mut YamlTokenT,
    ) -> Success {
        let current_block: u64;
        let mut end_mark: YamlMarkT;
        let mut string = NULL_STRING;
        let mut leading_break = NULL_STRING;
        let mut trailing_breaks = NULL_STRING;
        let mut whitespaces = NULL_STRING;
        let mut leading_blanks = false;
        let indent: libc::c_int = (*parser).indent + 1;
        {
            string.start = yaml_malloc(16) as *mut yaml_char_t;
            if !string.start.is_null() {
                let _ = memset(string.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            string.pointer = string.start;
            string.end = string.start.wrapping_add(16);
            let _ = memset(string.start as *mut libc::c_void, 0, 16);
        };
        {
            leading_break.start = yaml_malloc(16) as *mut yaml_char_t;
            if !leading_break.start.is_null() {
                let _ = memset(leading_break.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            leading_break.pointer = leading_break.start;
            leading_break.end = leading_break.start.wrapping_add(16);
            let _ = memset(leading_break.start as *mut libc::c_void, 0, 16);
        };
        {
            trailing_breaks.start = yaml_malloc(16) as *mut yaml_char_t;
            if !trailing_breaks.start.is_null() {
                let _ = memset(trailing_breaks.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            trailing_breaks.pointer = trailing_breaks.start;
            trailing_breaks.end = trailing_breaks.start.wrapping_add(16);
            let _ = memset(trailing_breaks.start as *mut libc::c_void, 0, 16);
        };
        {
            whitespaces.start = yaml_malloc(16) as *mut yaml_char_t;
            if !whitespaces.start.is_null() {
                let _ = memset(whitespaces.start as *mut libc::c_void, 0, 16);
            } else {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Failed to allocate memory for string"),
                    );
                };
            }
            whitespaces.pointer = whitespaces.start;
            whitespaces.end = whitespaces.start.wrapping_add(16);
            let _ = memset(whitespaces.start as *mut libc::c_void, 0, 16);
        };
        end_mark = (*parser).mark;
        let start_mark: YamlMarkT = end_mark;
        's_57: loop {
            if cache(parser, 4_u64).fail {
                current_block = 16642808987012640029;
                break;
            }
            if (*parser).mark.column == 0_u64
                && (*(*parser).buffer.pointer.offset(0) == b'-'
                    && *(*parser).buffer.pointer.offset(1) == b'-'
                    && *(*parser).buffer.pointer.offset(2) == b'-'
                    || *(*parser).buffer.pointer.offset(0) == b'.'
                        && *(*parser).buffer.pointer.offset(1) == b'.'
                        && *(*parser).buffer.pointer.offset(2) == b'.')
                && (*(*parser).buffer.pointer.offset(3) == b' '
                    || *(*parser).buffer.pointer.offset(3) == b'\t'
                    || (*(*parser).buffer.pointer.offset(3) == b'\r'
                        || *(*parser).buffer.pointer.offset(3) == b'\n'
                        || *(*parser).buffer.pointer.offset(3) == b'\xC2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((3 + 1).try_into().unwrap()) == b'\x85'
                        || *(*parser).buffer.pointer.offset(3) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((3 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((3 + 2).try_into().unwrap()) == b'\xA8'
                        || *(*parser).buffer.pointer.offset(3) == b'\xE2'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((3 + 1).try_into().unwrap()) == b'\x80'
                            && *(*parser)
                                .buffer
                                .pointer
                                .offset((3 + 2).try_into().unwrap()) == b'\xA9'
                        || *(*parser).buffer.pointer.offset(3) == b'\0'))
            {
                current_block = 6281126495347172768;
                break;
            }
            if *(*parser).buffer.pointer == b'#' {
                current_block = 6281126495347172768;
                break;
            }
            while !(*(*parser).buffer.pointer.offset(0) == b' '
                || *(*parser).buffer.pointer.offset(0) == b'\t'
                || (*(*parser).buffer.pointer.offset(0) == b'\r'
                    || *(*parser).buffer.pointer.offset(0) == b'\n'
                    || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                        && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                            == b'\x85'
                    || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                        && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                            == b'\x80'
                        && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                            == b'\xA8'
                    || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                        && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                            == b'\x80'
                        && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                            == b'\xA9' || *(*parser).buffer.pointer.offset(0) == b'\0'))
            {
                if (*parser).flow_level != 0 && *(*parser).buffer.pointer == b':'
                    && (*(*parser).buffer.pointer.offset(1) == b','
                        || *(*parser).buffer.pointer.offset(1) == b'?'
                        || *(*parser).buffer.pointer.offset(1) == b'['
                        || *(*parser).buffer.pointer.offset(1) == b']'
                        || *(*parser).buffer.pointer.offset(1) == b'{'
                        || *(*parser).buffer.pointer.offset(1) == b'}')
                {
                    yaml_parser_set_scanner_error(
                        parser,
                        b"while scanning a plain scalar\0" as *const u8
                            as *const libc::c_char,
                        start_mark,
                        b"found unexpected ':'\0" as *const u8 as *const libc::c_char,
                    );
                    current_block = 16642808987012640029;
                    break 's_57;
                } else {
                    if *(*parser).buffer.pointer == b':'
                        && (*(*parser).buffer.pointer.offset(1) == b' '
                            || *(*parser).buffer.pointer.offset(1) == b'\t'
                            || (*(*parser).buffer.pointer.offset(1) == b'\r'
                                || *(*parser).buffer.pointer.offset(1) == b'\n'
                                || *(*parser).buffer.pointer.offset(1) == b'\xC2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((1 + 1).try_into().unwrap()) == b'\x85'
                                || *(*parser).buffer.pointer.offset(1) == b'\xE2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((1 + 1).try_into().unwrap()) == b'\x80'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((1 + 2).try_into().unwrap()) == b'\xA8'
                                || *(*parser).buffer.pointer.offset(1) == b'\xE2'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((1 + 1).try_into().unwrap()) == b'\x80'
                                    && *(*parser)
                                        .buffer
                                        .pointer
                                        .offset((1 + 2).try_into().unwrap()) == b'\xA9'
                                || *(*parser).buffer.pointer.offset(1) == b'\0'))
                        || (*parser).flow_level != 0
                            && (*(*parser).buffer.pointer == b','
                                || *(*parser).buffer.pointer == b'['
                                || *(*parser).buffer.pointer == b']'
                                || *(*parser).buffer.pointer == b'{'
                                || *(*parser).buffer.pointer == b'}')
                    {
                        break;
                    }
                    if leading_blanks || whitespaces.start != whitespaces.pointer {
                        if leading_blanks {
                            if *leading_break.start == b'\n' {
                                if *trailing_breaks.start == b'\0' {
                                    let new_end = string.pointer.wrapping_add(5);
                                    if new_end >= string.end {
                                        yaml_string_extend(
                                            &raw mut string.start,
                                            &raw mut string.pointer,
                                            &raw mut string.end,
                                        );
                                    }
                                    let fresh717 = string.pointer;
                                    string.pointer = string.pointer.wrapping_offset(1);
                                    *fresh717 = b' ';
                                } else {
                                    {
                                        let a_len = string.pointer.offset_from(string.start)
                                            as usize;
                                        let b_len = trailing_breaks
                                            .pointer
                                            .offset_from(trailing_breaks.start) as usize;
                                        if a_len.checked_add(b_len).is_some()
                                            && string.pointer.add(b_len) <= string.end
                                        {
                                            yaml_string_join(
                                                &raw mut string.start,
                                                &raw mut string.pointer,
                                                &raw mut string.end,
                                                &raw mut trailing_breaks.start,
                                                &raw mut trailing_breaks.pointer,
                                                &raw mut trailing_breaks.end,
                                            );
                                            trailing_breaks.pointer = trailing_breaks.start;
                                        } else {
                                            {
                                                ::core::panicking::panic_fmt(
                                                    format_args!("String join would overflow memory bounds"),
                                                );
                                            };
                                        }
                                    };
                                    {
                                        trailing_breaks.pointer = trailing_breaks.start;
                                        let _ = memset(
                                            trailing_breaks.start as *mut libc::c_void,
                                            0,
                                            trailing_breaks.end.offset_from(trailing_breaks.start)
                                                as libc::c_ulong,
                                        );
                                    };
                                }
                                {
                                    leading_break.pointer = leading_break.start;
                                    let _ = memset(
                                        leading_break.start as *mut libc::c_void,
                                        0,
                                        leading_break.end.offset_from(leading_break.start)
                                            as libc::c_ulong,
                                    );
                                };
                            } else {
                                {
                                    let a_len = string.pointer.offset_from(string.start)
                                        as usize;
                                    let b_len = leading_break
                                        .pointer
                                        .offset_from(leading_break.start) as usize;
                                    if a_len.checked_add(b_len).is_some()
                                        && string.pointer.add(b_len) <= string.end
                                    {
                                        yaml_string_join(
                                            &raw mut string.start,
                                            &raw mut string.pointer,
                                            &raw mut string.end,
                                            &raw mut leading_break.start,
                                            &raw mut leading_break.pointer,
                                            &raw mut leading_break.end,
                                        );
                                        leading_break.pointer = leading_break.start;
                                    } else {
                                        {
                                            ::core::panicking::panic_fmt(
                                                format_args!("String join would overflow memory bounds"),
                                            );
                                        };
                                    }
                                };
                                {
                                    let a_len = string.pointer.offset_from(string.start)
                                        as usize;
                                    let b_len = trailing_breaks
                                        .pointer
                                        .offset_from(trailing_breaks.start) as usize;
                                    if a_len.checked_add(b_len).is_some()
                                        && string.pointer.add(b_len) <= string.end
                                    {
                                        yaml_string_join(
                                            &raw mut string.start,
                                            &raw mut string.pointer,
                                            &raw mut string.end,
                                            &raw mut trailing_breaks.start,
                                            &raw mut trailing_breaks.pointer,
                                            &raw mut trailing_breaks.end,
                                        );
                                        trailing_breaks.pointer = trailing_breaks.start;
                                    } else {
                                        {
                                            ::core::panicking::panic_fmt(
                                                format_args!("String join would overflow memory bounds"),
                                            );
                                        };
                                    }
                                };
                                {
                                    leading_break.pointer = leading_break.start;
                                    let _ = memset(
                                        leading_break.start as *mut libc::c_void,
                                        0,
                                        leading_break.end.offset_from(leading_break.start)
                                            as libc::c_ulong,
                                    );
                                };
                                {
                                    trailing_breaks.pointer = trailing_breaks.start;
                                    let _ = memset(
                                        trailing_breaks.start as *mut libc::c_void,
                                        0,
                                        trailing_breaks.end.offset_from(trailing_breaks.start)
                                            as libc::c_ulong,
                                    );
                                };
                            }
                            leading_blanks = false;
                        } else {
                            {
                                let a_len = string.pointer.offset_from(string.start)
                                    as usize;
                                let b_len = whitespaces
                                    .pointer
                                    .offset_from(whitespaces.start) as usize;
                                if a_len.checked_add(b_len).is_some()
                                    && string.pointer.add(b_len) <= string.end
                                {
                                    yaml_string_join(
                                        &raw mut string.start,
                                        &raw mut string.pointer,
                                        &raw mut string.end,
                                        &raw mut whitespaces.start,
                                        &raw mut whitespaces.pointer,
                                        &raw mut whitespaces.end,
                                    );
                                    whitespaces.pointer = whitespaces.start;
                                } else {
                                    {
                                        ::core::panicking::panic_fmt(
                                            format_args!("String join would overflow memory bounds"),
                                        );
                                    };
                                }
                            };
                            {
                                whitespaces.pointer = whitespaces.start;
                                let _ = memset(
                                    whitespaces.start as *mut libc::c_void,
                                    0,
                                    whitespaces.end.offset_from(whitespaces.start)
                                        as libc::c_ulong,
                                );
                            };
                        }
                    }
                    read(parser, &raw mut string);
                    end_mark = (*parser).mark;
                    if cache(parser, 2_u64).fail {
                        current_block = 16642808987012640029;
                        break 's_57;
                    }
                }
            }
            if !(*(*parser).buffer.pointer.offset(0) == b' '
                || *(*parser).buffer.pointer.offset(0) == b'\t'
                || (*(*parser).buffer.pointer.offset(0) == b'\r'
                    || *(*parser).buffer.pointer.offset(0) == b'\n'
                    || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                        && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                            == b'\x85'
                    || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                        && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                            == b'\x80'
                        && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                            == b'\xA8'
                    || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                        && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                            == b'\x80'
                        && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                            == b'\xA9'))
            {
                current_block = 6281126495347172768;
                break;
            }
            if cache(parser, 1_u64).fail {
                current_block = 16642808987012640029;
                break;
            }
            while *(*parser).buffer.pointer.offset(0) == b' '
                || *(*parser).buffer.pointer.offset(0) == b'\t'
                || (*(*parser).buffer.pointer.offset(0) == b'\r'
                    || *(*parser).buffer.pointer.offset(0) == b'\n'
                    || *(*parser).buffer.pointer.offset(0) == b'\xC2'
                        && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                            == b'\x85'
                    || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                        && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                            == b'\x80'
                        && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                            == b'\xA8'
                    || *(*parser).buffer.pointer.offset(0) == b'\xE2'
                        && *(*parser).buffer.pointer.offset((0 + 1).try_into().unwrap())
                            == b'\x80'
                        && *(*parser).buffer.pointer.offset((0 + 2).try_into().unwrap())
                            == b'\xA9')
            {
                if *(*parser).buffer.pointer.offset(0) == b' '
                    || *(*parser).buffer.pointer.offset(0) == b'\t'
                {
                    if leading_blanks && ((*parser).mark.column as libc::c_int) < indent
                        && *(*parser).buffer.pointer.offset(0) == b'\t'
                    {
                        yaml_parser_set_scanner_error(
                            parser,
                            b"while scanning a plain scalar\0" as *const u8
                                as *const libc::c_char,
                            start_mark,
                            b"found a tab character that violates indentation\0"
                                as *const u8 as *const libc::c_char,
                        );
                        current_block = 16642808987012640029;
                        break 's_57;
                    } else if !leading_blanks {
                        read(parser, &raw mut whitespaces);
                    } else {
                        skip(parser);
                    }
                } else {
                    if cache(parser, 2_u64).fail {
                        current_block = 16642808987012640029;
                        break 's_57;
                    }
                    if !leading_blanks {
                        {
                            whitespaces.pointer = whitespaces.start;
                            let _ = memset(
                                whitespaces.start as *mut libc::c_void,
                                0,
                                whitespaces.end.offset_from(whitespaces.start)
                                    as libc::c_ulong,
                            );
                        };
                        read_line(parser, &raw mut leading_break);
                        leading_blanks = true;
                    } else {
                        read_line(parser, &raw mut trailing_breaks);
                    }
                }
                if cache(parser, 1_u64).fail {
                    current_block = 16642808987012640029;
                    break 's_57;
                }
            }
            if (*parser).flow_level == 0
                && ((*parser).mark.column as libc::c_int) < indent
            {
                current_block = 6281126495347172768;
                break;
            }
        }
        if current_block != 16642808987012640029 {
            let _ = memset(
                token as *mut libc::c_void,
                0,
                size_of::<YamlTokenT>() as libc::c_ulong,
            );
            (*token).type_ = YamlScalarToken;
            (*token).start_mark = start_mark;
            (*token).end_mark = end_mark;
            let fresh842 = &raw mut (*token).data.scalar.value;
            *fresh842 = string.start;
            (*token).data.scalar.length = string.pointer.c_offset_from(string.start)
                as size_t;
            (*token).data.scalar.style = YamlPlainScalarStyle;
            if leading_blanks {
                (*parser).simple_key_allowed = true;
            }
            {
                yaml_free(leading_break.start as *mut libc::c_void);
                leading_break.end = ptr::null_mut::<yaml_char_t>();
                leading_break.pointer = leading_break.end;
                leading_break.start = leading_break.pointer;
            };
            {
                yaml_free(trailing_breaks.start as *mut libc::c_void);
                trailing_breaks.end = ptr::null_mut::<yaml_char_t>();
                trailing_breaks.pointer = trailing_breaks.end;
                trailing_breaks.start = trailing_breaks.pointer;
            };
            {
                yaml_free(whitespaces.start as *mut libc::c_void);
                whitespaces.end = ptr::null_mut::<yaml_char_t>();
                whitespaces.pointer = whitespaces.end;
                whitespaces.start = whitespaces.pointer;
            };
            return OK;
        }
        {
            yaml_free(string.start as *mut libc::c_void);
            string.end = ptr::null_mut::<yaml_char_t>();
            string.pointer = string.end;
            string.start = string.pointer;
        };
        {
            yaml_free(leading_break.start as *mut libc::c_void);
            leading_break.end = ptr::null_mut::<yaml_char_t>();
            leading_break.pointer = leading_break.end;
            leading_break.start = leading_break.pointer;
        };
        {
            yaml_free(trailing_breaks.start as *mut libc::c_void);
            trailing_breaks.end = ptr::null_mut::<yaml_char_t>();
            trailing_breaks.pointer = trailing_breaks.end;
            trailing_breaks.start = trailing_breaks.pointer;
        };
        {
            yaml_free(whitespaces.start as *mut libc::c_void);
            whitespaces.end = ptr::null_mut::<yaml_char_t>();
            whitespaces.pointer = whitespaces.end;
            whitespaces.start = whitespaces.pointer;
        };
        FAIL
    }
}
/// Success and Failure types for LibYML.
///
/// This module provides types for representing the success and failure of various operations within the library.
pub mod success {
    use core::ops::Deref;
    /// Constant representing a successful operation.
    pub const OK: Success = Success { ok: true };
    /// Constant representing a failed operation.
    pub const FAIL: Success = Success { ok: false };
    /// Structure representing the success state of an operation.
    #[must_use]
    pub struct Success {
        /// Boolean indicating whether the operation was successful.
        pub ok: bool,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Success {}
    #[automatically_derived]
    impl ::core::clone::Clone for Success {
        #[inline]
        fn clone(&self) -> Success {
            let _: ::core::clone::AssertParamIsClone<bool>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Success {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "Success",
                "ok",
                &&self.ok,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for Success {
        #[inline]
        fn default() -> Success {
            Success {
                ok: ::core::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Success {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Success {
        #[inline]
        fn eq(&self, other: &Success) -> bool {
            self.ok == other.ok
        }
    }
    /// Structure representing the failure state of an operation.
    pub struct Failure {
        /// Boolean indicating whether the operation failed.
        pub fail: bool,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Failure {}
    #[automatically_derived]
    impl ::core::clone::Clone for Failure {
        #[inline]
        fn clone(&self) -> Failure {
            let _: ::core::clone::AssertParamIsClone<bool>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Failure {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "Failure",
                "fail",
                &&self.fail,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for Failure {
        #[inline]
        fn default() -> Failure {
            Failure {
                fail: ::core::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for Failure {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for Failure {
        #[inline]
        fn eq(&self, other: &Failure) -> bool {
            self.fail == other.fail
        }
    }
    impl Deref for Success {
        type Target = Failure;
        /// Returns a reference to the corresponding `Failure` instance based on the success state.
        fn deref(&self) -> &Self::Target {
            if self.ok { &Failure { fail: false } } else { &Failure { fail: true } }
        }
    }
    /// Checks if the given `Success` result indicates a successful operation.
    ///
    /// # Arguments
    ///
    /// * `result` - A `Success` struct representing the result of an operation.
    ///
    /// # Returns
    ///
    /// * `true` if the operation was successful.
    /// * `false` if the operation failed.
    pub fn is_success(result: Success) -> bool {
        result.ok
    }
    /// Checks if the given `Success` result indicates a failure operation.
    ///
    /// # Arguments
    ///
    /// * `result` - A `Success` struct representing the result of an operation.
    ///
    /// # Returns
    ///
    /// * `true` if the operation failed.
    /// * `false` if the operation was successful.
    pub fn is_failure(result: Success) -> bool {
        !result.ok
    }
}
mod writer {
    use crate::{
        libc, ops::ForceAdd as _, success::{Success, FAIL, OK},
        yaml::size_t, PointerExt, YamlAnyEncoding, YamlEmitterT, YamlUtf16leEncoding,
        YamlUtf8Encoding, YamlWriterError,
    };
    use core::ptr::addr_of_mut;
    /// Sets the writer error for the emitter.
    ///
    /// This function sets the error of the emitter and updates the problem string.
    ///
    /// # Arguments
    ///
    /// * `emitter` - A pointer to the YamlEmitterT struct.
    /// * `problem` - A pointer to the string that describes the error.
    ///
    /// # Returns
    ///
    /// * `Success::FAIL` - The function sets the error of the emitter and returns FAIL.
    ///
    /// # Safety
    ///
    /// This function is marked unsafe as it dereferences the pointer passed to it.
    /// The caller must ensure that the pointer is valid and points to a valid memory location.
    /// The caller must also ensure that the pointer is not null.
    ///
    pub(crate) unsafe fn yaml_emitter_set_writer_error(
        emitter: *mut YamlEmitterT,
        problem: *const libc::c_char,
    ) -> Success {
        (*emitter).error = YamlWriterError;
        let fresh0 = &raw mut (*emitter).problem;
        *fresh0 = problem;
        FAIL
    }
    /// Flushes the buffer of the emitter and writes the content to the output stream.
    ///
    ///  This function is called when the emitter needs to flush its buffer to the output stream.
    ///  It first checks if the emitter is not null and if the write handler is not null.
    ///  It also checks if the encoding of the emitter is not YamlAnyEncoding.
    ///
    ///  If the conditions are met, it updates the last and pointer of the buffer.
    ///  If the encoding is YamlUtf8Encoding, it writes the content of the buffer to the output stream.
    ///  If an error occurs during the write operation, it sets the error of the emitter and returns FAIL.
    ///
    ///  If the encoding is not YamlUtf8Encoding, it writes the content of the buffer to the raw buffer.
    ///  It then writes the raw buffer to the output stream.
    ///  If an error occurs during the write operation, it sets the error of the emitter and returns FAIL.
    ///  If the write operation is successful, it returns OK.
    ///
    ///  # Arguments
    ///
    /// * `emitter` - A pointer to the YamlEmitterT struct.
    ///
    ///  # Returns
    ///
    /// * `Success` - An enum representing the success or failure of the operation.
    ///
    /// # Safety
    ///
    /// * The function is marked unsafe as it dereferences the pointer passed to it.
    /// * The caller must ensure that the pointer is valid and points to a valid memory location.
    /// * The caller must also ensure that the pointer is not null.
    /// * The caller must ensure that the write handler is not null.
    /// * The caller must ensure that the encoding is not YamlAnyEncoding.
    /// * The caller must ensure that the write handler is a valid function pointer.
    ///
    pub unsafe fn yaml_emitter_flush(emitter: *mut YamlEmitterT) -> Success {
        if !!emitter.is_null() {
            crate::externs::__assert_fail("!emitter.is_null()", "src/writer.rs", 75u32);
        }
        if !((*emitter).write_handler).is_some() {
            crate::externs::__assert_fail(
                "((*emitter).write_handler).is_some()",
                "src/writer.rs",
                76u32,
            );
        }
        if !((*emitter).encoding != YamlAnyEncoding) {
            crate::externs::__assert_fail(
                "(*emitter).encoding != YamlAnyEncoding",
                "src/writer.rs",
                77u32,
            );
        }
        let fresh1 = &raw mut (*emitter).buffer.last;
        *fresh1 = (*emitter).buffer.pointer;
        let fresh2 = &raw mut (*emitter).buffer.pointer;
        *fresh2 = (*emitter).buffer.start;
        if (*emitter).buffer.start == (*emitter).buffer.last {
            return OK;
        }
        if (*emitter).encoding == YamlUtf8Encoding {
            if (*emitter)
                .write_handler
                .expect(
                    "non-null function pointer",
                )(
                (*emitter).write_handler_data,
                (*emitter).buffer.start,
                (*emitter).buffer.last.c_offset_from((*emitter).buffer.start) as size_t,
            ) != 0
            {
                let fresh3 = &raw mut (*emitter).buffer.last;
                *fresh3 = (*emitter).buffer.start;
                let fresh4 = &raw mut (*emitter).buffer.pointer;
                *fresh4 = (*emitter).buffer.start;
                return OK;
            } else {
                return yaml_emitter_set_writer_error(
                    emitter,
                    b"write error\0" as *const u8 as *const libc::c_char,
                );
            }
        }
        let low: libc::c_int = if (*emitter).encoding == YamlUtf16leEncoding {
            0
        } else {
            1
        };
        let high: libc::c_int = if (*emitter).encoding == YamlUtf16leEncoding {
            1
        } else {
            0
        };
        while (*emitter).buffer.pointer != (*emitter).buffer.last {
            let mut octet: libc::c_uchar;
            let mut value: libc::c_uint;
            let mut k: size_t;
            octet = *(*emitter).buffer.pointer;
            let width: libc::c_uint = if octet & 0x80 == 0 {
                1
            } else if octet & 0xE0 == 0xC0 {
                2
            } else if octet & 0xF0 == 0xE0 {
                3
            } else if octet & 0xF8 == 0xF0 {
                4
            } else {
                0
            } as libc::c_uint;
            value = if octet & 0x80 == 0 {
                octet & 0x7F
            } else if octet & 0xE0 == 0xC0 {
                octet & 0x1F
            } else if octet & 0xF0 == 0xE0 {
                octet & 0xF
            } else if octet & 0xF8 == 0xF0 {
                octet & 0x7
            } else {
                0
            } as libc::c_uint;
            k = 1_u64;
            while k < width as libc::c_ulong {
                octet = *(*emitter).buffer.pointer.wrapping_offset(k as isize);
                value = (value << 6).force_add((octet & 0x3F) as libc::c_uint);
                k = k.force_add(1);
            }
            let fresh5 = &raw mut (*emitter).buffer.pointer;
            *fresh5 = (*fresh5).wrapping_offset(width as isize);
            if value < 0x10000 {
                *(*emitter).raw_buffer.last.wrapping_offset(high as isize) = (value >> 8)
                    as libc::c_uchar;
                *(*emitter).raw_buffer.last.wrapping_offset(low as isize) = (value
                    & 0xFF) as libc::c_uchar;
                let fresh6 = &raw mut (*emitter).raw_buffer.last;
                *fresh6 = (*fresh6).wrapping_offset(2_isize);
            } else {
                value = value.wrapping_sub(0x10000);
                *(*emitter).raw_buffer.last.wrapping_offset(high as isize) = 0xD8_u32
                    .force_add(value >> 18) as libc::c_uchar;
                *(*emitter).raw_buffer.last.wrapping_offset(low as isize) = (value >> 10
                    & 0xFF) as libc::c_uchar;
                *(*emitter).raw_buffer.last.wrapping_offset((high + 2) as isize) = 0xDC_u32
                    .force_add(value >> 8 & 0xFF) as libc::c_uchar;
                *(*emitter).raw_buffer.last.wrapping_offset((low + 2) as isize) = (value
                    & 0xFF) as libc::c_uchar;
                let fresh7 = &raw mut (*emitter).raw_buffer.last;
                *fresh7 = (*fresh7).wrapping_offset(4_isize);
            }
        }
        if (*emitter)
            .write_handler
            .expect(
                "non-null function pointer",
            )(
            (*emitter).write_handler_data,
            (*emitter).raw_buffer.start,
            (*emitter).raw_buffer.last.c_offset_from((*emitter).raw_buffer.start)
                as size_t,
        ) != 0
        {
            let fresh8 = &raw mut (*emitter).buffer.last;
            *fresh8 = (*emitter).buffer.start;
            let fresh9 = &raw mut (*emitter).buffer.pointer;
            *fresh9 = (*emitter).buffer.start;
            let fresh10 = &raw mut (*emitter).raw_buffer.last;
            *fresh10 = (*emitter).raw_buffer.start;
            let fresh11 = &raw mut (*emitter).raw_buffer.pointer;
            *fresh11 = (*emitter).raw_buffer.start;
            OK
        } else {
            yaml_emitter_set_writer_error(
                emitter,
                b"write error\0" as *const u8 as *const libc::c_char,
            )
        }
    }
}
/// YAML API module for LibYML.
///
/// This module provides functions and types for working directly with YAML data structures.
pub mod yaml {
    use crate::libc;
    use crate::memory::yaml_free;
    use core::ops::Deref;
    use core::ptr::{self, addr_of};
    pub(crate) use self::{YamlEncodingT::*, YamlEventTypeT::*, YamlNodeTypeT::*};
    pub use core::primitive::{i64 as ptrdiff_t, u64 as size_t, u8 as yaml_char_t};
    /// The version directive data.
    #[repr(C)]
    #[non_exhaustive]
    pub struct YamlVersionDirectiveT {
        /// The major version number.
        pub major: libc::c_int,
        /// The minor version number.
        pub minor: libc::c_int,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlVersionDirectiveT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlVersionDirectiveT {
        #[inline]
        fn clone(&self) -> YamlVersionDirectiveT {
            let _: ::core::clone::AssertParamIsClone<libc::c_int>;
            let _: ::core::clone::AssertParamIsClone<libc::c_int>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlVersionDirectiveT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "YamlVersionDirectiveT",
                "major",
                &self.major,
                "minor",
                &&self.minor,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for YamlVersionDirectiveT {
        #[inline]
        fn default() -> YamlVersionDirectiveT {
            YamlVersionDirectiveT {
                major: ::core::default::Default::default(),
                minor: ::core::default::Default::default(),
            }
        }
    }
    impl YamlVersionDirectiveT {
        /// Constructor for `YamlVersionDirectiveT`.
        pub fn new(major: libc::c_int, minor: libc::c_int) -> Self {
            YamlVersionDirectiveT {
                major,
                minor,
            }
        }
    }
    /// The tag directive data.
    #[repr(C)]
    #[non_exhaustive]
    pub struct YamlTagDirectiveT {
        /// The tag handle.
        pub handle: *mut yaml_char_t,
        /// The tag prefix.
        pub prefix: *mut yaml_char_t,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlTagDirectiveT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlTagDirectiveT {
        #[inline]
        fn clone(&self) -> YamlTagDirectiveT {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlTagDirectiveT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "YamlTagDirectiveT",
                "handle",
                &self.handle,
                "prefix",
                &&self.prefix,
            )
        }
    }
    /// The stream encoding.
    #[repr(u32)]
    #[non_exhaustive]
    pub enum YamlEncodingT {
        /// Let the parser choose the encoding.
        YamlAnyEncoding = 0,
        /// The default UTF-8 encoding.
        YamlUtf8Encoding = 1,
        /// The UTF-16-LE encoding with BOM.
        YamlUtf16leEncoding = 2,
        /// The UTF-16-BE encoding with BOM.
        YamlUtf16beEncoding = 3,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlEncodingT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlEncodingT {
        #[inline]
        fn clone(&self) -> YamlEncodingT {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for YamlEncodingT {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for YamlEncodingT {
        #[inline]
        fn eq(&self, other: &YamlEncodingT) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for YamlEncodingT {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for YamlEncodingT {
        #[inline]
        fn partial_cmp(
            &self,
            other: &YamlEncodingT,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for YamlEncodingT {
        #[inline]
        fn cmp(&self, other: &YamlEncodingT) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for YamlEncodingT {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlEncodingT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    YamlEncodingT::YamlAnyEncoding => "YamlAnyEncoding",
                    YamlEncodingT::YamlUtf8Encoding => "YamlUtf8Encoding",
                    YamlEncodingT::YamlUtf16leEncoding => "YamlUtf16leEncoding",
                    YamlEncodingT::YamlUtf16beEncoding => "YamlUtf16beEncoding",
                },
            )
        }
    }
    /// Line break type.
    #[repr(u32)]
    #[non_exhaustive]
    pub enum YamlBreakT {
        /// Let the parser choose the break type.
        YamlAnyBreak = 0,
        /// Use CR for line breaks (Mac style).
        YamlCrBreak = 1,
        /// Use LN for line breaks (Unix style).
        YamlLnBreak = 2,
        /// Use CR LN for line breaks (DOS style).
        YamlCrlnBreak = 3,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlBreakT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlBreakT {
        #[inline]
        fn clone(&self) -> YamlBreakT {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for YamlBreakT {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for YamlBreakT {
        #[inline]
        fn eq(&self, other: &YamlBreakT) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for YamlBreakT {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for YamlBreakT {
        #[inline]
        fn partial_cmp(
            &self,
            other: &YamlBreakT,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for YamlBreakT {
        #[inline]
        fn cmp(&self, other: &YamlBreakT) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for YamlBreakT {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlBreakT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    YamlBreakT::YamlAnyBreak => "YamlAnyBreak",
                    YamlBreakT::YamlCrBreak => "YamlCrBreak",
                    YamlBreakT::YamlLnBreak => "YamlLnBreak",
                    YamlBreakT::YamlCrlnBreak => "YamlCrlnBreak",
                },
            )
        }
    }
    /// Many bad things could happen with the parser and emitter.
    #[repr(u32)]
    #[non_exhaustive]
    pub enum YamlErrorTypeT {
        /// No error.
        YamlNoError = 0,
        /// Cannot allocate or reallocate a block of memory.
        YamlMemoryError = 1,
        /// Cannot read or decode the input stream.
        YamlReaderError = 2,
        /// Cannot scan the input stream.
        YamlScannerError = 3,
        /// Cannot parse the input stream.
        YamlParserError = 4,
        /// Cannot compose a YAML document.
        YamlComposerError = 5,
        /// Cannot write to the output stream.
        YamlWriterError = 6,
        /// Cannot emit a YAML stream.
        YamlEmitterError = 7,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlErrorTypeT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlErrorTypeT {
        #[inline]
        fn clone(&self) -> YamlErrorTypeT {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for YamlErrorTypeT {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for YamlErrorTypeT {
        #[inline]
        fn eq(&self, other: &YamlErrorTypeT) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for YamlErrorTypeT {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for YamlErrorTypeT {
        #[inline]
        fn partial_cmp(
            &self,
            other: &YamlErrorTypeT,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for YamlErrorTypeT {
        #[inline]
        fn cmp(&self, other: &YamlErrorTypeT) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for YamlErrorTypeT {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlErrorTypeT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    YamlErrorTypeT::YamlNoError => "YamlNoError",
                    YamlErrorTypeT::YamlMemoryError => "YamlMemoryError",
                    YamlErrorTypeT::YamlReaderError => "YamlReaderError",
                    YamlErrorTypeT::YamlScannerError => "YamlScannerError",
                    YamlErrorTypeT::YamlParserError => "YamlParserError",
                    YamlErrorTypeT::YamlComposerError => "YamlComposerError",
                    YamlErrorTypeT::YamlWriterError => "YamlWriterError",
                    YamlErrorTypeT::YamlEmitterError => "YamlEmitterError",
                },
            )
        }
    }
    impl Default for YamlErrorTypeT {
        fn default() -> Self {
            YamlErrorTypeT::YamlNoError
        }
    }
    /// The pointer position.
    #[repr(C)]
    #[non_exhaustive]
    pub struct YamlMarkT {
        /// The position index.
        pub index: size_t,
        /// The position line.
        pub line: size_t,
        /// The position column.
        pub column: size_t,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlMarkT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlMarkT {
        #[inline]
        fn clone(&self) -> YamlMarkT {
            let _: ::core::clone::AssertParamIsClone<size_t>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlMarkT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "YamlMarkT",
                "index",
                &self.index,
                "line",
                &self.line,
                "column",
                &&self.column,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for YamlMarkT {
        #[inline]
        fn default() -> YamlMarkT {
            YamlMarkT {
                index: ::core::default::Default::default(),
                line: ::core::default::Default::default(),
                column: ::core::default::Default::default(),
            }
        }
    }
    /// Scalar styles.
    #[repr(u32)]
    #[non_exhaustive]
    pub enum YamlScalarStyleT {
        /// Let the emitter choose the style.
        YamlAnyScalarStyle = 0,
        /// The plain scalar style.
        YamlPlainScalarStyle = 1,
        /// The single-quoted scalar style.
        YamlSingleQuotedScalarStyle = 2,
        /// The double-quoted scalar style.
        YamlDoubleQuotedScalarStyle = 3,
        /// The literal scalar style.
        YamlLiteralScalarStyle = 4,
        /// The folded scalar style.
        YamlFoldedScalarStyle = 5,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlScalarStyleT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlScalarStyleT {
        #[inline]
        fn clone(&self) -> YamlScalarStyleT {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for YamlScalarStyleT {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for YamlScalarStyleT {
        #[inline]
        fn eq(&self, other: &YamlScalarStyleT) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for YamlScalarStyleT {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for YamlScalarStyleT {
        #[inline]
        fn partial_cmp(
            &self,
            other: &YamlScalarStyleT,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for YamlScalarStyleT {
        #[inline]
        fn cmp(&self, other: &YamlScalarStyleT) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for YamlScalarStyleT {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlScalarStyleT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    YamlScalarStyleT::YamlAnyScalarStyle => "YamlAnyScalarStyle",
                    YamlScalarStyleT::YamlPlainScalarStyle => "YamlPlainScalarStyle",
                    YamlScalarStyleT::YamlSingleQuotedScalarStyle => {
                        "YamlSingleQuotedScalarStyle"
                    }
                    YamlScalarStyleT::YamlDoubleQuotedScalarStyle => {
                        "YamlDoubleQuotedScalarStyle"
                    }
                    YamlScalarStyleT::YamlLiteralScalarStyle => "YamlLiteralScalarStyle",
                    YamlScalarStyleT::YamlFoldedScalarStyle => "YamlFoldedScalarStyle",
                },
            )
        }
    }
    /// Sequence styles.
    #[repr(u32)]
    #[non_exhaustive]
    pub enum YamlSequenceStyleT {
        /// Let the emitter choose the style.
        YamlAnySequenceStyle = 0,
        /// The block sequence style.
        YamlBlockSequenceStyle = 1,
        /// The flow sequence style.
        YamlFlowSequenceStyle = 2,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlSequenceStyleT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlSequenceStyleT {
        #[inline]
        fn clone(&self) -> YamlSequenceStyleT {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for YamlSequenceStyleT {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for YamlSequenceStyleT {
        #[inline]
        fn eq(&self, other: &YamlSequenceStyleT) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for YamlSequenceStyleT {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for YamlSequenceStyleT {
        #[inline]
        fn partial_cmp(
            &self,
            other: &YamlSequenceStyleT,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for YamlSequenceStyleT {
        #[inline]
        fn cmp(&self, other: &YamlSequenceStyleT) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for YamlSequenceStyleT {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlSequenceStyleT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    YamlSequenceStyleT::YamlAnySequenceStyle => "YamlAnySequenceStyle",
                    YamlSequenceStyleT::YamlBlockSequenceStyle => {
                        "YamlBlockSequenceStyle"
                    }
                    YamlSequenceStyleT::YamlFlowSequenceStyle => "YamlFlowSequenceStyle",
                },
            )
        }
    }
    /// Mapping styles.
    #[repr(u32)]
    #[non_exhaustive]
    pub enum YamlMappingStyleT {
        /// Let the emitter choose the style.
        YamlAnyMappingStyle = 0,
        /// The block mapping style.
        YamlBlockMappingStyle = 1,
        /// The flow mapping style.
        YamlFlowMappingStyle = 2,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlMappingStyleT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlMappingStyleT {
        #[inline]
        fn clone(&self) -> YamlMappingStyleT {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for YamlMappingStyleT {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for YamlMappingStyleT {
        #[inline]
        fn eq(&self, other: &YamlMappingStyleT) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for YamlMappingStyleT {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for YamlMappingStyleT {
        #[inline]
        fn partial_cmp(
            &self,
            other: &YamlMappingStyleT,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for YamlMappingStyleT {
        #[inline]
        fn cmp(&self, other: &YamlMappingStyleT) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for YamlMappingStyleT {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlMappingStyleT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    YamlMappingStyleT::YamlAnyMappingStyle => "YamlAnyMappingStyle",
                    YamlMappingStyleT::YamlBlockMappingStyle => "YamlBlockMappingStyle",
                    YamlMappingStyleT::YamlFlowMappingStyle => "YamlFlowMappingStyle",
                },
            )
        }
    }
    /// The token types.
    #[repr(u32)]
    #[non_exhaustive]
    pub enum YamlTokenTypeT {
        /// An empty token.
        YamlNoToken = 0,
        /// A stream-start token.
        YamlStreamStartToken = 1,
        /// A stream-end token.
        YamlStreamEndToken = 2,
        /// A version-directive token.
        YamlVersionDirectiveToken = 3,
        /// A tag-directive token.
        YamlTagDirectiveToken = 4,
        /// A document-start token.
        YamlDocumentStartToken = 5,
        /// A document-end token.
        YamlDocumentEndToken = 6,
        /// A block-sequence-start token.
        YamlBlockSequenceStartToken = 7,
        /// A block-mapping-start token.
        YamlBlockMappingStartToken = 8,
        /// A block-end token.
        YamlBlockEndToken = 9,
        /// A flow-sequence-start token.
        YamlFlowSequenceStartToken = 10,
        /// A flow-sequence-end token.
        YamlFlowSequenceEndToken = 11,
        /// A flow-mapping-start token.
        YamlFlowMappingStartToken = 12,
        /// A flow-mapping-end token.
        YamlFlowMappingEndToken = 13,
        /// A block-entry token.
        YamlBlockEntryToken = 14,
        /// A flow-entry token.
        YamlFlowEntryToken = 15,
        /// A key token.
        YamlKeyToken = 16,
        /// A value token.
        YamlValueToken = 17,
        /// An alias token.
        YamlAliasToken = 18,
        /// An anchor token.
        YamlAnchorToken = 19,
        /// A tag token.
        YamlTagToken = 20,
        /// A scalar token.
        YamlScalarToken = 21,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlTokenTypeT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlTokenTypeT {
        #[inline]
        fn clone(&self) -> YamlTokenTypeT {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for YamlTokenTypeT {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for YamlTokenTypeT {
        #[inline]
        fn eq(&self, other: &YamlTokenTypeT) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for YamlTokenTypeT {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for YamlTokenTypeT {
        #[inline]
        fn partial_cmp(
            &self,
            other: &YamlTokenTypeT,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for YamlTokenTypeT {
        #[inline]
        fn cmp(&self, other: &YamlTokenTypeT) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for YamlTokenTypeT {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlTokenTypeT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    YamlTokenTypeT::YamlNoToken => "YamlNoToken",
                    YamlTokenTypeT::YamlStreamStartToken => "YamlStreamStartToken",
                    YamlTokenTypeT::YamlStreamEndToken => "YamlStreamEndToken",
                    YamlTokenTypeT::YamlVersionDirectiveToken => {
                        "YamlVersionDirectiveToken"
                    }
                    YamlTokenTypeT::YamlTagDirectiveToken => "YamlTagDirectiveToken",
                    YamlTokenTypeT::YamlDocumentStartToken => "YamlDocumentStartToken",
                    YamlTokenTypeT::YamlDocumentEndToken => "YamlDocumentEndToken",
                    YamlTokenTypeT::YamlBlockSequenceStartToken => {
                        "YamlBlockSequenceStartToken"
                    }
                    YamlTokenTypeT::YamlBlockMappingStartToken => {
                        "YamlBlockMappingStartToken"
                    }
                    YamlTokenTypeT::YamlBlockEndToken => "YamlBlockEndToken",
                    YamlTokenTypeT::YamlFlowSequenceStartToken => {
                        "YamlFlowSequenceStartToken"
                    }
                    YamlTokenTypeT::YamlFlowSequenceEndToken => {
                        "YamlFlowSequenceEndToken"
                    }
                    YamlTokenTypeT::YamlFlowMappingStartToken => {
                        "YamlFlowMappingStartToken"
                    }
                    YamlTokenTypeT::YamlFlowMappingEndToken => "YamlFlowMappingEndToken",
                    YamlTokenTypeT::YamlBlockEntryToken => "YamlBlockEntryToken",
                    YamlTokenTypeT::YamlFlowEntryToken => "YamlFlowEntryToken",
                    YamlTokenTypeT::YamlKeyToken => "YamlKeyToken",
                    YamlTokenTypeT::YamlValueToken => "YamlValueToken",
                    YamlTokenTypeT::YamlAliasToken => "YamlAliasToken",
                    YamlTokenTypeT::YamlAnchorToken => "YamlAnchorToken",
                    YamlTokenTypeT::YamlTagToken => "YamlTagToken",
                    YamlTokenTypeT::YamlScalarToken => "YamlScalarToken",
                },
            )
        }
    }
    /// The token structure.
    #[repr(C)]
    #[non_exhaustive]
    pub struct YamlTokenT {
        /// The token type.
        pub type_: YamlTokenTypeT,
        /// The token data.
        pub data: UnnamedYamlTokenTData,
        /// The beginning of the token.
        pub start_mark: YamlMarkT,
        /// The end of the token.
        pub end_mark: YamlMarkT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlTokenT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlTokenT {
        #[inline]
        fn clone(&self) -> YamlTokenT {
            let _: ::core::clone::AssertParamIsClone<YamlTokenTypeT>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlTokenTData>;
            let _: ::core::clone::AssertParamIsClone<YamlMarkT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlTokenT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "YamlTokenT",
                "type_",
                &self.type_,
                "data",
                &self.data,
                "start_mark",
                &self.start_mark,
                "end_mark",
                &&self.end_mark,
            )
        }
    }
    #[repr(C)]
    /// The data structure for YAML tokens.
    pub struct UnnamedYamlTokenTData {
        /// The stream start (for YamlStreamStartToken).
        pub stream_start: UnnamedYamlTokenTdataStreamStart,
        /// The alias (for YamlAliasToken).
        pub alias: UnnamedYamlTokenTdataAlias,
        /// The anchor (for YamlAnchorToken).
        pub anchor: UnnamedYamlTokenTdataAnchor,
        /// The tag (for YamlTagToken).
        pub tag: UnnamedYamlTokenTdataTag,
        /// The scalar value (for YamlScalarToken).
        pub scalar: UnnamedYamlTokenTdataScalar,
        /// The version directive (for YamlVersionDirectiveToken).
        pub version_directive: UnnamedYamlTokenTdataVersionDirective,
        /// The tag directive (for YamlTagDirectiveToken).
        pub tag_directive: UnnamedYamlTokenTdataTagDirective,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlTokenTData {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlTokenTData {
        #[inline]
        fn clone(&self) -> UnnamedYamlTokenTData {
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlTokenTdataStreamStart>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlTokenTdataAlias>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlTokenTdataAnchor>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlTokenTdataTag>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlTokenTdataScalar>;
            let _: ::core::clone::AssertParamIsClone<
                UnnamedYamlTokenTdataVersionDirective,
            >;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlTokenTdataTagDirective>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlTokenTData {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "stream_start",
                "alias",
                "anchor",
                "tag",
                "scalar",
                "version_directive",
                "tag_directive",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.stream_start,
                &self.alias,
                &self.anchor,
                &self.tag,
                &self.scalar,
                &self.version_directive,
                &&self.tag_directive,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "UnnamedYamlTokenTData",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for UnnamedYamlTokenTData {
        #[inline]
        fn default() -> UnnamedYamlTokenTData {
            UnnamedYamlTokenTData {
                stream_start: ::core::default::Default::default(),
                alias: ::core::default::Default::default(),
                anchor: ::core::default::Default::default(),
                tag: ::core::default::Default::default(),
                scalar: ::core::default::Default::default(),
                version_directive: ::core::default::Default::default(),
                tag_directive: ::core::default::Default::default(),
            }
        }
    }
    /// Represents the start of a YAML data stream.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlTokenTdataStreamStart {
        /// The stream encoding.
        pub encoding: YamlEncodingT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlTokenTdataStreamStart {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlTokenTdataStreamStart {
        #[inline]
        fn clone(&self) -> UnnamedYamlTokenTdataStreamStart {
            let _: ::core::clone::AssertParamIsClone<YamlEncodingT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlTokenTdataStreamStart {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "UnnamedYamlTokenTdataStreamStart",
                "encoding",
                &&self.encoding,
            )
        }
    }
    /// Represents an alias in a YAML document.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlTokenTdataAlias {
        /// The alias value.
        pub value: *mut yaml_char_t,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlTokenTdataAlias {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlTokenTdataAlias {
        #[inline]
        fn clone(&self) -> UnnamedYamlTokenTdataAlias {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlTokenTdataAlias {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "UnnamedYamlTokenTdataAlias",
                "value",
                &&self.value,
            )
        }
    }
    /// Represents an anchor in a YAML document.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlTokenTdataAnchor {
        /// The anchor value.
        pub value: *mut yaml_char_t,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlTokenTdataAnchor {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlTokenTdataAnchor {
        #[inline]
        fn clone(&self) -> UnnamedYamlTokenTdataAnchor {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlTokenTdataAnchor {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "UnnamedYamlTokenTdataAnchor",
                "value",
                &&self.value,
            )
        }
    }
    /// Represents a tag in a YAML document.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlTokenTdataTag {
        /// The tag handle.
        pub handle: *mut yaml_char_t,
        /// The tag suffix.
        pub suffix: *mut yaml_char_t,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlTokenTdataTag {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlTokenTdataTag {
        #[inline]
        fn clone(&self) -> UnnamedYamlTokenTdataTag {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlTokenTdataTag {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "UnnamedYamlTokenTdataTag",
                "handle",
                &self.handle,
                "suffix",
                &&self.suffix,
            )
        }
    }
    /// Represents a scalar value in a YAML document.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlTokenTdataScalar {
        /// The scalar value.
        pub value: *mut yaml_char_t,
        /// The length of the scalar value.
        pub length: size_t,
        /// The scalar style.
        pub style: YamlScalarStyleT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlTokenTdataScalar {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlTokenTdataScalar {
        #[inline]
        fn clone(&self) -> UnnamedYamlTokenTdataScalar {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<size_t>;
            let _: ::core::clone::AssertParamIsClone<YamlScalarStyleT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlTokenTdataScalar {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "UnnamedYamlTokenTdataScalar",
                "value",
                &self.value,
                "length",
                &self.length,
                "style",
                &&self.style,
            )
        }
    }
    /// Represents the version directive in a YAML document.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlTokenTdataVersionDirective {
        /// The major version number.
        pub major: libc::c_int,
        /// The minor version number.
        pub minor: libc::c_int,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlTokenTdataVersionDirective {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlTokenTdataVersionDirective {
        #[inline]
        fn clone(&self) -> UnnamedYamlTokenTdataVersionDirective {
            let _: ::core::clone::AssertParamIsClone<libc::c_int>;
            let _: ::core::clone::AssertParamIsClone<libc::c_int>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlTokenTdataVersionDirective {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "UnnamedYamlTokenTdataVersionDirective",
                "major",
                &self.major,
                "minor",
                &&self.minor,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for UnnamedYamlTokenTdataVersionDirective {
        #[inline]
        fn default() -> UnnamedYamlTokenTdataVersionDirective {
            UnnamedYamlTokenTdataVersionDirective {
                major: ::core::default::Default::default(),
                minor: ::core::default::Default::default(),
            }
        }
    }
    /// Represents the tag directive in a YAML document.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlTokenTdataTagDirective {
        /// The tag handle.
        pub handle: *mut yaml_char_t,
        /// The tag prefix.
        pub prefix: *mut yaml_char_t,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlTokenTdataTagDirective {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlTokenTdataTagDirective {
        #[inline]
        fn clone(&self) -> UnnamedYamlTokenTdataTagDirective {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlTokenTdataTagDirective {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "UnnamedYamlTokenTdataTagDirective",
                "handle",
                &self.handle,
                "prefix",
                &&self.prefix,
            )
        }
    }
    /// Event types.
    #[repr(u32)]
    #[non_exhaustive]
    pub enum YamlEventTypeT {
        /// An empty event.
        YamlNoEvent = 0,
        /// A stream-start event.
        YamlStreamStartEvent = 1,
        /// A stream-end event.
        YamlStreamEndEvent = 2,
        /// A document-start event.
        YamlDocumentStartEvent = 3,
        /// A document-end event.
        YamlDocumentEndEvent = 4,
        /// An alias event.
        YamlAliasEvent = 5,
        /// A scalar event.
        YamlScalarEvent = 6,
        /// A sequence-start event.
        YamlSequenceStartEvent = 7,
        /// A sequence-end event.
        YamlSequenceEndEvent = 8,
        /// A mapping-start event.
        YamlMappingStartEvent = 9,
        /// A mapping-end event.
        YamlMappingEndEvent = 10,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlEventTypeT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlEventTypeT {
        #[inline]
        fn clone(&self) -> YamlEventTypeT {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for YamlEventTypeT {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for YamlEventTypeT {
        #[inline]
        fn eq(&self, other: &YamlEventTypeT) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for YamlEventTypeT {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for YamlEventTypeT {
        #[inline]
        fn partial_cmp(
            &self,
            other: &YamlEventTypeT,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for YamlEventTypeT {
        #[inline]
        fn cmp(&self, other: &YamlEventTypeT) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for YamlEventTypeT {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlEventTypeT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    YamlEventTypeT::YamlNoEvent => "YamlNoEvent",
                    YamlEventTypeT::YamlStreamStartEvent => "YamlStreamStartEvent",
                    YamlEventTypeT::YamlStreamEndEvent => "YamlStreamEndEvent",
                    YamlEventTypeT::YamlDocumentStartEvent => "YamlDocumentStartEvent",
                    YamlEventTypeT::YamlDocumentEndEvent => "YamlDocumentEndEvent",
                    YamlEventTypeT::YamlAliasEvent => "YamlAliasEvent",
                    YamlEventTypeT::YamlScalarEvent => "YamlScalarEvent",
                    YamlEventTypeT::YamlSequenceStartEvent => "YamlSequenceStartEvent",
                    YamlEventTypeT::YamlSequenceEndEvent => "YamlSequenceEndEvent",
                    YamlEventTypeT::YamlMappingStartEvent => "YamlMappingStartEvent",
                    YamlEventTypeT::YamlMappingEndEvent => "YamlMappingEndEvent",
                },
            )
        }
    }
    /// The event structure.
    #[repr(C)]
    #[non_exhaustive]
    pub struct YamlEventT {
        /// The event type.
        pub type_: YamlEventTypeT,
        /// The event data.
        pub data: UnnamedYamlEventTData,
        /// The beginning of the event.
        pub start_mark: YamlMarkT,
        /// The end of the event.
        pub end_mark: YamlMarkT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlEventT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlEventT {
        #[inline]
        fn clone(&self) -> YamlEventT {
            let _: ::core::clone::AssertParamIsClone<YamlEventTypeT>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlEventTData>;
            let _: ::core::clone::AssertParamIsClone<YamlMarkT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlEventT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "YamlEventT",
                "type_",
                &self.type_,
                "data",
                &self.data,
                "start_mark",
                &self.start_mark,
                "end_mark",
                &&self.end_mark,
            )
        }
    }
    /// Represents the data associated with a YAML event.
    #[repr(C)]
    pub struct UnnamedYamlEventTData {
        /// The stream parameters (for YamlStreamStartEvent).
        pub stream_start: UnnamedYamlEventTdataStreamStart,
        /// The document parameters (for YamlDocumentStartEvent).
        pub document_start: UnnamedYamlEventTdataDocumentStart,
        /// The document end parameters (for YamlDocumentEndEvent).
        pub document_end: UnnamedYamlEventTdataDocumentEnd,
        /// The alias parameters (for YamlAliasEvent).
        pub alias: UnnamedYamlEventTdataAlias,
        /// The scalar parameters (for YamlScalarEvent).
        pub scalar: UnnamedYamlEventTdataScalar,
        /// The sequence parameters (for YamlSequenceStartEvent).
        pub sequence_start: UnnamedYamlEventTdataSequenceStart,
        /// The mapping parameters (for YamlMappingStartEvent).
        pub mapping_start: UnnamedYamlEventTdataMappingStart,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlEventTData {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlEventTData {
        #[inline]
        fn clone(&self) -> UnnamedYamlEventTData {
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlEventTdataStreamStart>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlEventTdataDocumentStart>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlEventTdataDocumentEnd>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlEventTdataAlias>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlEventTdataScalar>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlEventTdataSequenceStart>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlEventTdataMappingStart>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlEventTData {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "stream_start",
                "document_start",
                "document_end",
                "alias",
                "scalar",
                "sequence_start",
                "mapping_start",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.stream_start,
                &self.document_start,
                &self.document_end,
                &self.alias,
                &self.scalar,
                &self.sequence_start,
                &&self.mapping_start,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "UnnamedYamlEventTData",
                names,
                values,
            )
        }
    }
    /// Represents the data associated with the start of a YAML stream.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlEventTdataStreamStart {
        /// The document encoding.
        pub encoding: YamlEncodingT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlEventTdataStreamStart {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlEventTdataStreamStart {
        #[inline]
        fn clone(&self) -> UnnamedYamlEventTdataStreamStart {
            let _: ::core::clone::AssertParamIsClone<YamlEncodingT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlEventTdataStreamStart {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "UnnamedYamlEventTdataStreamStart",
                "encoding",
                &&self.encoding,
            )
        }
    }
    /// Represents the data associated with the start of a YAML document.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlEventTdataDocumentStart {
        /// The version directive.
        pub version_directive: *mut YamlVersionDirectiveT,
        /// The list of tag directives.
        pub tag_directives: UnnamedYamlEventTdataDocumentStartTagDirectives,
        /// Is the document indicator implicit?
        pub implicit: bool,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlEventTdataDocumentStart {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlEventTdataDocumentStart {
        #[inline]
        fn clone(&self) -> UnnamedYamlEventTdataDocumentStart {
            let _: ::core::clone::AssertParamIsClone<*mut YamlVersionDirectiveT>;
            let _: ::core::clone::AssertParamIsClone<
                UnnamedYamlEventTdataDocumentStartTagDirectives,
            >;
            let _: ::core::clone::AssertParamIsClone<bool>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlEventTdataDocumentStart {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "UnnamedYamlEventTdataDocumentStart",
                "version_directive",
                &self.version_directive,
                "tag_directives",
                &self.tag_directives,
                "implicit",
                &&self.implicit,
            )
        }
    }
    /// Represents the list of tag directives at the start of a YAML document.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlEventTdataDocumentStartTagDirectives {
        /// The beginning of the tag directives list.
        pub start: *mut YamlTagDirectiveT,
        /// The end of the tag directives list.
        pub end: *mut YamlTagDirectiveT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlEventTdataDocumentStartTagDirectives {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlEventTdataDocumentStartTagDirectives {
        #[inline]
        fn clone(&self) -> UnnamedYamlEventTdataDocumentStartTagDirectives {
            let _: ::core::clone::AssertParamIsClone<*mut YamlTagDirectiveT>;
            let _: ::core::clone::AssertParamIsClone<*mut YamlTagDirectiveT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlEventTdataDocumentStartTagDirectives {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "UnnamedYamlEventTdataDocumentStartTagDirectives",
                "start",
                &self.start,
                "end",
                &&self.end,
            )
        }
    }
    /// Represents the data associated with the end of a YAML document.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlEventTdataDocumentEnd {
        /// Is the document end indicator implicit?
        pub implicit: bool,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlEventTdataDocumentEnd {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlEventTdataDocumentEnd {
        #[inline]
        fn clone(&self) -> UnnamedYamlEventTdataDocumentEnd {
            let _: ::core::clone::AssertParamIsClone<bool>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlEventTdataDocumentEnd {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "UnnamedYamlEventTdataDocumentEnd",
                "implicit",
                &&self.implicit,
            )
        }
    }
    /// Represents the data associated with a YAML alias event.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlEventTdataAlias {
        /// The anchor.
        pub anchor: *mut yaml_char_t,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlEventTdataAlias {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlEventTdataAlias {
        #[inline]
        fn clone(&self) -> UnnamedYamlEventTdataAlias {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlEventTdataAlias {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "UnnamedYamlEventTdataAlias",
                "anchor",
                &&self.anchor,
            )
        }
    }
    /// Represents the data associated with a YAML scalar event.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlEventTdataScalar {
        /// The anchor.
        pub anchor: *mut yaml_char_t,
        /// The tag.
        pub tag: *mut yaml_char_t,
        /// The scalar value.
        pub value: *mut yaml_char_t,
        /// The length of the scalar value.
        pub length: size_t,
        /// Is the tag optional for the plain style?
        pub plain_implicit: bool,
        /// Is the tag optional for any non-plain style?
        pub quoted_implicit: bool,
        /// The scalar style.
        pub style: YamlScalarStyleT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlEventTdataScalar {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlEventTdataScalar {
        #[inline]
        fn clone(&self) -> UnnamedYamlEventTdataScalar {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<size_t>;
            let _: ::core::clone::AssertParamIsClone<bool>;
            let _: ::core::clone::AssertParamIsClone<YamlScalarStyleT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlEventTdataScalar {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "anchor",
                "tag",
                "value",
                "length",
                "plain_implicit",
                "quoted_implicit",
                "style",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.anchor,
                &self.tag,
                &self.value,
                &self.length,
                &self.plain_implicit,
                &self.quoted_implicit,
                &&self.style,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "UnnamedYamlEventTdataScalar",
                names,
                values,
            )
        }
    }
    /// Represents the data associated with the start of a YAML sequence.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlEventTdataSequenceStart {
        /// The anchor.
        pub anchor: *mut yaml_char_t,
        /// The tag.
        pub tag: *mut yaml_char_t,
        /// Is the tag optional?
        pub implicit: bool,
        /// The sequence style.
        pub style: YamlSequenceStyleT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlEventTdataSequenceStart {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlEventTdataSequenceStart {
        #[inline]
        fn clone(&self) -> UnnamedYamlEventTdataSequenceStart {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<bool>;
            let _: ::core::clone::AssertParamIsClone<YamlSequenceStyleT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlEventTdataSequenceStart {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "UnnamedYamlEventTdataSequenceStart",
                "anchor",
                &self.anchor,
                "tag",
                &self.tag,
                "implicit",
                &self.implicit,
                "style",
                &&self.style,
            )
        }
    }
    /// Represents the data associated with the start of a YAML mapping.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlEventTdataMappingStart {
        /// The anchor.
        pub anchor: *mut yaml_char_t,
        /// The tag.
        pub tag: *mut yaml_char_t,
        /// Is the tag optional?
        pub implicit: bool,
        /// The mapping style.
        pub style: YamlMappingStyleT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlEventTdataMappingStart {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlEventTdataMappingStart {
        #[inline]
        fn clone(&self) -> UnnamedYamlEventTdataMappingStart {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<bool>;
            let _: ::core::clone::AssertParamIsClone<YamlMappingStyleT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlEventTdataMappingStart {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "UnnamedYamlEventTdataMappingStart",
                "anchor",
                &self.anchor,
                "tag",
                &self.tag,
                "implicit",
                &self.implicit,
                "style",
                &&self.style,
            )
        }
    }
    /// Node types.
    #[repr(u32)]
    #[non_exhaustive]
    pub enum YamlNodeTypeT {
        /// An empty node.
        YamlNoNode = 0,
        /// A scalar node.
        YamlScalarNode = 1,
        /// A sequence node.
        YamlSequenceNode = 2,
        /// A mapping node.
        YamlMappingNode = 3,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlNodeTypeT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlNodeTypeT {
        #[inline]
        fn clone(&self) -> YamlNodeTypeT {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for YamlNodeTypeT {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for YamlNodeTypeT {
        #[inline]
        fn eq(&self, other: &YamlNodeTypeT) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for YamlNodeTypeT {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for YamlNodeTypeT {
        #[inline]
        fn partial_cmp(
            &self,
            other: &YamlNodeTypeT,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for YamlNodeTypeT {
        #[inline]
        fn cmp(&self, other: &YamlNodeTypeT) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for YamlNodeTypeT {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlNodeTypeT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    YamlNodeTypeT::YamlNoNode => "YamlNoNode",
                    YamlNodeTypeT::YamlScalarNode => "YamlScalarNode",
                    YamlNodeTypeT::YamlSequenceNode => "YamlSequenceNode",
                    YamlNodeTypeT::YamlMappingNode => "YamlMappingNode",
                },
            )
        }
    }
    /// The node structure.
    #[repr(C)]
    #[non_exhaustive]
    pub struct YamlNodeT {
        /// The node type.
        pub type_: YamlNodeTypeT,
        /// The node tag.
        pub tag: *mut yaml_char_t,
        /// The node data.
        pub data: UnnamedYamlNodeTData,
        /// The beginning of the node.
        pub start_mark: YamlMarkT,
        /// The end of the node.
        pub end_mark: YamlMarkT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlNodeT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlNodeT {
        #[inline]
        fn clone(&self) -> YamlNodeT {
            let _: ::core::clone::AssertParamIsClone<YamlNodeTypeT>;
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlNodeTData>;
            let _: ::core::clone::AssertParamIsClone<YamlMarkT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlNodeT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field5_finish(
                f,
                "YamlNodeT",
                "type_",
                &self.type_,
                "tag",
                &self.tag,
                "data",
                &self.data,
                "start_mark",
                &self.start_mark,
                "end_mark",
                &&self.end_mark,
            )
        }
    }
    /// Represents the data associated with a YAML node.
    #[repr(C)]
    pub struct UnnamedYamlNodeTData {
        /// The scalar parameters (for YamlScalarNode).
        pub scalar: UnnamedYamlNodeTDataScalar,
        /// The sequence parameters (for YamlSequenceNode).
        pub sequence: UnnamedYamlNodeTDataSequence,
        /// The mapping parameters (for YamlMappingNode).
        pub mapping: UnnamedYamlNodeTDataMapping,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlNodeTData {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlNodeTData {
        #[inline]
        fn clone(&self) -> UnnamedYamlNodeTData {
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlNodeTDataScalar>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlNodeTDataSequence>;
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlNodeTDataMapping>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlNodeTData {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "UnnamedYamlNodeTData",
                "scalar",
                &self.scalar,
                "sequence",
                &self.sequence,
                "mapping",
                &&self.mapping,
            )
        }
    }
    /// Represents the data associated with a YAML scalar node.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlNodeTDataScalar {
        /// The scalar value.
        pub value: *mut yaml_char_t,
        /// The length of the scalar value.
        pub length: size_t,
        /// The scalar style.
        pub style: YamlScalarStyleT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlNodeTDataScalar {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlNodeTDataScalar {
        #[inline]
        fn clone(&self) -> UnnamedYamlNodeTDataScalar {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<size_t>;
            let _: ::core::clone::AssertParamIsClone<YamlScalarStyleT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlNodeTDataScalar {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "UnnamedYamlNodeTDataScalar",
                "value",
                &self.value,
                "length",
                &self.length,
                "style",
                &&self.style,
            )
        }
    }
    /// Represents an element of a YAML sequence node.
    pub type YamlNodeItemT = libc::c_int;
    /// Represents the data associated with a YAML sequence node.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlNodeTDataSequence {
        /// The stack of sequence items.
        pub items: YamlStackT<YamlNodeItemT>,
        /// The sequence style.
        pub style: YamlSequenceStyleT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlNodeTDataSequence {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlNodeTDataSequence {
        #[inline]
        fn clone(&self) -> UnnamedYamlNodeTDataSequence {
            let _: ::core::clone::AssertParamIsClone<YamlStackT<YamlNodeItemT>>;
            let _: ::core::clone::AssertParamIsClone<YamlSequenceStyleT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlNodeTDataSequence {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "UnnamedYamlNodeTDataSequence",
                "items",
                &self.items,
                "style",
                &&self.style,
            )
        }
    }
    /// Represents the data associated with a YAML mapping node.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlNodeTDataMapping {
        /// The stack of mapping pairs (key, value).
        pub pairs: YamlStackT<YamlNodePairT>,
        /// The mapping style.
        pub style: YamlMappingStyleT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlNodeTDataMapping {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlNodeTDataMapping {
        #[inline]
        fn clone(&self) -> UnnamedYamlNodeTDataMapping {
            let _: ::core::clone::AssertParamIsClone<YamlStackT<YamlNodePairT>>;
            let _: ::core::clone::AssertParamIsClone<YamlMappingStyleT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlNodeTDataMapping {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "UnnamedYamlNodeTDataMapping",
                "pairs",
                &self.pairs,
                "style",
                &&self.style,
            )
        }
    }
    /// An element of a mapping node.
    #[repr(C)]
    #[non_exhaustive]
    pub struct YamlNodePairT {
        /// The key of the element.
        pub key: libc::c_int,
        /// The value of the element.
        pub value: libc::c_int,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlNodePairT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlNodePairT {
        #[inline]
        fn clone(&self) -> YamlNodePairT {
            let _: ::core::clone::AssertParamIsClone<libc::c_int>;
            let _: ::core::clone::AssertParamIsClone<libc::c_int>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlNodePairT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "YamlNodePairT",
                "key",
                &self.key,
                "value",
                &&self.value,
            )
        }
    }
    /// The document structure.
    #[repr(C)]
    #[non_exhaustive]
    pub struct YamlDocumentT {
        /// The document nodes.
        pub nodes: YamlStackT<YamlNodeT>,
        /// The version directive.
        pub version_directive: *mut YamlVersionDirectiveT,
        /// The list of tag directives.
        pub tag_directives: UnnamedYamlDocumentTTagDirectives,
        /// Is the document start indicator implicit?
        pub start_implicit: bool,
        /// Is the document end indicator implicit?
        pub end_implicit: bool,
        /// The beginning of the document.
        pub start_mark: YamlMarkT,
        /// The end of the document.
        pub end_mark: YamlMarkT,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for YamlDocumentT {
        #[inline]
        fn clone(&self) -> YamlDocumentT {
            YamlDocumentT {
                nodes: ::core::clone::Clone::clone(&self.nodes),
                version_directive: ::core::clone::Clone::clone(&self.version_directive),
                tag_directives: ::core::clone::Clone::clone(&self.tag_directives),
                start_implicit: ::core::clone::Clone::clone(&self.start_implicit),
                end_implicit: ::core::clone::Clone::clone(&self.end_implicit),
                start_mark: ::core::clone::Clone::clone(&self.start_mark),
                end_mark: ::core::clone::Clone::clone(&self.end_mark),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlDocumentT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "nodes",
                "version_directive",
                "tag_directives",
                "start_implicit",
                "end_implicit",
                "start_mark",
                "end_mark",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.nodes,
                &self.version_directive,
                &self.tag_directives,
                &self.start_implicit,
                &self.end_implicit,
                &self.start_mark,
                &&self.end_mark,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "YamlDocumentT",
                names,
                values,
            )
        }
    }
    /// Represents the list of tag directives in a YAML document.
    #[repr(C)]
    #[non_exhaustive]
    pub struct UnnamedYamlDocumentTTagDirectives {
        /// The beginning of the tag directives list.
        pub start: *mut YamlTagDirectiveT,
        /// The end of the tag directives list.
        pub end: *mut YamlTagDirectiveT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlDocumentTTagDirectives {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlDocumentTTagDirectives {
        #[inline]
        fn clone(&self) -> UnnamedYamlDocumentTTagDirectives {
            let _: ::core::clone::AssertParamIsClone<*mut YamlTagDirectiveT>;
            let _: ::core::clone::AssertParamIsClone<*mut YamlTagDirectiveT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlDocumentTTagDirectives {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "UnnamedYamlDocumentTTagDirectives",
                "start",
                &self.start,
                "end",
                &&self.end,
            )
        }
    }
    /// The prototype of a read handler.
    ///
    /// The read handler is called when the parser needs to read more bytes from the
    /// source. The handler should write not more than `size` bytes to the `buffer`.
    /// The number of written bytes should be set to the `length` variable.
    ///
    /// On success, the handler should return 1. If the handler failed, the returned
    /// value should be 0. On EOF, the handler should set the `size_read` to 0 and
    /// return 1.
    pub type YamlReadHandlerT = unsafe fn(
        data: *mut libc::c_void,
        buffer: *mut libc::c_uchar,
        size: size_t,
        size_read: *mut size_t,
    ) -> libc::c_int;
    /// This structure holds information about a potential simple key.
    #[repr(C)]
    #[non_exhaustive]
    pub struct YamlSimpleKeyT {
        /// Is a simple key possible?
        pub possible: bool,
        /// Is a simple key required?
        pub required: bool,
        /// The number of the token.
        pub token_number: size_t,
        /// The position mark.
        pub mark: YamlMarkT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlSimpleKeyT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlSimpleKeyT {
        #[inline]
        fn clone(&self) -> YamlSimpleKeyT {
            let _: ::core::clone::AssertParamIsClone<bool>;
            let _: ::core::clone::AssertParamIsClone<size_t>;
            let _: ::core::clone::AssertParamIsClone<YamlMarkT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlSimpleKeyT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "YamlSimpleKeyT",
                "possible",
                &self.possible,
                "required",
                &self.required,
                "token_number",
                &self.token_number,
                "mark",
                &&self.mark,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for YamlSimpleKeyT {
        #[inline]
        fn default() -> YamlSimpleKeyT {
            YamlSimpleKeyT {
                possible: ::core::default::Default::default(),
                required: ::core::default::Default::default(),
                token_number: ::core::default::Default::default(),
                mark: ::core::default::Default::default(),
            }
        }
    }
    /// The states of the parser.
    #[repr(u32)]
    #[non_exhaustive]
    pub enum YamlParserStateT {
        /// Expect stream-start.
        YamlParseStreamStartState = 0,
        /// Expect the beginning of an implicit document.
        YamlParseImplicitDocumentStartState = 1,
        /// Expect document-start.
        YamlParseDocumentStartState = 2,
        /// Expect the content of a document.
        YamlParseDocumentContentState = 3,
        /// Expect document-end.
        YamlParseDocumentEndState = 4,
        /// Expect a block node.
        YamlParseBlockNodeState = 5,
        /// Expect a block node or indentless sequence.
        YamlParseBlockNodeOrIndentlessSequenceState = 6,
        /// Expect a flow node.
        YamlParseFlowNodeState = 7,
        /// Expect the first entry of a block sequence.
        YamlParseBlockSequenceFirstEntryState = 8,
        /// Expect an entry of a block sequence.
        YamlParseBlockSequenceEntryState = 9,
        /// Expect an entry of an indentless sequence.
        YamlParseIndentlessSequenceEntryState = 10,
        /// Expect the first key of a block mapping.
        YamlParseBlockMappingFirstKeyState = 11,
        /// Expect a block mapping key.
        YamlParseBlockMappingKeyState = 12,
        /// Expect a block mapping value.
        YamlParseBlockMappingValueState = 13,
        /// Expect the first entry of a flow sequence.
        YamlParseFlowSequenceFirstEntryState = 14,
        /// Expect an entry of a flow sequence.
        YamlParseFlowSequenceEntryState = 15,
        /// Expect a key of an ordered mapping.
        YamlParseFlowSequenceEntryMappingKeyState = 16,
        /// Expect a value of an ordered mapping.
        YamlParseFlowSequenceEntryMappingValueState = 17,
        /// Expect the and of an ordered mapping entry.
        YamlParseFlowSequenceEntryMappingEndState = 18,
        /// Expect the first key of a flow mapping.
        YamlParseFlowMappingFirstKeyState = 19,
        /// Expect a key of a flow mapping.
        YamlParseFlowMappingKeyState = 20,
        /// Expect a value of a flow mapping.
        YamlParseFlowMappingValueState = 21,
        /// Expect an empty value of a flow mapping.
        YamlParseFlowMappingEmptyValueState = 22,
        /// Expect nothing.
        YamlParseEndState = 23,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlParserStateT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlParserStateT {
        #[inline]
        fn clone(&self) -> YamlParserStateT {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for YamlParserStateT {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for YamlParserStateT {
        #[inline]
        fn eq(&self, other: &YamlParserStateT) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for YamlParserStateT {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for YamlParserStateT {
        #[inline]
        fn partial_cmp(
            &self,
            other: &YamlParserStateT,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for YamlParserStateT {
        #[inline]
        fn cmp(&self, other: &YamlParserStateT) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for YamlParserStateT {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlParserStateT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    YamlParserStateT::YamlParseStreamStartState => {
                        "YamlParseStreamStartState"
                    }
                    YamlParserStateT::YamlParseImplicitDocumentStartState => {
                        "YamlParseImplicitDocumentStartState"
                    }
                    YamlParserStateT::YamlParseDocumentStartState => {
                        "YamlParseDocumentStartState"
                    }
                    YamlParserStateT::YamlParseDocumentContentState => {
                        "YamlParseDocumentContentState"
                    }
                    YamlParserStateT::YamlParseDocumentEndState => {
                        "YamlParseDocumentEndState"
                    }
                    YamlParserStateT::YamlParseBlockNodeState => {
                        "YamlParseBlockNodeState"
                    }
                    YamlParserStateT::YamlParseBlockNodeOrIndentlessSequenceState => {
                        "YamlParseBlockNodeOrIndentlessSequenceState"
                    }
                    YamlParserStateT::YamlParseFlowNodeState => "YamlParseFlowNodeState",
                    YamlParserStateT::YamlParseBlockSequenceFirstEntryState => {
                        "YamlParseBlockSequenceFirstEntryState"
                    }
                    YamlParserStateT::YamlParseBlockSequenceEntryState => {
                        "YamlParseBlockSequenceEntryState"
                    }
                    YamlParserStateT::YamlParseIndentlessSequenceEntryState => {
                        "YamlParseIndentlessSequenceEntryState"
                    }
                    YamlParserStateT::YamlParseBlockMappingFirstKeyState => {
                        "YamlParseBlockMappingFirstKeyState"
                    }
                    YamlParserStateT::YamlParseBlockMappingKeyState => {
                        "YamlParseBlockMappingKeyState"
                    }
                    YamlParserStateT::YamlParseBlockMappingValueState => {
                        "YamlParseBlockMappingValueState"
                    }
                    YamlParserStateT::YamlParseFlowSequenceFirstEntryState => {
                        "YamlParseFlowSequenceFirstEntryState"
                    }
                    YamlParserStateT::YamlParseFlowSequenceEntryState => {
                        "YamlParseFlowSequenceEntryState"
                    }
                    YamlParserStateT::YamlParseFlowSequenceEntryMappingKeyState => {
                        "YamlParseFlowSequenceEntryMappingKeyState"
                    }
                    YamlParserStateT::YamlParseFlowSequenceEntryMappingValueState => {
                        "YamlParseFlowSequenceEntryMappingValueState"
                    }
                    YamlParserStateT::YamlParseFlowSequenceEntryMappingEndState => {
                        "YamlParseFlowSequenceEntryMappingEndState"
                    }
                    YamlParserStateT::YamlParseFlowMappingFirstKeyState => {
                        "YamlParseFlowMappingFirstKeyState"
                    }
                    YamlParserStateT::YamlParseFlowMappingKeyState => {
                        "YamlParseFlowMappingKeyState"
                    }
                    YamlParserStateT::YamlParseFlowMappingValueState => {
                        "YamlParseFlowMappingValueState"
                    }
                    YamlParserStateT::YamlParseFlowMappingEmptyValueState => {
                        "YamlParseFlowMappingEmptyValueState"
                    }
                    YamlParserStateT::YamlParseEndState => "YamlParseEndState",
                },
            )
        }
    }
    /// This structure holds aliases data.
    #[repr(C)]
    #[non_exhaustive]
    pub struct YamlAliasDataT {
        /// The anchor.
        pub anchor: *mut yaml_char_t,
        /// The node id.
        pub index: libc::c_int,
        /// The anchor mark.
        pub mark: YamlMarkT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlAliasDataT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlAliasDataT {
        #[inline]
        fn clone(&self) -> YamlAliasDataT {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<libc::c_int>;
            let _: ::core::clone::AssertParamIsClone<YamlMarkT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlAliasDataT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "YamlAliasDataT",
                "anchor",
                &self.anchor,
                "index",
                &self.index,
                "mark",
                &&self.mark,
            )
        }
    }
    /// The parser structure.
    ///
    /// All members are internal. Manage the structure using the `yaml_parser_`
    /// family of functions.
    #[repr(C)]
    #[non_exhaustive]
    pub struct YamlParserT {
        #[cfg(not(doc))]
        pub(crate) error: YamlErrorTypeT,
        #[cfg(not(doc))]
        pub(crate) problem: *const libc::c_char,
        #[cfg(not(doc))]
        pub(crate) problem_offset: size_t,
        #[cfg(not(doc))]
        pub(crate) problem_value: libc::c_int,
        #[cfg(not(doc))]
        pub(crate) problem_mark: YamlMarkT,
        #[cfg(not(doc))]
        pub(crate) context: *const libc::c_char,
        #[cfg(not(doc))]
        pub(crate) context_mark: YamlMarkT,
        /// Read handler.
        pub(crate) read_handler: Option<YamlReadHandlerT>,
        /// A pointer for passing to the read handler.
        pub(crate) read_handler_data: *mut libc::c_void,
        /// Standard (string or file) input data.
        pub(crate) input: UnnamedYamlParserTInput,
        /// EOF flag
        pub(crate) eof: bool,
        /// The working buffer.
        pub(crate) buffer: YamlBufferT<yaml_char_t>,
        /// The number of unread characters in the buffer.
        pub(crate) unread: size_t,
        /// The raw buffer.
        pub(crate) raw_buffer: YamlBufferT<libc::c_uchar>,
        /// The input encoding.
        pub(crate) encoding: YamlEncodingT,
        /// The offset of the current position (in bytes).
        pub(crate) offset: size_t,
        /// The mark of the current position.
        pub(crate) mark: YamlMarkT,
        /// Have we started to scan the input stream?
        pub(crate) stream_start_produced: bool,
        /// Have we reached the end of the input stream?
        pub(crate) stream_end_produced: bool,
        /// The number of unclosed '[' and '{' indicators.
        pub(crate) flow_level: libc::c_int,
        /// The tokens queue.
        pub(crate) tokens: YamlQueueT<YamlTokenT>,
        /// The number of tokens fetched from the queue.
        pub(crate) tokens_parsed: size_t,
        /// Does the tokens queue contain a token ready for dequeueing.
        pub(crate) token_available: bool,
        /// The indentation levels stack.
        pub(crate) indents: YamlStackT<libc::c_int>,
        /// The current indentation level.
        pub(crate) indent: libc::c_int,
        /// May a simple key occur at the current position?
        pub(crate) simple_key_allowed: bool,
        /// The stack of simple keys.
        pub(crate) simple_keys: YamlStackT<YamlSimpleKeyT>,
        /// At least this many leading elements of simple_keys have possible=0.
        pub(crate) not_simple_keys: libc::c_int,
        /// The parser states stack.
        pub(crate) states: YamlStackT<YamlParserStateT>,
        /// The current parser state.
        pub(crate) state: YamlParserStateT,
        /// The stack of marks.
        pub(crate) marks: YamlStackT<YamlMarkT>,
        /// The list of TAG directives.
        pub(crate) tag_directives: YamlStackT<YamlTagDirectiveT>,
        /// The alias data.
        pub(crate) aliases: YamlStackT<YamlAliasDataT>,
        /// The currently parsed document.
        pub(crate) document: *mut YamlDocumentT,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for YamlParserT {
        #[inline]
        fn clone(&self) -> YamlParserT {
            YamlParserT {
                error: ::core::clone::Clone::clone(&self.error),
                problem: ::core::clone::Clone::clone(&self.problem),
                problem_offset: ::core::clone::Clone::clone(&self.problem_offset),
                problem_value: ::core::clone::Clone::clone(&self.problem_value),
                problem_mark: ::core::clone::Clone::clone(&self.problem_mark),
                context: ::core::clone::Clone::clone(&self.context),
                context_mark: ::core::clone::Clone::clone(&self.context_mark),
                read_handler: ::core::clone::Clone::clone(&self.read_handler),
                read_handler_data: ::core::clone::Clone::clone(&self.read_handler_data),
                input: ::core::clone::Clone::clone(&self.input),
                eof: ::core::clone::Clone::clone(&self.eof),
                buffer: ::core::clone::Clone::clone(&self.buffer),
                unread: ::core::clone::Clone::clone(&self.unread),
                raw_buffer: ::core::clone::Clone::clone(&self.raw_buffer),
                encoding: ::core::clone::Clone::clone(&self.encoding),
                offset: ::core::clone::Clone::clone(&self.offset),
                mark: ::core::clone::Clone::clone(&self.mark),
                stream_start_produced: ::core::clone::Clone::clone(
                    &self.stream_start_produced,
                ),
                stream_end_produced: ::core::clone::Clone::clone(
                    &self.stream_end_produced,
                ),
                flow_level: ::core::clone::Clone::clone(&self.flow_level),
                tokens: ::core::clone::Clone::clone(&self.tokens),
                tokens_parsed: ::core::clone::Clone::clone(&self.tokens_parsed),
                token_available: ::core::clone::Clone::clone(&self.token_available),
                indents: ::core::clone::Clone::clone(&self.indents),
                indent: ::core::clone::Clone::clone(&self.indent),
                simple_key_allowed: ::core::clone::Clone::clone(
                    &self.simple_key_allowed,
                ),
                simple_keys: ::core::clone::Clone::clone(&self.simple_keys),
                not_simple_keys: ::core::clone::Clone::clone(&self.not_simple_keys),
                states: ::core::clone::Clone::clone(&self.states),
                state: ::core::clone::Clone::clone(&self.state),
                marks: ::core::clone::Clone::clone(&self.marks),
                tag_directives: ::core::clone::Clone::clone(&self.tag_directives),
                aliases: ::core::clone::Clone::clone(&self.aliases),
                document: ::core::clone::Clone::clone(&self.document),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlParserT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "error",
                "problem",
                "problem_offset",
                "problem_value",
                "problem_mark",
                "context",
                "context_mark",
                "read_handler",
                "read_handler_data",
                "input",
                "eof",
                "buffer",
                "unread",
                "raw_buffer",
                "encoding",
                "offset",
                "mark",
                "stream_start_produced",
                "stream_end_produced",
                "flow_level",
                "tokens",
                "tokens_parsed",
                "token_available",
                "indents",
                "indent",
                "simple_key_allowed",
                "simple_keys",
                "not_simple_keys",
                "states",
                "state",
                "marks",
                "tag_directives",
                "aliases",
                "document",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.error,
                &self.problem,
                &self.problem_offset,
                &self.problem_value,
                &self.problem_mark,
                &self.context,
                &self.context_mark,
                &self.read_handler,
                &self.read_handler_data,
                &self.input,
                &self.eof,
                &self.buffer,
                &self.unread,
                &self.raw_buffer,
                &self.encoding,
                &self.offset,
                &self.mark,
                &self.stream_start_produced,
                &self.stream_end_produced,
                &self.flow_level,
                &self.tokens,
                &self.tokens_parsed,
                &self.token_available,
                &self.indents,
                &self.indent,
                &self.simple_key_allowed,
                &self.simple_keys,
                &self.not_simple_keys,
                &self.states,
                &self.state,
                &self.marks,
                &self.tag_directives,
                &self.aliases,
                &&self.document,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "YamlParserT",
                names,
                values,
            )
        }
    }
    /// Represents the prefix data associated with a YAML parser.
    #[repr(C)]
    #[non_exhaustive]
    pub struct YamlParserTPrefix {
        /// The error type.
        pub error: YamlErrorTypeT,
        /// The error description.
        pub problem: *const libc::c_char,
        /// The byte about which the problem occurred.
        pub problem_offset: size_t,
        /// The problematic value (-1 is none).
        pub problem_value: libc::c_int,
        /// The problem position.
        pub problem_mark: YamlMarkT,
        /// The error context.
        pub context: *const libc::c_char,
        /// The context position.
        pub context_mark: YamlMarkT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlParserTPrefix {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlParserTPrefix {
        #[inline]
        fn clone(&self) -> YamlParserTPrefix {
            let _: ::core::clone::AssertParamIsClone<YamlErrorTypeT>;
            let _: ::core::clone::AssertParamIsClone<*const libc::c_char>;
            let _: ::core::clone::AssertParamIsClone<size_t>;
            let _: ::core::clone::AssertParamIsClone<libc::c_int>;
            let _: ::core::clone::AssertParamIsClone<YamlMarkT>;
            let _: ::core::clone::AssertParamIsClone<*const libc::c_char>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlParserTPrefix {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "error",
                "problem",
                "problem_offset",
                "problem_value",
                "problem_mark",
                "context",
                "context_mark",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.error,
                &self.problem,
                &self.problem_offset,
                &self.problem_value,
                &self.problem_mark,
                &self.context,
                &&self.context_mark,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "YamlParserTPrefix",
                names,
                values,
            )
        }
    }
    #[doc(hidden)]
    impl Deref for YamlParserT {
        type Target = YamlParserTPrefix;
        fn deref(&self) -> &Self::Target {
            unsafe { &*(&raw const *self).cast() }
        }
    }
    #[repr(C)]
    pub(crate) struct UnnamedYamlParserTInput {
        /// String input data.
        pub(crate) string: UnnamedYamlParserTInputString,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlParserTInput {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlParserTInput {
        #[inline]
        fn clone(&self) -> UnnamedYamlParserTInput {
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlParserTInputString>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlParserTInput {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "UnnamedYamlParserTInput",
                "string",
                &&self.string,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for UnnamedYamlParserTInput {
        #[inline]
        fn default() -> UnnamedYamlParserTInput {
            UnnamedYamlParserTInput {
                string: ::core::default::Default::default(),
            }
        }
    }
    #[repr(C)]
    pub(crate) struct UnnamedYamlParserTInputString {
        /// The string start pointer.
        pub(crate) start: *const libc::c_uchar,
        /// The string end pointer.
        pub(crate) end: *const libc::c_uchar,
        /// The string current position.
        pub(crate) current: *const libc::c_uchar,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlParserTInputString {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlParserTInputString {
        #[inline]
        fn clone(&self) -> UnnamedYamlParserTInputString {
            let _: ::core::clone::AssertParamIsClone<*const libc::c_uchar>;
            let _: ::core::clone::AssertParamIsClone<*const libc::c_uchar>;
            let _: ::core::clone::AssertParamIsClone<*const libc::c_uchar>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlParserTInputString {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "UnnamedYamlParserTInputString",
                "start",
                &self.start,
                "end",
                &self.end,
                "current",
                &&self.current,
            )
        }
    }
    /// The prototype of a write handler.
    ///
    /// The write handler is called when the emitter needs to flush the accumulated
    /// characters to the output. The handler should write `size` bytes of the
    /// `buffer` to the output.
    ///
    /// On success, the handler should return 1. If the handler failed, the returned
    /// value should be 0.
    pub type YamlWriteHandlerT = unsafe fn(
        data: *mut libc::c_void,
        buffer: *mut libc::c_uchar,
        size: size_t,
    ) -> libc::c_int;
    /// The emitter states.
    #[repr(u32)]
    #[non_exhaustive]
    pub enum YamlEmitterStateT {
        /// Expect stream-start.
        YamlEmitStreamStartState = 0,
        /// Expect the first document-start or stream-end.
        YamlEmitFirstDocumentStartState = 1,
        /// Expect document-start or stream-end.
        YamlEmitDocumentStartState = 2,
        /// Expect the content of a document.
        YamlEmitDocumentContentState = 3,
        /// Expect document-end.
        YamlEmitDocumentEndState = 4,
        /// Expect the first item of a flow sequence.
        YamlEmitFlowSequenceFirstItemState = 5,
        /// Expect an item of a flow sequence.
        YamlEmitFlowSequenceItemState = 6,
        /// Expect the first key of a flow mapping.
        YamlEmitFlowMappingFirstKeyState = 7,
        /// Expect a key of a flow mapping.
        YamlEmitFlowMappingKeyState = 8,
        /// Expect a value for a simple key of a flow mapping.
        YamlEmitFlowMappingSimpleValueState = 9,
        /// Expect a value of a flow mapping.
        YamlEmitFlowMappingValueState = 10,
        /// Expect the first item of a block sequence.
        YamlEmitBlockSequenceFirstItemState = 11,
        /// Expect an item of a block sequence.
        YamlEmitBlockSequenceItemState = 12,
        /// Expect the first key of a block mapping.
        YamlEmitBlockMappingFirstKeyState = 13,
        /// Expect the key of a block mapping.
        YamlEmitBlockMappingKeyState = 14,
        /// Expect a value for a simple key of a block mapping.
        YamlEmitBlockMappingSimpleValueState = 15,
        /// Expect a value of a block mapping.
        YamlEmitBlockMappingValueState = 16,
        /// Expect nothing.
        YamlEmitEndState = 17,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlEmitterStateT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlEmitterStateT {
        #[inline]
        fn clone(&self) -> YamlEmitterStateT {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for YamlEmitterStateT {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for YamlEmitterStateT {
        #[inline]
        fn eq(&self, other: &YamlEmitterStateT) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for YamlEmitterStateT {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for YamlEmitterStateT {
        #[inline]
        fn partial_cmp(
            &self,
            other: &YamlEmitterStateT,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::PartialOrd::partial_cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for YamlEmitterStateT {
        #[inline]
        fn cmp(&self, other: &YamlEmitterStateT) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for YamlEmitterStateT {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            ::core::hash::Hash::hash(&__self_discr, state)
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlEmitterStateT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    YamlEmitterStateT::YamlEmitStreamStartState => {
                        "YamlEmitStreamStartState"
                    }
                    YamlEmitterStateT::YamlEmitFirstDocumentStartState => {
                        "YamlEmitFirstDocumentStartState"
                    }
                    YamlEmitterStateT::YamlEmitDocumentStartState => {
                        "YamlEmitDocumentStartState"
                    }
                    YamlEmitterStateT::YamlEmitDocumentContentState => {
                        "YamlEmitDocumentContentState"
                    }
                    YamlEmitterStateT::YamlEmitDocumentEndState => {
                        "YamlEmitDocumentEndState"
                    }
                    YamlEmitterStateT::YamlEmitFlowSequenceFirstItemState => {
                        "YamlEmitFlowSequenceFirstItemState"
                    }
                    YamlEmitterStateT::YamlEmitFlowSequenceItemState => {
                        "YamlEmitFlowSequenceItemState"
                    }
                    YamlEmitterStateT::YamlEmitFlowMappingFirstKeyState => {
                        "YamlEmitFlowMappingFirstKeyState"
                    }
                    YamlEmitterStateT::YamlEmitFlowMappingKeyState => {
                        "YamlEmitFlowMappingKeyState"
                    }
                    YamlEmitterStateT::YamlEmitFlowMappingSimpleValueState => {
                        "YamlEmitFlowMappingSimpleValueState"
                    }
                    YamlEmitterStateT::YamlEmitFlowMappingValueState => {
                        "YamlEmitFlowMappingValueState"
                    }
                    YamlEmitterStateT::YamlEmitBlockSequenceFirstItemState => {
                        "YamlEmitBlockSequenceFirstItemState"
                    }
                    YamlEmitterStateT::YamlEmitBlockSequenceItemState => {
                        "YamlEmitBlockSequenceItemState"
                    }
                    YamlEmitterStateT::YamlEmitBlockMappingFirstKeyState => {
                        "YamlEmitBlockMappingFirstKeyState"
                    }
                    YamlEmitterStateT::YamlEmitBlockMappingKeyState => {
                        "YamlEmitBlockMappingKeyState"
                    }
                    YamlEmitterStateT::YamlEmitBlockMappingSimpleValueState => {
                        "YamlEmitBlockMappingSimpleValueState"
                    }
                    YamlEmitterStateT::YamlEmitBlockMappingValueState => {
                        "YamlEmitBlockMappingValueState"
                    }
                    YamlEmitterStateT::YamlEmitEndState => "YamlEmitEndState",
                },
            )
        }
    }
    #[repr(C)]
    pub(crate) struct YamlAnchorsT {
        /// The number of references.
        pub(crate) references: libc::c_int,
        /// The anchor id.
        pub(crate) anchor: libc::c_int,
        /// If the node has been emitted?
        pub(crate) serialized: bool,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlAnchorsT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlAnchorsT {
        #[inline]
        fn clone(&self) -> YamlAnchorsT {
            let _: ::core::clone::AssertParamIsClone<libc::c_int>;
            let _: ::core::clone::AssertParamIsClone<libc::c_int>;
            let _: ::core::clone::AssertParamIsClone<bool>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlAnchorsT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "YamlAnchorsT",
                "references",
                &self.references,
                "anchor",
                &self.anchor,
                "serialized",
                &&self.serialized,
            )
        }
    }
    /// The emitter structure.
    ///
    /// All members are internal. Manage the structure using the `yaml_emitter_`
    /// family of functions.
    #[repr(C)]
    #[non_exhaustive]
    pub struct YamlEmitterT {
        #[cfg(not(doc))]
        pub(crate) error: YamlErrorTypeT,
        #[cfg(not(doc))]
        pub(crate) problem: *const libc::c_char,
        /// Write handler.
        pub write_handler: Option<YamlWriteHandlerT>,
        /// A pointer for passing to the write handler.
        pub(crate) write_handler_data: *mut libc::c_void,
        /// Standard (string or file) output data.
        pub output: UnnamedYamlEmitterTOutput,
        /// The working buffer.
        pub buffer: YamlBufferT<yaml_char_t>,
        /// The raw buffer.
        pub(crate) raw_buffer: YamlBufferT<libc::c_uchar>,
        /// The stream encoding.
        pub(crate) encoding: YamlEncodingT,
        /// If the output is in the canonical style?
        pub(crate) canonical: bool,
        /// The number of indentation spaces.
        pub(crate) best_indent: libc::c_int,
        /// The preferred width of the output lines.
        pub(crate) best_width: libc::c_int,
        /// Allow unescaped non-ASCII characters?
        pub(crate) unicode: bool,
        /// The preferred line break.
        pub(crate) line_break: YamlBreakT,
        /// The stack of states.
        pub(crate) states: YamlStackT<YamlEmitterStateT>,
        /// The current emitter state.
        pub(crate) state: YamlEmitterStateT,
        /// The event queue.
        pub(crate) events: YamlQueueT<YamlEventT>,
        /// The stack of indentation levels.
        pub(crate) indents: YamlStackT<libc::c_int>,
        /// The list of tag directives.
        pub(crate) tag_directives: YamlStackT<YamlTagDirectiveT>,
        /// The current indentation level.
        pub(crate) indent: libc::c_int,
        /// The current flow level.
        pub(crate) flow_level: libc::c_int,
        /// Is it the document root context?
        pub(crate) root_context: bool,
        /// Is it a sequence context?
        pub(crate) sequence_context: bool,
        /// Is it a mapping context?
        pub(crate) mapping_context: bool,
        /// Is it a simple mapping key context?
        pub(crate) simple_key_context: bool,
        /// The current line.
        pub(crate) line: libc::c_int,
        /// The current column.
        pub(crate) column: libc::c_int,
        /// If the last character was a whitespace?
        pub(crate) whitespace: bool,
        /// If the last character was an indentation character (' ', '-', '?', ':')?
        pub(crate) indention: bool,
        /// If an explicit document end is required?
        pub(crate) open_ended: libc::c_int,
        /// Anchor analysis.
        pub(crate) anchor_data: UnnamedYamlEmitterTAnchorData,
        /// Tag analysis.
        pub(crate) tag_data: UnnamedYamlEmitterTTagData,
        /// Scalar analysis.
        pub(crate) scalar_data: UnnamedYamlEmitterTScalarData,
        /// If the stream was already opened?
        pub opened: bool,
        /// If the stream was already closed?
        pub closed: bool,
        /// The information associated with the document nodes.
        pub(crate) anchors: *mut YamlAnchorsT,
        /// The last assigned anchor id.
        pub(crate) last_anchor_id: libc::c_int,
        /// The currently emitted document.
        pub(crate) document: *mut YamlDocumentT,
    }
    #[automatically_derived]
    impl ::core::clone::Clone for YamlEmitterT {
        #[inline]
        fn clone(&self) -> YamlEmitterT {
            YamlEmitterT {
                error: ::core::clone::Clone::clone(&self.error),
                problem: ::core::clone::Clone::clone(&self.problem),
                write_handler: ::core::clone::Clone::clone(&self.write_handler),
                write_handler_data: ::core::clone::Clone::clone(
                    &self.write_handler_data,
                ),
                output: ::core::clone::Clone::clone(&self.output),
                buffer: ::core::clone::Clone::clone(&self.buffer),
                raw_buffer: ::core::clone::Clone::clone(&self.raw_buffer),
                encoding: ::core::clone::Clone::clone(&self.encoding),
                canonical: ::core::clone::Clone::clone(&self.canonical),
                best_indent: ::core::clone::Clone::clone(&self.best_indent),
                best_width: ::core::clone::Clone::clone(&self.best_width),
                unicode: ::core::clone::Clone::clone(&self.unicode),
                line_break: ::core::clone::Clone::clone(&self.line_break),
                states: ::core::clone::Clone::clone(&self.states),
                state: ::core::clone::Clone::clone(&self.state),
                events: ::core::clone::Clone::clone(&self.events),
                indents: ::core::clone::Clone::clone(&self.indents),
                tag_directives: ::core::clone::Clone::clone(&self.tag_directives),
                indent: ::core::clone::Clone::clone(&self.indent),
                flow_level: ::core::clone::Clone::clone(&self.flow_level),
                root_context: ::core::clone::Clone::clone(&self.root_context),
                sequence_context: ::core::clone::Clone::clone(&self.sequence_context),
                mapping_context: ::core::clone::Clone::clone(&self.mapping_context),
                simple_key_context: ::core::clone::Clone::clone(
                    &self.simple_key_context,
                ),
                line: ::core::clone::Clone::clone(&self.line),
                column: ::core::clone::Clone::clone(&self.column),
                whitespace: ::core::clone::Clone::clone(&self.whitespace),
                indention: ::core::clone::Clone::clone(&self.indention),
                open_ended: ::core::clone::Clone::clone(&self.open_ended),
                anchor_data: ::core::clone::Clone::clone(&self.anchor_data),
                tag_data: ::core::clone::Clone::clone(&self.tag_data),
                scalar_data: ::core::clone::Clone::clone(&self.scalar_data),
                opened: ::core::clone::Clone::clone(&self.opened),
                closed: ::core::clone::Clone::clone(&self.closed),
                anchors: ::core::clone::Clone::clone(&self.anchors),
                last_anchor_id: ::core::clone::Clone::clone(&self.last_anchor_id),
                document: ::core::clone::Clone::clone(&self.document),
            }
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlEmitterT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "error",
                "problem",
                "write_handler",
                "write_handler_data",
                "output",
                "buffer",
                "raw_buffer",
                "encoding",
                "canonical",
                "best_indent",
                "best_width",
                "unicode",
                "line_break",
                "states",
                "state",
                "events",
                "indents",
                "tag_directives",
                "indent",
                "flow_level",
                "root_context",
                "sequence_context",
                "mapping_context",
                "simple_key_context",
                "line",
                "column",
                "whitespace",
                "indention",
                "open_ended",
                "anchor_data",
                "tag_data",
                "scalar_data",
                "opened",
                "closed",
                "anchors",
                "last_anchor_id",
                "document",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.error,
                &self.problem,
                &self.write_handler,
                &self.write_handler_data,
                &self.output,
                &self.buffer,
                &self.raw_buffer,
                &self.encoding,
                &self.canonical,
                &self.best_indent,
                &self.best_width,
                &self.unicode,
                &self.line_break,
                &self.states,
                &self.state,
                &self.events,
                &self.indents,
                &self.tag_directives,
                &self.indent,
                &self.flow_level,
                &self.root_context,
                &self.sequence_context,
                &self.mapping_context,
                &self.simple_key_context,
                &self.line,
                &self.column,
                &self.whitespace,
                &self.indention,
                &self.open_ended,
                &self.anchor_data,
                &self.tag_data,
                &self.scalar_data,
                &self.opened,
                &self.closed,
                &self.anchors,
                &self.last_anchor_id,
                &&self.document,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "YamlEmitterT",
                names,
                values,
            )
        }
    }
    /// Represents the prefix data associated with a YAML emitter.
    #[repr(C)]
    #[non_exhaustive]
    pub struct YamlEmitterTPrefix {
        /// The error type.
        pub error: YamlErrorTypeT,
        /// The error description.
        pub problem: *const libc::c_char,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlEmitterTPrefix {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlEmitterTPrefix {
        #[inline]
        fn clone(&self) -> YamlEmitterTPrefix {
            let _: ::core::clone::AssertParamIsClone<YamlErrorTypeT>;
            let _: ::core::clone::AssertParamIsClone<*const libc::c_char>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlEmitterTPrefix {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "YamlEmitterTPrefix",
                "error",
                &self.error,
                "problem",
                &&self.problem,
            )
        }
    }
    #[doc(hidden)]
    impl Deref for YamlEmitterT {
        type Target = YamlEmitterTPrefix;
        fn deref(&self) -> &Self::Target {
            unsafe { &*(&raw const *self).cast() }
        }
    }
    #[repr(C)]
    /// Represents the output data associated with a YAML emitter.
    pub struct UnnamedYamlEmitterTOutput {
        /// String output data.
        pub string: UnnamedYamlEmitterTOutputString,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlEmitterTOutput {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlEmitterTOutput {
        #[inline]
        fn clone(&self) -> UnnamedYamlEmitterTOutput {
            let _: ::core::clone::AssertParamIsClone<UnnamedYamlEmitterTOutputString>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlEmitterTOutput {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field1_finish(
                f,
                "UnnamedYamlEmitterTOutput",
                "string",
                &&self.string,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for UnnamedYamlEmitterTOutput {
        #[inline]
        fn default() -> UnnamedYamlEmitterTOutput {
            UnnamedYamlEmitterTOutput {
                string: ::core::default::Default::default(),
            }
        }
    }
    #[repr(C)]
    /// Represents the unamed output string data associated with a YAML emitter.
    pub struct UnnamedYamlEmitterTOutputString {
        /// The buffer pointer.
        pub buffer: *mut libc::c_uchar,
        /// The buffer size.
        pub size: size_t,
        /// The number of written bytes.
        pub size_written: *mut size_t,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlEmitterTOutputString {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlEmitterTOutputString {
        #[inline]
        fn clone(&self) -> UnnamedYamlEmitterTOutputString {
            let _: ::core::clone::AssertParamIsClone<*mut libc::c_uchar>;
            let _: ::core::clone::AssertParamIsClone<size_t>;
            let _: ::core::clone::AssertParamIsClone<*mut size_t>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlEmitterTOutputString {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "UnnamedYamlEmitterTOutputString",
                "buffer",
                &self.buffer,
                "size",
                &self.size,
                "size_written",
                &&self.size_written,
            )
        }
    }
    #[repr(C)]
    pub(crate) struct UnnamedYamlEmitterTAnchorData {
        /// The anchor value.
        pub(crate) anchor: *mut yaml_char_t,
        /// The anchor length.
        pub(crate) anchor_length: size_t,
        /// Is it an alias?
        pub(crate) alias: bool,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlEmitterTAnchorData {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlEmitterTAnchorData {
        #[inline]
        fn clone(&self) -> UnnamedYamlEmitterTAnchorData {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<size_t>;
            let _: ::core::clone::AssertParamIsClone<bool>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlEmitterTAnchorData {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "UnnamedYamlEmitterTAnchorData",
                "anchor",
                &self.anchor,
                "anchor_length",
                &self.anchor_length,
                "alias",
                &&self.alias,
            )
        }
    }
    #[repr(C)]
    pub(crate) struct UnnamedYamlEmitterTTagData {
        /// The tag handle.
        pub(crate) handle: *mut yaml_char_t,
        /// The tag handle length.
        pub(crate) handle_length: size_t,
        /// The tag suffix.
        pub(crate) suffix: *mut yaml_char_t,
        /// The tag suffix length.
        pub(crate) suffix_length: size_t,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlEmitterTTagData {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlEmitterTTagData {
        #[inline]
        fn clone(&self) -> UnnamedYamlEmitterTTagData {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<size_t>;
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlEmitterTTagData {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "UnnamedYamlEmitterTTagData",
                "handle",
                &self.handle,
                "handle_length",
                &self.handle_length,
                "suffix",
                &self.suffix,
                "suffix_length",
                &&self.suffix_length,
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for UnnamedYamlEmitterTTagData {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for UnnamedYamlEmitterTTagData {
        #[inline]
        fn eq(&self, other: &UnnamedYamlEmitterTTagData) -> bool {
            self.handle == other.handle && self.handle_length == other.handle_length
                && self.suffix == other.suffix
                && self.suffix_length == other.suffix_length
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for UnnamedYamlEmitterTTagData {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {
            let _: ::core::cmp::AssertParamIsEq<*mut yaml_char_t>;
            let _: ::core::cmp::AssertParamIsEq<size_t>;
            let _: ::core::cmp::AssertParamIsEq<*mut yaml_char_t>;
        }
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for UnnamedYamlEmitterTTagData {
        #[inline]
        fn partial_cmp(
            &self,
            other: &UnnamedYamlEmitterTTagData,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            match ::core::cmp::PartialOrd::partial_cmp(&self.handle, &other.handle) {
                ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                    match ::core::cmp::PartialOrd::partial_cmp(
                        &self.handle_length,
                        &other.handle_length,
                    ) {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal) => {
                            match ::core::cmp::PartialOrd::partial_cmp(
                                &self.suffix,
                                &other.suffix,
                            ) {
                                ::core::option::Option::Some(
                                    ::core::cmp::Ordering::Equal,
                                ) => {
                                    ::core::cmp::PartialOrd::partial_cmp(
                                        &self.suffix_length,
                                        &other.suffix_length,
                                    )
                                }
                                cmp => cmp,
                            }
                        }
                        cmp => cmp,
                    }
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for UnnamedYamlEmitterTTagData {
        #[inline]
        fn cmp(&self, other: &UnnamedYamlEmitterTTagData) -> ::core::cmp::Ordering {
            match ::core::cmp::Ord::cmp(&self.handle, &other.handle) {
                ::core::cmp::Ordering::Equal => {
                    match ::core::cmp::Ord::cmp(
                        &self.handle_length,
                        &other.handle_length,
                    ) {
                        ::core::cmp::Ordering::Equal => {
                            match ::core::cmp::Ord::cmp(&self.suffix, &other.suffix) {
                                ::core::cmp::Ordering::Equal => {
                                    ::core::cmp::Ord::cmp(
                                        &self.suffix_length,
                                        &other.suffix_length,
                                    )
                                }
                                cmp => cmp,
                            }
                        }
                        cmp => cmp,
                    }
                }
                cmp => cmp,
            }
        }
    }
    #[automatically_derived]
    impl ::core::hash::Hash for UnnamedYamlEmitterTTagData {
        #[inline]
        fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
            ::core::hash::Hash::hash(&self.handle, state);
            ::core::hash::Hash::hash(&self.handle_length, state);
            ::core::hash::Hash::hash(&self.suffix, state);
            ::core::hash::Hash::hash(&self.suffix_length, state)
        }
    }
    #[repr(C)]
    pub(crate) struct UnnamedYamlEmitterTScalarData {
        /// The scalar value.
        pub(crate) value: *mut yaml_char_t,
        /// The scalar length.
        pub(crate) length: size_t,
        /// Does the scalar contain line breaks?
        pub(crate) multiline: bool,
        /// Can the scalar be expressed in the flow plain style?
        pub(crate) flow_plain_allowed: bool,
        /// Can the scalar be expressed in the block plain style?
        pub(crate) block_plain_allowed: bool,
        /// Can the scalar be expressed in the single quoted style?
        pub(crate) single_quoted_allowed: bool,
        /// Can the scalar be expressed in the literal or folded styles?
        pub(crate) block_allowed: bool,
        /// The output style.
        pub(crate) style: YamlScalarStyleT,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for UnnamedYamlEmitterTScalarData {}
    #[automatically_derived]
    impl ::core::clone::Clone for UnnamedYamlEmitterTScalarData {
        #[inline]
        fn clone(&self) -> UnnamedYamlEmitterTScalarData {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<size_t>;
            let _: ::core::clone::AssertParamIsClone<bool>;
            let _: ::core::clone::AssertParamIsClone<YamlScalarStyleT>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnnamedYamlEmitterTScalarData {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "value",
                "length",
                "multiline",
                "flow_plain_allowed",
                "block_plain_allowed",
                "single_quoted_allowed",
                "block_allowed",
                "style",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.value,
                &self.length,
                &self.multiline,
                &self.flow_plain_allowed,
                &self.block_plain_allowed,
                &self.single_quoted_allowed,
                &self.block_allowed,
                &&self.style,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "UnnamedYamlEmitterTScalarData",
                names,
                values,
            )
        }
    }
    #[repr(C)]
    pub(crate) struct YamlStringT {
        pub(crate) start: *mut yaml_char_t,
        pub(crate) end: *mut yaml_char_t,
        pub(crate) pointer: *mut yaml_char_t,
    }
    #[automatically_derived]
    impl ::core::marker::Copy for YamlStringT {}
    #[automatically_derived]
    impl ::core::clone::Clone for YamlStringT {
        #[inline]
        fn clone(&self) -> YamlStringT {
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            let _: ::core::clone::AssertParamIsClone<*mut yaml_char_t>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for YamlStringT {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "YamlStringT",
                "start",
                &self.start,
                "end",
                &self.end,
                "pointer",
                &&self.pointer,
            )
        }
    }
    pub(crate) const NULL_STRING: YamlStringT = YamlStringT {
        start: ptr::null_mut::<yaml_char_t>(),
        end: ptr::null_mut::<yaml_char_t>(),
        pointer: ptr::null_mut::<yaml_char_t>(),
    };
    #[repr(C)]
    /// Represents the data associated with a YAML token.
    pub struct YamlBufferT<T> {
        /// The beginning of the buffer.
        pub start: *mut T,
        /// The end of the buffer.
        pub end: *mut T,
        /// The current position of the buffer.
        pub pointer: *mut T,
        /// The last filled position of the buffer.
        pub last: *mut T,
    }
    #[automatically_derived]
    impl<T: ::core::marker::Copy> ::core::marker::Copy for YamlBufferT<T> {}
    #[automatically_derived]
    impl<T: ::core::clone::Clone> ::core::clone::Clone for YamlBufferT<T> {
        #[inline]
        fn clone(&self) -> YamlBufferT<T> {
            YamlBufferT {
                start: ::core::clone::Clone::clone(&self.start),
                end: ::core::clone::Clone::clone(&self.end),
                pointer: ::core::clone::Clone::clone(&self.pointer),
                last: ::core::clone::Clone::clone(&self.last),
            }
        }
    }
    #[automatically_derived]
    impl<T: ::core::fmt::Debug> ::core::fmt::Debug for YamlBufferT<T> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "YamlBufferT",
                "start",
                &self.start,
                "end",
                &self.end,
                "pointer",
                &self.pointer,
                "last",
                &&self.last,
            )
        }
    }
    impl<T> YamlBufferT<T> {
        /// Is the buffer empty?
        pub(crate) fn is_empty(&self) -> bool {
            self.pointer == self.last
        }
        /// Advances the buffer by one character.
        pub(crate) fn next(&mut self) {
            if !self.is_empty() {
                unsafe {
                    self.pointer = self.pointer.add(1);
                }
            }
        }
    }
    impl<T> Default for YamlBufferT<T> {
        fn default() -> Self {
            YamlBufferT {
                start: ptr::null_mut(),
                end: ptr::null_mut(),
                pointer: ptr::null_mut(),
                last: ptr::null_mut(),
            }
        }
    }
    /// The beginning of the stack.
    #[repr(C)]
    pub struct YamlStackT<T> {
        /// The beginning of the stack.
        pub start: *mut T,
        /// The end of the stack.
        pub end: *mut T,
        /// The top of the stack.
        pub top: *mut T,
    }
    #[automatically_derived]
    impl<T: ::core::fmt::Debug> ::core::fmt::Debug for YamlStackT<T> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field3_finish(
                f,
                "YamlStackT",
                "start",
                &self.start,
                "end",
                &self.end,
                "top",
                &&self.top,
            )
        }
    }
    impl<T> Copy for YamlStackT<T> {}
    impl<T> Clone for YamlStackT<T> {
        fn clone(&self) -> Self {
            *self
        }
    }
    /// The beginning of the queue.
    #[repr(C)]
    pub(crate) struct YamlQueueT<T> {
        /// The beginning of the queue.
        pub(crate) start: *mut T,
        /// The end of the queue.
        pub(crate) end: *mut T,
        /// The head of the queue.
        pub(crate) head: *mut T,
        /// The tail of the queue.
        pub(crate) tail: *mut T,
    }
    #[automatically_derived]
    impl<T: ::core::fmt::Debug> ::core::fmt::Debug for YamlQueueT<T> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field4_finish(
                f,
                "YamlQueueT",
                "start",
                &self.start,
                "end",
                &self.end,
                "head",
                &self.head,
                "tail",
                &&self.tail,
            )
        }
    }
    impl<T> Copy for YamlQueueT<T> {}
    impl<T> Clone for YamlQueueT<T> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<T: Default> Default for YamlQueueT<T> {
        fn default() -> Self {
            YamlQueueT {
                start: ptr::null_mut(),
                end: ptr::null_mut(),
                head: ptr::null_mut(),
                tail: ptr::null_mut(),
            }
        }
    }
    impl Default for YamlStringT {
        fn default() -> Self {
            YamlStringT {
                start: ptr::null_mut(),
                end: ptr::null_mut(),
                pointer: ptr::null_mut(),
            }
        }
    }
    impl Default for UnnamedYamlEmitterTScalarData {
        fn default() -> Self {
            UnnamedYamlEmitterTScalarData {
                value: ptr::null_mut(),
                length: 0,
                multiline: false,
                flow_plain_allowed: false,
                block_plain_allowed: false,
                single_quoted_allowed: false,
                block_allowed: false,
                style: YamlScalarStyleT::YamlAnyScalarStyle,
            }
        }
    }
    impl Default for UnnamedYamlEmitterTTagData {
        fn default() -> Self {
            UnnamedYamlEmitterTTagData {
                handle: ptr::null_mut(),
                handle_length: 0,
                suffix: ptr::null_mut(),
                suffix_length: 0,
            }
        }
    }
    impl Default for UnnamedYamlEmitterTAnchorData {
        fn default() -> Self {
            UnnamedYamlEmitterTAnchorData {
                anchor: ptr::null_mut(),
                anchor_length: 0,
                alias: false,
            }
        }
    }
    impl Default for UnnamedYamlEmitterTOutputString {
        fn default() -> Self {
            UnnamedYamlEmitterTOutputString {
                buffer: ptr::null_mut(),
                size: 0,
                size_written: ptr::null_mut(),
            }
        }
    }
    impl Default for UnnamedYamlParserTInputString {
        fn default() -> Self {
            UnnamedYamlParserTInputString {
                start: ptr::null(),
                end: ptr::null(),
                current: ptr::null(),
            }
        }
    }
    impl Default for UnnamedYamlDocumentTTagDirectives {
        fn default() -> Self {
            UnnamedYamlDocumentTTagDirectives {
                start: ptr::null_mut(),
                end: ptr::null_mut(),
            }
        }
    }
    impl Default for YamlEncodingT {
        fn default() -> Self {
            YamlAnyEncoding
        }
    }
    impl Default for YamlParserStateT {
        fn default() -> Self {
            YamlParserStateT::YamlParseStreamStartState
        }
    }
    impl Default for YamlScalarStyleT {
        fn default() -> Self {
            YamlScalarStyleT::YamlAnyScalarStyle
        }
    }
    impl Default for YamlTokenT {
        fn default() -> Self {
            YamlTokenT {
                type_: YamlTokenTypeT::YamlNoToken,
                data: UnnamedYamlTokenTData::default(),
                start_mark: YamlMarkT::default(),
                end_mark: YamlMarkT::default(),
            }
        }
    }
    impl Default for UnnamedYamlTokenTdataStreamStart {
        fn default() -> Self {
            UnnamedYamlTokenTdataStreamStart {
                encoding: YamlAnyEncoding,
            }
        }
    }
    impl Default for UnnamedYamlTokenTdataAlias {
        fn default() -> Self {
            UnnamedYamlTokenTdataAlias {
                value: ptr::null_mut(),
            }
        }
    }
    impl Default for UnnamedYamlTokenTdataAnchor {
        fn default() -> Self {
            UnnamedYamlTokenTdataAnchor {
                value: ptr::null_mut(),
            }
        }
    }
    impl Default for UnnamedYamlTokenTdataTag {
        fn default() -> Self {
            UnnamedYamlTokenTdataTag {
                handle: ptr::null_mut(),
                suffix: ptr::null_mut(),
            }
        }
    }
    impl Default for UnnamedYamlTokenTdataScalar {
        fn default() -> Self {
            UnnamedYamlTokenTdataScalar {
                value: ptr::null_mut(),
                length: 0,
                style: YamlScalarStyleT::YamlAnyScalarStyle,
            }
        }
    }
    impl Default for UnnamedYamlTokenTdataTagDirective {
        fn default() -> Self {
            UnnamedYamlTokenTdataTagDirective {
                handle: ptr::null_mut(),
                prefix: ptr::null_mut(),
            }
        }
    }
    impl Default for YamlStackT<YamlSimpleKeyT> {
        fn default() -> Self {
            YamlStackT {
                start: ptr::null_mut(),
                end: ptr::null_mut(),
                top: ptr::null_mut(),
            }
        }
    }
    impl Default for YamlStackT<YamlTagDirectiveT> {
        fn default() -> Self {
            YamlStackT {
                start: ptr::null_mut(),
                end: ptr::null_mut(),
                top: ptr::null_mut(),
            }
        }
    }
    impl Default for YamlStackT<YamlAliasDataT> {
        fn default() -> Self {
            YamlStackT {
                start: ptr::null_mut(),
                end: ptr::null_mut(),
                top: ptr::null_mut(),
            }
        }
    }
    impl Default for YamlTagDirectiveT {
        fn default() -> Self {
            YamlTagDirectiveT {
                handle: ptr::null_mut(),
                prefix: ptr::null_mut(),
            }
        }
    }
    impl Default for YamlBreakT {
        fn default() -> Self {
            YamlBreakT::YamlAnyBreak
        }
    }
    impl Default for YamlSequenceStyleT {
        fn default() -> Self {
            YamlSequenceStyleT::YamlAnySequenceStyle
        }
    }
    impl Default for YamlMappingStyleT {
        fn default() -> Self {
            YamlMappingStyleT::YamlAnyMappingStyle
        }
    }
    impl Default for YamlEventTypeT {
        fn default() -> Self {
            YamlNoEvent
        }
    }
    impl Default for YamlAliasDataT {
        fn default() -> Self {
            YamlAliasDataT {
                anchor: ptr::null_mut(),
                index: 0,
                mark: YamlMarkT::default(),
            }
        }
    }
    impl Default for YamlNodeTypeT {
        fn default() -> Self {
            YamlNoNode
        }
    }
    impl Default for YamlEmitterStateT {
        fn default() -> Self {
            YamlEmitterStateT::YamlEmitStreamStartState
        }
    }
    impl YamlDocumentT {
        /// Cleans up all dynamically allocated memory within `YamlDocumentT`.
        ///
        /// Frees:
        /// - `version_directive` pointer
        /// - All `tag_directives` (including each directive’s `handle`/`prefix`)
        /// - The `nodes` array, and per-node allocations (e.g., `tag`, `data.scalar.value`,
        ///   `data.sequence.items`, `data.mapping.pairs`)
        ///
        /// # Safety
        ///
        /// - Assumes pointers are either valid or null.
        /// - Must not be called if other code is still using these pointers.
        /// - Must match the memory management strategy used in allocations (`yaml_free` here).
        pub unsafe fn cleanup(&mut self) {
            if !self.version_directive.is_null() {
                yaml_free(self.version_directive as *mut libc::c_void);
                self.version_directive = ptr::null_mut();
            }
            let mut tag_ptr = self.tag_directives.start;
            while tag_ptr < self.tag_directives.end {
                yaml_free((*tag_ptr).handle as *mut libc::c_void);
                yaml_free((*tag_ptr).prefix as *mut libc::c_void);
                tag_ptr = tag_ptr.add(1);
            }
            if !self.tag_directives.start.is_null() {
                yaml_free(self.tag_directives.start as *mut libc::c_void);
            }
            self.tag_directives.start = ptr::null_mut();
            self.tag_directives.end = ptr::null_mut();
            let mut node_ptr = self.nodes.start;
            while node_ptr < self.nodes.top {
                let node = &mut *node_ptr;
                if !node.tag.is_null() {
                    yaml_free(node.tag as *mut libc::c_void);
                    node.tag = ptr::null_mut();
                }
                match node.type_ {
                    YamlScalarNode => {
                        let scalar_val = node.data.scalar.value;
                        if !scalar_val.is_null() {
                            yaml_free(scalar_val as *mut libc::c_void);
                            node.data.scalar.value = ptr::null_mut();
                        }
                    }
                    YamlSequenceNode => {
                        let items_start = node.data.sequence.items.start;
                        if !items_start.is_null() {
                            yaml_free(items_start as *mut libc::c_void);
                            node.data.sequence.items.start = ptr::null_mut();
                            node.data.sequence.items.end = ptr::null_mut();
                            node.data.sequence.items.top = ptr::null_mut();
                        }
                    }
                    YamlMappingNode => {
                        let pairs_start = node.data.mapping.pairs.start;
                        if !pairs_start.is_null() {
                            yaml_free(pairs_start as *mut libc::c_void);
                            node.data.mapping.pairs.start = ptr::null_mut();
                            node.data.mapping.pairs.end = ptr::null_mut();
                            node.data.mapping.pairs.top = ptr::null_mut();
                        }
                    }
                    _ => {}
                }
                node_ptr = node_ptr.add(1);
            }
            if !self.nodes.start.is_null() {
                yaml_free(self.nodes.start as *mut libc::c_void);
            }
            self.nodes.start = ptr::null_mut();
            self.nodes.end = ptr::null_mut();
            self.nodes.top = ptr::null_mut();
        }
    }
    impl Default for YamlDocumentT {
        fn default() -> Self {
            YamlDocumentT {
                nodes: YamlStackT {
                    start: ptr::null_mut(),
                    end: ptr::null_mut(),
                    top: ptr::null_mut(),
                },
                version_directive: ptr::null_mut(),
                tag_directives: UnnamedYamlDocumentTTagDirectives::default(),
                start_implicit: false,
                end_implicit: false,
                start_mark: YamlMarkT::default(),
                end_mark: YamlMarkT::default(),
            }
        }
    }
}
pub use crate::api::{
    yaml_alias_event_initialize, yaml_emitter_delete, yaml_emitter_initialize,
    yaml_emitter_set_break, yaml_emitter_set_canonical, yaml_emitter_set_encoding,
    yaml_emitter_set_indent, yaml_emitter_set_output, yaml_emitter_set_output_string,
    yaml_emitter_set_unicode, yaml_emitter_set_width, yaml_event_delete,
    yaml_mapping_end_event_initialize, yaml_mapping_start_event_initialize,
    yaml_parser_set_encoding, yaml_parser_set_input, yaml_parser_set_input_string,
    yaml_scalar_event_initialize, yaml_sequence_end_event_initialize,
    yaml_sequence_start_event_initialize, yaml_stream_end_event_initialize,
    yaml_stream_start_event_initialize, yaml_token_delete,
};
pub use crate::decode::{yaml_parser_delete, yaml_parser_initialize};
pub use crate::document::{
    yaml_document_delete, yaml_document_get_node, yaml_document_get_root_node,
    yaml_document_initialize,
};
pub use crate::dumper::{yaml_emitter_close, yaml_emitter_dump, yaml_emitter_open};
pub use crate::emitter::yaml_emitter_emit;
pub use crate::loader::yaml_parser_load;
pub use crate::parser::yaml_parser_parse;
pub use crate::scanner::yaml_parser_scan;
pub use crate::writer::yaml_emitter_flush;
pub use crate::yaml::{
    YamlAliasDataT, YamlBreakT, YamlDocumentT, YamlEmitterStateT, YamlEmitterT,
    YamlEncodingT, YamlErrorTypeT, YamlEventT, YamlEventTypeT, YamlMappingStyleT,
    YamlMarkT, YamlNodeItemT, YamlNodePairT, YamlNodeT, YamlNodeTypeT, YamlParserStateT,
    YamlParserT, YamlReadHandlerT, YamlScalarStyleT, YamlSequenceStyleT, YamlSimpleKeyT,
    YamlStackT, YamlTagDirectiveT, YamlTokenT, YamlTokenTypeT, YamlVersionDirectiveT,
    YamlWriteHandlerT,
};
#[doc(hidden)]
pub use crate::yaml::{
    YamlBreakT::*, YamlEmitterStateT::*, YamlEncodingT::*, YamlErrorTypeT::*,
    YamlEventTypeT::*, YamlMappingStyleT::*, YamlNodeTypeT::*, YamlParserStateT::*,
    YamlScalarStyleT::*, YamlSequenceStyleT::*, YamlTokenTypeT::*,
};
