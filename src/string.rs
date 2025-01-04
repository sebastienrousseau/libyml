// string.rs

use crate::{
    externs::memset,
    libc,
    memory::{yaml_realloc, yaml_strdup},
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
    // 1) Compute the current capacity (before reallocation), storing as `size_t`.
    let current_size: size_t = (*end).offset_from(*start) as size_t;

    // 2) Record the old offset between `pointer` and `start`, also as `size_t`.
    let old_offset: size_t = (*pointer).offset_from(*start) as size_t;

    // 3) Decide the new size (for example, double).
    let new_size: size_t = current_size * 2;

    // 4) Reallocate. This may move (and free) the old pointer.
    //    It's crucial we computed `old_offset` first.
    let new_start =
        yaml_realloc((*start).cast::<libc::c_void>(), new_size)
            as *mut yaml_char_t;
    if new_start.is_null() {
        // handle out-of-memory, or let it panic
        panic!("yaml_string_extend: reallocation failed");
    }

    // 5) Optionally zero out the newly allocated region:
    //    from `current_size` up to `new_size`.
    memset(
        new_start.add(current_size as usize).cast::<libc::c_void>(),
        0,
        current_size,
    );

    // 6) Update pointers:
    //    - `*start` => new pointer
    //    - `*pointer` => offset from new_start by old_offset
    //    - `*end` => new_start + new_size
    *start = new_start;
    *pointer = new_start.add(old_offset as usize);
    *end = new_start.add(new_size as usize);
}

/// Duplicate a null-terminated string.
/// # Safety
/// - This function is unsafe because it involves memory allocation.
pub unsafe fn yaml_string_duplicate(
    str: *const yaml_char_t,
) -> *mut yaml_char_t {
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
    // If b_start is equal to b_pointer, there's nothing to join
    if *b_start == *b_pointer {
        return;
    }

    // Calculate the length of the data in b
    let b_length = ((*b_pointer).offset_from(*b_start))
        .min((*b_end).offset_from(*b_start))
        as usize;

    // If the length of b is 0, there's nothing to copy
    if b_length == 0 {
        return;
    }

    // Ensure there's enough space in a to hold b's content
    while ((*a_end).offset_from(*a_pointer) as usize) < b_length {
        yaml_string_extend(a_start, a_pointer, a_end);
    }

    // Copy b's content to a
    core::ptr::copy_nonoverlapping(*b_start, *a_pointer, b_length);

    // Move a's pointer forward by the length of the copied data
    *a_pointer = (*a_pointer).add(b_length);
}
