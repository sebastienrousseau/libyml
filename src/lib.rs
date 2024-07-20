//! # LibYML (a fork of unsafe-libyaml)
//!
//! [![GitHub][github-badge]][06]
//! [![Crates.io][crates-badge]][07]
//! [![lib.rs][libs-badge]][06]
//! [![Docs.rs][docs-badge]][08]
//! [![Codecov][codecov-badge]][09]
//! [![Build Status][build-badge]][10]
//!
//! This library is a fork of [unsafe-libyaml][01], a version of [libyaml][02]
//! translated from C to unsafe Rust with the assistance of [c2rust][03].
//!
//! This project, has been renamed to [LibYML][00] for simplicity and to avoid
//! confusion with the original [unsafe-libyaml][01] crate which is now
//! archived and no longer maintained.
//!
//! ## Credits and Acknowledgements
//!
//! This library is a fork of the excellent work done by [David Tolnay][04] and the
//! maintainers of the [unsafe-libyaml][01] library.
//!
//! LibYML has now evolved into a separate library with its own goals and direction
//! in mind and does not intend to replace the original unsafe-libyaml crate.
//!
//! If you are currently using unsafe-libyaml in your projects, we recommend
//! carefully evaluating your requirements and considering the stability and
//! maturity of the original library as well as looking at the features and
//! improvements offered by other libraries in the Rust ecosystem.
//!
//! I would like to express my sincere gratitude to [David Tolnay][04] and the
//! [unsafe-libyaml][01] and [libyaml][02] maintainers for their valuable
//! contributions to the Rust and C programming communities.
//!
//! ```toml
//! [dependencies]
//! libyml = "0.0.4"
//! ```
//!
//! Release notes are available under [GitHub releases][05].
//!
//! ### Rust Version Compatibility
//!
//! This library is compatible with Rust 1.60 and above.
//!
//! ## Features
//!
//! - **Serialization and Deserialization**: LibYML provides easy-to-use APIs for
//!   serializing Rust structs and enums into YAML format and deserializing YAML
//!   data into Rust types.
//! - **Custom Struct and Enum Support**: The library allows you to work with
//!   custom structs and enums, enabling seamless serialization and deserialization
//!   of your own data types.
//! - **Error Handling**: LibYML provides comprehensive error handling mechanisms,
//!   including detailed error messages and the ability to handle and recover from
//!   parsing and emission errors.
//! - **Streaming Support**: The library supports streaming of YAML data, allowing
//!   you to efficiently process large YAML documents incrementally.
//! - **Alias and Anchor Support**: LibYML handles YAML aliases and anchors,
//!   enabling you to work with complex YAML structures that involve references and
//!   duplicated data.
//! - **Tag Handling**: The library provides support for custom tags, allowing you
//!   to serialize and deserialize YAML data with specific type information.
//! - **Configurable Emitter**: LibYML allows you to customize the emitter
//!   settings, such as indentation, line width, and scalar style, to generate YAML
//!   output according to your preferences.
//! - **Extensive Documentation**: The library comes with detailed documentation
//!   and examples, making it easy to get started and understand how to use its
//!   various features effectively.
//! - **Safety and Efficiency**: LibYML is designed with safety and efficiency in
//!   mind. It minimizes the use of unsafe code and provides an interface that
//!   helps prevent common pitfalls and errors.
//!
//! ## Examples
//!
//! To get started with LibYML, you can use the examples provided in the
//! `examples` directory of the project.
//!
//! LibYML provides a set of comprehensive examples to demonstrate its usage and
//! capabilities. The examples cover various aspects of the library, including
//! initializing and deleting parsers, setting input strings, memory management,
//! string manipulation, and more.
//!
//! To run the examples, clone the repository and navigate to the examples
//! directory. Each example is contained in a separate file, focusing on specific
//! functionalities or use cases.
//!
//! For instance, to run all the example demonstrating the usage of the LibYML
//! APIs, you can execute the following command:
//!
//! ```shell
//! cargo run --example example
//! ```
//!
//! The command will execute all the examples code, demonstrating all the features
//! and use cases of the LibYML library.
//!
//! Here are a few notable examples you can also run individually:
//!
//! ### 1) The `apis` Example
//!
//! The `api.rs` file provides a set of low-level API functions for working with
//! YAML data in the LibYML library. The `apis` example showcases the usage of
//! these low-level APIs covering functions such as:
//!
//! - `yaml_parser_initialize` - Initialize a YAML parser,
//! - `yaml_parser_delete` - Delete a YAML parser,
//! - `yaml_parser_set_input_string` - Set the input string for the parser,
//! - `yaml_malloc` - Allocate memory,
//! - `yaml_realloc` - Reallocate memory,
//! - `yaml_free` - Free memory,
//! - `yaml_strdup` - Duplicate a string,
//! - `yaml_string_extend` - Extend a string,
//! - `yaml_string_join` - Join strings,
//!
//! You can run the `apis` example using the following command:
//!
//! ```shell
//! cargo run --example apis
//! ```
//!
//! ## License
//!
//! [MIT license](https://opensource.org/license/MIT), same as libyaml.
//!
//! [00]: https://libyml.com
//! [01]: https://github.com/dtolnay/unsafe-libyaml
//! [02]: https://github.com/yaml/libyaml/tree/2c891fc7a770e8ba2fec34fc6b545c672beb37e6
//! [03]: https://github.com/immunant/c2rust
//! [04]: https://github.com/dtolnay
//! [05]: https://github.com/sebastienrousseau/libyml/releases
//! [06]: https://github.com/sebastienrousseau/libyml
//! [07]: https://crates.io/crates/libyml
//! [08]: https://docs.rs/libyml
//! [09]: https://codecov.io/gh/sebastienrousseau/libyml
//! [10]: https://github.com/sebastienrousseau/libyml/actions?query=branch%3Amaster
//! [build-badge]: https://img.shields.io/github/actions/workflow/status/sebastienrousseau/libyml/release.yml?branch=master&style=for-the-badge&logo=github
//! [codecov-badge]: https://img.shields.io/codecov/c/github/sebastienrousseau/libyml?style=for-the-badge&token=yc9s578xIk&logo=codecov
//! [crates-badge]: https://img.shields.io/crates/v/libyml.svg?style=for-the-badge&color=fc8d62&logo=rust
//! [libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.4-orange.svg?style=for-the-badge
//! [docs-badge]: https://img.shields.io/badge/docs.rs-libyml-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//! [github-badge]: https://img.shields.io/badge/github-sebastienrousseau/libyml-8da0cb?style=for-the-badge&labelColor=555555&logo=github

//!

#![no_std]
#![doc(
    html_favicon_url = "https://kura.pro/libyml/images/favicon.ico",
    html_logo_url = "https://kura.pro/libyml/images/logos/libyml.svg",
    html_root_url = "https://docs.rs/libyml"
)]
#![crate_name = "libyml"]
#![crate_type = "lib"]

extern crate alloc;
use core::mem::size_of;

/// This module contains declarations for C library functions.
///
/// # Purpose
///
/// This module provides declarations for C library functions that are used in the LibYAML library.
/// These declarations are necessary for interfacing with the C library functions from Rust.
///
/// # Parameters
///
/// None.
///
/// # Return Value
///
/// This module does not return any value. It only contains declarations for C library functions.
pub mod libc {
    pub(crate) use core::ffi::c_void;
    pub(crate) use core::primitive::{
        i32 as c_int, i64 as c_long, i8 as c_char, u32 as c_uint,
        u64 as c_ulong, u8 as c_uchar,
    };
}

/// Externs for libyaml
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
        // Round up to multiple of MALLOC_ALIGN.
        (need_len + MALLOC_ALIGN - 1) & !(MALLOC_ALIGN - 1)
    };

    // `max_align_t` may be bigger than this, but libyaml does not use `long
    // double` or u128.
    const MALLOC_ALIGN: usize = {
        let int_align = align_of::<libc::c_ulong>();
        let ptr_align = align_of::<usize>();
        if int_align >= ptr_align {
            int_align
        } else {
            ptr_align
        }
    };

    pub(crate) unsafe fn malloc(
        size: libc::c_ulong,
    ) -> *mut libc::c_void {
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

    pub(crate) unsafe fn realloc(
        ptr: *mut libc::c_void,
        new_size: libc::c_ulong,
    ) -> *mut libc::c_void {
        let mut memory = ptr.cast::<u8>().sub(HEADER);
        let size = memory.cast::<usize>().read();
        let layout =
            Layout::from_size_align_unchecked(size, MALLOC_ALIGN);
        let new_size = HEADER.force_add(new_size.force_into());
        memory = rust::realloc(memory, layout, new_size);
        if memory.is_null() {
            return die();
        }
        memory.cast::<usize>().write(new_size);
        memory.add(HEADER).cast()
    }

    /// Free memory.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it deallocates memory pointed to by `ptr`.
    /// The caller must ensure that:
    /// - `ptr` is a valid pointer that was previously allocated by this allocator.
    /// - `ptr` has not been freed before.
    /// - After this call, `ptr` should not be used.
    pub unsafe fn free(ptr: *mut libc::c_void) {
        let memory = ptr.cast::<u8>().sub(HEADER);
        let size = memory.cast::<usize>().read();
        let layout =
            Layout::from_size_align_unchecked(size, MALLOC_ALIGN);
        rust::dealloc(memory, layout);
    }

    pub(crate) unsafe fn memcmp(
        lhs: *const libc::c_void,
        rhs: *const libc::c_void,
        count: libc::c_ulong,
    ) -> libc::c_int {
        let lhs =
            slice::from_raw_parts(lhs.cast::<u8>(), count as usize);
        let rhs =
            slice::from_raw_parts(rhs.cast::<u8>(), count as usize);
        lhs.cmp(rhs) as libc::c_int
    }

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

    pub(crate) unsafe fn memmove(
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

    /// Sets the first `count` bytes of the memory block pointed to by `dest` to the specified `ch` byte.
    ///
    /// # Purpose
    ///
    /// This function fills a block of memory with a specified byte value. It is commonly used to initialize or clear memory.
    ///
    /// # Parameters
    ///
    /// - `dest`: A pointer to the memory block to be filled.
    /// - `ch`: The byte value to fill the memory block with.
    /// - `count`: The number of bytes to fill in the memory block.
    ///
    /// # Return Value
    ///
    /// The function returns a pointer to the memory block (`dest`) after filling it.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it directly manipulates raw memory. The caller must ensure that:
    /// - `dest` is a valid pointer to a memory block.
    /// - `count` is not greater than the size of the memory block pointed to by `dest`.
    /// - The memory block pointed to by `dest` is properly aligned and initialized.
    pub unsafe fn memset(
        dest: *mut libc::c_void,
        ch: libc::c_int,
        count: libc::c_ulong,
    ) -> *mut libc::c_void {
        ptr::write_bytes(dest.cast::<u8>(), ch as u8, count as usize);
        dest
    }

    pub(crate) unsafe fn strcmp(
        lhs: *const libc::c_char,
        rhs: *const libc::c_char,
    ) -> libc::c_int {
        if lhs.is_null() || rhs.is_null() {
            return die();
        }
        let lhs = slice::from_raw_parts(
            lhs.cast::<u8>(),
            strlen(lhs) as usize,
        );
        let rhs = slice::from_raw_parts(
            rhs.cast::<u8>(),
            strlen(rhs) as usize,
        );
        lhs.cmp(rhs) as libc::c_int
    }

    pub(crate) unsafe fn strlen(
        str: *const libc::c_char,
    ) -> libc::c_ulong {
        let mut end = str;
        while *end != 0 {
            end = end.add(1);
        }
        end.offset_from(str) as libc::c_ulong
    }

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
        if count == 0 {
            0
        } else {
            (*lhs).cmp(&*rhs) as libc::c_int
        }
    }

    macro_rules! __assert {
        (false $(,)?) => {
            $crate::externs::__assert_fail(
                stringify!(false),
                file!(),
                line!(),
            )
        };
        ($assertion:expr $(,)?) => {
            if !$assertion {
                $crate::externs::__assert_fail(
                    stringify!($assertion),
                    file!(),
                    line!(),
                );
            }
        };
    }

    pub(crate) unsafe fn __assert_fail(
        __assertion: &'static str,
        __file: &'static str,
        __line: u32,
    ) -> ! {
        struct Abort;
        impl Drop for Abort {
            fn drop(&mut self) {
                panic!("arithmetic overflow");
            }
        }
        let _abort_on_panic = Abort;
        panic!(
            "{}:{}: Assertion `{}` failed.",
            __file, __line, __assertion
        );
    }
}

mod fmt {
    use crate::yaml::yaml_char_t;
    use core::fmt::{self, Write};
    use core::ptr;

    pub(crate) struct WriteToPtr {
        ptr: *mut yaml_char_t,
    }

    impl WriteToPtr {
        pub(crate) unsafe fn new(ptr: *mut yaml_char_t) -> Self {
            WriteToPtr { ptr }
        }

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

#[macro_use]
mod macros;

/// Macros module for LibYML
#[macro_use]
pub mod utils;

/// API module for LibYML
pub mod api;

/// String module for LibYML
pub mod string;

mod dumper;
mod emitter;
/// Loader module for LibYML
pub mod loader;

/// Decode module for LibYML
pub mod decode;

/// Document module for LibYML
pub mod document;

/// Internal module for LibYML
pub mod internal;

/// Memory module for LibYML
pub mod memory;
mod ops;
mod parser;
mod reader;
mod scanner;
/// Success and Failure types for LibYML
pub mod success;
mod writer;
/// YAML API module for LibYML
pub mod yaml;

pub use crate::api::{
    yaml_alias_event_initialize, yaml_emitter_delete,
    yaml_emitter_initialize, yaml_emitter_set_break,
    yaml_emitter_set_canonical, yaml_emitter_set_encoding,
    yaml_emitter_set_indent, yaml_emitter_set_output,
    yaml_emitter_set_output_string, yaml_emitter_set_unicode,
    yaml_emitter_set_width, yaml_event_delete,
    yaml_mapping_end_event_initialize,
    yaml_mapping_start_event_initialize, yaml_parser_set_encoding,
    yaml_parser_set_input, yaml_parser_set_input_string,
    yaml_scalar_event_initialize, yaml_sequence_end_event_initialize,
    yaml_sequence_start_event_initialize,
    yaml_stream_end_event_initialize,
    yaml_stream_start_event_initialize, yaml_token_delete,
};
pub use crate::decode::{yaml_parser_delete, yaml_parser_initialize};
pub use crate::document::{
    yaml_document_delete, yaml_document_get_node,
    yaml_document_get_root_node, yaml_document_initialize,
};
pub use crate::dumper::{
    yaml_emitter_close, yaml_emitter_dump, yaml_emitter_open,
};
pub use crate::emitter::yaml_emitter_emit;
pub use crate::loader::yaml_parser_load;
pub use crate::parser::yaml_parser_parse;
pub use crate::scanner::yaml_parser_scan;
pub use crate::writer::yaml_emitter_flush;
pub use crate::yaml::{
    YamlAliasDataT, YamlBreakT, YamlDocumentT, YamlEmitterStateT,
    YamlEmitterT, YamlEncodingT, YamlErrorTypeT, YamlEventT,
    YamlEventTypeT, YamlMappingStyleT, YamlMarkT, YamlNodeItemT,
    YamlNodePairT, YamlNodeT, YamlNodeTypeT, YamlParserStateT,
    YamlParserT, YamlReadHandlerT, YamlScalarStyleT,
    YamlSequenceStyleT, YamlSimpleKeyT, YamlStackT, YamlTagDirectiveT,
    YamlTokenT, YamlTokenTypeT, YamlVersionDirectiveT,
    YamlWriteHandlerT,
};
#[doc(hidden)]
pub use crate::yaml::{
    YamlBreakT::*, YamlEmitterStateT::*, YamlEncodingT::*,
    YamlErrorTypeT::*, YamlEventTypeT::*, YamlMappingStyleT::*,
    YamlNodeTypeT::*, YamlParserStateT::*, YamlScalarStyleT::*,
    YamlSequenceStyleT::*, YamlTokenTypeT::*,
};
