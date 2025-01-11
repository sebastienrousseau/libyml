// src/loader/memory.rs
use crate::{
    externs::memset,
    libc,
    memory::{yaml_free, yaml_strdup},
    yaml::yaml_char_t,
    YamlNodeT,
};
use alloc::boxed::Box;
use core::{mem::size_of, ptr};

/// A trait for initializing YAML nodes with default values.
pub trait YamlInitialize {
    /// Initializes the YAML node with default values.
    ///
    /// # Safety
    ///
    /// - This function must be called with a valid, non-null pointer to a `YamlNodeT` struct.
    unsafe fn initialize(&mut self);
}

impl YamlInitialize for YamlNodeT {
    #[inline]
    unsafe fn initialize(&mut self) {
        let ptr: *mut Self = self;
        memset(
            ptr as *mut libc::c_void,
            0,
            size_of::<Self>() as libc::c_ulong,
        );
    }
}

/// Small string optimization for YAML scalar values.
#[repr(C)]
#[derive(
    Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub struct YamlScalarValue {
    /// Inline buffer for small strings
    buffer: [u8; 24],
    /// Length of the string
    len: u8,
    /// Heap allocation for larger strings
    heap: Option<Box<[u8]>>,
}

impl YamlScalarValue {
    /// The maximum length for inline storage
    pub const INLINE_CAPACITY: usize = 24;

    #[inline]
    /// Creates a new empty `YamlScalarValue`.
    pub const fn new() -> Self {
        Self {
            buffer: [0; Self::INLINE_CAPACITY],
            len: 0,
            heap: None,
        }
    }

    /// Creates a new `YamlScalarValue` from a byte slice.
    ///
    /// If the input is longer than the inline capacity, it will be stored on the heap.
    #[inline]
    pub fn from_slice(bytes: &[u8]) -> Self {
        if bytes.len() <= Self::INLINE_CAPACITY {
            let mut result = Self::new();
            result.buffer[..bytes.len()].copy_from_slice(bytes);
            result.len = bytes.len() as u8;
            result
        } else {
            Self {
                buffer: [0; Self::INLINE_CAPACITY],
                len: 0,
                heap: Some(bytes.into()),
            }
        }
    }

    /// Returns a slice of the contained bytes.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        if let Some(heap) = &self.heap {
            heap
        } else {
            &self.buffer[..self.len as usize]
        }
    }

    /// Returns true if the value is stored inline.
    #[inline]
    pub const fn is_inline(&self) -> bool {
        self.heap.is_none()
    }

    /// Returns the length of the contained value.
    #[inline]
    pub fn len(&self) -> usize {
        if let Some(heap) = &self.heap {
            heap.len()
        } else {
            self.len as usize
        }
    }

    /// Returns true if the value is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0 && self.heap.is_none()
    }
}

/// Allocates memory for a YAML scalar value.
///
/// # Arguments
///
/// * `value` - A C string representing the scalar value.
///
/// # Returns
///
/// * A pointer to the allocated YAML character array.
///
/// # Safety
///
/// - `value` must be a valid, null-terminated C string.
#[inline]
pub unsafe fn allocate_yaml_scalar(
    value: *const libc::c_char,
) -> *mut yaml_char_t {
    if value.is_null() {
        return ptr::null_mut();
    }
    yaml_strdup(value as *const u8)
}

/// Frees the allocated YAML memory.
///
/// # Arguments
///
/// * `ptr` - A pointer to the memory to be freed.
///
/// # Safety
///
/// - `ptr` must be a valid pointer allocated by `yaml_malloc` or related functions.
#[inline]
pub unsafe fn free_yaml_memory(ptr: *mut libc::c_void) {
    if !ptr.is_null() {
        yaml_free(ptr);
    }
}

/// Initializes a YAML node with default values.
///
/// # Arguments
///
/// * `node` - A mutable pointer to the `YamlNodeT` struct.
///
/// # Safety
///
/// - `node` must be a valid, non-null pointer to a `YamlNodeT` struct.
#[inline]
pub unsafe fn initialize_yaml_node(node: *mut YamlNodeT) {
    if !node.is_null() {
        (*node).initialize();
    }
}
